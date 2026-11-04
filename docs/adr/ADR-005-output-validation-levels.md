# ADR-005: Output validation levels

- Status: Accepted
- Date: 2025-11-04

## Context

An engine that exits with code zero has produced a file, but that file
is not necessarily usable. It can be empty, truncated, in the wrong
format, or a file that only the producing engine can read. Reporting
such a run as a success is the classic false success, and it is exactly
what the product rules forbid.

Not every format can be verified in the same way. A PDF can be opened
by an independent parser, a PNG can be checked by its signature and its
dimensions, but a text format has no signature at all.

## Decision

Every conversion result reports the strongest validation level that was
reached. The levels are ordered and comparable:

| Level | Name                | Meaning                                                                  |
| ----- | ------------------- | ------------------------------------------------------------------------ |
| 0     | `Exists`            | The output exists and is a regular file.                                 |
| 1     | `NonEmptySignature` | The output is not empty and matches the expected signature or container. |
| 2     | `Parsed`            | The output can be parsed or opened by an independent reader.             |
| 3     | `Semantic`          | The output matches the semantic or visual expectations of the fixture.   |

- The exit code of the engine is one signal among several and never
  the sole success criterion.
- A result that only reaches level 0 is not reported as a plain
  success; it is reported with the level that was actually achieved,
  and a failed validation produces `OutputValidationFailed`.
- Not every format supports every level. The registry entry of a pair
  names the validation strategy that is used for it.
- Level 3 requires a fixture with known expectations and is the reason
  a pair can only be promoted to `Stable` after a comparison has been
  performed.

## Consequences

- The user interface can state honestly how far the output was checked.
- A successful conversion carries a weaker claim when only levels 0 or
  1 were possible, and the interface has to show that difference.
- Validation needs an independent reader, which means additional test
  dependencies per format family.
- The validation level belongs in the job history, so that a failure
  can be reproduced and compared against an earlier run.

## Alternatives considered

- **Trust the exit code**: rejected. It is the single most common
  source of false success.
- **Always validate at the highest level**: rejected. Several formats
  cannot be checked semantically, and pretending otherwise would be
  dishonest.
- **Do not report the level, only the outcome**: rejected. The user
  could then not tell a fully verified conversion from a barely
  checked one.
