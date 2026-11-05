//! Shared application state managed by Tauri.

use std::sync::Arc;

use ufc_engines::conversion::ConversionService;

use crate::services::jobs::JobManager;
use crate::services::settings::SettingsService;

/// State shared by every Tauri command.
#[derive(Debug)]
pub struct AppState {
    settings: Arc<SettingsService>,
    conversion: Arc<ConversionService>,
    jobs: Arc<JobManager>,
}

impl AppState {
    /// Creates the initial application state.
    ///
    /// The conversion service detects the engines of this device while
    /// it is created, so the capability registry already reflects the
    /// machine the application runs on.
    pub fn new(settings: Arc<SettingsService>) -> Self {
        Self {
            settings,
            conversion: Arc::new(ConversionService::new()),
            jobs: Arc::new(JobManager::new()),
        }
    }

    /// Returns the settings service.
    pub fn settings(&self) -> &SettingsService {
        &self.settings
    }

    /// Returns the conversion service.
    pub fn conversion(&self) -> &Arc<ConversionService> {
        &self.conversion
    }

    /// Returns the job manager.
    pub fn jobs(&self) -> &Arc<JobManager> {
        &self.jobs
    }
}
