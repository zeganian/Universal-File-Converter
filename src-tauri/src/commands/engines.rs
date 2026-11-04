//! Engine status reporting.

use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;
use ufc_core::prelude::*;

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
/// No engine adapter is implemented in this milestone, therefore the
/// report is empty and the UI shows that no engine has been detected
/// yet. The application never claims an engine is present when it is
/// not (`PRD.md` section 6, "Explicit dependencies").
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn get_engine_status(state: State<'_, AppState>) -> Result<EngineReport, CommandError> {
    let engines = known_engines(state.registry());

    Ok(EngineReport {
        available_count: engines
            .iter()
            .filter(|engine| engine.availability.is_usable())
            .count(),
        engines,
    })
}

/// Builds the engine status list from the capability registry.
///
/// Every engine referenced by a registered pair is reported. Engines
/// that are referenced by no pair are not reported at all, because the
/// application has no knowledge of them yet.
fn known_engines(registry: &CapabilityRegistry) -> Vec<EngineStatus> {
    let mut availability: BTreeMap<EngineId, EngineAvailability> = BTreeMap::new();

    for capability in registry.all() {
        availability
            .entry(capability.engine_id.clone())
            .or_insert(EngineAvailability::Missing);
    }

    availability
        .into_iter()
        .map(|(engine_id, engine_availability)| EngineStatus {
            display_name: engine_id.as_str().to_owned(),
            engine_id,
            availability: engine_availability,
            source: None,
            version: None,
            installation_hint: None,
            reason: Some(
                ConversionError::new(
                    ConversionErrorCode::MissingDependency,
                    "No engine detection is implemented in this build.",
                )
                .with_remediation("Engine detection will be available in a later milestone."),
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ufc_core::domain::registry::{ConversionCapability, PairStatus};

    #[test]
    fn empty_registry_reports_no_engines() {
        let registry = CapabilityRegistry::empty();
        let engines = known_engines(&registry);
        assert!(engines.is_empty());
    }

    #[test]
    fn referenced_engine_is_reported_as_missing() {
        let mut capability = ConversionCapability::planned(
            FormatId::new("png").expect("valid"),
            FormatId::new("pdf").expect("valid"),
            EngineId::new("imagemagick").expect("valid"),
        );
        capability.status = PairStatus::Stable;

        let registry = CapabilityRegistry::new(vec![capability]);
        let engines = known_engines(&registry);

        assert_eq!(engines.len(), 1);
        assert_eq!(engines[0].availability, EngineAvailability::Missing);
        assert!(!engines[0].availability.is_usable());
        assert!(engines[0].reason.is_some());
    }
}
