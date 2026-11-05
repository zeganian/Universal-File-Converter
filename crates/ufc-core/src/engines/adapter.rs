//! The engine adapter contract.
//!
//! Every conversion engine implements [`EngineAdapter`]. The contract
//! mirrors `ARCHITECTURE.md` section 4.
//!
//! No adapter is implemented in this milestone. The trait exists so that
//! the boundary between the application and a future engine is
//! explicit and reviewable before any engine code is written.

use std::path::PathBuf;

use crate::domain::conversion::{ConversionOptions, EngineId, EngineStatus, OverwritePolicy};
use crate::domain::error::DomainResult;
use crate::domain::format::FormatId;
use crate::domain::job::Progress;
use crate::domain::registry::ConversionCapability;

/// A fully resolved engine invocation.
///
/// The invocation is a program path plus an argument vector. It is
/// deliberately *not* a command string: `RULES.md` section 5 and
/// `SECURITY.md` section 4 forbid building shell command strings from
/// user controlled data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInvocation {
    /// Absolute path of the executable.
    pub program: PathBuf,
    /// Arguments passed to the executable, already separated.
    pub args: Vec<String>,
    /// Working directory for the process.
    pub working_directory: PathBuf,
}

impl EngineInvocation {
    /// Creates an invocation.
    pub fn new(
        program: impl Into<PathBuf>,
        args: Vec<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            program: program.into(),
            args,
            working_directory: working_directory.into(),
        }
    }

    /// Returns the argument vector as it would be passed to the OS.
    pub fn argv(&self) -> Vec<&str> {
        self.args.iter().map(String::as_str).collect()
    }
}

/// A validated request to run a conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedConversion {
    /// The source file to convert.
    pub input_path: PathBuf,
    /// The format of the source file.
    pub source_format: FormatId,
    /// The desired output format.
    pub target_format: FormatId,
    /// Directory that receives the output.
    pub output_directory: PathBuf,
    /// Policy for an already existing output file.
    pub overwrite_policy: OverwritePolicy,
    /// Options that influence the conversion.
    pub options: ConversionOptions,
}

/// The outcome of running an engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOutcome {
    /// Process exit code, when the process ran to completion.
    pub exit_code: Option<i32>,
    /// Captured standard output, already size limited and redacted.
    pub stdout: String,
    /// Captured standard error, already size limited and redacted.
    pub stderr: String,
    /// Files produced by the engine.
    pub produced_files: Vec<PathBuf>,
    /// Progress updates reported by the engine.
    pub progress: Vec<Progress>,
}

/// A conversion engine.
///
/// Implementations must never construct a shell command string and must
/// never accept an executable path from the frontend.
pub trait EngineAdapter: Send + Sync {
    /// Stable identifier of the engine.
    fn engine_id(&self) -> EngineId;

    /// Human readable name shown in the UI.
    fn display_name(&self) -> &'static str;

    /// Detects the engine installation on this machine.
    fn detect_installation(&self) -> EngineStatus;

    /// Reports the source-to-target pairs this engine supports.
    fn capabilities(&self) -> Vec<ConversionCapability>;

    /// Validates a conversion request against this engine.
    fn validate_request(&self, request: &ValidatedConversion) -> DomainResult<()>;

    /// Builds the process invocation for a validated request.
    fn build_invocation(&self, request: &ValidatedConversion) -> DomainResult<EngineInvocation>;

    /// Runs the engine and reports the outcome.
    ///
    /// The default implementation returns an error, because running a
    /// process is the responsibility of the infrastructure layer and not
    /// of the adapter itself.
    fn execute(
        &self,
        _invocation: &EngineInvocation,
        _cancel: &dyn CancellationSignal,
    ) -> DomainResult<EngineOutcome> {
        Err(crate::domain::error::ConversionError::new(
            crate::domain::error::ConversionErrorCode::Internal,
            "Engine execution is not available in this build.",
        ))
    }

    /// Cancels a running conversion.
    fn cancel(&self, _invocation: &EngineInvocation) -> DomainResult<()> {
        Err(crate::domain::error::ConversionError::new(
            crate::domain::error::ConversionErrorCode::Internal,
            "Engine cancellation is not available in this build.",
        ))
    }

    /// Validates a produced output file.
    ///
    /// The expected format is part of the call: an adapter validates
    /// the output against the format the user asked for, not against
    /// the format the engine likes to write.
    fn validate_output(
        &self,
        _path: &std::path::Path,
        _target_format: &FormatId,
    ) -> DomainResult<crate::domain::conversion::ValidationLevel> {
        Err(crate::domain::error::ConversionError::new(
            crate::domain::error::ConversionErrorCode::Internal,
            "Output validation is not available in this build.",
        ))
    }

    /// Returns sanitized diagnostic details for the engine.
    fn diagnostics(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Cooperative cancellation signal handed to a running engine.
pub trait CancellationSignal: Send + Sync {
    /// Returns `true` when the caller requested cancellation.
    fn is_cancelled(&self) -> bool;
}

/// A cancellation signal that never fires.
#[derive(Debug, Clone, Copy, Default)]
pub struct NeverCancelled;

impl CancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_keeps_arguments_separated() {
        let invocation = EngineInvocation::new(
            "/usr/bin/magick",
            vec!["convert".into(), "a b.png".into(), "a b.jpg".into()],
            "/tmp",
        );
        assert_eq!(invocation.argv(), ["convert", "a b.png", "a b.jpg"]);
        assert_eq!(invocation.program, PathBuf::from("/usr/bin/magick"));
    }

    #[test]
    fn arguments_with_shell_metacharacters_are_not_interpreted() {
        let invocation = EngineInvocation::new(
            "/usr/bin/tool",
            vec![
                "--name=$(whoami)".into(),
                "*.png".into(),
                "; rm -rf /".into(),
            ],
            "/tmp",
        );
        assert_eq!(invocation.args.len(), 3);
        assert_eq!(invocation.args[0], "--name=$(whoami)");
    }

    #[test]
    fn never_cancelled_signal_does_not_fire() {
        assert!(!NeverCancelled.is_cancelled());
    }
}
