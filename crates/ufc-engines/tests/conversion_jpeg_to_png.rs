//! Integration test for the JPEG to PNG pair.
//!
//! The test performs a real conversion with ImageMagick and verifies
//! the result with an independent reader. It is skipped when
//! ImageMagick is not installed, because the application must work
//! without any engine (`ADR-002`).

use std::path::{Path, PathBuf};

use ufc_core::domain::conversion::{ConversionOptions, ConversionRequest, OverwritePolicy};
use ufc_core::domain::format::from_extension;
use ufc_core::domain::job::JobStatus;
use ufc_engines::conversion::{ConversionService, NullObserver};
use ufc_engines::engines::imagemagick::ImageMagickAdapter;
use ufc_engines::engines::NeverCancelled;
use uuid::Uuid;

/// Returns `true` when ImageMagick 7 is installed on this device.
fn imagemagick_is_available() -> bool {
    ImageMagickAdapter::detect().installation().is_some()
}

/// Creates a JPEG image.
fn write_source_jpeg(path: &Path, width: u32, height: u32) {
    let image = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x * 4) as u8, (y * 4) as u8, 128])
    });

    image
        .save_with_format(path, image::ImageFormat::Jpeg)
        .expect("the source image is written");
}

/// Builds a request for the JPEG to PNG pair.
fn request(source: &Path, output_directory: &Path) -> ConversionRequest {
    ConversionRequest::new(
        Uuid::new_v4(),
        source.to_string_lossy().to_string(),
        from_extension("png").expect("a valid format"),
        output_directory.to_string_lossy().to_string(),
    )
}

#[test]
fn a_jpeg_is_converted_into_a_valid_png() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 32, 24);

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Succeeded, "{:?}", result.error);
    assert!(result.error.is_none());
    assert_eq!(result.engine_id.as_deref(), Some("imagemagick"));
    assert!(
        result.engine_version.is_some(),
        "the engine version is recorded"
    );

    let produced = PathBuf::from(result.output_path.expect("an output path is reported"));
    assert_eq!(produced, output.join("photo.png"));
    assert!(produced.is_file(), "the output file exists");

    // The output is a real PNG, not a file with the right name.
    let bytes = std::fs::read(&produced).expect("the output is read");
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
        "the PNG signature"
    );
    assert!(bytes.len() > 100, "the output is not empty");

    // The independent reader can open it.
    let decoded = image::open(&produced).expect("the output is decodable");
    assert_eq!(decoded.width(), 32);
    assert_eq!(decoded.height(), 24);

    assert_eq!(
        result.validation_level,
        ufc_core::domain::conversion::ValidationLevel::Parsed
    );
}

#[test]
fn the_source_file_is_never_modified() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 16, 16);
    let before = std::fs::read(&source).expect("the source is read");

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Succeeded);
    assert_eq!(
        std::fs::read(&source).expect("the source is read"),
        before,
        "the source file must not change"
    );
}

#[test]
fn a_conversion_without_a_loss_of_transparency_warns_about_nothing() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 8, 8);

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Succeeded);
    assert!(
        result.warnings.is_empty(),
        "a JPEG to PNG conversion loses nothing: {:?}",
        result.warnings
    );
}

#[test]
fn a_quality_setting_is_rejected_for_the_lossless_target() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 8, 8);

    let mut request = request(&source, &output);
    request.options = ConversionOptions::new().with_quality(80);

    let service = ConversionService::new();
    let result = service.convert(&request, &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Failed);
    assert_eq!(
        result.error.expect("an error is reported").code,
        ufc_core::domain::error::ConversionErrorCode::InvalidOptions
    );
}

#[test]
fn a_malformed_source_is_rejected_before_the_engine_runs() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("broken.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    std::fs::write(&source, b"this is not a jpeg image").expect("the file is written");

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Failed);
    assert_eq!(
        result.error.expect("an error is reported").code,
        ufc_core::domain::error::ConversionErrorCode::UndetectedFormat
    );
}

#[test]
fn an_existing_output_is_not_replaced_without_a_decision() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 8, 8);

    let service = ConversionService::new();

    let first = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);
    assert_eq!(first.status, JobStatus::Succeeded);

    let second = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(second.status, JobStatus::Failed);
    assert_eq!(
        second.error.expect("an error is reported").code,
        ufc_core::domain::error::ConversionErrorCode::OutputExists
    );
}

#[test]
fn a_renaming_policy_creates_a_second_file() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.jpg");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_jpeg(&source, 8, 8);

    let service = ConversionService::new();

    let first = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);
    assert_eq!(first.status, JobStatus::Succeeded);

    let mut renaming = request(&source, &output);
    renaming.overwrite_policy = OverwritePolicy::Rename;

    let second = service.convert(&renaming, &NeverCancelled, &NullObserver);

    assert_eq!(second.status, JobStatus::Succeeded);
    assert_eq!(
        PathBuf::from(second.output_path.expect("an output path is reported"))
            .file_name()
            .expect("a file name"),
        "photo (2).png"
    );
}
