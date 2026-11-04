//! Conversion request and result models.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::domain::error::{ConversionError, ConversionErrorCode};
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
}

impl ConversionRequest {
    /// Creates a request with the `Skip` overwrite policy.
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
        }
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
}
