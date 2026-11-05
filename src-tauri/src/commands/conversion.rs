//! Starting, inspecting and cancelling conversions.

use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use ufc_core::domain::conversion::{ConversionRequest, ConversionResult};
use ufc_core::domain::job::{JobStatus, Progress};
use ufc_engines::conversion::JobObserver;
use uuid::Uuid;

use crate::commands::app::CommandError;
use crate::services::jobs::JobManager;
use crate::services::state::AppState;

/// The event that carries a state change of a job.
pub const JOB_UPDATED_EVENT: &str = "job://updated";

/// Starts a conversion.
///
/// The conversion runs in the background: the command returns the job
/// immediately, and every state change is sent to the interface as a
/// `job://updated` event.
///
/// # Errors
/// Fails when the request itself is invalid. Failures during the
/// conversion are reported through the job result.
#[tauri::command]
pub async fn start_conversion(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ConversionRequest,
) -> Result<ConversionResult, CommandError> {
    let conversion = Arc::clone(state.conversion());
    let jobs = Arc::clone(state.jobs());

    let (job_id, token) = jobs.create(source_name(&request.input_path));

    let observer = EventObserver {
        app: app.clone(),
        jobs: Arc::clone(&jobs),
        job_id,
    };

    let job_id_for_task = job_id;
    let jobs_for_task = Arc::clone(&jobs);

    tauri::async_runtime::spawn_blocking(move || {
        let result = conversion.convert(&request, &token, &observer);

        jobs_for_task.finish(result.clone());

        let _ = app.emit(JOB_UPDATED_EVENT, &result);
    });

    Ok(jobs
        .get(job_id_for_task)
        .unwrap_or_else(|| ConversionResult::in_progress(job_id_for_task, JobStatus::Queued)))
}

/// Returns the state of a job.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn get_job(
    state: State<'_, AppState>,
    job_id: Uuid,
) -> Result<Option<ConversionResult>, CommandError> {
    Ok(state.jobs().get(job_id))
}

/// Returns the state of every job.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn list_jobs(state: State<'_, AppState>) -> Result<Vec<ConversionResult>, CommandError> {
    Ok(state.jobs().list())
}

/// Cancels a running job.
///
/// Returns `false` when the job is unknown or already finished.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, job_id: Uuid) -> Result<bool, CommandError> {
    Ok(state.jobs().cancel(job_id))
}

/// Reports the state changes of a job to the interface.
struct EventObserver {
    app: AppHandle,
    jobs: Arc<JobManager>,
    job_id: Uuid,
}

impl JobObserver for EventObserver {
    fn on_update(&self, status: JobStatus, progress: Progress) {
        let result = self.jobs.update(self.job_id, status, progress);
        let _ = self.app.emit(JOB_UPDATED_EVENT, &result);
    }
}

/// Returns the name of the source file, without its directory.
///
/// Only the name is kept, never the full path, so that the queue can
/// show what is being converted without storing the location of the
/// file (`ARCHITECTURE.md` section 12).
fn source_name(input_path: &str) -> Option<String> {
    std::path::Path::new(input_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}
