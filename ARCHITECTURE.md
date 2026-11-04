# Architecture and Technical Design
## Universal File Converter

## 1. Architecture overview

Use a layered desktop architecture:

- Frontend: React + TypeScript.
- Desktop shell and privileged bridge: Tauri 2.
- Core application services: Rust.
- Conversion engines: isolated external processes or vetted libraries.
- Storage: local filesystem plus a small local metadata database if history is enabled.

The frontend must never execute conversion commands directly. It requests a typed operation from Rust commands. Rust validates the request, selects an adapter, starts the engine safely, tracks progress, validates output, and returns a structured result.

## 2. Logical components

1. UI layer: file intake, target selection, queue, progress, settings, results.
2. Tauri command layer: narrow, typed commands exposed to the frontend.
3. Application service layer: conversion orchestration, queue, cancellation, settings.
4. Domain layer: file types, conversion pair registry, job state machine, result/error models.
5. Engine adapter layer: LibreOffice, FFmpeg, Poppler, ImageMagick, archive adapters.
6. Infrastructure layer: filesystem, process runner, database, logging, platform paths.

## 3. Suggested repository structure

```text
universal-file-converter/
├── src/                         # React frontend
│   ├── app/
│   ├── components/
│   ├── features/
│   │   ├── conversion/
│   │   ├── queue/
│   │   ├── history/
│   │   ├── engines/
│   │   └── settings/
│   ├── hooks/
│   ├── lib/
│   ├── types/
│   └── styles/
├── src-tauri/
│   ├── capabilities/
│   ├── icons/
│   ├── src/
│   │   ├── commands/
│   │   ├── domain/
│   │   ├── services/
│   │   ├── engines/
│   │   ├── process/
│   │   ├── storage/
│   │   ├── security/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── resources/
│   ├── engines/
│   │   ├── windows/
│   │   ├── linux/
│   │   └── macos/
│   └── licenses/
├── tests/
│   ├── fixtures/
│   ├── integration/
│   ├── e2e/
│   └── quality/
├── docs/
├── scripts/
├── .github/workflows/
├── PRD.md
├── ARCHITECTURE.md
├── RULES.md
├── DESIGN.md
├── SUPPORTED_FORMATS.md
├── TESTING.md
├── SECURITY.md
├── README.md
└── LICENSE
```

## 4. Engine adapter contract

Every engine adapter should implement a consistent conceptual contract:

- `detect_installation()`: find bundled/system engine and version.
- `capabilities()`: report supported input/output pairs and options.
- `validate_request()`: validate format pair and options.
- `build_invocation()`: create executable path and argument vector (never a shell string).
- `execute()`: run process, capture output safely, emit progress if available.
- `cancel()`: terminate process tree safely.
- `validate_output()`: verify output type and usability.
- `diagnostics()`: return sanitized diagnostic details.

Do not expose arbitrary executable paths or arbitrary command arguments to the frontend.

## 5. Engine selection and hybrid strategy

Selection order:
1. Prefer a compatible bundled engine when present and approved for redistribution.
2. Otherwise detect a supported system installation.
3. Verify engine version and required capabilities.
4. If neither is usable, return `MissingDependency` with platform-specific installation guidance.

Never silently switch to a different engine if it may change output quality or behavior. Show the selected engine and source (bundled/system) in job details.

Bundled and system engines must be version-pinned or constrained by a tested compatibility range. Record exact engine version in diagnostics.

## 6. Process execution

- Use Rust process APIs with executable path and argument vector.
- Do not invoke `sh -c`, `cmd /C`, PowerShell, or equivalent for ordinary conversion.
- Do not interpolate file names into command strings.
- Use unique per-job temporary directories with restrictive permissions.
- Set a controlled working directory and minimal environment.
- Capture stdout/stderr with size limits and redaction.
- Enforce timeouts where appropriate.
- Limit concurrent processes based on engine and resource profile.
- Terminate child process trees on cancellation.
- Treat engine exit code as one signal, not the sole success criterion.

## 7. File handling pipeline

1. Receive selected paths through a narrow command.
2. Check existence, regular-file status, read permission, and path policy.
3. Detect format using extension plus signatures where feasible.
4. Reject or warn on extension/content mismatch.
5. Resolve conversion pair from explicit capability registry.
6. Check free disk space when meaningful.
7. Create isolated temp/output staging directory.
8. Execute engine.
9. Validate output format, size, readability, and expected metadata.
10. Move validated output atomically to final destination where possible.
11. Clean temporary files in success/failure/cancel paths.
12. Persist minimal job metadata.

Do not modify the original source file.

## 8. Output validation levels

- Level 0: output exists and is a regular file.
- Level 1: non-empty and expected signature/container detected.
- Level 2: output can be parsed or opened by an independent library/tool.
- Level 3: semantic/visual checks against fixture expectations.

A result should include the validation level achieved. Not every format supports Level 3.

## 9. Progress model

Represent progress as:
- `Determinate { percent }` when engine provides trustworthy progress.
- `Indeterminate` when work is underway without reliable progress.
- `Stage { name }` for phases such as preparing, converting, validating.

Do not fabricate a percentage based only on elapsed time.

## 10. Suggested Rust domain types

```rust
enum JobStatus {
    Queued,
    Validating,
    WaitingForEngine,
    Running,
    ValidatingOutput,
    Succeeded,
    SucceededWithWarnings,
    Failed,
    Cancelled,
}

enum Progress {
    Determinate { percent: u8 },
    Indeterminate,
    Stage { name: String },
}

struct ConversionRequest {
    input_path: String,
    target_format: String,
    output_directory: Option<String>,
    overwrite_policy: OverwritePolicy,
}

struct ConversionResult {
    job_id: String,
    status: JobStatus,
    output_path: Option<String>,
    engine_name: Option<String>,
    engine_version: Option<String>,
    warnings: Vec<String>,
    error: Option<ConversionError>,
    validation_level: u8,
}
```

Treat these as design examples; refine types and serialization before implementation. Prefer path types internally (`PathBuf`) rather than strings.

## 11. Frontend/backend boundary

Expose typed commands such as:
- `inspect_files(paths)`
- `get_conversion_options(file_id)`
- `start_conversion(requests)`
- `cancel_job(job_id)`
- `get_job(job_id)`
- `list_history(filters)`
- `get_engine_status()`
- `update_settings(settings)`

Use Tauri events for job state/progress updates. Validate every payload in Rust even if the frontend already validates it.

## 12. Local storage

Store only what is necessary:
- job ID;
- source filename (optional/configurable);
- source and output format;
- timestamp and duration;
- engine/version;
- status and validation level;
- sanitized error summary.

Do not store document contents, extracted text, thumbnails, or full paths by default. Make history retention configurable and allow clearing it.

## 13. Cross-platform considerations

- Resolve app/resource paths through Tauri APIs and platform-specific Rust code.
- Do not assume executable names or install paths are identical across operating systems.
- Account for macOS app signing/notarization and quarantine behavior.
- Account for Linux package formats, executable permissions, shared-library dependencies, and sandboxing.
- Account for Windows process-tree termination, path length, and installer conventions.
- Test each engine on each supported OS before marking the corresponding pair stable.

## 14. Architecture decision records

Create an ADR for major decisions:
- ADR-001: Tauri 2 and Rust backend.
- ADR-002: hybrid bundled/system engine discovery.
- ADR-003: local-only processing and no telemetry by default.
- ADR-004: conversion-pair capability registry.
- ADR-005: output validation levels.
