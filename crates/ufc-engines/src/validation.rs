//! Output validation.
//!
//! The output of a conversion is verified with a reader that is
//! independent of the engine that produced it. An exit code of zero is
//! never treated as proof that the output is usable
//! (`ARCHITECTURE.md` section 8, `ADR-005`).

use std::path::Path;

use ufc_core::domain::conversion::ValidationLevel;
use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use ufc_core::domain::format::FormatId;

/// The first bytes of every JPEG file.
const JPEG_SIGNATURE: [u8; 3] = [0xff, 0xd8, 0xff];

/// The first bytes of every PNG file.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// Validates a produced file against the format the user asked for.
///
/// The reader is independent of the engine that produced the file, so
/// a successful exit code of the engine is never treated as proof
/// that the output is usable (`ARCHITECTURE.md` section 8).
///
/// # Errors
/// Returns an error when the format has no validation strategy, or
/// when the file does not match the expected format.
pub fn validate_image(path: &Path, target_format: &FormatId) -> DomainResult<ValidationLevel> {
    match target_format.as_str() {
        "jpeg" => validate_jpeg(path),
        "png" => validate_png(path),
        other => Err(ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            format!("There is no output validation for the '{other}' format."),
        )),
    }
}

/// Validates a produced JPEG file.
///
/// The levels are reached in order:
///
/// - the file exists and is a regular file (`Exists`),
/// - the file is not empty and starts with the JPEG signature
///   (`NonEmptySignature`),
/// - the file can be decoded by an independent reader (`Parsed`).
///
/// # Errors
/// Returns an error when the file is missing, empty, not a JPEG or
/// cannot be decoded.
pub fn validate_jpeg(path: &Path) -> DomainResult<ValidationLevel> {
    validate_with_signature(path, &JPEG_SIGNATURE, "JPEG")
}

/// Validates a produced PNG file.
///
/// The levels are reached in order:
///
/// - the file exists and is a regular file (`Exists`),
/// - the file is not empty and starts with the PNG signature
///   (`NonEmptySignature`),
/// - the file can be decoded by an independent reader (`Parsed`).
///
/// # Errors
/// Returns an error when the file is missing, empty, not a PNG or
/// cannot be decoded.
pub fn validate_png(path: &Path) -> DomainResult<ValidationLevel> {
    validate_with_signature(path, &PNG_SIGNATURE, "PNG")
}

/// Validates a file against the signature of a format.
///
/// The name of the format is used in the error messages, so that the
/// user learns what the produced file actually is.
fn validate_with_signature(
    path: &Path,
    signature: &[u8],
    format_name: &str,
) -> DomainResult<ValidationLevel> {
    let metadata = std::fs::metadata(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::MissingOutput,
            "The conversion did not produce an output file.",
        )
        .with_detail(source.to_string())
    })?;

    if !metadata.is_file() {
        return Err(ConversionError::new(
            ConversionErrorCode::MissingOutput,
            "The conversion did not produce a regular output file.",
        ));
    }

    if metadata.len() == 0 {
        return Err(ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            "The produced file is empty.",
        ));
    }

    if !has_signature(path, signature)? {
        return Err(ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            format!("The produced file is not a {format_name} image."),
        ));
    }

    // The independent reader: the file has to be decodable without the
    // engine that wrote it.
    image::open(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            format!("The produced file could not be read as a {format_name} image."),
        )
        .with_detail(source.to_string())
    })?;

    Ok(ValidationLevel::Parsed)
}

/// Returns `true` when the file starts with the given signature.
fn has_signature(path: &Path, signature: &[u8]) -> DomainResult<bool> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::MissingOutput,
            "The produced file could not be opened.",
        )
        .with_detail(source.to_string())
    })?;

    let mut header = vec![0_u8; signature.len()];
    let read = file.read(&mut header).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::MissingOutput,
            "The produced file could not be read.",
        )
        .with_detail(source.to_string())
    })?;

    Ok(read == signature.len() && header == signature)
}

/// Returns the dimensions of an image.
pub fn dimensions(path: &Path) -> DomainResult<(u32, u32)> {
    let image = image::open(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            "The image could not be read.",
        )
        .with_detail(source.to_string())
    })?;

    Ok((image.width(), image.height()))
}

/// Compares the dimensions of the source and of the output.
///
/// A conversion that changes the size of the image is reported as a
/// failed validation: the user asked for a format change, not for a
/// resize.
pub fn compare_dimensions(source: &Path, output: &Path) -> DomainResult<()> {
    let (source_width, source_height) = dimensions(source)?;
    let (output_width, output_height) = dimensions(output)?;

    if source_width != output_width || source_height != output_height {
        return Err(ConversionError::new(
            ConversionErrorCode::OutputValidationFailed,
            format!(
                "The converted image is {output_width}x{output_height}, but the source is {source_width}x{source_height}."
            ),
        ));
    }

    Ok(())
}

/// Returns `true` when a PNG image carries an alpha channel.
///
/// The check reads the colour type of the PNG header. It is used to
/// warn the user that transparency is lost when the image becomes a
/// JPEG (`SUPPORTED_FORMATS.md`, "Images").
pub fn png_has_alpha(path: &Path) -> bool {
    use std::io::Read;

    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut header = [0_u8; 26];
    if file.read_exact(&mut header).is_err() {
        return false;
    }

    // The signature, the chunk length and the chunk type occupy the
    // first sixteen bytes. The colour type is the byte at index 25.
    if header[..8] != PNG_SIGNATURE {
        return false;
    }

    // 4: greyscale with alpha, 6: truecolour with alpha.
    matches!(header[25], 4 | 6)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};

    fn write_jpeg(path: &Path, width: u32, height: u32) {
        let image = RgbImage::from_pixel(width, height, image::Rgb([10, 20, 30]));
        image
            .save_with_format(path, ImageFormat::Jpeg)
            .expect("the fixture is written");
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        let image = RgbImage::from_pixel(width, height, image::Rgb([10, 20, 30]));
        image
            .save_with_format(path, ImageFormat::Png)
            .expect("the fixture is written");
    }

    #[test]
    fn a_valid_jpeg_reaches_the_parsed_level() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("out.jpg");
        write_jpeg(&path, 4, 4);

        let level = validate_jpeg(&path).expect("the file is valid");

        assert_eq!(level, ValidationLevel::Parsed);
        assert_eq!(level.level(), 2);
    }

    #[test]
    fn a_missing_file_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("missing.jpg");

        let error = validate_jpeg(&path).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::MissingOutput);
    }

    #[test]
    fn an_empty_file_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("empty.jpg");
        std::fs::write(&path, b"").expect("the file is written");

        let error = validate_jpeg(&path).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
    }

    #[test]
    fn a_png_is_not_accepted_as_a_jpeg() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("actually-a-png.jpg");
        write_png(&path, 4, 4);

        let error = validate_jpeg(&path).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
        assert!(error.message.contains("JPEG"));
    }

    #[test]
    fn a_truncated_jpeg_is_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("truncated.jpg");
        write_jpeg(&path, 16, 16);
        let bytes = std::fs::read(&path).expect("the file is read");
        std::fs::write(&path, &bytes[..bytes.len() / 2]).expect("the file is written");

        let error = validate_jpeg(&path).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
    }

    #[test]
    fn equal_dimensions_are_accepted() {
        let dir = tempfile::tempdir().expect("temp dir");
        let source = dir.path().join("source.png");
        let output = dir.path().join("output.jpg");
        write_png(&source, 8, 8);
        write_jpeg(&output, 8, 8);

        compare_dimensions(&source, &output).expect("the sizes match");
    }

    #[test]
    fn different_dimensions_are_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let source = dir.path().join("source.png");
        let output = dir.path().join("output.jpg");
        write_png(&source, 8, 8);
        write_jpeg(&output, 4, 4);

        let error = compare_dimensions(&source, &output).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
        assert!(error.message.contains("8x8"));
    }

    #[test]
    fn a_valid_png_reaches_the_parsed_level() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("out.png");
        write_png(&path, 4, 4);

        let level = validate_png(&path).expect("the file is valid");

        assert_eq!(level, ValidationLevel::Parsed);
    }

    #[test]
    fn a_jpeg_is_not_accepted_as_a_png() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("actually-a-jpeg.png");
        write_jpeg(&path, 4, 4);

        let error = validate_png(&path).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
        assert!(error.message.contains("PNG"));
    }

    #[test]
    fn the_validation_follows_the_requested_format() {
        let dir = tempfile::tempdir().expect("temp dir");
        let png = dir.path().join("image.png");
        let jpeg = dir.path().join("image.jpg");
        write_png(&png, 4, 4);
        write_jpeg(&jpeg, 4, 4);

        let png_format = FormatId::new("png").expect("valid");
        let jpeg_format = FormatId::new("jpeg").expect("valid");

        assert_eq!(
            validate_image(&png, &png_format).expect("the png is valid"),
            ValidationLevel::Parsed
        );
        assert_eq!(
            validate_image(&jpeg, &jpeg_format).expect("the jpeg is valid"),
            ValidationLevel::Parsed
        );
        assert!(validate_image(&png, &jpeg_format).is_err());
        assert!(validate_image(&jpeg, &png_format).is_err());
    }

    #[test]
    fn a_format_without_a_validation_strategy_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("image.png");
        write_png(&path, 4, 4);

        let pdf = FormatId::new("pdf").expect("valid");

        let error = validate_image(&path, &pdf).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputValidationFailed);
    }
}
