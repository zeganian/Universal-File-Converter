//! Conversion request and result models.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use crate::domain::format::FormatId;
use crate::domain::job::{JobStatus, Progress};

/// Policy applied when the output file already exists.
///
/// `PRD.md` FR-009 forbids overwriting an existing output without an
/// explicit user decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum OverwritePolicy {
    /// Leave the existing file untouched and report a structured error.
    #[default]
    Skip,
    /// Write to a new name such as `report (2).pdf`.
    Rename,
    /// Replace the existing file.
    Overwrite,
}

/// How a single output file was verified.
///
/// Levels are ordered by strength, matching `ARCHITECTURE.md` section 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ValidationLevel {
    /// The output exists and is a regular file.
    Exists,
    /// The output is non-empty and matches the expected signature.
    NonEmptySignature,
    /// The output could be parsed by an independent reader.
    Parsed,
    /// The output matched semantic or visual expectations.
    Semantic,
}

impl ValidationLevel {
    /// Returns the numeric level as defined in `ARCHITECTURE.md`.
    pub fn level(self) -> u8 {
        match self {
            ValidationLevel::Exists => 0,
            ValidationLevel::NonEmptySignature => 1,
            ValidationLevel::Parsed => 2,
            ValidationLevel::Semantic => 3,
        }
    }
}

/// Options that influence how a conversion is performed.
///
/// The options are part of the domain model, not of an engine: they
/// describe what the user asked for, and each adapter decides how to
/// translate them into engine arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", default)]
pub struct ConversionOptions {
    /// Encoder quality between 1 and 100. `None` uses the engine default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    /// Background colour used when the target format cannot store
    /// transparency, for example when a PNG becomes a JPEG.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

impl ConversionOptions {
    /// Lowest accepted quality.
    pub const MIN_QUALITY: u8 = 1;
    /// Highest accepted quality.
    pub const MAX_QUALITY: u8 = 100;

    /// Returns the options without any change.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the encoder quality.
    #[must_use]
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Sets the background colour.
    #[must_use]
    pub fn with_background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Returns `true` when no option is set.
    pub fn is_empty(&self) -> bool {
        self.quality.is_none() && self.background.is_none()
    }

    /// Validates the options.
    ///
    /// # Errors
    /// Returns [`ConversionErrorCode::InvalidOptions`] when the quality
    /// is outside the accepted range or when the background colour is
    /// not a plain colour token.
    pub fn validate(&self) -> DomainResult<()> {
        if let Some(quality) = self.quality {
            if !(Self::MIN_QUALITY..=Self::MAX_QUALITY).contains(&quality) {
                return Err(ConversionError::new(
                    ConversionErrorCode::InvalidOptions,
                    format!(
                        "Quality must be between {} and {}.",
                        Self::MIN_QUALITY,
                        Self::MAX_QUALITY
                    ),
                )
                .with_remediation("Choose a quality between 1 and 100."));
            }
        }

        if let Some(background) = &self.background {
            if !is_colour_token(background) {
                return Err(ConversionError::new(
                    ConversionErrorCode::InvalidOptions,
                    "The background colour is not a valid colour.",
                )
                .with_remediation(
                    "Use a colour name such as `white` or a hex value such as `#ffffff`.",
                ));
            }
        }

        Ok(())
    }
}

/// Returns `true` when the value is a plain colour token.
///
/// Only names, hex values and simple functional notations are accepted.
/// The value ends up in an engine argument vector, so it stays
/// deliberately narrow.
fn is_colour_token(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }

    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'#' || byte == b'-' || byte == b'_')
}

/// A request to convert one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    /// Identifier of the source file being converted.
    pub file_id: Uuid,
    /// Absolute path of the source file.
    pub input_path: String,
    /// Target format identifier.
    pub target_format: FormatId,
    /// Absolute path of the directory that receives the output.
    pub output_directory: String,
    /// How to behave when the output file already exists.
    pub overwrite_policy: OverwritePolicy,
    /// Options that influence the conversion.
    #[serde(default)]
    pub options: ConversionOptions,
}

impl ConversionRequest {
    /// Creates a request with the `Skip` overwrite policy and no options.
    pub fn new(
        file_id: Uuid,
        input_path: impl Into<String>,
        target_format: FormatId,
        output_directory: impl Into<String>,
    ) -> Self {
        Self {
            file_id,
            input_path: input_path.into(),
            target_format,
            output_directory: output_directory.into(),
            overwrite_policy: OverwritePolicy::default(),
            options: ConversionOptions::default(),
        }
    }

    /// Replaces the options of the request.
    #[must_use]
    pub fn with_options(mut self, options: ConversionOptions) -> Self {
        self.options = options;
        self
    }
}

/// The outcome of a conversion job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    /// Identifier of the job.
    pub job_id: Uuid,
    /// Final status of the job.
    pub status: JobStatus,
    /// Name of the source file, without its directory.
    ///
    /// Only the name is kept, never the full path, so that the queue can
    /// show what is being converted without storing the location of the
    /// file (`ARCHITECTURE.md` section 12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    /// Absolute path of the produced output, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    /// Identifier of the engine that performed the conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_id: Option<String>,
    /// Exact engine version used for the conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version: Option<String>,
    /// Non-fatal remarks about the conversion.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// The structured failure, when the job did not succeed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ConversionError>,
    /// Strongest validation level the output reached.
    pub validation_level: ValidationLevel,
    /// Last known progress of the job.
    pub progress: Progress,
}

impl ConversionResult {
    /// Creates a result for a job that is still in flight.
    pub fn in_progress(job_id: Uuid, status: JobStatus) -> Self {
        Self {
            job_id,
            status,
            source_name: None,
            output_path: None,
            engine_id: None,
            engine_version: None,
            warnings: Vec::new(),
            error: None,
            validation_level: ValidationLevel::Exists,
            progress: Progress::Indeterminate,
        }
    }

    /// Returns `true` when the job finished successfully.
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Succeeded | JobStatus::SucceededWithWarnings
        )
    }
}

/// Identifies an engine implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EngineId(String);

impl EngineId {
    /// Creates an engine identifier from a slug.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if !value.is_empty()
            && value.len() <= 64
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where an engine installation was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum EngineSource {
    /// Shipped with the application.
    Bundled,
    /// Found on the system.
    System,
}

impl EngineSource {
    /// Returns a short, stable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            EngineSource::Bundled => "Bundled",
            EngineSource::System => "System",
        }
    }
}

/// Availability of an engine on the current machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum EngineAvailability {
    /// The engine was detected and is usable.
    Available,
    /// The engine was not found on this machine.
    Missing,
    /// The engine was found but its version is outside the supported
    /// range.
    IncompatibleVersion,
}

impl EngineAvailability {
    /// Returns `true` when the engine can be used for a conversion.
    pub fn is_usable(self) -> bool {
        matches!(self, EngineAvailability::Available)
    }
}

/// Detected installation of a conversion engine.
///
/// `ARCHITECTURE.md` section 5 requires the selected engine and its
/// source to be visible to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    /// Identifier of the engine.
    pub engine_id: EngineId,
    /// Human readable engine name, for example `FFmpeg`.
    pub display_name: String,
    /// Whether the engine is usable right now.
    pub availability: EngineAvailability,
    /// Where the engine was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<EngineSource>,
    /// Exact detected version, when the engine was found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Platform specific installation guidance shown when the engine is
    /// missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation_hint: Option<String>,
    /// Reason the engine is not usable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ConversionError>,
}

impl EngineStatus {
    /// Creates a status describing an engine that was not detected.
    pub fn missing(
        engine_id: EngineId,
        display_name: impl Into<String>,
        reason: ConversionError,
    ) -> Self {
        Self {
            engine_id,
            display_name: display_name.into(),
            availability: EngineAvailability::Missing,
            source: None,
            version: None,
            installation_hint: None,
            reason: Some(reason),
        }
    }
}

/// Error returned when a conversion pair is not supported.
pub fn unsupported_pair(source: &FormatId, target: &FormatId) -> ConversionError {
    ConversionError::new(
        ConversionErrorCode::UnsupportedPair,
        format!("Conversion from {source} to {target} is not supported."),
    )
}

/// Returns the file extension that is used for a format.
///
/// The extension is a naming rule of the product, not a property of the
/// engine: `jpeg` is written as `.jpg`, which is what users expect.
pub fn preferred_extension(format: &FormatId) -> &str {
    match format.as_str() {
        "jpeg" => "jpg",
        other => other,
    }
}

/// Returns the file name of the converted output.
///
/// The name is derived from the source file, so that the result is
/// predictable for the user.
pub fn output_file_name(input: &std::path::Path, target: &FormatId) -> String {
    let stem = input
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "output".to_owned());

    format!("{stem}.{}", preferred_extension(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_levels_are_ordered() {
        assert!(ValidationLevel::Exists < ValidationLevel::NonEmptySignature);
        assert!(ValidationLevel::NonEmptySignature < ValidationLevel::Parsed);
        assert!(ValidationLevel::Parsed < ValidationLevel::Semantic);
    }

    #[test]
    fn validation_level_numbers_match_architecture_document() {
        assert_eq!(ValidationLevel::Exists.level(), 0);
        assert_eq!(ValidationLevel::NonEmptySignature.level(), 1);
        assert_eq!(ValidationLevel::Parsed.level(), 2);
        assert_eq!(ValidationLevel::Semantic.level(), 3);
    }

    #[test]
    fn default_overwrite_policy_is_skip() {
        assert_eq!(OverwritePolicy::default(), OverwritePolicy::Skip);
    }

    #[test]
    fn request_defaults_to_skip_policy() {
        let request = ConversionRequest::new(
            Uuid::nil(),
            "/tmp/a.pdf",
            FormatId::new("png").expect("valid"),
            "/tmp/out",
        );
        assert_eq!(request.overwrite_policy, OverwritePolicy::Skip);
    }

    #[test]
    fn engine_id_validates_slug() {
        assert!(EngineId::new("ffmpeg").is_some());
        assert!(EngineId::new("").is_none());
        assert!(EngineId::new("FFmpeg").is_none());
        assert!(EngineId::new("poppler utils").is_none());
    }

    #[test]
    fn availability_reports_usability() {
        assert!(EngineAvailability::Available.is_usable());
        assert!(!EngineAvailability::Missing.is_usable());
        assert!(!EngineAvailability::IncompatibleVersion.is_usable());
    }

    #[test]
    fn in_progress_result_has_no_output_and_no_error() {
        let result = ConversionResult::in_progress(Uuid::nil(), JobStatus::Queued);
        assert!(result.output_path.is_none());
        assert!(result.error.is_none());
        assert!(!result.is_success());
    }

    #[test]
    fn unsupported_pair_error_names_both_formats() {
        let source = FormatId::new("pdf").expect("valid");
        let target = FormatId::new("docx").expect("valid");
        let error = unsupported_pair(&source, &target);
        assert_eq!(error.code, ConversionErrorCode::UnsupportedPair);
        assert!(error.message.contains("pdf"));
        assert!(error.message.contains("docx"));
    }

    #[test]
    fn options_are_empty_by_default() {
        let options = ConversionOptions::default();
        assert!(options.is_empty());
        assert!(options.quality.is_none());
        assert!(options.background.is_none());
        options.validate().expect("empty options are valid");
    }

    #[test]
    fn quality_is_accepted_inside_the_range() {
        for quality in [1, 50, 92, 100] {
            ConversionOptions::new()
                .with_quality(quality)
                .validate()
                .expect("quality must be accepted");
        }
    }

    #[test]
    fn quality_outside_the_range_is_rejected() {
        for quality in [0, 101, 255] {
            let error = ConversionOptions::new()
                .with_quality(quality)
                .validate()
                .expect_err("quality must be rejected");
            assert_eq!(error.code, ConversionErrorCode::InvalidOptions);
            assert!(error.remediation.is_some());
        }
    }

    #[test]
    fn background_colour_accepts_names_and_hex_values() {
        for background in ["white", "black", "#ffffff", "#fff", "grey50"] {
            ConversionOptions::new()
                .with_background(background)
                .validate()
                .expect("colour must be accepted");
        }
    }

    #[test]
    fn background_colour_rejects_anything_else() {
        for background in ["", "white; rm -rf /", "rgb(0,0,0", "a b", "#ff f"] {
            let error = ConversionOptions::new()
                .with_background(background)
                .validate()
                .expect_err("colour must be rejected");
            assert_eq!(error.code, ConversionErrorCode::InvalidOptions);
        }
    }

    #[test]
    fn a_request_carries_its_options() {
        let request = ConversionRequest::new(
            Uuid::nil(),
            "/tmp/a.png",
            FormatId::new("jpg").expect("valid"),
            "/tmp/out",
        )
        .with_options(ConversionOptions::new().with_quality(80));

        assert_eq!(request.options.quality, Some(80));
        assert!(!request.options.is_empty());
    }
}
