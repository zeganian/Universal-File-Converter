//! Job status, progress and the job state machine.
//!
//! The state machine is the single source of truth for job lifecycle
//! rules. It is deliberately free of any I/O so that it can be unit
//! tested exhaustively (see `tests/job_state.rs`).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Lifecycle state of a conversion job.
///
/// Mirrors the lifecycle defined in `PRD.md` section 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum JobStatus {
    /// Job accepted by the queue and waiting for a worker slot.
    Queued,
    /// Source file and requested conversion pair are being verified.
    Validating,
    /// The required engine is not available yet.
    WaitingForEngine,
    /// The engine process is running.
    Running,
    /// The produced output is being verified.
    ValidatingOutput,
    /// Conversion finished and the output passed validation.
    Succeeded,
    /// Conversion finished, output is usable, but warnings were raised.
    SucceededWithWarnings,
    /// Conversion failed.
    Failed,
    /// Conversion was cancelled by the user.
    Cancelled,
}

impl JobStatus {
    /// Returns `true` when the job can no longer change state.
    ///
    /// Terminal states are `Succeeded`, `SucceededWithWarnings`, `Failed`
    /// and `Cancelled`. A job must never leave a terminal state
    /// (`PRD.md` section 10).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobStatus::Succeeded
                | JobStatus::SucceededWithWarnings
                | JobStatus::Failed
                | JobStatus::Cancelled
        )
    }

    /// Returns `true` when the job is still in flight.
    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }

    /// Returns the user facing label for this status.
    ///
    /// The labels follow `DESIGN.md` section 8. Labels never rely on
    /// colour alone to convey meaning.
    pub fn label(self) -> &'static str {
        match self {
            JobStatus::Queued => "Queued",
            JobStatus::Validating => "Checking file",
            JobStatus::WaitingForEngine => "Waiting for engine",
            JobStatus::Running => "Converting",
            JobStatus::ValidatingOutput => "Validating output",
            JobStatus::Succeeded => "Completed",
            JobStatus::SucceededWithWarnings => "Completed with warnings",
            JobStatus::Failed => "Failed",
            JobStatus::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Progress information for a running job.
///
/// The model distinguishes determinate progress from indeterminate
/// progress so that the UI never animates a fabricated percentage
/// (`ARCHITECTURE.md` section 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum Progress {
    /// The engine reported a trustworthy percentage.
    Determinate {
        /// Completion percentage in the range `0..=100`.
        percent: u8,
    },
    /// Work is underway but no reliable percentage is available.
    Indeterminate,
    /// The job is currently performing a named phase.
    Stage {
        /// Human readable phase name.
        name: String,
    },
}

impl Progress {
    /// Creates determinate progress, clamping `percent` into `0..=100`.
    pub fn determinate(percent: u8) -> Self {
        Progress::Determinate {
            percent: percent.min(100),
        }
    }

    /// Returns the determinate percentage, if any.
    pub fn percent(&self) -> Option<u8> {
        match self {
            Progress::Determinate { percent } => Some(*percent),
            _ => None,
        }
    }
}

/// Error produced by an illegal job state transition.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidTransition {
    /// A terminal job was asked to change state again.
    #[error("cannot transition from terminal state `{from}`")]
    FromTerminal {
        /// The terminal state the job was in.
        from: JobStatus,
    },
    /// The requested transition is not part of the state machine.
    #[error("cannot transition from `{from}` to `{to}`")]
    NotAllowed {
        /// Current state.
        from: JobStatus,
        /// Requested state.
        to: JobStatus,
    },
}

/// Determines whether a job may move from `from` to `to`.
///
/// Transition table (see `ARCHITECTURE.md` section 10):
///
/// ```text
/// Queued            -> Validating | Cancelled | Failed
/// Validating        -> WaitingForEngine | Running | Failed | Cancelled
/// WaitingForEngine  -> Running | Failed | Cancelled
/// Running           -> ValidatingOutput | Failed | Cancelled
/// ValidatingOutput  -> Succeeded | SucceededWithWarnings | Failed | Cancelled
/// <terminal>        -> (no transitions)
/// ```
pub fn can_transition(from: JobStatus, to: JobStatus) -> bool {
    use JobStatus::*;

    if from.is_terminal() {
        return false;
    }

    matches!(
        (from, to),
        (Queued, Validating)
            | (Queued, Cancelled)
            | (Queued, Failed)
            | (Validating, WaitingForEngine)
            | (Validating, Running)
            | (Validating, Failed)
            | (Validating, Cancelled)
            | (WaitingForEngine, Running)
            | (WaitingForEngine, Failed)
            | (WaitingForEngine, Cancelled)
            | (Running, ValidatingOutput)
            | (Running, Failed)
            | (Running, Cancelled)
            | (ValidatingOutput, Succeeded)
            | (ValidatingOutput, SucceededWithWarnings)
            | (ValidatingOutput, Failed)
            | (ValidatingOutput, Cancelled)
    )
}

/// Validates a job state transition.
///
/// Returns [`InvalidTransition::FromTerminal`] when the job already
/// reached a terminal state, and
/// [`InvalidTransition::NotAllowed`] when the transition is simply not
/// part of the state machine.
pub fn validate_transition(from: JobStatus, to: JobStatus) -> Result<(), InvalidTransition> {
    if from.is_terminal() {
        return Err(InvalidTransition::FromTerminal { from });
    }
    if can_transition(from, to) {
        Ok(())
    } else {
        Err(InvalidTransition::NotAllowed { from, to })
    }
}

/// Returns every status a job in `from` may transition to.
pub fn allowed_transitions(from: JobStatus) -> Vec<JobStatus> {
    use JobStatus::*;

    const ALL: [JobStatus; 9] = [
        Queued,
        Validating,
        WaitingForEngine,
        Running,
        ValidatingOutput,
        Succeeded,
        SucceededWithWarnings,
        Failed,
        Cancelled,
    ];

    if from.is_terminal() {
        return Vec::new();
    }
    ALL.into_iter()
        .filter(|to| can_transition(from, *to))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_states_are_recognised() {
        assert!(JobStatus::Succeeded.is_terminal());
        assert!(JobStatus::SucceededWithWarnings.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
    }

    #[test]
    fn happy_path_is_allowed() {
        assert!(can_transition(JobStatus::Queued, JobStatus::Validating));
        assert!(can_transition(JobStatus::Validating, JobStatus::Running));
        assert!(can_transition(
            JobStatus::Running,
            JobStatus::ValidatingOutput
        ));
        assert!(can_transition(
            JobStatus::ValidatingOutput,
            JobStatus::Succeeded
        ));
    }

    #[test]
    fn engine_waiting_path_is_allowed() {
        assert!(can_transition(
            JobStatus::Validating,
            JobStatus::WaitingForEngine
        ));
        assert!(can_transition(
            JobStatus::WaitingForEngine,
            JobStatus::Running
        ));
    }

    #[test]
    fn skipping_validation_is_rejected() {
        assert!(!can_transition(JobStatus::Queued, JobStatus::Running));
        assert!(!can_transition(JobStatus::Queued, JobStatus::Succeeded));
    }

    #[test]
    fn terminal_state_cannot_transition_again() {
        let err = validate_transition(JobStatus::Succeeded, JobStatus::Running)
            .expect_err("terminal must be locked");
        assert_eq!(
            err,
            InvalidTransition::FromTerminal {
                from: JobStatus::Succeeded
            }
        );
        assert!(allowed_transitions(JobStatus::Succeeded).is_empty());
    }

    #[test]
    fn invalid_transition_reports_both_states() {
        let err = validate_transition(JobStatus::Queued, JobStatus::Running)
            .expect_err("must be rejected");
        assert_eq!(
            err,
            InvalidTransition::NotAllowed {
                from: JobStatus::Queued,
                to: JobStatus::Running,
            }
        );
    }

    #[test]
    fn every_non_terminal_state_can_be_cancelled() {
        for status in [
            JobStatus::Queued,
            JobStatus::Validating,
            JobStatus::WaitingForEngine,
            JobStatus::Running,
            JobStatus::ValidatingOutput,
        ] {
            assert!(
                can_transition(status, JobStatus::Cancelled),
                "{status:?} must be cancellable"
            );
        }
    }

    #[test]
    fn every_non_terminal_state_can_fail() {
        for status in [
            JobStatus::Queued,
            JobStatus::Validating,
            JobStatus::WaitingForEngine,
            JobStatus::Running,
            JobStatus::ValidatingOutput,
        ] {
            assert!(
                can_transition(status, JobStatus::Failed),
                "{status:?} must be able to fail"
            );
        }
    }

    #[test]
    fn determinate_progress_is_clamped() {
        assert_eq!(
            Progress::determinate(50),
            Progress::Determinate { percent: 50 }
        );
        assert_eq!(
            Progress::determinate(255),
            Progress::Determinate { percent: 100 }
        );
    }

    #[test]
    fn indeterminate_progress_has_no_percent() {
        assert_eq!(Progress::Indeterminate.percent(), None);
        assert_eq!(
            Progress::Stage {
                name: "converting".into()
            }
            .percent(),
            None
        );
    }

    #[test]
    fn labels_match_design_document() {
        assert_eq!(JobStatus::Validating.label(), "Checking file");
        assert_eq!(JobStatus::Running.label(), "Converting");
        assert_eq!(JobStatus::Succeeded.label(), "Completed");
        assert_eq!(
            JobStatus::SucceededWithWarnings.label(),
            "Completed with warnings"
        );
    }
}
