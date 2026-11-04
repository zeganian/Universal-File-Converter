//! Shared application state managed by Tauri.

use std::sync::Arc;

use ufc_core::prelude::*;

use crate::services::settings::SettingsService;

/// State shared by every Tauri command.
///
/// The state is immutable from the command layer's point of view: it
/// exposes the settings service and the capability registry. Mutating
/// settings goes through [`SettingsService::update`], which keeps the
/// write path in one place.
#[derive(Debug)]
pub struct AppState {
    settings: Arc<SettingsService>,
    registry: CapabilityRegistry,
}

impl AppState {
    /// Creates the initial application state.
    pub fn new(settings: Arc<SettingsService>) -> Self {
        Self {
            settings,
            // The registry is empty because no engine adapter is
            // implemented yet. It must stay empty until a real adapter,
            // an integration fixture and a documentation entry exist
            // (`CLAUDE.md` completion rule).
            registry: ufc_core::domain::registry::default_registry(),
        }
    }

    /// Returns the settings service.
    pub fn settings(&self) -> &SettingsService {
        &self.settings
    }

    /// Returns the capability registry.
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }
}
