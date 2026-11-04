# Claude Code Project Instructions

Before making changes, read these files in order:
1. `PRD.md`
2. `ARCHITECTURE.md`
3. `RULES.md`
4. `DESIGN.md`
5. `SECURITY.md`
6. `SUPPORTED_FORMATS.md`
7. `TESTING.md`

## Working method

- First inspect the existing repository, package files, Rust modules, and git status.
- Summarize the current state and propose a small implementation plan.
- Implement only the requested milestone.
- Do not scaffold the entire product in one pass.
- Ask before changing core architecture, adding network behavior, or bundling third-party binaries.
- At the end, report changed files, commands run, test results, limitations, and next steps.

## Initial milestone

Build only the application foundation:
- Tauri 2 + React + TypeScript scaffold.
- Basic navigation and empty Convert screen.
- Typed Rust command boundary.
- Domain types for conversion requests, job states, progress, and errors.
- Capability registry with no falsely advertised conversion pairs.
- Engine status page with placeholder/unavailable state until real detection exists.
- Unit tests for job state transitions and registry behavior.
- CI checks for formatting, linting, type checking, and Rust tests where practical.

Do not implement all conversion engines in the initial milestone.

## Completion rule

Do not report a conversion capability as implemented unless a real engine adapter, integration test fixture, output validation, and documentation entry exist.
