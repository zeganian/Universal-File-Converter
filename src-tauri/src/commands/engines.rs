//! Engine status reporting.

use serde::Serialize;
use tauri::State;
use ufc_core::domain::conversion::EngineStatus;

use crate::commands::app::CommandError;
use crate::services::state::AppState;

/// A report about the engines known to the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineReport {
    /// Status of every known engine.
    pub engines: Vec<EngineStatus>,
    /// Number of engines that are usable right now.
    pub available_count: usize,
}

/// Returns the status of every known conversion engine.
///
/// The engines are detected on this device: the report contains the
/// exact version and the source of every installation, or the reason
/// why the engine cannot be used (`PRD.md` FR-004, `DESIGN.md` section
/// 9).
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn get_engine_status(state: State<'_, AppState>) -> Result<EngineReport, CommandError> {
    let engines = state.conversion().engine_status();

    Ok(EngineReport {
        available_count: engines
            .iter()
            .filter(|engine| engine.availability.is_usable())
            .count(),
        engines,
    })
}

/// Detects the engines again and returns the new status.
///
/// The user can call this after installing an engine, without having to
/// restart the application.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn refresh_engine_status(state: State<'_, AppState>) -> Result<EngineReport, CommandError> {
    state.conversion().refresh();

    let engines = state.conversion().engine_status();

    Ok(EngineReport {
        available_count: engines
            .iter()
            .filter(|engine| engine.availability.is_usable())
            .count(),
        engines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ufc_core::domain::conversion::{EngineAvailability, EngineId};
    use ufc_core::domain::error::{ConversionError, ConversionErrorCode};

    #[test]
    fn a_missing_engine_carries_a_reason_and_a_hint() {
        let engine = EngineStatus {
            engine_id: EngineId::new("imagemagick").expect("valid"),
            display_name: "ImageMagick".to_owned(),
            availability: EngineAvailability::Missing,
            source: None,
            version: None,
            installation_hint: Some("Install ImageMagick 7.".to_owned()),
            reason: Some(
                ConversionError::new(
                    ConversionErrorCode::MissingDependency,
                    "ImageMagick was not found on this device.",
                )
                .with_remediation("Install ImageMagick 7."),
            ),
        };

        assert!(!engine.availability.is_usable());
        assert!(engine.reason.is_some());
        assert!(engine.installation_hint.is_some());
    }
}
