# ADR-004: Conversion pair capability registry

- Status: Accepted
- Date: 2025-11-04

## Context

A converter is often judged by the number of formats it lists. That
number is misleading: a file format being recognised does not mean that
every conversion involving it works, and a conversion that fails after
the user waited for it is worse than an option that was never offered.

The product rules require that only tested pairs are advertised, and
that the list of selectable formats is derived from a single source of
truth.

## Decision

The Rust capability registry in `crates/ufc-core/src/domain/registry.rs`
is the single source of truth for conversion capabilities.

A pair is offerable only when three independent conditions hold:

1. its status is `Stable` or `Experimental`,
2. it is implemented, and
3. the engine it needs is available on this machine.

The registry entry records the source format, the target format, the
engine, the tested version range, the maturity status, the validation
strategy, the user facing limitations and the computed availability.

- The frontend obtains the list of selectable targets from the
  `get_capabilities` command. It never keeps its own hardcoded list.
- `SUPPORTED_FORMATS.md` documents the matrix and uses the same status
  vocabulary as the registry.
- A pair is only promoted to `Stable` when the release gates in
  `PRD.md` section 13 pass: automated tests for valid and invalid
  input, an independent reader for the output, a visual or content
  comparison, large and malformed file behaviour, documented platform
  support and a reviewed licence position.
- Until an engine adapter, an integration fixture, an output validation
  and a documentation entry exist, the pair stays unavailable. The
  registry starts empty and stays empty until that is true.

## Consequences

- The user interface cannot offer a conversion that the backend cannot
  perform.
- Adding a format label is no longer a one-line change: it requires an
  adapter, tests, validation and documentation.
- The registry and the documentation can drift apart, so both have to
  be updated in the same change.
- Users see fewer options, but every option that is shown works.

## Alternatives considered

- **A list in the frontend**: rejected. It would inevitably disagree
  with the backend and would advertise pairs that cannot be performed.
- **Discovering pairs at runtime from the engine**: rejected as the
  single mechanism. The engine can describe what it is capable of, but
  it cannot know whether the pair passed the product's quality gates.
  It remains useful as an additional source for the detection step.
