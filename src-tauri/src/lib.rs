//! Universal File Converter — Tauri application shell.
//!
//! This crate is the desktop shell: it owns the Tauri runtime, the
//! typed command boundary and the wiring to the framework-agnostic
//! domain logic in `ufc-core`.
//!
//! All business rules live in `ufc-core`. Nothing in this crate may
//! re-implement them.

#![forbid(unsafe_code)]

pub mod commands;
pub mod domain;
pub mod engines;
pub mod services;

use std::sync::Arc;

use tauri::Manager;

use services::settings::SettingsService;
use services::state::AppState;

/// Builds and runs the Tauri application.
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("UFC_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = SettingsService::load(app.handle())?;
            let state = AppState::new(Arc::new(settings));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app::get_app_info,
            commands::capabilities::get_capabilities,
            commands::engines::get_engine_status,
            commands::engines::refresh_engine_status,
            commands::conversion::start_conversion,
            commands::conversion::get_job,
            commands::conversion::list_jobs,
            commands::conversion::cancel_job,
            commands::files::inspect_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Universal File Converter");
}
