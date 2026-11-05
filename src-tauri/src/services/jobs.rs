//! The job manager.
//!
//! It keeps the state of the conversion jobs, so that the interface can
//! show what is happening and so that a job can be cancelled.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ufc_core::domain::conversion::ConversionResult;
use ufc_core::domain::job::{JobStatus, Progress};
use ufc_core::engines::adapter::CancellationSignal;
use uuid::Uuid;

/// A cancellation signal that the caller can trigger.
#[derive(Debug, Clone, Default)]
pub struct CancelToken {
    cancelled: Arc<AtomicBool>,
}

impl CancelToken {
    /// Creates a token that has not been triggered.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests the cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Returns `true` when the cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl CancellationSignal for CancelToken {
    fn is_cancelled(&self) -> bool {
        CancelToken::is_cancelled(self)
    }
}

/// The state of a running or finished job.
#[derive(Debug, Clone)]
struct JobEntry {
    result: ConversionResult,
    token: CancelToken,
}

/// Keeps the jobs of the application.
#[derive(Debug, Default)]
pub struct JobManager {
    jobs: Mutex<HashMap<Uuid, JobEntry>>,
}

impl JobManager {
    /// Creates a manager without any job.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new job and returns its identifier and its
    /// cancellation token.
    ///
    /// The name of the source file is shown in the queue while the job
    /// is still running, so that the user knows what is being
    /// converted.
    pub fn create(&self, source_name: Option<String>) -> (Uuid, CancelToken) {
        let job_id = Uuid::new_v4();
        let token = CancelToken::new();

        let mut result = ConversionResult::in_progress(job_id, JobStatus::Queued);
        result.source_name = source_name;

        self.lock().insert(
            job_id,
            JobEntry {
                result,
                token: token.clone(),
            },
        );

        (job_id, token)
    }

    /// Records a state change of a job.
    pub fn update(&self, job_id: Uuid, status: JobStatus, progress: Progress) -> ConversionResult {
        let mut jobs = self.lock();

        let Some(entry) = jobs.get_mut(&job_id) else {
            return ConversionResult::in_progress(job_id, status);
        };

        entry.result.status = status;
        entry.result.progress = progress;

        entry.result.clone()
    }

    /// Records the final result of a job.
    pub fn finish(&self, result: ConversionResult) {
        let mut jobs = self.lock();

        if let Some(entry) = jobs.get_mut(&result.job_id) {
            entry.result = result;
        } else {
            jobs.insert(
                result.job_id,
                JobEntry {
                    result,
                    token: CancelToken::new(),
                },
            );
        }
    }

    /// Returns the state of a job.
    pub fn get(&self, job_id: Uuid) -> Option<ConversionResult> {
        self.lock().get(&job_id).map(|entry| entry.result.clone())
    }

    /// Returns the state of every job, newest first.
    pub fn list(&self) -> Vec<ConversionResult> {
        let jobs = self.lock();

        let mut results: Vec<ConversionResult> =
            jobs.values().map(|entry| entry.result.clone()).collect();

        results.sort_by_key(|entry| std::cmp::Reverse(entry.job_id));

        results
    }

    /// Requests the cancellation of a job.
    ///
    /// Returns `false` when the job is unknown or already finished.
    pub fn cancel(&self, job_id: Uuid) -> bool {
        let jobs = self.lock();

        match jobs.get(&job_id) {
            Some(entry) if entry.result.status.is_active() => {
                entry.token.cancel();
                true
            }
            _ => false,
        }
    }

    /// Locks the job map, even when a previous holder panicked.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<Uuid, JobEntry>> {
        self.jobs.lock().unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_job_is_queued() {
        let manager = JobManager::new();
        let (job_id, _token) = manager.create(None);

        let result = manager.get(job_id).expect("the job exists");

        assert_eq!(result.status, JobStatus::Queued);
        assert!(result.status.is_active());
    }

    #[test]
    fn a_state_change_is_recorded() {
        let manager = JobManager::new();
        let (job_id, _token) = manager.create(None);

        manager.update(
            job_id,
            JobStatus::Running,
            Progress::Stage {
                name: "converting".to_owned(),
            },
        );

        let result = manager.get(job_id).expect("the job exists");

        assert_eq!(result.status, JobStatus::Running);
    }

    #[test]
    fn a_running_job_can_be_cancelled() {
        let manager = JobManager::new();
        let (job_id, token) = manager.create(None);

        assert!(manager.cancel(job_id));
        assert!(token.is_cancelled());
    }

    #[test]
    fn a_finished_job_cannot_be_cancelled() {
        let manager = JobManager::new();
        let (job_id, _token) = manager.create(None);

        manager.update(job_id, JobStatus::Succeeded, Progress::Indeterminate);

        assert!(!manager.cancel(job_id));
    }

    #[test]
    fn the_jobs_are_listed() {
        let manager = JobManager::new();
        manager.create(None);
        manager.create(None);

        assert_eq!(manager.list().len(), 2);
    }

    #[test]
    fn a_job_remembers_the_name_of_its_source_file() {
        let manager = JobManager::new();
        let (job_id, _token) = manager.create(Some("photo.png".to_owned()));

        let result = manager.get(job_id).expect("the job exists");

        assert_eq!(result.source_name.as_deref(), Some("photo.png"));
    }
}
