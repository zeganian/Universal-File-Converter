# Architecture Decision Records

This directory holds the architecture decision records of Universal File
Converter, as required by `ARCHITECTURE.md` section 14.

| ADR                                            | Title                                      | Status   |
| ---------------------------------------------- | ------------------------------------------ | -------- |
| [ADR-001](ADR-001-tauri-2-rust-backend.md)     | Tauri 2 with a Rust backend                | Accepted |
| [ADR-002](ADR-002-hybrid-engine-discovery.md)  | Hybrid bundled and system engine discovery | Accepted |
| [ADR-003](ADR-003-local-only-no-telemetry.md)  | Local-only processing without telemetry    | Accepted |
| [ADR-004](ADR-004-capability-registry.md)      | Conversion pair capability registry        | Accepted |
| [ADR-005](ADR-005-output-validation-levels.md) | Output validation levels                   | Accepted |

## How to write a new ADR

1. Copy the structure of an existing record: status, date, context,
   decision, consequences, alternatives.
2. Use a sequential file name: `ADR-NNN-short-title.md`.
3. Keep the record focused on one decision. If a change reverses an
   earlier record, write a new record and mark the old one as
   superseded instead of editing its decision.
4. Reference the record from `ARCHITECTURE.md` and from the pull
   request that introduces it.
