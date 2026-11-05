//! Process execution infrastructure.
//!
//! Every external engine is started through [`ProcessRunner`]. The
//! runner exists to make the rules of `SECURITY.md` section 4 and
//! `RULES.md` section 5 impossible to bypass:
//!
//! - an executable path and an argument vector, never a shell command
//!   string,
//! - a controlled working directory and a minimal environment,
//! - a timeout that kills the process,
//! - a cancellation signal that kills the process,
//! - a size limit on the captured output,
//! - redaction of the paths that belong to the job.

pub mod runner;

pub use runner::{ProcessOutcome, ProcessRequest, ProcessRunner};
