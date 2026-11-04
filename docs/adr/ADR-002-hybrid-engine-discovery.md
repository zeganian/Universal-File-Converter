# ADR-002: Hybrid bundled and system engine discovery

- Status: Accepted
- Date: 2025-11-04

## Context

Conversions are performed by mature, format specific engines such as
LibreOffice, FFmpeg, Poppler, ImageMagick and 7-Zip. Every one of them
has its own licence, its own build configuration and its own
installation procedure on each operating system.

Bundling every engine would produce a very large installer and would
require a licence review for each build. Requiring the user to install
everything manually would make the first run frustrating. Some engines
also differ significantly between builds, for example the codecs and
licensing configuration of FFmpeg.

## Decision

Engines are discovered at runtime with a hybrid strategy, in this
order:

1. Prefer a bundled engine when one is present and approved for
   redistribution.
2. Otherwise detect a supported system installation.
3. Verify the engine version against the tested compatibility range.
4. When neither is usable, report `MissingDependency` together with
   platform specific installation guidance.

Additional rules:

- The selected engine, its source (`bundled` or `system`) and its exact
  version are shown in the job details.
- The application never silently switches to a different engine when
  that could change the output quality or behaviour.
- No binary is downloaded at runtime in the MVP.
- A bundled engine may only be added after its licence, build
  configuration, notices and redistribution obligations have been
  reviewed, and only together with the platform specific installation
  documentation.

## Consequences

- The application stays small and works offline once the engines it
  uses are installed.
- Some pairs are unavailable on a given machine, and the user interface
  has to explain why instead of offering the conversion.
- Detection, version parsing and version ranges become testable units
  per platform.
- Engine versions must be recorded in the job diagnostics so that a
  dependency upgrade can be compared against previous results.

## Alternatives considered

- **Bundle every engine**: rejected for now. The installer size and
  the licence obligations are too large relative to the benefit.
- **System engines only**: rejected. It would leave the application
  unusable until the user installs a long list of programs.
- **Download engines on demand**: rejected for the MVP. It needs a
  network permission, a download mechanism and a verification scheme,
  all of which conflict with the offline-first product promise.
