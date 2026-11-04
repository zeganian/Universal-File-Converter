# Engineering Rules for AI Coding Agents

## Universal File Converter

These rules are mandatory for Claude Code and other contributors.

## 1. Product rules

- Build a cross-platform desktop app for Windows, Linux, and macOS.
- Use Tauri 2, React, TypeScript, and Rust.
- The product is local-first and offline-capable.
- Reliability is more important than adding many format labels.
- Never claim a format pair is supported until it is implemented and tested.
- Never claim "unlimited file size". Explain that device resources and engine limits apply.
- No cloud upload, account, telemetry, or network conversion in the MVP.

## 2. Language and communication

- User-facing UI language: English initially; keep strings ready for localization.
- Documentation: English, with clear and concise technical writing.
- Code identifiers, comments, logs, and commit messages: English.
- Explain architectural changes and trade-offs in Indonesian when reporting to the project owner.

## 3. Technology rules

- Frontend: React + TypeScript.
- Desktop framework: Tauri 2.
- Backend: stable Rust toolchain.
- Use package versions compatible with the current Tauri 2 release and lock dependencies.
- Use strict TypeScript settings.
- Prefer small, focused Rust modules and typed errors.
- Use established libraries and external engines where appropriate; do not reinvent complex format parsers.

## 4. Code quality

- Keep functions small and single-purpose.
- Avoid `any` in TypeScript; use explicit types and runtime validation.
- Avoid `unwrap()` and `expect()` in production Rust paths. Handle errors explicitly.
- Do not swallow errors or return success after partial failure.
- Do not use global mutable state unless justified and documented.
- Do not duplicate conversion logic across UI components.
- Keep UI state separate from conversion orchestration.
- Add tests with every new conversion capability.
- Run formatting, linting, type checks, and tests before claiming a task is complete.

## 5. Process and security rules

- Never build shell command strings from user input.
- Use executable path plus argument vector.
- Do not execute arbitrary paths or commands supplied by the frontend.
- Validate input paths, output paths, file types, and conversion options in Rust.
- Never overwrite the source file.
- Never overwrite an existing output without the selected policy and user consent.
- Use unique temporary directories per job.
- Clean temporary files on success, failure, and cancellation.
- Apply output/log size limits.
- Do not log file contents, extracted document text, credentials, or sensitive metadata.
- Treat all input files as untrusted.
- Use least-privilege Tauri capabilities and narrow command permissions.
- Do not add network permissions unless a documented feature requires them.

## 6. Conversion reliability rules

- Maintain an explicit source-format/target-format capability registry.
- UI options must be derived from the registry and detected engine capabilities.
- Check engine presence and version before starting.
- Capture engine exit code and sanitized stderr.
- Validate output independently; exit code zero alone is not enough.
- Use determinate progress only when the engine provides reliable progress.
- Distinguish warning, partial success, cancellation, and failure.
- Preserve original input files.
- Add fixtures for valid, malformed, empty, large, and edge-case files where practical.
- Record engine name and version in each result.

## 7. Dependency and licensing rules

- Do not bundle an engine until its license, build configuration, notices, and redistribution obligations are reviewed.
- Preserve third-party license and attribution files.
- Do not assume all builds of FFmpeg or ImageMagick have identical licensing/features.
- Provide system-engine fallback only for tested versions.
- Document required runtime libraries and installation guidance per OS.
- Never download binaries at runtime in the MVP.

## 8. UI/UX rules

- Show unsupported conversions as unavailable, not as selectable actions that fail later.
- Show why an engine is missing and how to resolve it.
- Never fake progress.
- Keep errors understandable, with optional technical details.
- Confirm destructive actions and overwrites.
- Support keyboard navigation and visible focus.
- Show the selected output folder and collision policy.

## 9. Git workflow

- Use conventional commits where practical: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `build:`.
- One logical change per commit.
- Do not commit generated binaries, user files, secrets, or local environment paths.
- Update docs and supported-format matrix when capabilities change.
- Keep lockfiles committed for reproducible builds.

## 10. Definition of done

A task is done only when:

1. Implementation matches the requirement.
2. Types and errors are handled.
3. Unit/integration tests are added or updated.
4. Formatting, lint, typecheck, and relevant tests pass.
5. Security and path handling have been reviewed.
6. Documentation and supported-format matrix are updated.
7. The agent reports files changed, commands run, test results, and known limitations.

## 11. Agent behavior

- Inspect the repository before editing.
- Read PRD, ARCHITECTURE, DESIGN, SECURITY, TESTING, and SUPPORTED_FORMATS first.
- Do not replace working architecture or dependencies without explaining why.
- Do not invent APIs or assume engine behavior; inspect official documentation and installed versions.
- Ask before making irreversible changes or broad refactors.
- Work in small phases and stop at the end of each milestone for review.
