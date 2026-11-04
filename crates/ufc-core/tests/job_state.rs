//! Integration tests for the job state machine.
//!
//! The unit tests inside `src/domain/job.rs` cover the transition
//! table exhaustively. These tests exercise the same rules through the
//! public crate API, the way an application service would use them.

use ufc_core::domain::job::{allowed_transitions, can_transition, validate_transition};
use ufc_core::domain::job::{InvalidTransition, JobStatus};

/// The happy path a successful conversion walks through.
const HAPPY_PATH: [JobStatus; 6] = [
    JobStatus::Queued,
    JobStatus::Validating,
    JobStatus::WaitingForEngine,
    JobStatus::Running,
    JobStatus::ValidatingOutput,
    JobStatus::Succeeded,
];

#[test]
fn a_job_can_walk_the_full_lifecycle() {
    for window in HAPPY_PATH.windows(2) {
        let (from, to) = (window[0], window[1]);
        assert!(
            can_transition(from, to),
            "expected {from:?} -> {to:?} to be allowed"
        );
        validate_transition(from, to).expect("transition must be valid");
    }
}

#[test]
fn the_lifecycle_ends_in_a_terminal_state() {
    let last = HAPPY_PATH[HAPPY_PATH.len() - 1];
    assert!(last.is_terminal());
    assert!(!last.is_active());
}

#[test]
fn every_terminal_state_locks_the_job() {
    let terminal = [
        JobStatus::Succeeded,
        JobStatus::SucceededWithWarnings,
        JobStatus::Failed,
        JobStatus::Cancelled,
    ];

    for state in terminal {
        assert!(state.is_terminal(), "{state:?} must be terminal");
        assert!(allowed_transitions(state).is_empty());
        assert!(!can_transition(state, JobStatus::Running));

        let error = validate_transition(state, JobStatus::Running)
            .expect_err("a terminal job must not change state");
        assert_eq!(error, InvalidTransition::FromTerminal { from: state });
    }
}

#[test]
fn a_job_may_not_skip_the_validation_step() {
    assert!(!can_transition(JobStatus::Queued, JobStatus::Running));
    assert!(!can_transition(JobStatus::Queued, JobStatus::Succeeded));
    assert!(!can_transition(JobStatus::Validating, JobStatus::Succeeded));
}

#[test]
fn a_running_job_may_not_return_to_running() {
    assert!(!can_transition(JobStatus::Running, JobStatus::Running));
    assert!(!can_transition(JobStatus::Running, JobStatus::Queued));
    assert!(!can_transition(
        JobStatus::Running,
        JobStatus::WaitingForEngine
    ));
}

#[test]
fn every_active_state_can_be_cancelled_and_can_fail() {
    let active = [
        JobStatus::Queued,
        JobStatus::Validating,
        JobStatus::WaitingForEngine,
        JobStatus::Running,
        JobStatus::ValidatingOutput,
    ];

    for state in active {
        assert!(state.is_active(), "{state:?} must be active");
        assert!(can_transition(state, JobStatus::Cancelled));
        assert!(can_transition(state, JobStatus::Failed));
    }
}

#[test]
fn allowed_transitions_match_the_documented_table() {
    // The order follows the declaration order of `JobStatus`, so that
    // the UI can render the options in a stable, predictable sequence.
    assert_eq!(
        allowed_transitions(JobStatus::Queued),
        vec![
            JobStatus::Validating,
            JobStatus::Failed,
            JobStatus::Cancelled
        ]
    );
    assert_eq!(
        allowed_transitions(JobStatus::Validating),
        vec![
            JobStatus::WaitingForEngine,
            JobStatus::Running,
            JobStatus::Failed,
            JobStatus::Cancelled
        ]
    );
    assert_eq!(
        allowed_transitions(JobStatus::WaitingForEngine),
        vec![JobStatus::Running, JobStatus::Failed, JobStatus::Cancelled]
    );
    assert_eq!(
        allowed_transitions(JobStatus::Running),
        vec![
            JobStatus::ValidatingOutput,
            JobStatus::Failed,
            JobStatus::Cancelled
        ]
    );
    assert_eq!(
        allowed_transitions(JobStatus::ValidatingOutput),
        vec![
            JobStatus::Succeeded,
            JobStatus::SucceededWithWarnings,
            JobStatus::Failed,
            JobStatus::Cancelled
        ]
    );
}

#[test]
fn status_labels_are_stable() {
    // The frontend mirrors these labels in `src/features/queue/jobStatus.ts`.
    assert_eq!(JobStatus::Queued.label(), "Queued");
    assert_eq!(JobStatus::Validating.label(), "Checking file");
    assert_eq!(JobStatus::WaitingForEngine.label(), "Waiting for engine");
    assert_eq!(JobStatus::Running.label(), "Converting");
    assert_eq!(JobStatus::ValidatingOutput.label(), "Validating output");
    assert_eq!(JobStatus::Succeeded.label(), "Completed");
    assert_eq!(
        JobStatus::SucceededWithWarnings.label(),
        "Completed with warnings"
    );
    assert_eq!(JobStatus::Failed.label(), "Failed");
    assert_eq!(JobStatus::Cancelled.label(), "Cancelled");
}
