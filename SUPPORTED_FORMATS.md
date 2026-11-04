# Supported Formats and Conversion Matrix
## Universal File Converter

This document is the source of truth for user-visible conversion capabilities.

**Important:** A format being recognized does not mean every conversion involving it is supported. Each source-to-target pair must be implemented, tested, and enabled in the capability registry before being advertised.

## Status definitions

- `Planned`: not implemented.
- `Experimental`: implemented but quality/platform coverage is incomplete.
- `Stable`: release-gated tests pass on the listed platforms.
- `Unavailable`: engine or required capability is not available.
- `Not supported`: intentionally unsupported.

## Initial candidate matrix

| Source | Target | Candidate engine | Initial status | Notes |
|---|---|---|---|---|
| DOCX | PDF | LibreOffice | Planned | Verify fonts, page breaks, tables |
| ODT | PDF | LibreOffice | Planned | Verify layout |
| XLSX | PDF | LibreOffice | Planned | Sheet selection and print areas |
| ODS | PDF | LibreOffice | Planned | Sheet selection and print areas |
| PPTX | PDF | LibreOffice | Planned | Fonts, animations are not represented in PDF |
| ODP | PDF | LibreOffice | Planned | Verify layout |
| PDF | PNG | Poppler | Planned | Page range and DPI options |
| PDF | JPEG | Poppler | Planned | Page range and quality options |
| PDF | TXT | Poppler | Planned | Text extraction only; scanned pages need OCR |
| PNG | JPEG | ImageMagick / Rust image library | Planned | Transparency handling must be explicit |
| JPEG | PNG | ImageMagick / Rust image library | Planned | Lossy source remains lossy |
| PNG | WebP | ImageMagick / Rust image library | Planned | Quality option |
| WebP | PNG | ImageMagick / Rust image library | Planned | Animated WebP requires separate handling |
| WAV | MP3 | FFmpeg | Planned | Codec availability varies by build |
| FLAC | MP3 | FFmpeg | Planned | Lossy target |
| MP4 | MKV | FFmpeg | Planned | Container remux vs re-encode must be explicit |
| MP4 | WebM | FFmpeg | Planned | Codec conversion generally required |
| ZIP | 7Z | 7-Zip | Planned | Preserve paths and metadata where possible |
| TAR.GZ | ZIP | 7-Zip / archive library | Planned | Nested archive behavior documented |

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
