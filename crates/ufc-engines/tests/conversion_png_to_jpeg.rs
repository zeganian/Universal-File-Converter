//! Integration test for the first implemented conversion pair.
//!
//! The test performs a real conversion with ImageMagick and verifies
//! the result with an independent reader. It is skipped when
//! ImageMagick is not installed, because the application must work
//! without any engine (`ADR-002`).
//!
//! Run `node scripts/generate-fixtures.mjs` first to also exercise the
//! fixtures of the repository.

use std::path::{Path, PathBuf};

use ufc_core::domain::conversion::{ConversionOptions, ConversionRequest, OverwritePolicy};
use ufc_core::domain::format::from_extension;
use ufc_core::domain::job::JobStatus;
use ufc_core::engines::adapter::NeverCancelled;
use ufc_engines::conversion::{ConversionService, NullObserver};
use uuid::Uuid;

/// Returns `true` when ImageMagick 7 is installed on this device.
fn imagemagick_is_available() -> bool {
    ufc_engines::engines::imagemagick::ImageMagickAdapter::detect()
        .installation()
        .is_some()
}

/// Creates a PNG image with an alpha channel.
fn write_source_png(path: &Path, width: u32, height: u32, with_alpha: bool) {
    let image = if with_alpha {
        image::RgbaImage::from_pixel(width, height, image::Rgba([10, 20, 30, 128]))
    } else {
        image::RgbaImage::from_pixel(width, height, image::Rgba([10, 20, 30, 255]))
    };

    image
        .save_with_format(path, image::ImageFormat::Png)
        .expect("the source image is written");
}

/// Builds a request for the PNG to JPEG pair.
fn request(source: &Path, output_directory: &Path) -> ConversionRequest {
    ConversionRequest::new(
        Uuid::new_v4(),
        source.to_string_lossy().to_string(),
        from_extension("jpg").expect("a valid format"),
        output_directory.to_string_lossy().to_string(),
    )
}

#[test]
fn a_png_is_converted_into_a_valid_jpeg() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_png(&source, 32, 24, false);

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(
        result.status,
        JobStatus::Succeeded,
        "the conversion succeeds"
    );
    assert!(result.error.is_none());
    assert_eq!(result.engine_id.as_deref(), Some("imagemagick"));
    assert!(
        result.engine_version.is_some(),
        "the engine version is recorded"
    );

    let produced = PathBuf::from(result.output_path.expect("an output path is reported"));
    assert_eq!(produced, output.join("photo.jpg"));
    assert!(produced.is_file(), "the output file exists");

    // The output is a real JPEG, not a file with the right name.
    let bytes = std::fs::read(&produced).expect("the output is read");
    assert_eq!(&bytes[..3], &[0xff, 0xd8, 0xff], "the JPEG signature");
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
    let source = directory.path().join("photo.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_png(&source, 16, 16, false);
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
fn a_transparent_source_produces_a_warning() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("transparent.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_png(&source, 8, 8, true);

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Succeeded);
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("alpha")),
        "the loss of transparency is reported: {:?}",
        result.warnings
    );
}

#[test]
fn an_existing_output_is_not_replaced_without_a_decision() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_png(&source, 8, 8, false);

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
    let source = directory.path().join("photo.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    write_source_png(&source, 8, 8, false);

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
        "photo (2).jpg"
    );
}

#[test]
fn a_malformed_source_is_rejected_before_the_engine_runs() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("broken.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    std::fs::write(&source, b"this is not a png image").expect("the file is written");

    let service = ConversionService::new();
    let result = service.convert(&request(&source, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Failed);
    assert_eq!(
        result.error.expect("an error is reported").code,
        ufc_core::domain::error::ConversionErrorCode::UndetectedFormat
    );
}

#[test]
fn the_quality_option_is_applied() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let source = directory.path().join("photo.png");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    // A gradient compresses very differently at a low quality.
    let image = image::RgbaImage::from_fn(64, 64, |x, y| {
        image::Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255])
    });
    image
        .save_with_format(&source, image::ImageFormat::Png)
        .expect("the source image is written");

    let service = ConversionService::new();

    let mut low = request(&source, &output);
    low.options = ConversionOptions::new().with_quality(10);
    let low_result = service.convert(&low, &NeverCancelled, &NullObserver);

    let mut high = request(&source, &output);
    high.options = ConversionOptions::new().with_quality(95);
    high.overwrite_policy = OverwritePolicy::Rename;
    let high_result = service.convert(&high, &NeverCancelled, &NullObserver);

    assert_eq!(low_result.status, JobStatus::Succeeded);
    assert_eq!(high_result.status, JobStatus::Succeeded);

    let low_size = std::fs::metadata(low_result.output_path.expect("an output path is reported"))
        .expect("the file exists")
        .len();
    let high_size = std::fs::metadata(high_result.output_path.expect("an output path is reported"))
        .expect("the file exists")
        .len();

    assert!(
        low_size < high_size,
        "a lower quality has to produce a smaller file ({low_size} < {high_size})"
    );
}

#[test]
fn the_repository_fixture_is_converted_as_well() {
    if !imagemagick_is_available() {
        eprintln!("skipped: ImageMagick 7 is not installed");
        return;
    }

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/generated/valid/gradient.png");

    if !fixture.is_file() {
        eprintln!(
            "skipped: the repository fixtures are missing, run 'node scripts/generate-fixtures.mjs'"
        );
        return;
    }

    let directory = tempfile::tempdir().expect("temp dir");
    let output = directory.path().join("out");
    std::fs::create_dir_all(&output).expect("the output directory is created");

    let service = ConversionService::new();
    let result = service.convert(&request(&fixture, &output), &NeverCancelled, &NullObserver);

    assert_eq!(result.status, JobStatus::Succeeded, "{:?}", result.error);

    let produced = PathBuf::from(result.output_path.expect("an output path is reported"));
    let decoded = image::open(&produced).expect("the output is decodable");
    assert_eq!(decoded.width(), 64);
    assert_eq!(decoded.height(), 64);
}
