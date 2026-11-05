//! The conversion service.
//!
//! The service orchestrates a conversion. It owns the order of the
//! steps described in `ARCHITECTURE.md` section 7:
//!
//! 1. the source path is canonicalised and inspected,
//! 2. the pair is resolved from the capability registry,
//! 3. the engine is asked to validate the request,
//! 4. the engine runs in an isolated job directory,
//! 5. the output is validated independently,
//! 6. the output is moved to its destination,
//! 7. the job directory is removed.
//!
//! Every state change goes through the state machine of the domain
//! crate, and the service reports it to an observer so that the
//! interface can show the progress.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use ufc_core::domain::conversion::{
    output_file_name, ConversionRequest, ConversionResult, EngineAvailability, EngineId,
    EngineStatus, ValidationLevel,
};
use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use ufc_core::domain::format::from_extension;
use ufc_core::domain::job::{validate_transition, JobStatus, Progress};
use ufc_core::domain::registry::{CapabilityRegistry, ConversionCapability};
use ufc_core::engines::adapter::{
    CancellationSignal, EngineAdapter, EngineInvocation, ValidatedConversion,
};
use uuid::Uuid;

use crate::engines;
use crate::output::resolve_destination;
use crate::validation;

/// Receives the state changes of a running job.
pub trait JobObserver {
    /// Called whenever the job changes state.
    fn on_update(&self, status: JobStatus, progress: Progress);
}

/// An observer that ignores every update.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullObserver;

impl JobObserver for NullObserver {
    fn on_update(&self, _status: JobStatus, _progress: Progress) {}
}

/// Orchestrates conversions with the adapters of the application.
pub struct ConversionService {
    adapters: Vec<Arc<dyn EngineAdapter>>,
    registry: RwLock<CapabilityRegistry>,
}

impl std::fmt::Debug for ConversionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversionService")
            .field("adapters", &self.adapters.len())
            .field("capabilities", &read_lock(&self.registry).len())
            .finish()
    }
}

impl Default for ConversionService {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConversionService {
    fn clone(&self) -> Self {
        Self {
            adapters: self.adapters.clone(),
            registry: RwLock::new(self.registry()),
        }
    }
}

impl ConversionService {
    /// Creates a service and detects the engines of this device.
    pub fn new() -> Self {
        let adapters = engines::adapters();
        let registry = build_registry(&adapters);

        Self {
            adapters,
            registry: RwLock::new(registry),
        }
    }

    /// Creates a service with a fixed set of adapters.
    pub fn with_adapters(adapters: Vec<Arc<dyn EngineAdapter>>) -> Self {
        let registry = build_registry(&adapters);

        Self {
            adapters,
            registry: RwLock::new(registry),
        }
    }

    /// Returns the capability registry of the application.
    pub fn registry(&self) -> CapabilityRegistry {
        read_lock(&self.registry).clone()
    }

    /// Detects the engines again and rebuilds the registry.
    ///
    /// The application calls this when the user asks for a new
    /// detection, for example after installing an engine.
    pub fn refresh(&self) {
        let registry = build_registry(&self.adapters);
        *write_lock(&self.registry) = registry;
    }

    /// Returns the status of every engine.
    pub fn engine_status(&self) -> Vec<EngineStatus> {
        engines::engine_statuses(&self.adapters)
    }

    /// Converts one file.
    ///
    /// The method never panics and never reports a success that was not
    /// verified: the output is validated before the job is marked as
    /// succeeded (`PRD.md` FR-005, FR-010).
    pub fn convert(
        &self,
        request: &ConversionRequest,
        cancel: &dyn CancellationSignal,
        observer: &dyn JobObserver,
    ) -> ConversionResult {
        let job_id = Uuid::new_v4();
        let mut result = ConversionResult::in_progress(job_id, JobStatus::Queued);
        result.source_name = source_name(&request.input_path);

        observer.on_update(
            JobStatus::Queued,
            Progress::Stage {
                name: "preparing".to_owned(),
            },
        );

        match self.run(request, cancel, observer) {
            Ok(success) => {
                result.status = success.status;
                result.output_path = Some(success.output_path.to_string_lossy().to_string());
                result.engine_id = success.engine_id;
                result.engine_version = success.engine_version;
                result.warnings = success.warnings;
                result.validation_level = success.validation_level;
                result.progress = Progress::Stage {
                    name: "done".to_owned(),
                };
            }
            Err(error) => {
                result.status = if error.code == ConversionErrorCode::Cancelled {
                    JobStatus::Cancelled
                } else {
                    JobStatus::Failed
                };
                result.error = Some(error);
                result.progress = Progress::Indeterminate;
            }
        }

        result
    }

    /// Runs the steps of a conversion.
    fn run(
        &self,
        request: &ConversionRequest,
        cancel: &dyn CancellationSignal,
        observer: &dyn JobObserver,
    ) -> DomainResult<Success> {
        // Queued -> Validating
        transition(
            observer,
            JobStatus::Queued,
            JobStatus::Validating,
            Progress::Stage {
                name: "checking file".to_owned(),
            },
        )?;

        let input_path = canonicalise_source(&request.input_path)?;
        let source_format = detect_source_format(&input_path)?;

        let capability = self.find_capability(&source_format, &request.target_format)?;
        let adapter = self.find_adapter(&capability.engine_id)?;
        let engine_status = adapter.detect_installation();

        if !capability.available {
            let reason = engine_status.reason.clone().unwrap_or_else(|| {
                ConversionError::new(
                    ConversionErrorCode::MissingDependency,
                    format!(
                        "{} is not available on this device.",
                        capability.engine_id.as_str()
                    ),
                )
            });

            return Err(reason);
        }

        let output_directory = PathBuf::from(&request.output_directory);
        if !output_directory.is_dir() {
            return Err(ConversionError::new(
                ConversionErrorCode::InvalidOutputLocation,
                "The output directory does not exist.",
            ));
        }

        // The engine writes into an isolated job directory. The
        // validated file is only moved to its destination afterwards.
        let job_directory = create_job_directory()?;

        let validated = ValidatedConversion {
            input_path: input_path.clone(),
            source_format: source_format.clone(),
            target_format: request.target_format.clone(),
            output_directory: job_directory.clone(),
            overwrite_policy: request.overwrite_policy,
            options: request.options.clone(),
        };

        let outcome = adapter
            .validate_request(&validated)
            .and_then(|()| adapter.build_invocation(&validated))
            .and_then(|invocation| self.execute(adapter.as_ref(), &invocation, cancel, observer));

        if let Err(error) = outcome {
            let _ = std::fs::remove_dir_all(&job_directory);
            return Err(error);
        }

        // The output is validated before the job is marked as
        // succeeded.
        if let Err(error) = transition(
            observer,
            JobStatus::Running,
            JobStatus::ValidatingOutput,
            Progress::Stage {
                name: "validating output".to_owned(),
            },
        ) {
            let _ = std::fs::remove_dir_all(&job_directory);
            return Err(error);
        }

        let produced = job_directory.join(output_file_name(&input_path, &request.target_format));

        let validation_level = match adapter
            .validate_output(&produced)
            .and_then(|level| validation::compare_dimensions(&input_path, &produced).map(|_| level))
        {
            Ok(level) => level,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&job_directory);
                return Err(error);
            }
        };

        let destination = match resolve_destination(
            &output_directory,
            &produced_name(&input_path, request),
            request.overwrite_policy,
        ) {
            Ok(destination) => destination,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&job_directory);
                return Err(error);
            }
        };

        if let Err(error) = move_file(&produced, &destination) {
            let _ = std::fs::remove_dir_all(&job_directory);
            return Err(error);
        }

        let _ = std::fs::remove_dir_all(&job_directory);

        let mut warnings = Vec::new();
        if validation::png_has_alpha(&input_path) {
            warnings.push(
                "The source image has an alpha channel. It was flattened onto a solid background, because JPEG cannot store transparency."
                    .to_owned(),
            );
        }

        transition(
            observer,
            JobStatus::ValidatingOutput,
            JobStatus::Succeeded,
            Progress::Stage {
                name: "done".to_owned(),
            },
        )?;

        Ok(Success {
            status: JobStatus::Succeeded,
            output_path: destination,
            engine_id: Some(capability.engine_id.to_string()),
            engine_version: engine_status.version.clone(),
            warnings,
            validation_level,
        })
    }

    /// Runs the engine and reports the state change.
    fn execute(
        &self,
        adapter: &dyn EngineAdapter,
        invocation: &EngineInvocation,
        cancel: &dyn CancellationSignal,
        observer: &dyn JobObserver,
    ) -> DomainResult<ufc_core::engines::adapter::EngineOutcome> {
        // Validating -> Running
        transition(
            observer,
            JobStatus::Validating,
            JobStatus::Running,
            Progress::Stage {
                name: "converting".to_owned(),
            },
        )?;

        adapter.execute(invocation, cancel)
    }

    /// Looks up the capability of a pair.
    fn find_capability(
        &self,
        source_format: &ufc_core::domain::format::FormatId,
        target_format: &ufc_core::domain::format::FormatId,
    ) -> DomainResult<ConversionCapability> {
        read_lock(&self.registry)
            .find(source_format, target_format)
            .cloned()
            .ok_or_else(|| {
                ufc_core::domain::conversion::unsupported_pair(source_format, target_format)
            })
    }

    /// Looks up the adapter of an engine.
    fn find_adapter(&self, engine_id: &EngineId) -> DomainResult<Arc<dyn EngineAdapter>> {
        self.adapters
            .iter()
            .find(|adapter| &adapter.engine_id() == engine_id)
            .cloned()
            .ok_or_else(|| {
                ConversionError::new(
                    ConversionErrorCode::MissingDependency,
                    format!("{engine_id} is not available on this device."),
                )
            })
    }
}

/// The successful outcome of a conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Success {
    status: JobStatus,
    output_path: PathBuf,
    engine_id: Option<String>,
    engine_version: Option<String>,
    warnings: Vec<String>,
    validation_level: ValidationLevel,
}

/// Returns the name of the source file, without its directory.
fn source_name(input_path: &str) -> Option<String> {
    std::path::Path::new(input_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}

/// Reads a lock, even when a previous holder panicked.
fn read_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|error| error.into_inner())
}

/// Writes a lock, even when a previous holder panicked.
fn write_lock<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(|error| error.into_inner())
}

/// Builds the registry from the adapters and their detection result.
fn build_registry(adapters: &[Arc<dyn EngineAdapter>]) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::empty();

    for adapter in adapters {
        for capability in adapter.capabilities() {
            registry.insert(capability);
        }
    }

    let mut availability: BTreeMap<EngineId, EngineAvailability> = BTreeMap::new();
    for adapter in adapters {
        let status = adapter.detect_installation();
        availability.insert(status.engine_id.clone(), status.availability);
    }

    registry.apply_availability(&availability);

    registry
}

/// Reports a state change and validates it against the state machine.
fn transition(
    observer: &dyn JobObserver,
    from: JobStatus,
    to: JobStatus,
    progress: Progress,
) -> DomainResult<()> {
    validate_transition(from, to).map_err(|error| {
        ConversionError::new(
            ConversionErrorCode::Internal,
            format!("The job could not change its state: {error}"),
        )
    })?;

    observer.on_update(to, progress);

    Ok(())
}

/// Canonicalises the source path and makes sure it is a regular file.
fn canonicalise_source(path: &str) -> DomainResult<PathBuf> {
    if path.trim().is_empty() {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "No source file was selected.",
        ));
    }

    let canonical = std::fs::canonicalize(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source file could not be found.",
        )
        .with_detail(source.to_string())
    })?;

    let metadata = std::fs::metadata(&canonical).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source file could not be read.",
        )
        .with_detail(source.to_string())
    })?;

    if !metadata.is_file() {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The source is not a regular file.",
        ));
    }

    Ok(canonical)
}

/// Detects the format of the source file from its extension.
fn detect_source_format(path: &Path) -> DomainResult<ufc_core::domain::format::FormatId> {
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();

    from_extension(&extension).ok_or_else(|| {
        ConversionError::new(
            ConversionErrorCode::UndetectedFormat,
            format!("The format of '{}' is not known.", extension),
        )
        .with_remediation("The file name has to end in a known extension.")
    })
}

/// Creates the isolated directory of a job.
fn create_job_directory() -> DomainResult<PathBuf> {
    let base = std::env::temp_dir();
    let directory = base.join(format!("ufc-job-{}", Uuid::new_v4()));

    std::fs::create_dir_all(&directory).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::IoError,
            "The job directory could not be created.",
        )
        .with_detail(source.to_string())
    })?;

    restrict_permissions(&directory);

    Ok(directory)
}

/// Removes the access permissions of other users on Unix.
fn restrict_permissions(directory: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700));
    }

    #[cfg(not(unix))]
    {
        let _ = directory;
    }
}

/// Returns the name of the produced file.
fn produced_name(input_path: &Path, request: &ConversionRequest) -> String {
    output_file_name(input_path, &request.target_format)
}

/// Moves a file, also when it crosses a file system boundary.
fn move_file(from: &Path, to: &Path) -> DomainResult<()> {
    match std::fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(from, to).map_err(|source| {
                ConversionError::new(
                    ConversionErrorCode::IoError,
                    "The converted file could not be moved to the output directory.",
                )
                .with_detail(source.to_string())
            })?;

            let _ = std::fs::remove_file(from);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ufc_core::engines::adapter::NeverCancelled;

    fn service() -> ConversionService {
        ConversionService::new()
    }

    #[test]
    fn the_registry_lists_the_implemented_pairs() {
        let service = service();
        let registry = service.registry();

        assert!(!registry.is_empty());
        assert!(
            registry
                .find(
                    &from_extension("png").expect("valid"),
                    &from_extension("jpg").expect("valid")
                )
                .is_some(),
            "the PNG to JPEG pair must be registered"
        );
    }

    #[test]
    fn a_pair_without_an_implementation_is_not_offered() {
        let service = service();
        let registry = service.registry();

        assert!(
            registry
                .find(
                    &from_extension("docx").expect("valid"),
                    &from_extension("pdf").expect("valid")
                )
                .is_none(),
            "a planned pair must not be registered"
        );
    }

    #[test]
    fn a_missing_source_file_is_reported() {
        let request = ConversionRequest::new(
            Uuid::nil(),
            "/definitely/not/here.png",
            from_extension("jpg").expect("valid"),
            std::env::temp_dir().to_string_lossy().to_string(),
        );

        let result = service().convert(&request, &NeverCancelled, &NullObserver);

        assert_eq!(result.status, JobStatus::Failed);
        assert_eq!(
            result.error.expect("an error is reported").code,
            ConversionErrorCode::InvalidSource
        );
    }

    #[test]
    fn an_unsupported_pair_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let source = dir.path().join("document.docx");
        std::fs::write(&source, b"not really a document").expect("the file is written");

        let request = ConversionRequest::new(
            Uuid::nil(),
            source.to_string_lossy().to_string(),
            from_extension("pdf").expect("valid"),
            dir.path().to_string_lossy().to_string(),
        );

        let result = service().convert(&request, &NeverCancelled, &NullObserver);

        assert_eq!(result.status, JobStatus::Failed);
        assert_eq!(
            result.error.expect("an error is reported").code,
            ConversionErrorCode::UnsupportedPair
        );
    }

    #[test]
    fn a_missing_output_directory_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let source = dir.path().join("image.png");
        write_png(&source, 8, 8);

        let request = ConversionRequest::new(
            Uuid::nil(),
            source.to_string_lossy().to_string(),
            from_extension("jpg").expect("valid"),
            dir.path().join("missing").to_string_lossy().to_string(),
        );

        let result = service().convert(&request, &NeverCancelled, &NullObserver);

        assert_eq!(result.status, JobStatus::Failed);
        assert_eq!(
            result.error.expect("an error is reported").code,
            ConversionErrorCode::InvalidOutputLocation
        );
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        let image = image::RgbImage::from_pixel(width, height, image::Rgb([10, 20, 30]));
        image
            .save_with_format(path, image::ImageFormat::Png)
            .expect("the fixture is written");
    }
}
