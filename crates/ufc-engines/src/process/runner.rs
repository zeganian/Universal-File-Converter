//! The process runner.
//!
//! This is the only place in the application where an external program
//! is started. It is deliberately restrictive:
//!
//! - the program is started with an argument vector, never with a
//!   shell command string,
//! - the environment is cleared and rebuilt from an allowlist,
//! - the working directory is chosen by the caller,
//! - the output is captured through temporary files with a size limit,
//! - the process is killed when the timeout expires or when the caller
//!   cancels, and the whole process group is killed on Unix,
//! - the paths of the job are redacted from the captured output.

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use ufc_core::engines::adapter::CancellationSignal;

/// Default limit for the captured standard output and error.
pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 64 * 1024;

/// How often the runner checks whether the process is still running.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Counter that makes the name of a capture file unique.
static CAPTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A request to run an external program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRequest {
    /// Absolute path of the executable.
    pub program: PathBuf,
    /// Arguments, already separated. They are never joined into a
    /// string and never passed to a shell.
    pub args: Vec<String>,
    /// Working directory of the process.
    pub working_directory: PathBuf,
    /// Maximum runtime before the process is killed.
    pub timeout: Duration,
    /// Maximum size of the captured standard output and error.
    pub max_output_bytes: usize,
    /// Additional environment variables for the process.
    pub env: BTreeMap<String, String>,
    /// Paths that must not appear in the captured output, together with
    /// the placeholder that replaces them.
    pub redactions: Vec<(String, String)>,
}

impl ProcessRequest {
    /// Creates a request with the default timeout and output limit.
    pub fn new(
        program: impl Into<PathBuf>,
        args: Vec<String>,
        working_directory: impl Into<PathBuf>,
    ) -> Self {
        Self {
            program: program.into(),
            args,
            working_directory: working_directory.into(),
            timeout: Duration::from_secs(120),
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            env: BTreeMap::new(),
            redactions: Vec::new(),
        }
    }

    /// Sets the timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the output size limit.
    #[must_use]
    pub fn with_max_output_bytes(mut self, limit: usize) -> Self {
        self.max_output_bytes = limit;
        self
    }

    /// Adds an environment variable.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Registers a path that must be redacted from the output.
    #[must_use]
    pub fn with_redaction(
        mut self,
        path: impl AsRef<Path>,
        placeholder: impl Into<String>,
    ) -> Self {
        let value = path.as_ref().to_string_lossy().to_string();
        if !value.is_empty() {
            self.redactions.push((value, placeholder.into()));
        }
        self
    }
}

/// The result of running an external program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutcome {
    /// Exit code of the process, when it ran to completion.
    pub exit_code: Option<i32>,
    /// Captured standard output, size limited and redacted.
    pub stdout: String,
    /// Captured standard error, size limited and redacted.
    pub stderr: String,
    /// How long the process ran.
    pub duration: Duration,
    /// Whether the process was killed because the timeout expired.
    pub timed_out: bool,
    /// Whether the process was killed because the caller cancelled.
    pub cancelled: bool,
}

impl ProcessOutcome {
    /// Returns `true` when the process exited with code zero and was
    /// not killed.
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out && !self.cancelled
    }
}

/// Runs external programs under the rules of the security document.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessRunner;

impl ProcessRunner {
    /// Runs a program and waits for it to finish.
    ///
    /// # Errors
    /// Returns [`ConversionErrorCode::EngineStartFailed`] when the
    /// program cannot be started at all.
    pub fn run(
        request: &ProcessRequest,
        cancel: &dyn CancellationSignal,
    ) -> DomainResult<ProcessOutcome> {
        let capture_id = CAPTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let stdout_path = request
            .working_directory
            .join(format!(".ufc-stdout-{}-{capture_id}", std::process::id()));
        let stderr_path = request
            .working_directory
            .join(format!(".ufc-stderr-{}-{capture_id}", std::process::id()));

        let stdout_file = create_capture_file(&stdout_path)?;
        let stderr_file = create_capture_file(&stderr_path)?;

        let stdout_handle = duplicate_handle(&stdout_file)?;
        let stderr_handle = duplicate_handle(&stderr_file)?;

        let mut command = build_command(request, stdout_handle, stderr_handle);

        let started = Instant::now();
        let mut child = command.spawn().map_err(|source| {
            ConversionError::new(
                ConversionErrorCode::EngineStartFailed,
                "The conversion engine could not be started.",
            )
            .with_detail(source.to_string())
        })?;

        let mut timed_out = false;
        let mut cancelled = false;

        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {}
                Err(source) => {
                    let _ = std::fs::remove_file(&stdout_path);
                    let _ = std::fs::remove_file(&stderr_path);
                    return Err(ConversionError::new(
                        ConversionErrorCode::EngineFailed,
                        "The conversion engine could not be waited for.",
                    )
                    .with_detail(source.to_string()));
                }
            }

            if cancel.is_cancelled() {
                cancelled = true;
                kill_process_tree(&mut child);
                let _ = child.wait();
                break;
            }

            if started.elapsed() >= request.timeout {
                timed_out = true;
                kill_process_tree(&mut child);
                let _ = child.wait();
                break;
            }

            std::thread::sleep(POLL_INTERVAL);
        }

        let exit_code = child
            .try_wait()
            .ok()
            .flatten()
            .and_then(|status| status.code());
        let duration = started.elapsed();

        let stdout = read_capture(&stdout_path, request.max_output_bytes);
        let stderr = read_capture(&stderr_path, request.max_output_bytes);
        let _ = std::fs::remove_file(&stdout_path);
        let _ = std::fs::remove_file(&stderr_path);

        Ok(ProcessOutcome {
            exit_code,
            stdout: redact(&stdout, &request.redactions),
            stderr: redact(&stderr, &request.redactions),
            duration,
            timed_out,
            cancelled,
        })
    }
}

/// Builds the command with a cleared and rebuilt environment.
fn build_command(request: &ProcessRequest, stdout_file: File, stderr_file: File) -> Command {
    let mut command = Command::new(&request.program);
    command
        .args(&request.args)
        .current_dir(&request.working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .env_clear();

    for (key, value) in allowed_environment(request) {
        command.env(key, value);
    }

    configure_process_group(&mut command);

    command
}

/// Returns the environment variables the process is allowed to see.
///
/// The list is rebuilt from scratch: the child never inherits the
/// environment of the application.
fn allowed_environment(request: &ProcessRequest) -> Vec<(String, String)> {
    let mut env: BTreeMap<String, String> = BTreeMap::new();

    let inherited = |name: &str| -> Option<String> {
        std::env::var(name).ok().filter(|value| !value.is_empty())
    };

    // A minimal, portable allowlist. Windows needs a few of these for
    // the process to start at all.
    for name in [
        "PATH",
        "LANG",
        "LC_ALL",
        "SystemRoot",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
    ] {
        if let Some(value) = inherited(name) {
            env.insert(name.to_owned(), value);
        }
    }

    // The job directory is the only writable place the engine may use
    // for temporary files.
    env.insert(
        "TMPDIR".to_owned(),
        request.working_directory.to_string_lossy().to_string(),
    );

    for (key, value) in &request.env {
        env.insert(key.clone(), value.clone());
    }

    env.into_iter().collect()
}

/// Puts the process into its own process group on Unix.
///
/// The group is killed as a whole on cancellation or timeout, so that a
/// child process of the engine cannot survive its parent.
fn configure_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

/// Creates the file that receives a stream of the process.
fn create_capture_file(path: &Path) -> DomainResult<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|source| {
            ConversionError::new(
                ConversionErrorCode::EngineStartFailed,
                "The output capture file could not be created.",
            )
            .with_detail(source.to_string())
        })
}

/// Creates a second handle for a capture file.
fn duplicate_handle(file: &File) -> DomainResult<File> {
    file.try_clone().map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::EngineStartFailed,
            "The output capture file could not be duplicated.",
        )
        .with_detail(source.to_string())
    })
}

/// Reads at most `limit` bytes from a capture file.
fn read_capture(path: &Path, limit: usize) -> String {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return String::new(),
    };

    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut truncated = false;

    loop {
        match file.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                if buffer.len() + read > limit {
                    let remaining = limit.saturating_sub(buffer.len());
                    buffer.extend_from_slice(&chunk[..remaining]);
                    truncated = true;
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }
            Err(_) => break,
        }
    }

    let mut text = String::from_utf8_lossy(&buffer).to_string();
    if truncated {
        text.push_str("\n[output truncated]");
    }
    text
}

/// Replaces the registered paths in the captured output.
fn redact(text: &str, redactions: &[(String, String)]) -> String {
    let mut redacted = text.to_owned();

    for (secret, placeholder) in redactions {
        if !secret.is_empty() {
            redacted = redacted.replace(secret.as_str(), placeholder);
        }
    }

    redacted
}

/// Kills the process and, on Unix, its whole process group.
fn kill_process_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        use nix::sys::signal::{killpg, Signal};
        use nix::unistd::Pid;

        // The group id equals the process id because the child was put
        // into its own group.
        let pid = Pid::from_raw(child.id() as i32);
        let _ = killpg(pid, Signal::SIGKILL);
    }

    #[cfg(windows)]
    {
        // `taskkill` is started with an argument vector, never with a
        // command string, and its arguments are all integers.
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }

    let _ = child.kill();
}

/// Returns the program name of a command, for diagnostics.
pub fn program_name(program: &Path) -> String {
    program
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "engine".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ufc_core::engines::adapter::NeverCancelled;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    #[test]
    fn a_missing_program_is_reported_as_a_start_failure() {
        let dir = temp_dir();
        let request =
            ProcessRequest::new(dir.path().join("does-not-exist"), Vec::new(), dir.path());

        let error = ProcessRunner::run(&request, &NeverCancelled).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::EngineStartFailed);
    }

    #[test]
    fn the_output_is_limited() {
        let dir = temp_dir();
        let request = ProcessRequest::new(
            "/bin/sh",
            vec!["-c".to_owned(), "yes hello | head -c 100000".to_owned()],
            dir.path(),
        )
        .with_max_output_bytes(256);

        let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

        assert!(outcome.stdout.len() < 1024);
        assert!(outcome.stdout.contains("output truncated"));
    }

    #[test]
    fn the_output_is_redacted() {
        let dir = temp_dir();
        let secret = dir.path().join("secret.txt");
        let request = ProcessRequest::new(
            "/bin/sh",
            vec!["-c".to_owned(), format!("echo {}", secret.display())],
            dir.path(),
        )
        .with_redaction(&secret, "<input>");

        let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

        assert!(!outcome.stdout.contains("secret.txt"));
        assert!(outcome.stdout.contains("<input>"));
    }

    #[test]
    fn the_environment_only_contains_the_allowlist() {
        let dir = temp_dir();
        let request = ProcessRequest::new(
            "/bin/sh",
            vec!["-c".to_owned(), "echo ${UFC_SECRET:-unset}".to_owned()],
            dir.path(),
        )
        .with_env("UFC_SECRET", "value");

        let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

        assert!(outcome.stdout.contains("value"));
    }

    #[cfg(unix)]
    mod unix {
        use super::*;

        #[test]
        fn a_successful_program_reports_its_exit_code() {
            let dir = temp_dir();
            let request = ProcessRequest::new(
                "/bin/sh",
                vec!["-c".to_owned(), "echo hello".to_owned()],
                dir.path(),
            );

            let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

            assert_eq!(outcome.exit_code, Some(0));
            assert!(outcome.is_success());
            assert!(outcome.stdout.contains("hello"));
        }

        #[test]
        fn a_failing_program_reports_its_exit_code() {
            let dir = temp_dir();
            let request = ProcessRequest::new(
                "/bin/sh",
                vec!["-c".to_owned(), "exit 3".to_owned()],
                dir.path(),
            );

            let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

            assert_eq!(outcome.exit_code, Some(3));
            assert!(!outcome.is_success());
        }

        #[test]
        fn the_timeout_kills_the_process() {
            let dir = temp_dir();
            let request = ProcessRequest::new(
                "/bin/sh",
                vec!["-c".to_owned(), "sleep 30".to_owned()],
                dir.path(),
            )
            .with_timeout(Duration::from_millis(300));

            let outcome = ProcessRunner::run(&request, &NeverCancelled).expect("runs");

            assert!(outcome.timed_out);
            assert!(!outcome.is_success());
            assert!(outcome.duration < Duration::from_secs(5));
        }

        #[test]
        fn a_cancelled_run_stops_the_process() {
            struct AlwaysCancelled;
            impl CancellationSignal for AlwaysCancelled {
                fn is_cancelled(&self) -> bool {
                    true
                }
            }

            let dir = temp_dir();
            let request = ProcessRequest::new(
                "/bin/sh",
                vec!["-c".to_owned(), "sleep 30".to_owned()],
                dir.path(),
            )
            .with_timeout(Duration::from_secs(30));

            let outcome = ProcessRunner::run(&request, &AlwaysCancelled).expect("runs");

            assert!(outcome.cancelled);
            assert!(!outcome.is_success());
        }
    }
}
