# ADR-003: Local-only processing without telemetry

- Status: Accepted
- Date: 2025-11-04

## Context

The product promise is that files stay on the user's device. The
preprocessing of online converters is one of the main reasons users
avoid them, and documents are frequently confidential.

The application also has to work without a network connection, and the
Tauri capability set is part of the security model: every permission
that is not needed is a liability.

## Decision

- Files are processed locally. No file content ever leaves the device.
- The application has no telemetry and no analytics.
- No network permission is requested. The capability file
  `src-tauri/capabilities/default.json` contains no network scope.
- Local diagnostics stay on the device. A bug report is opt-in and the
  user attaches it manually.
- The optional local history stores metadata only: job identifier,
  format pair, timestamp, duration, engine and version, status and
  validation level. It never stores file contents, extracted text or
  full source paths, and it is disabled by default.
- Any future feature that transmits data needs a separate decision
  record, an explicit user consent and a documentation update.

## Consequences

- The application can be installed, used and verified without a network
  connection.
- There is no usage analytics, so the success metrics in `PRD.md`
  section 12 are measured with local test reports only.
- No crash reports arrive automatically, which means issues are found
  through user reports and the release test matrix.
- The absence of network permissions keeps the attack surface small and
  makes the security review simpler.

## Alternatives considered

- **Opt-in anonymous telemetry**: rejected for now. The value does not
  justify the permanent capability and the privacy explanation that the
  product would need.
- **Optional cloud conversion**: rejected. It contradicts the product
  promise and is out of scope for the MVP.
