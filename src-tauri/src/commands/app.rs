//! General application information.

use serde::Serialize;
use tauri::State;
use ufc_core::prelude::*;

use crate::services::state::AppState;

/// Version information about the running application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Name of the application.
    pub name: String,
    /// Semantic version of the application.
    pub version: String,
    /// Version of the domain core crate.
    pub core_version: String,
    /// Whether at least one conversion pair is currently available.
    pub has_available_conversions: bool,
}

/// Returns basic information about the running application.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, CommandError> {
    Ok(AppInfo {
        name: "Universal File Converter".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        core_version: ufc_core::CORE_VERSION.to_owned(),
        has_available_conversions: state.conversion().registry().available_count() > 0,
    })
}

/// Error returned by a Tauri command.
///
/// The frontend receives a stable, structured payload rather than an
/// opaque string (`PRD.md` FR-005).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Machine readable classification.
    pub code: ConversionErrorCode,
    /// Concise, user facing summary.
    pub message: String,
    /// Optional actionable hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Optional technical detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl From<ConversionError> for CommandError {
    fn from(error: ConversionError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            remediation: error.remediation,
            detail: error.detail,
        }
    }
}

impl From<CommandError> for ConversionError {
    fn from(error: CommandError) -> Self {
        let mut conversion_error = ConversionError::new(error.code, error.message);
        if let Some(remediation) = error.remediation {
            conversion_error = conversion_error.with_remediation(remediation);
        }
        if let Some(detail) = error.detail {
            conversion_error = conversion_error.with_detail(detail);
        }
        conversion_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_error_round_trips_through_conversion_error() {
        let original =
            ConversionError::new(ConversionErrorCode::MissingDependency, "ffmpeg missing")
                .with_remediation("Install FFmpeg")
                .with_detail("probe exited with 1");

        let command_error = CommandError::from(original.clone());
        assert_eq!(command_error.code, ConversionErrorCode::MissingDependency);
        assert_eq!(command_error.remediation.as_deref(), Some("Install FFmpeg"));

        let restored = ConversionError::from(command_error);
        assert_eq!(restored, original);
    }
}
