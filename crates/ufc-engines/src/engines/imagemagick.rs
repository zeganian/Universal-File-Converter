//! The ImageMagick adapter.
//!
//! The adapter converts a PNG image into a JPEG image and a JPEG image
//! into a PNG image with ImageMagick 7. It is the first implemented
//! engine of the application and the reference implementation of the
//! adapter contract (`ARCHITECTURE.md` section 4).
//!
//! The adapter never builds a shell command string: it produces an
//! executable path and an argument vector, and the paths it uses are
//! canonical, absolute paths. An absolute path can never be mistaken
//! for an option by the engine.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ufc_core::domain::conversion::{
    output_file_name, EngineAvailability, EngineId, EngineSource, EngineStatus, OverwritePolicy,
    ValidationLevel,
};
use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use ufc_core::domain::format::{stores_transparency, FormatId};
use ufc_core::domain::job::Progress;
use ufc_core::domain::registry::{
    CapabilityOptions, ConversionCapability, EngineVersionRange, PairStatus,
};
use ufc_core::engines::adapter::{
    CancellationSignal, EngineAdapter, EngineInvocation, EngineOutcome, ValidatedConversion,
};

use crate::engines::detection;
use crate::process::{ProcessRequest, ProcessRunner};
use crate::validation;

/// The identifier of the engine.
pub const ENGINE_ID: &str = "imagemagick";

/// The name that is shown to the user.
pub const DISPLAY_NAME: &str = "ImageMagick";

/// The lowest tested ImageMagick version.
pub const MIN_TESTED_VERSION: &str = "7.1.0";

/// The highest tested ImageMagick version.
pub const MAX_TESTED_VERSION: &str = "7.99.99";

/// The maximum runtime of a single conversion.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// The default JPEG quality.
pub const DEFAULT_QUALITY: u8 = 92;

/// The default background colour for a target that cannot store
/// transparency.
pub const DEFAULT_BACKGROUND: &str = "white";

/// The compression level of a PNG file.
///
/// PNG is lossless, so the size of the file is the only thing the
/// encoder can trade. The highest level produces the smallest file.
const PNG_COMPRESSION_LEVEL: u8 = 9;

/// A detected ImageMagick installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installation {
    /// The program that is used to run a conversion.
    pub program: PathBuf,
    /// The exact version that was reported by the program.
    pub version: String,
}

/// The ImageMagick adapter.
#[derive(Debug, Clone, Default)]
pub struct ImageMagickAdapter {
    installation: Option<Installation>,
}

impl ImageMagickAdapter {
    /// Creates an adapter without a detected installation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an adapter and looks for ImageMagick on this device.
    pub fn detect() -> Self {
        let mut adapter = Self::new();
        adapter.installation = detect_imagemagick().ok();
        adapter
    }

    /// Returns the detected installation, when there is one.
    pub fn installation(&self) -> Option<&Installation> {
        self.installation.as_ref()
    }

    /// Returns the source format of the PNG to JPEG pair.
    pub fn source_format() -> FormatId {
        FormatId::new("png").expect("the source format is a valid identifier")
    }

    /// Returns the target format of the PNG to JPEG pair.
    pub fn target_format() -> FormatId {
        FormatId::new("jpeg").expect("the target format is a valid identifier")
    }

    /// Returns the source format of the JPEG to PNG pair.
    pub fn jpeg_source_format() -> FormatId {
        FormatId::new("jpeg").expect("the source format is a valid identifier")
    }

    /// Returns the target format of the JPEG to PNG pair.
    pub fn jpeg_target_format() -> FormatId {
        FormatId::new("png").expect("the target format is a valid identifier")
    }

    /// Returns the capability of the PNG to JPEG pair.
    pub fn capability() -> ConversionCapability {
        ConversionCapability {
            source_format: Self::source_format(),
            target_format: Self::target_format(),
            engine_id: EngineId::new(ENGINE_ID).expect("the engine id is valid"),
            engine_version_range: Some(EngineVersionRange {
                min: MIN_TESTED_VERSION.to_owned(),
                max: MAX_TESTED_VERSION.to_owned(),
            }),
            // The pair is implemented and tested on Linux. It stays
            // experimental until the release gates of `PRD.md` section
            // 13 pass on the other platforms as well.
            status: PairStatus::Experimental,
            validation_strategy: "independent-jpeg-decoder".to_owned(),
            options: CapabilityOptions {
                quality: true,
                background: true,
            },
            limitations: vec![
                "JPEG cannot store transparency. The source is flattened onto a solid background, by default white.".to_owned(),
                "JPEG is a lossy format. The output does not reproduce the source pixel for pixel.".to_owned(),
            ],
            available: false,
        }
    }

    /// Returns the capability of the JPEG to PNG pair.
    pub fn jpeg_to_png_capability() -> ConversionCapability {
        ConversionCapability {
            source_format: Self::jpeg_source_format(),
            target_format: Self::jpeg_target_format(),
            engine_id: EngineId::new(ENGINE_ID).expect("the engine id is valid"),
            engine_version_range: Some(EngineVersionRange {
                min: MIN_TESTED_VERSION.to_owned(),
                max: MAX_TESTED_VERSION.to_owned(),
            }),
            status: PairStatus::Experimental,
            validation_strategy: "independent-png-decoder".to_owned(),
            options: CapabilityOptions {
                // PNG is lossless. There is no encoder quality that
                // would change the picture.
                quality: false,
                background: false,
            },
            limitations: vec![
                "PNG is lossless. The output reproduces the source pixel for pixel, but it is usually larger than the JPEG.".to_owned(),
                "The size of the file is decided by the compression level, not by a quality setting.".to_owned(),
            ],
            available: false,
        }
    }

    /// Returns the capability of a pair, when the adapter performs it.
    pub fn capability_for(source: &FormatId, target: &FormatId) -> Option<ConversionCapability> {
        if source == &Self::source_format() && target == &Self::target_format() {
            return Some(Self::capability());
        }

        if source == &Self::jpeg_source_format() && target == &Self::jpeg_target_format() {
            return Some(Self::jpeg_to_png_capability());
        }

        None
    }
}

/// Looks for ImageMagick 7 on this device.
fn detect_imagemagick() -> DomainResult<Installation> {
    let hint = detection::imagemagick_install_hint();

    let program = detection::find_executable(&["magick", "magick.exe"]).ok_or_else(|| {
        // ImageMagick 6 is a different program with a different command
        // line. It is reported separately, because the user can act on
        // it.
        if let Some(legacy) = detection::find_executable(&["convert", "convert.exe"]) {
            if let Some(version) = legacy_version(&legacy) {
                return detection::incompatible_version_error(
                    DISPLAY_NAME,
                    &version,
                    "ImageMagick 7",
                    &hint,
                );
            }
        }

        detection::missing_engine_error(DISPLAY_NAME, &hint)
    })?;

    let output = run_version_probe(&program)?;
    let version = detection::parse_imagemagick_version(&output).ok_or_else(|| {
        ConversionError::new(
            ConversionErrorCode::IncompatibleEngineVersion,
            format!("The installed {DISPLAY_NAME} did not report a usable version."),
        )
        .with_remediation(hint.clone())
    })?;

    if !is_supported_version(&version) {
        return Err(detection::incompatible_version_error(
            DISPLAY_NAME,
            &version,
            "ImageMagick 7.1 or newer",
            &hint,
        ));
    }

    Ok(Installation { program, version })
}

/// Reads the version of a legacy `convert` program, if it is one.
fn legacy_version(program: &Path) -> Option<String> {
    let output = run_version_probe(program).ok()?;
    let version = detection::parse_imagemagick_version(&output)?;

    match detection::imagemagick_major(&version) {
        Some(6) => Some(version),
        _ => None,
    }
}

/// Runs `<program> -version` and returns its output.
fn run_version_probe(program: &Path) -> DomainResult<String> {
    let working_directory = std::env::temp_dir();
    let request = ProcessRequest::new(program, vec!["-version".to_owned()], working_directory)
        .with_timeout(Duration::from_secs(15));

    let outcome = ProcessRunner::run(&request, &crate::engines::NeverCancelled)?;

    if !outcome.is_success() {
        return Err(ConversionError::new(
            ConversionErrorCode::IncompatibleEngineVersion,
            format!("The installed {DISPLAY_NAME} could not be queried."),
        )
        .with_detail(outcome.stderr.trim().to_owned()));
    }

    Ok(outcome.stdout)
}

/// Returns `true` when the version is inside the tested range.
fn is_supported_version(version: &str) -> bool {
    let parsed = match semver::Version::parse(version) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };

    let min = semver::Version::parse(MIN_TESTED_VERSION);
    let max = semver::Version::parse(MAX_TESTED_VERSION);

    match (min, max) {
        (Ok(min), Ok(max)) => parsed >= min && parsed <= max,
        _ => false,
    }
}

impl EngineAdapter for ImageMagickAdapter {
    fn engine_id(&self) -> EngineId {
        EngineId::new(ENGINE_ID).expect("the engine id is valid")
    }

    fn display_name(&self) -> &'static str {
        DISPLAY_NAME
    }

    fn detect_installation(&self) -> EngineStatus {
        let hint = detection::imagemagick_install_hint();

        match &self.installation {
            Some(installation) => EngineStatus {
                engine_id: self.engine_id(),
                display_name: DISPLAY_NAME.to_owned(),
                availability: EngineAvailability::Available,
                source: Some(EngineSource::System),
                version: Some(installation.version.clone()),
                installation_hint: None,
                reason: None,
            },
            None => {
                let reason = detect_imagemagick()
                    .err()
                    .unwrap_or_else(|| detection::missing_engine_error(DISPLAY_NAME, &hint));

                EngineStatus {
                    engine_id: self.engine_id(),
                    display_name: DISPLAY_NAME.to_owned(),
                    availability: match reason.code {
                        ConversionErrorCode::IncompatibleEngineVersion => {
                            EngineAvailability::IncompatibleVersion
                        }
                        _ => EngineAvailability::Missing,
                    },
                    source: None,
                    version: None,
                    installation_hint: Some(hint),
                    reason: Some(reason),
                }
            }
        }
    }

    fn capabilities(&self) -> Vec<ConversionCapability> {
        vec![Self::capability(), Self::jpeg_to_png_capability()]
    }

    fn validate_request(&self, request: &ValidatedConversion) -> DomainResult<()> {
        let capability = Self::capability_for(&request.source_format, &request.target_format)
            .ok_or_else(|| {
                ufc_core::domain::conversion::unsupported_pair(
                    &request.source_format,
                    &request.target_format,
                )
            })?;

        request.options.validate()?;
        validate_options(&request.options, &capability.options)?;

        validate_source_file(&request.input_path, &request.source_format)?;

        let metadata = std::fs::metadata(&request.output_directory).map_err(|source| {
            ConversionError::new(
                ConversionErrorCode::InvalidOutputLocation,
                "The output directory could not be read.",
            )
            .with_detail(source.to_string())
        })?;

        if !metadata.is_dir() {
            return Err(ConversionError::new(
                ConversionErrorCode::InvalidOutputLocation,
                "The output location is not a directory.",
            ));
        }

        Ok(())
    }

    fn build_invocation(&self, request: &ValidatedConversion) -> DomainResult<EngineInvocation> {
        let installation = self.installation.as_ref().ok_or_else(|| {
            detection::missing_engine_error(DISPLAY_NAME, &detection::imagemagick_install_hint())
        })?;

        let output_path = request.output_directory.join(output_file_name(
            &request.input_path,
            &request.target_format,
        ));

        let mut args = vec![request.input_path.to_string_lossy().to_string()];

        if stores_transparency(&request.target_format) {
            // The target keeps its alpha channel, so nothing is
            // flattened and no background is needed.
            args.extend([
                "-define".to_owned(),
                format!("png:compression-level={PNG_COMPRESSION_LEVEL}"),
            ]);
        } else {
            let quality = request.options.quality.unwrap_or(DEFAULT_QUALITY);
            let background = request
                .options
                .background
                .as_deref()
                .unwrap_or(DEFAULT_BACKGROUND);

            // JPEG has no alpha channel. The source is composited onto a
            // solid background so that transparent areas do not turn
            // into black.
            args.extend([
                "-background".to_owned(),
                background.to_owned(),
                "-alpha".to_owned(),
                "remove".to_owned(),
                "-alpha".to_owned(),
                "off".to_owned(),
                "-quality".to_owned(),
                quality.to_string(),
            ]);
        }

        // Resource limits keep a hostile or broken input from
        // exhausting the machine.
        args.extend([
            "-limit".to_owned(),
            "memory".to_owned(),
            "256MiB".to_owned(),
            "-limit".to_owned(),
            "map".to_owned(),
            "512MiB".to_owned(),
            output_path.to_string_lossy().to_string(),
        ]);

        Ok(EngineInvocation::new(
            installation.program.clone(),
            args,
            request.output_directory.clone(),
        ))
    }

    fn execute(
        &self,
        invocation: &EngineInvocation,
        cancel: &dyn CancellationSignal,
    ) -> DomainResult<EngineOutcome> {
        let mut request = ProcessRequest::new(
            invocation.program.clone(),
            invocation.args.clone(),
            invocation.working_directory.clone(),
        )
        .with_timeout(DEFAULT_TIMEOUT)
        .with_redaction(&invocation.working_directory, "<job>");

        if let Some(output) = invocation.args.last() {
            request = request.with_redaction(output, "<output>");
        }
        if let Some(input) = invocation.args.first() {
            request = request.with_redaction(input, "<input>");
        }

        let outcome = ProcessRunner::run(&request, cancel)?;

        if outcome.cancelled {
            return Err(ConversionError::new(
                ConversionErrorCode::Cancelled,
                "The conversion was cancelled.",
            ));
        }

        if outcome.timed_out {
            return Err(ConversionError::new(
                ConversionErrorCode::Timeout,
                "The conversion engine took too long and was stopped.",
            )
            .with_remediation("Try a smaller file or convert it again later."));
        }

        if !outcome.is_success() {
            return Err(ConversionError::new(
                ConversionErrorCode::EngineFailed,
                format!("{DISPLAY_NAME} could not convert the file."),
            )
            .with_detail(sanitise_stderr(&outcome.stderr)));
        }

        Ok(EngineOutcome {
            exit_code: outcome.exit_code,
            stdout: outcome.stdout,
            stderr: outcome.stderr,
            produced_files: Vec::new(),
            progress: vec![Progress::Stage {
                name: "converting".to_owned(),
            }],
        })
    }

    fn validate_output(
        &self,
        path: &Path,
        target_format: &FormatId,
    ) -> DomainResult<ValidationLevel> {
        validation::validate_image(path, target_format)
    }
}

/// Checks the options against the ones the pair accepts.
///
/// An option that the pair does not accept is rejected instead of
/// being silently ignored: a quality setting on a lossless target
/// would tell the user that the file is compressed, while the encoder
/// ignores the setting completely.
fn validate_options(
    options: &ufc_core::domain::conversion::ConversionOptions,
    accepted: &CapabilityOptions,
) -> DomainResult<()> {
    if options.quality.is_some() && !accepted.quality {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidOptions,
            "This conversion does not accept a quality setting.",
        )
        .with_remediation("Remove the quality setting and convert the file again."));
    }

    if options.background.is_some() && !accepted.background {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidOptions,
            "This conversion does not accept a background colour.",
        )
        .with_remediation("Remove the background colour and convert the file again."));
    }

    Ok(())
}

/// Checks the source file of a conversion.
///
/// The extension and the content have to agree: a file that is named
/// `.png` but contains something else is rejected instead of being
/// handed to the engine (`ARCHITECTURE.md` section 7).
fn validate_source_file(path: &Path, source_format: &FormatId) -> DomainResult<()> {
    let metadata = std::fs::metadata(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source file could not be read.",
        )
        .with_detail(source.to_string())
    })?;

    if !metadata.is_file() {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source is not a regular file.",
        ));
    }

    let detected = infer::get_from_path(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source file could not be inspected.",
        )
        .with_detail(source.to_string())
    })?;

    // The content is checked, not only the name of the file: a text
    // file that is called `.png` is not a PNG image.
    let expected = expected_extension(source_format).ok_or_else(|| {
        ufc_core::domain::conversion::unsupported_pair(source_format, source_format)
    })?;

    let is_expected = detected
        .map(|kind| kind.extension() == expected)
        .unwrap_or(false);

    if !is_expected {
        return Err(ConversionError::new(
            ConversionErrorCode::UndetectedFormat,
            format!(
                "The source file is not a {} image.",
                expected_name(source_format)
            ),
        ));
    }

    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let extension_format =
        ufc_core::domain::format::from_extension(&extension).map(|id| id.to_string());

    if extension_format.as_deref() != Some(source_format.as_str()) {
        return Err(ConversionError::new(
            ConversionErrorCode::FormatMismatch,
            format!(
                "The file extension is '{extension}', but the content of the file is a {} image.",
                expected_name(source_format)
            ),
        )
        .with_remediation(format!(
            "Rename the file to end in '.{}' to convert it.",
            expected_extension(source_format).unwrap_or_default()
        )));
    }

    Ok(())
}

/// Returns the extension that a source file of a format carries.
///
/// The extension is the one of the format, not of the engine: JPEG
/// files are called `.jpg`, like the users expect.
fn expected_extension(format: &FormatId) -> Option<&'static str> {
    match format.as_str() {
        "png" => Some("png"),
        "jpeg" => Some("jpg"),
        _ => None,
    }
}

/// Returns the name of a format, as it is shown to the user.
fn expected_name(format: &FormatId) -> &str {
    match format.as_str() {
        "png" => "PNG",
        "jpeg" => "JPEG",
        other => other,
    }
}

/// Shortens the captured error output of the engine.
fn sanitise_stderr(stderr: &str) -> String {
    let trimmed = stderr.trim();

    if trimmed.is_empty() {
        return "The engine did not report an error.".to_owned();
    }

    let max = 800;
    if trimmed.chars().count() <= max {
        return trimmed.to_owned();
    }

    let head: String = trimmed.chars().take(max).collect();
    format!("{head}…")
}

/// The overwrite policy that the adapter uses.
///
/// The adapter never decides whether an existing file is replaced: the
/// service resolves the destination and applies the policy before the
/// engine is started (`PRD.md` FR-009).
pub const OVERWRITE_POLICY: OverwritePolicy = OverwritePolicy::Skip;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_range_accepts_tested_versions() {
        assert!(is_supported_version("7.1.2-34"));
        assert!(is_supported_version("7.1.0"));
        assert!(is_supported_version("7.99.99"));
    }

    #[test]
    fn the_version_range_rejects_other_versions() {
        assert!(!is_supported_version("6.9.11-60"));
        assert!(!is_supported_version("8.0.0"));
        assert!(!is_supported_version("not-a-version"));
    }

    #[test]
    fn the_capability_describes_the_implemented_pair() {
        let capability = ImageMagickAdapter::capability();

        assert_eq!(capability.source_format.as_str(), "png");
        assert_eq!(capability.target_format.as_str(), "jpeg");
        assert_eq!(capability.engine_id.as_str(), ENGINE_ID);
        assert_eq!(capability.status, PairStatus::Experimental);
        assert!(
            !capability.available,
            "the capability is only available when the engine was detected"
        );
        assert!(capability
            .limitations
            .iter()
            .any(|limitation| limitation.contains("transparency")));
    }

    #[test]
    fn the_invocation_keeps_the_arguments_separated() {
        let installation = Installation {
            program: PathBuf::from("/usr/bin/magick"),
            version: "7.1.2-34".to_owned(),
        };
        let adapter = ImageMagickAdapter {
            installation: Some(installation),
        };

        let request = ValidatedConversion {
            input_path: PathBuf::from("/tmp/a b.png"),
            source_format: ImageMagickAdapter::source_format(),
            target_format: ImageMagickAdapter::target_format(),
            output_directory: PathBuf::from("/tmp/out"),
            overwrite_policy: OverwritePolicy::Skip,
            options: Default::default(),
        };

        let invocation = adapter.build_invocation(&request).expect("invocation");

        assert_eq!(invocation.program, PathBuf::from("/usr/bin/magick"));
        assert_eq!(invocation.args.first().unwrap(), "/tmp/a b.png");
        assert!(invocation.args.contains(&"remove".to_owned()));
        assert!(invocation.args.contains(&"92".to_owned()));
        assert_eq!(
            invocation.args.last().unwrap(),
            "/tmp/out/a b.jpg",
            "the output keeps the name of the source"
        );
    }

    #[test]
    fn the_invocation_uses_the_requested_quality_and_background() {
        let adapter = ImageMagickAdapter {
            installation: Some(Installation {
                program: PathBuf::from("/usr/bin/magick"),
                version: "7.1.2-34".to_owned(),
            }),
        };

        let request = ValidatedConversion {
            input_path: PathBuf::from("/tmp/a.png"),
            source_format: ImageMagickAdapter::source_format(),
            target_format: ImageMagickAdapter::target_format(),
            output_directory: PathBuf::from("/tmp/out"),
            overwrite_policy: OverwritePolicy::Skip,
            options: ufc_core::domain::conversion::ConversionOptions::new()
                .with_quality(70)
                .with_background("black"),
        };

        let invocation = adapter.build_invocation(&request).expect("invocation");

        assert!(invocation.args.contains(&"70".to_owned()));
        assert!(invocation.args.contains(&"black".to_owned()));
    }

    #[test]
    fn an_adapter_without_an_installation_cannot_build_an_invocation() {
        let adapter = ImageMagickAdapter::new();
        let request = ValidatedConversion {
            input_path: PathBuf::from("/tmp/a.png"),
            source_format: ImageMagickAdapter::source_format(),
            target_format: ImageMagickAdapter::target_format(),
            output_directory: PathBuf::from("/tmp/out"),
            overwrite_policy: OverwritePolicy::Skip,
            options: Default::default(),
        };

        let error = adapter.build_invocation(&request).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::MissingDependency);
    }

    #[test]
    fn the_capability_is_found_for_every_implemented_pair() {
        let png = ImageMagickAdapter::source_format();
        let jpeg = ImageMagickAdapter::target_format();

        let capability =
            ImageMagickAdapter::capability_for(&png, &jpeg).expect("the pair is implemented");
        assert_eq!(capability.source_format, png);
        assert_eq!(capability.target_format, jpeg);
        assert!(capability.options.quality);
        assert!(capability.options.background);

        let capability =
            ImageMagickAdapter::capability_for(&jpeg, &png).expect("the pair is implemented");
        assert_eq!(capability.source_format, jpeg);
        assert_eq!(capability.target_format, png);
        assert!(
            !capability.options.quality,
            "PNG is lossless, there is no quality to choose"
        );
        assert!(
            !capability.options.background,
            "PNG stores transparency, there is no background to choose"
        );
    }

    #[test]
    fn a_pair_that_is_not_implemented_has_no_capability() {
        let png = ImageMagickAdapter::source_format();
        let pdf = FormatId::new("pdf").expect("valid");

        assert!(ImageMagickAdapter::capability_for(&png, &pdf).is_none());
    }

    #[test]
    fn the_invocation_of_the_jpeg_to_png_pair_keeps_the_alpha_channel() {
        let adapter = ImageMagickAdapter {
            installation: Some(Installation {
                program: PathBuf::from("/usr/bin/magick"),
                version: "7.1.2-34".to_owned(),
            }),
        };

        let request = ValidatedConversion {
            input_path: PathBuf::from("/tmp/photo.jpg"),
            source_format: ImageMagickAdapter::jpeg_source_format(),
            target_format: ImageMagickAdapter::jpeg_target_format(),
            output_directory: PathBuf::from("/tmp/out"),
            overwrite_policy: OverwritePolicy::Skip,
            options: Default::default(),
        };

        let invocation = adapter.build_invocation(&request).expect("invocation");

        assert_eq!(invocation.args.first().unwrap(), "/tmp/photo.jpg");
        assert!(
            !invocation.args.iter().any(|argument| argument == "remove"),
            "a target that stores transparency is never flattened"
        );
        assert!(
            !invocation
                .args
                .iter()
                .any(|argument| argument == "-quality"),
            "a lossless target has no quality setting"
        );
        assert!(
            invocation
                .args
                .iter()
                .any(|argument| argument.starts_with("png:compression-level")),
            "the compression level of the PNG is set"
        );
        assert_eq!(invocation.args.last().unwrap(), "/tmp/out/photo.png");
    }

    #[test]
    fn a_quality_setting_is_rejected_for_a_pair_that_does_not_accept_it() {
        let adapter = ImageMagickAdapter::new();
        let request = ValidatedConversion {
            input_path: PathBuf::from("/tmp/photo.jpg"),
            source_format: ImageMagickAdapter::jpeg_source_format(),
            target_format: ImageMagickAdapter::jpeg_target_format(),
            output_directory: PathBuf::from("/tmp/out"),
            overwrite_policy: OverwritePolicy::Skip,
            options: ufc_core::domain::conversion::ConversionOptions::new().with_quality(80),
        };

        let error = adapter.validate_request(&request).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::InvalidOptions);
    }
}
