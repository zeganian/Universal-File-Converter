//! Capability registry access.

use serde::Serialize;
use tauri::State;
use ufc_core::domain::registry::ConversionCapability;

use crate::commands::app::CommandError;
use crate::services::state::AppState;

/// A summary of the conversion capabilities known to the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySummary {
    /// Every registered pair.
    pub capabilities: Vec<ConversionCapability>,
    /// Number of pairs that may currently be offered.
    pub available_count: usize,
    /// Whether any conversion is available at all.
    pub has_available_conversions: bool,
}

/// Returns the conversion capability registry.
///
/// The frontend must derive the list of selectable target formats from
/// this response and must not maintain its own hardcoded list
/// (`SUPPORTED_FORMATS.md`, "Registry rules"). A pair is only offered
/// when it is implemented and its engine is available on this device.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn get_capabilities(state: State<'_, AppState>) -> Result<CapabilitySummary, CommandError> {
    let registry = state.conversion().registry();
    let available_count = registry.available_count();

    Ok(CapabilitySummary {
        capabilities: registry.all().to_vec(),
        available_count,
        has_available_conversions: available_count > 0,
    })
}
