# Universal File Converter

A local-first, open-source desktop application for converting documents, PDFs, images, audio, video, and archives.

**No upload by default. Offline-capable. Cross-platform.**

> The project is under active development. The list of planned formats is not a guarantee of current support. Check `SUPPORTED_FORMATS.md` for the implementation status of each conversion pair.

## Vision

Universal File Converter processes files on the user's own computer using mature conversion engines. It aims to avoid server upload limits and reduce privacy concerns associated with online converters.

There is no server-imposed upload limit, but practical file size is still constrained by available RAM, CPU, disk space, engine capabilities, and file complexity.

## Technology

- Tauri 2
- React
- TypeScript
- Rust
- LibreOffice / Pandoc for selected document workflows
- Poppler for PDF rendering and text extraction
- ImageMagick or a vetted image library for image workflows
- FFmpeg for audio/video workflows
- 7-Zip or vetted archive libraries for archive workflows

Actual engine versions and availability will be documented per release and platform.

## Planned features

- Drag-and-drop and file picker
- Single and batch conversion
- Conversion queue and progress
- Local conversion history
- Engine detection and diagnostics
- Output validation
- Windows, Linux, and macOS support

## Development status

The application foundation is in place: the Tauri shell, the typed command boundary, the domain model, the capability registry, the five screens, and the automated checks.

**One conversion pair is implemented: PNG to JPEG, with ImageMagick 7.** The pair is _experimental_: the engine, the integration tests and the output validation exist, but the release gates of `TESTING.md` have not run on Windows and macOS yet. Every other pair in the matrix is still _planned_, and the interface reports it as such instead of offering a conversion that would fail. A pair only becomes selectable after an engine adapter, an integration fixture, an output validation, and a documentation entry exist.

No conversion pair should be considered stable until it is implemented and passes the quality gates in `TESTING.md`.

## Repository documentation

- `PRD.md` — product requirements and scope
- `ARCHITECTURE.md` — application architecture
- `RULES.md` — engineering rules for contributors and AI agents
- `DESIGN.md` — visual design system
- `SUPPORTED_FORMATS.md` — conversion capability matrix
- `TESTING.md` — testing and release gates
- `SECURITY.md` — security and privacy requirements
- `docs/adr/` — architecture decision records
- `docs/ci.md` — the continuous integration, its pinned versions and the system dependencies per operating system
- `tests/` — test layers and fixtures

## Development setup

Prerequisites:

- Git
- Node.js 22.12 or newer
- pnpm 12 (`corepack enable` provides it)
- Rust stable toolchain
- Tauri 2 platform prerequisites for the operating system

Follow the official Tauri 2 prerequisites for your OS. On Debian and Ubuntu the desktop build needs the web view development packages, for example `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev` and `patchelf`. The indicator library of Ayatana replaces the original `libappindicator3-dev`; the two of them cannot be installed together. The exact list per operating system is in `docs/ci.md`.

```bash
pnpm install
pnpm check          # every check that CI runs
pnpm tauri:dev      # start the desktop application
```

### Individual commands

| Command                     | Purpose                                                            |
| --------------------------- | ------------------------------------------------------------------ |
| `pnpm dev`                  | Frontend only, in the browser.                                     |
| `pnpm tauri:dev`            | The desktop application with hot reload.                           |
| `pnpm tauri:build`          | The installable application.                                       |
| `pnpm test`                 | Frontend unit and component tests.                                 |
| `pnpm lint`                 | ESLint, including the type aware rules.                            |
| `pnpm typecheck`            | TypeScript without emitting.                                       |
| `pnpm format:check`         | Prettier, in check mode.                                           |
| `pnpm check`                | Everything above plus the Rust checks and the binding drift check. |
| `pnpm generate:types`       | Regenerate the TypeScript bindings from the Rust types.            |
| `pnpm fixtures:generate`    | Generate the synthetic test fixtures.                              |
| `cargo test -p ufc-core`    | The domain tests, without a web view.                              |
| `cargo test -p ufc-engines` | The engine adapter tests, including the real conversions.          |
| `cargo test --workspace`    | The domain and the desktop crate tests.                            |

The generated TypeScript bindings in `src/types/generated/` are committed on purpose. CI regenerates them and fails when the committed copy is stale.

Do not install or bundle every conversion engine before the engine manager and licensing review exist.

## Privacy

Conversion is designed to happen locally. The MVP should not upload files or collect telemetry. Review `SECURITY.md` before adding network-dependent features.

## Contributing

Contributions should:

- follow `RULES.md`;
- add or update tests;
- update `SUPPORTED_FORMATS.md`;
- include engine/license notes when adding dependencies;
- document known conversion limitations.

## Licensing

The repository's own license must be selected before public release. Third-party engines retain their own licenses; see the notices distributed with each engine. Do not assume one license covers the entire application and all bundled dependencies.
