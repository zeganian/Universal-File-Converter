# Universal File Converter

A local-first, open-source desktop application for converting documents, PDFs, images, audio, video, and archives.

**No upload by default. Offline-capable. Cross-platform.**

> The project is under active development. The list of planned formats is not a guarantee of current support. Check `SUPPORTED_FORMATS.md` for the implementation status of each conversion pair.

## Vision

Universal File Converter processes files on the user's own computer using mature conversion engines. It aims to avoid server upload limits and reduce privacy concerns associated with online converters.

There is no server-imposed upload limit, but practical file size is still constrained by available RAM, CPU, disk space, engine capabilities, and file complexity.

## Planned technology

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

Early planning / foundation stage. No conversion pair should be considered stable until it is implemented and passes the quality gates in `TESTING.md`.

## Repository documentation

- `PRD.md` — product requirements and scope
- `ARCHITECTURE.md` — application architecture
- `RULES.md` — engineering rules for contributors and AI agents
- `DESIGN.md` — visual design system
- `SUPPORTED_FORMATS.md` — conversion capability matrix
- `TESTING.md` — testing and release gates
- `SECURITY.md` — security and privacy requirements

## Development setup

Prerequisites:
- Git
- Node.js LTS and the package manager selected by the repository
- Rust stable toolchain
- Tauri 2 platform prerequisites for the operating system

Follow the official Tauri 2 prerequisites for your OS. The exact commands will be added after the initial scaffold and package manager are selected.

Typical scaffold direction:
1. Create a Tauri 2 application with React and TypeScript.
2. Confirm the empty application builds on each target OS.
3. Add the typed Rust command boundary.
4. Implement the conversion registry and one end-to-end conversion pair.
5. Add tests before expanding format coverage.

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
