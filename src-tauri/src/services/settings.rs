//! Persistent application settings.
//!
//! Settings are stored as a JSON document in the platform application
//! configuration directory. Only a small, non-sensitive set of values is
//! stored: no file contents, no paths from the user's documents and no
//! credentials (`SECURITY.md` section 2).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

/// Theme preference for the user interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    /// Follow the operating system preference.
    #[default]
    System,
    /// Always use the light theme.
    Light,
    /// Always use the dark theme.
    Dark,
}

/// Collision handling applied when an output file already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CollisionPolicy {
    /// Keep the existing file and report the conflict.
    #[default]
    Skip,
    /// Write to a new file name.
    Rename,
    /// Replace the existing file after an explicit confirmation.
    Overwrite,
}

/// User configurable application settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Theme preference.
    pub theme: Theme,
    /// Default output directory. Empty means "ask every time".
    pub default_output_directory: String,
    /// Policy applied to an already existing output file.
    pub collision_policy: CollisionPolicy,
    /// Maximum number of conversions running at the same time.
    pub max_concurrency: u8,
    /// Whether conversion history is recorded locally.
    ///
    /// History is disabled by default so that no source path is stored
    /// without the user asking for it (`PRD.md` FR-011).
    pub history_enabled: bool,
    /// Number of days history entries are kept.
    pub history_retention_days: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            default_output_directory: String::new(),
            collision_policy: CollisionPolicy::Skip,
            // Conservative default: memory hungry engines should not run
            // in parallel by default (`PRD.md` section 9).
            max_concurrency: 2,
            history_enabled: false,
            history_retention_days: 30,
        }
    }
}

/// Service that loads and stores [`Settings`].
#[derive(Debug)]
pub struct SettingsService {
    path: PathBuf,
    settings: Settings,
}

impl SettingsService {
    /// Loads the settings from disk, falling back to defaults when the
    /// file is missing or unreadable.
    ///
    /// A malformed settings file must never prevent the application from
    /// starting; it is replaced by the defaults instead.
    pub fn load(app: &tauri::AppHandle) -> Result<Self, SettingsError> {
        let path = settings_path(app);
        let settings = match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Settings::default(),
            Err(error) => {
                return Err(SettingsError::Read {
                    path: path.clone(),
                    source: error,
                })
            }
        };

        Ok(Self { path, settings })
    }

    /// Creates a service backed by an explicit file path.
    ///
    /// Used by tests to avoid touching the real configuration directory.
    pub fn with_path(path: PathBuf, settings: Settings) -> Self {
        Self { path, settings }
    }

    /// Returns the current settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Replaces the current settings and persists them.
    pub fn update(&mut self, settings: Settings) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| SettingsError::Write {
                path: self.path.clone(),
                source,
            })?;
        }

        let encoded = serde_json::to_vec_pretty(&settings)
            .map_err(|source| SettingsError::Serialize { source })?;

        std::fs::write(&self.path, encoded).map_err(|source| SettingsError::Write {
            path: self.path.clone(),
            source,
        })?;

        self.settings = settings;
        Ok(())
    }
}

fn settings_path(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("settings.json")
}

/// Errors produced by the settings service.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// The settings file could not be read.
    #[error("could not read settings from {path}: {source}")]
    Read {
        /// Path of the settings file.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The settings file could not be written.
    #[error("could not write settings to {path}: {source}")]
    Write {
        /// Path of the settings file.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The settings could not be encoded.
    #[error("could not encode settings: {source}")]
    Serialize {
        /// Underlying encoding error.
        #[source]
        source: serde_json::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let settings = Settings::default();
        assert_eq!(settings.theme, Theme::System);
        assert_eq!(settings.collision_policy, CollisionPolicy::Skip);
        assert_eq!(settings.max_concurrency, 2);
        assert!(!settings.history_enabled, "history must be off by default");
    }

    #[test]
    fn settings_round_trip_through_json() {
        let settings = Settings {
            theme: Theme::Dark,
            default_output_directory: "/tmp/out".into(),
            collision_policy: CollisionPolicy::Rename,
            max_concurrency: 4,
            history_enabled: true,
            history_retention_days: 7,
        };

        let encoded = serde_json::to_string(&settings).expect("encodes");
        let decoded: Settings = serde_json::from_str(&encoded).expect("decodes");
        assert_eq!(settings, decoded);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let decoded: Settings = serde_json::from_str("{}").expect("decodes");
        assert_eq!(decoded.max_concurrency, Settings::default().max_concurrency);
        assert!(!decoded.history_enabled);
    }

    #[test]
    fn update_persists_to_disk() {
        let dir = std::env::temp_dir().join(format!("ufc-settings-{}", uuid::Uuid::new_v4()));
        let path = dir.join("nested").join("settings.json");
        let mut service = SettingsService::with_path(path.clone(), Settings::default());

        let updated = Settings {
            max_concurrency: 3,
            ..Settings::default()
        };
        service.update(updated.clone()).expect("update succeeds");

        let written = std::fs::read_to_string(&path).expect("file written");
        let decoded: Settings = serde_json::from_str(&written).expect("decodes");
        assert_eq!(decoded, updated);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
