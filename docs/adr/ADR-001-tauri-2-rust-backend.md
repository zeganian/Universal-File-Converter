# ADR-001: Tauri 2 with a Rust backend

- Status: Accepted
- Date: 2025-11-04

## Context

Universal File Converter converts documents, images, media and archives
locally. The conversion engines are external programs or native
libraries, the file handling must be safe, and the application must run
without a network connection. The user interface has to feel like a
normal desktop application on Windows, Linux and macOS.

The frontend must never execute a conversion itself: it may only ask
the backend to perform a typed operation.

## Decision

The application is a Tauri 2 desktop shell with a React and TypeScript
frontend and a Rust backend.

- The Rust backend owns all privileged work: file inspection, path
  validation, engine discovery, process execution, output validation
  and settings.
- The frontend communicates through narrow, typed Tauri commands.
  Payloads are validated in Rust even when the frontend already
  validated them.
- The business rules live in `crates/ufc-core`, a crate that does not
  depend on Tauri. It can be tested without a web view or a system web
  toolkit.
- The frontend receives the list of selectable conversion pairs from
  the Rust capability registry. It never keeps its own copy.

## Consequences

- The core domain is testable in isolation and survives UI rewrites.
- The application binary is small compared with an Electron shell,
  because the web view is provided by the operating system.
- The cost is a bridge: every backend feature needs a command, a
  serialisation format and a TypeScript type.
- Tauri brings platform specific requirements (web view development
  packages on Linux, code signing on macOS and Windows) that CI has to
  install.
- Process execution in Rust is safe by construction: an executable path
  plus an argument vector, never a shell command string.

## Alternatives considered

- **Electron**: rejected. Larger binary and memory footprint, and it
  bundles a Chromium that has to be updated for every security patch.
- **Pure Rust user interface**: rejected for now. The interface is not
  the hard part of this product, and a web frontend is faster to
  iterate on.
- **A local HTTP service**: rejected. It would add a port, an
  authentication story and an unnecessary attack surface.
