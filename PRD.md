# Product Requirements Document (PRD)

## Universal File Converter

- Status: Draft v1.0
- Product type: Cross-platform desktop application
- Platforms: Windows, Linux, macOS
- Architecture: Tauri 2 + React + TypeScript + Rust
- Processing model: Local-first, offline-capable
- Distribution: GitHub Releases; hybrid engine strategy

## 1. Product vision

Universal File Converter (UFC) is an open-source desktop application that converts files locally on the user's computer. It aims to provide reliable, transparent, and accessible conversions without requiring file uploads, accounts, subscriptions, or a remote conversion service.

**Product promise:** Files stay on the user's device by default. Conversion reliability and clear error reporting take priority over maximizing the number of supported format pairs.

## 2. Problem statement

Online converters commonly impose upload-size limits, queues, account requirements, privacy trade-offs, or paid tiers. Conversion may also fail silently or produce output that is technically created but unusable.

UFC addresses these issues by:

- processing files locally;
- using mature, format-specific conversion engines;
- validating outputs after conversion;
- explaining missing dependencies and conversion failures;
- supporting batch jobs and predictable output handling.

The application cannot promise unlimited file size or perfect fidelity. Actual limits depend on hardware, available disk space, engine capabilities, source-file complexity, and platform constraints.

## 3. Goals

1. Provide a dependable desktop conversion workflow on all three desktop platforms.
2. Keep user files local unless the user explicitly chooses otherwise in a future feature.
3. Make engine availability and compatibility understandable.
4. Detect unsupported or invalid conversion pairs before starting where possible.
5. Verify generated output rather than treating process exit code alone as success.
6. Provide actionable errors, progress, cancellation, and batch processing.
7. Make the project maintainable through modular converters and automated tests.

## 4. Non-goals for the initial release

- Cloud conversion, online accounts, synchronization, or telemetry by default.
- Guaranteeing pixel-perfect conversion for every document.
- Editing document contents or providing a full office suite.
- DRM removal, password cracking, or bypassing access controls.
- Supporting every legacy or proprietary format.
- Running untrusted conversion engines or plugins with unrestricted privileges.
- Mobile applications.

## 5. Target users and primary jobs

- Students and office users: convert DOCX, XLSX, PPTX, and PDF.
- Creators: convert and resize common image formats.
- Media users: transcode audio/video through FFmpeg.
- Developers and power users: batch-convert files and inspect detailed logs.

Primary job: "I have a file in format A and need a usable file in format B without uploading it to a website."

## 6. Product principles

- Local-first: input files are not uploaded.
- Reliability over breadth: only advertise conversion pairs that are tested.
- Explicit dependencies: never imply an engine is available when it is not.
- Safe by default: validate paths, avoid shell interpolation, and isolate temporary files.
- Honest outcomes: distinguish success, warning, partial success, cancellation, and failure.
- Reproducibility: record engine name/version and conversion options in job details.
- Accessibility: keyboard navigation, readable contrast, and clear status labels.

## 7. MVP scope

### 7.1 Core workflow

1. User selects or drags files into the application.
2. Application detects file type using extension plus content/signature checks where practical.
3. User selects an available output format.
4. Application validates the source/target pair and required engine.
5. User chooses output location and optional safe conversion settings.
6. Application queues and runs the job.
7. Application validates output and displays the result.
8. User can open the output folder or view the error details.

### 7.2 MVP capabilities

- File picker and drag-and-drop.
- Single and batch conversion.
- Conversion queue with per-file and overall progress where supported.
- Pause is not required for first MVP; cancellation should be supported where safe.
- Output directory selection and collision-safe filenames.
- Conversion history stored locally with configurable retention.
- Engine/dependency status screen.
- Clear unsupported-pair and missing-engine messages.
- Detailed, user-readable error panel.
- Temporary-file cleanup.
- Offline operation after application and required engines are installed.

### 7.3 Initial format families

The table is an initial target, not a promise that every pair is supported. Publish a tested source-to-target matrix in `SUPPORTED_FORMATS.md`.

| Family        | Initial candidates                                 | Engine                                                   |
| ------------- | -------------------------------------------------- | -------------------------------------------------------- |
| Documents     | DOCX, ODT, RTF, TXT, HTML, Markdown                | LibreOffice and/or Pandoc                                |
| Office to PDF | DOCX, ODT, XLSX, ODS, PPTX, ODP to PDF             | LibreOffice                                              |
| PDF           | PDF to PNG/JPEG/TIFF; PDF text extraction          | Poppler; dedicated PDF library where appropriate         |
| Images        | PNG, JPEG, WebP, BMP, TIFF                         | ImageMagick or a vetted Rust image library               |
| Spreadsheets  | XLSX, ODS, CSV                                     | LibreOffice; format-specific libraries for limited tasks |
| Presentations | PPTX, ODP                                          | LibreOffice                                              |
| Audio/video   | MP3, WAV, FLAC, AAC, OGG, MP4, MKV, MOV, AVI, WEBM | FFmpeg                                                   |
| Archives      | ZIP, TAR, GZ, 7Z                                   | 7-Zip or platform-compatible archive libraries           |

Important limitations:

- PDF-to-DOCX is complex and may not preserve layout. Treat as experimental until quality tests pass.
- Scanned PDF requires OCR; this is a separate capability and dependency, not equivalent to text extraction.
- CSV has no universal formatting or formula model; encoding, delimiter, locale, and sheet selection matter.
- SVG and PDF can contain active or external content. Handle them as untrusted input.
- Some codecs and formats depend on the FFmpeg build and licensing configuration.

## 8. Functional requirements

| ID     | Requirement        | Priority | Acceptance criteria                                                                                 |
| ------ | ------------------ | -------- | --------------------------------------------------------------------------------------------------- |
| FR-001 | Add files          | Must     | Picker and drag/drop add valid paths without reading entire large files into UI memory              |
| FR-002 | Detect file type   | Must     | Extension and available signature checks; ambiguous types are shown as uncertain                    |
| FR-003 | List valid targets | Must     | Only tested/available conversion pairs are offered                                                  |
| FR-004 | Dependency check   | Must     | Missing/incompatible engine is reported before job start                                            |
| FR-005 | Convert file       | Must     | Job produces output or a structured failure; no false success                                       |
| FR-006 | Batch queue        | Must     | Multiple files can be queued; one failure does not silently mark all as failed                      |
| FR-007 | Progress           | Must     | Show progress when engine provides it; otherwise show indeterminate status                          |
| FR-008 | Cancel             | Should   | Cancellation terminates child process safely and cleans temporary files                             |
| FR-009 | Output naming      | Must     | Never overwrite without explicit confirmation; resolve collisions safely                            |
| FR-010 | Output validation  | Must     | Check existence, non-zero size where applicable, expected type, and parser/open test when supported |
| FR-011 | History            | Should   | Store metadata only; no file contents; user can clear history                                       |
| FR-012 | Logs               | Must     | Provide redacted, actionable diagnostics without leaking file contents                              |
| FR-013 | Offline use        | Must     | No network required for conversion once app and engines are installed                               |
| FR-014 | Settings           | Should   | Default output folder, collision policy, history retention, theme                                   |
| FR-015 | Accessibility      | Should   | Keyboard operation, focus states, labels, scalable UI                                               |

## 9. Non-functional requirements

- Reliability: deterministic job state transitions and structured errors.
- Performance: avoid loading entire media/document files into memory unnecessarily.
- Resource control: configurable concurrency; default conservative concurrency for memory-heavy engines.
- Security: no shell command construction from user-controlled strings; canonicalize and validate paths.
- Privacy: no upload, analytics, or telemetry by default.
- Portability: platform-specific engine discovery and packaging.
- Maintainability: each engine adapter implements a common interface.
- Observability: logs include job ID, engine, version, duration, exit status, and validation result.

## 10. Job lifecycle

`Queued -> Validating -> WaitingForEngine -> Running -> ValidatingOutput -> Succeeded`

Alternative terminal states:

- `Failed`
- `Cancelled`
- `SucceededWithWarnings`

A job must not transition from a terminal state back to running. Every transition should be timestamped.

## 11. UX requirements

Main screens:

1. Home / Convert: drop zone, file list, target format, convert action.
2. Queue: queued/running/completed jobs and progress.
3. Results: output path, validation state, warnings, open-folder action.
4. History: previous job metadata and repeat-conversion shortcut.
5. Settings: output path, theme, concurrency, history retention.
6. Engines: detected engine, version, source (bundled/system), status, supported capabilities.

Avoid displaying "unlimited" as a guarantee. Use "No server-imposed upload limit; practical limits depend on your device."

## 12. Success metrics

- Conversion success rate by tested pair.
- Output validation pass rate.
- Crash-free conversion sessions.
- Median and 95th percentile conversion duration by file-size band and hardware class.
- Number of actionable vs. generic errors.
- Dependency detection accuracy.
- Regression count per release.

Do not collect these metrics remotely by default. Use local test reports and opt-in bug reports.

## 13. Release gates

A conversion pair may be marked Stable only when:

- automated tests cover representative valid and invalid inputs;
- output opens in an independent reader/parser;
- visual or content comparison has been performed where meaningful;
- large-file and malformed-file behavior is tested;
- platform support is documented;
- licensing and redistribution status of the engine/build is reviewed.

## 14. Open product decisions

- Whether OCR is included in v1 or a later optional module.
- Whether PDF merge/split is MVP or post-MVP.
- Exact bundled engine set for each OS and installer size budget.
- Whether history is enabled by default and its retention period.
- Exact supported format-pair matrix after prototype testing.
