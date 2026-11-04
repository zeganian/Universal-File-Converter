//! Structured error model for conversions.
//!
//! Every error carries a stable machine readable [`ConversionErrorCode`]
//! so that the frontend can branch on the cause without parsing human
//! readable text, plus an optional user facing message and an optional
//! technical detail that is only shown on request
//! (`PRD.md` FR-005, FR-012; `DESIGN.md` section 9).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Stable machine readable classification of a conversion failure.
///
/// The discriminant values are part of the public contract between the
/// Rust backend and the TypeScript frontend and must not change
/// without a migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ConversionErrorCode {
    /// The requested source-to-target pair is not supported.
    UnsupportedPair,
    /// A required conversion engine is not installed or not detected.
    MissingDependency,
    /// The installed engine version is outside the supported range.
    IncompatibleEngineVersion,
    /// The source path does not exist, is not a regular file, or is not
    /// readable.
    InvalidSource,
    /// The source file could not be identified reliably.
    UndetectedFormat,
    /// The source file extension and its content signature disagree.
    FormatMismatch,
    /// The requested conversion options are invalid.
    InvalidOptions,
    /// The output directory is missing or not writable.
    InvalidOutputLocation,
    /// The output file already exists and the policy forbids overwriting.
    OutputExists,
    /// The engine process exited with a non-zero status.
    EngineFailed,
    /// The engine produced no output at all.
    MissingOutput,
    /// The produced output did not pass validation.
    OutputValidationFailed,
    /// The job was cancelled by the user.
    Cancelled,
    /// The engine exceeded the configured time limit.
    Timeout,
    /// There is not enough free disk space.
    InsufficientDiskSpace,
    /// The input could not be read.
    IoError,
    /// The engine could not be started.
    EngineStartFailed,
    /// The application hit an internal invariant.
    Internal,
}

impl ConversionErrorCode {
    /// Returns a short, stable identifier suitable for logs.
    pub fn as_str(self) -> &'static str {
        match self {
            ConversionErrorCode::UnsupportedPair => "unsupported_pair",
            ConversionErrorCode::MissingDependency => "missing_dependency",
            ConversionErrorCode::IncompatibleEngineVersion => "incompatible_engine_version",
            ConversionErrorCode::InvalidSource => "invalid_source",
            ConversionErrorCode::UndetectedFormat => "undetected_format",
            ConversionErrorCode::FormatMismatch => "format_mismatch",
            ConversionErrorCode::InvalidOptions => "invalid_options",
            ConversionErrorCode::InvalidOutputLocation => "invalid_output_location",
            ConversionErrorCode::OutputExists => "output_exists",
            ConversionErrorCode::EngineFailed => "engine_failed",
            ConversionErrorCode::MissingOutput => "missing_output",
            ConversionErrorCode::OutputValidationFailed => "output_validation_failed",
            ConversionErrorCode::Cancelled => "cancelled",
            ConversionErrorCode::Timeout => "timeout",
            ConversionErrorCode::InsufficientDiskSpace => "insufficient_disk_space",
            ConversionErrorCode::IoError => "io_error",
            ConversionErrorCode::EngineStartFailed => "engine_start_failed",
            ConversionErrorCode::Internal => "internal",
        }
    }
}

/// A structured conversion error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConversionError {
    /// Machine readable classification.
    pub code: ConversionErrorCode,
    /// Concise, user facing summary. Never contains file contents.
    pub message: String,
    /// Optional actionable hint, for example how to install a missing
    /// engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Optional technical detail shown only in the expandable details
    /// panel. Redacted before it leaves the backend.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ConversionError {
    /// Creates an error with a code and a user facing message.
    pub fn new(code: ConversionErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            remediation: None,
            detail: None,
        }
    }

    /// Attaches an actionable remediation hint.
    #[must_use]
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    /// Attaches a technical detail.
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConversionError {}

/// Convenience alias for fallible domain operations.
pub type DomainResult<T> = Result<T, ConversionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_identifiers_are_stable_and_unique() {
        let codes = [
            ConversionErrorCode::UnsupportedPair,
            ConversionErrorCode::MissingDependency,
            ConversionErrorCode::IncompatibleEngineVersion,
            ConversionErrorCode::InvalidSource,
            ConversionErrorCode::UndetectedFormat,
            ConversionErrorCode::FormatMismatch,
            ConversionErrorCode::InvalidOptions,
            ConversionErrorCode::InvalidOutputLocation,
            ConversionErrorCode::OutputExists,
            ConversionErrorCode::EngineFailed,
            ConversionErrorCode::MissingOutput,
            ConversionErrorCode::OutputValidationFailed,
            ConversionErrorCode::Cancelled,
            ConversionErrorCode::Timeout,
            ConversionErrorCode::InsufficientDiskSpace,
            ConversionErrorCode::IoError,
            ConversionErrorCode::EngineStartFailed,
            ConversionErrorCode::Internal,
        ];

        let mut seen = std::collections::HashSet::new();
        for code in codes {
            assert!(
                seen.insert(code.as_str()),
                "duplicate code: {}",
                code.as_str()
            );
        }
        assert_eq!(seen.len(), codes.len());
    }

    #[test]
    fn optional_fields_are_omitted_when_absent() {
        let error = ConversionError::new(ConversionErrorCode::Internal, "boom");
        let json = serde_json::to_value(&error).expect("serializable");
        assert!(json.get("remediation").is_none());
        assert!(json.get("detail").is_none());
        assert_eq!(json["code"], "internal");
    }

    #[test]
    fn builders_attach_optional_fields() {
        let error = ConversionError::new(ConversionErrorCode::MissingDependency, "ffmpeg missing")
            .with_remediation("Install FFmpeg")
            .with_detail("probe failed: exit code 1");
        assert_eq!(error.remediation.as_deref(), Some("Install FFmpeg"));
        assert_eq!(error.detail.as_deref(), Some("probe failed: exit code 1"));
    }
}
