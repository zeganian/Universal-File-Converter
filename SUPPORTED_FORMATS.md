# Supported Formats and Conversion Matrix

## Universal File Converter

This document is the source of truth for user-visible conversion capabilities.

**Important:** A format being recognized does not mean every conversion involving it is supported. Each source-to-target pair must be implemented, tested, and enabled in the capability registry before being advertised.

## Current implementation status

**One conversion pair is implemented: PNG to JPEG, with ImageMagick 7.** It is registered in the capability registry with the status `Experimental`, so the interface offers it as soon as ImageMagick 7 is detected on the device. When the engine is missing, the pair is shown as unavailable together with the installation guidance, instead of failing later.

Every other pair of the matrix below is still `Planned`. The Convert screen shows honestly that no other conversion is available, and the Engines screen reports which engines were detected.

A row only leaves the `Planned` state when an engine adapter, an integration fixture, an output validation and a documentation entry exist.

## Status definitions

- `Planned`: not implemented.
- `Experimental`: implemented but quality/platform coverage is incomplete.
- `Stable`: release-gated tests pass on the listed platforms.
- `Unavailable`: engine or required capability is not available.
- `Not supported`: intentionally unsupported.

A pair keeps its maturity status and, independently of it, is offered to the user only when its engine is available on the machine. The registry therefore distinguishes "implemented but the engine is missing" from "not implemented yet"; the interface shows the first as unavailable with installation guidance and the second as not yet supported.

## Initial candidate matrix

| Source | Target | Candidate engine                 | Initial status | Notes                                         |
| ------ | ------ | -------------------------------- | -------------- | --------------------------------------------- |
| DOCX   | PDF    | LibreOffice                      | Planned        | Verify fonts, page breaks, tables             |
| ODT    | PDF    | LibreOffice                      | Planned        | Verify layout                                 |
| XLSX   | PDF    | LibreOffice                      | Planned        | Sheet selection and print areas               |
| ODS    | PDF    | LibreOffice                      | Planned        | Sheet selection and print areas               |
| PPTX   | PDF    | LibreOffice                      | Planned        | Fonts, animations are not represented in PDF  |
| ODP    | PDF    | LibreOffice                      | Planned        | Verify layout                                 |
| PDF    | PNG    | Poppler                          | Planned        | Page range and DPI options                    |
| PDF    | JPEG   | Poppler                          | Planned        | Page range and quality options                |
| PDF    | TXT    | Poppler                          | Planned        | Text extraction only; scanned pages need OCR  |
| PNG    | JPEG   | ImageMagick / Rust image library | Experimental   | Transparency handling must be explicit        |
| JPEG   | PNG    | ImageMagick / Rust image library | Planned        | Lossy source remains lossy                    |
| PNG    | WebP   | ImageMagick / Rust image library | Planned        | Quality option                                |
| WebP   | PNG    | ImageMagick / Rust image library | Planned        | Animated WebP requires separate handling      |
| WAV    | MP3    | FFmpeg                           | Planned        | Codec availability varies by build            |
| FLAC   | MP3    | FFmpeg                           | Planned        | Lossy target                                  |
| MP4    | MKV    | FFmpeg                           | Planned        | Container remux vs re-encode must be explicit |
| MP4    | WebM   | FFmpeg                           | Planned        | Codec conversion generally required           |
| ZIP    | 7Z     | 7-Zip                            | Planned        | Preserve paths and metadata where possible    |
| TAR.GZ | ZIP    | 7-Zip / archive library          | Planned        | Nested archive behavior documented            |

## Implemented pairs

### PNG to JPEG

- **Engine:** ImageMagick 7 (`magick`), system installation. Tested with 7.1.2-34 on Linux x64 and with the pinned 7.1.2-35 in the continuous integration. The adapter accepts versions from 7.1.0 up to 7.99.99 and reports an incompatible version with installation guidance otherwise.
- **Status:** `Experimental`. The pair is implemented and tested, but the release gates of `TESTING.md` section 5 have not run on Windows and macOS yet.
- **Options:** JPEG quality (1–100, default 92) and background colour for the alpha channel (default `white`).
- **Output validation:** level 2 (`Parsed`). The output is checked with an independent JPEG decoder, not only by the exit code of the engine. The dimensions of the source and the output are compared.
- **Fixtures and test cases:** `crates/ufc-engines/tests/conversion_png_to_jpeg.rs` covers the happy path, the unchanged source file, the transparency warning, an existing output, the renaming policy, a malformed source, the quality option and the repository fixture. The tests skip themselves when ImageMagick 7 is not installed.
- **Limitations:** JPEG has no alpha channel; transparent areas are flattened onto the selected background and a warning is shown. JPEG is lossy. The source file is never modified.
- **Maximum tested file size:** 64 × 64 pixels in the automated tests. This is a test result, not a product limit.
- **License notes:** ImageMagick is used as a system dependency and is not bundled. Its license applies to the installation of the user, not to this repository.

## Quality requirements

For each stable pair, record:

- OS and CPU architecture tested.
- Source fixtures and test case IDs.
- Engine name and exact tested version.
- Output validation level.
- Known fidelity limitations.
- Supported options and defaults.
- Maximum tested file size (a test result, not a product limit).
- License/redistribution notes.

## Registry rules

The frontend must obtain available pairs from the Rust capability registry. Do not maintain a separate hardcoded list in React.

The registry should include:

- source format;
- target format;
- engine ID;
- minimum/maximum tested engine version;
- status;
- options schema;
- platform availability;
- validation strategy;
- user-facing limitations.

## Special cases

### PDF to DOCX

Treat as experimental. PDF is a page-description format, not a full editable document model. Layout reconstruction may be approximate. Do not label it stable without representative quality tests.

### Scanned PDF and OCR

Text extraction does not perform OCR. OCR should be a separate optional engine/capability with language packs, privacy notes, and separate tests.

### CSV

CSV does not preserve spreadsheet formulas, styles, charts, or multiple sheets. Make delimiter, encoding, and locale behavior explicit.

### Images

JPEG does not support transparency. The UI must require or clearly apply a background color when converting transparent images to JPEG.

### Audio/video

Container remuxing is different from transcoding. Display whether streams are copied or re-encoded and expose codec options only when supported by the installed FFmpeg build.

### Archives

Prevent path traversal during extraction. Do not extract archive entries outside the selected destination. Consider symlinks, nested archives, decompression bombs, and resource limits.
