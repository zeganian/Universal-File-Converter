# Testing and Quality Strategy

## Universal File Converter

## 1. Testing objectives

Prove that conversions produce usable output, failures are understandable, and user data is not damaged or unexpectedly exposed.

## 2. Test layers

### Unit tests

- Format detection and extension/signature mismatch.
- Capability registry lookup.
- Conversion option validation.
- Job state transitions.
- Output filename collision policy.
- Path validation and temporary directory generation.
- Error mapping and log redaction.

### Adapter tests

- Engine discovery and version parsing.
- Argument construction.
- Exit code handling.
- Progress parsing.
- Cancellation and process cleanup.
- Missing executable and incompatible version behavior.

### Integration tests

- Real conversion using small known fixtures.
- Output existence and signature.
- Output opens with an independent reader/parser.
- Batch conversion with mixed success/failure.
- Output collision behavior.
- Temporary file cleanup.

### End-to-end tests

- Add files through picker/drop.
- Select a valid target.
- Start and cancel a job.
- Inspect success/failure details.
- Open output folder.
- Clear history.

## 3. Fixture categories

Maintain non-sensitive, redistributable test fixtures:

- valid minimal file;
- complex layout file;
- malformed/corrupt file;
- zero-byte file;
- large file;
- Unicode filename;
- long path;
- filename with spaces and shell metacharacters;
- file with extension/content mismatch;
- encrypted/password-protected file where relevant;
- file with unusual metadata or embedded resources.

Do not commit copyrighted or private user documents as fixtures. Generate synthetic fixtures or use files with suitable licenses.

## 4. Reliability checks

For every conversion:

1. Verify process start and termination.
2. Capture exit status.
3. Check expected output path.
4. Check file type/signature.
5. Check non-zero size where appropriate.
6. Parse/open output independently where possible.
7. Compare expected content or visual output for fixtures.
8. Record warnings and validation level.

## 5. Cross-platform matrix

Run CI on:

- Windows x64;
- Linux x64 (and ARM64 if officially targeted);
- macOS Apple Silicon and Intel if supported.

Test both:

- bundled engine path;
- system engine path;
- missing engine path.

Do not mark a conversion pair stable on an OS until its engine and output tests pass there.

## 6. Large-file and resource tests

Use increasing file-size bands, for example small, medium, large, and stress. Record:

- input size;
- output size;
- peak memory if measurable;
- duration;
- CPU usage if available;
- free disk space before/after;
- cancellation response time.

Avoid arbitrary hard-coded limits unless required for safety. Provide clear disk-space and resource errors.

## 7. Regression suite

Every bug report should become a regression test when a reproducible fixture can be created safely. Keep engine versions recorded so a dependency upgrade can be compared.

## 8. Release quality gates

- Format, lint, typecheck, and unit tests pass.
- Integration tests pass for all stable pairs on supported platforms.
- No known data-loss or path-traversal defects.
- Installer includes required license notices.
- Missing-dependency flows are tested.
- Release notes list engine versions and known limitations.
