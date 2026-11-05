//! Inspecting the files that the user selected.
//!
//! The inspection is deliberately cheap: the file name, the size and
//! the signature of the file are read, but the content is never loaded
//! into memory (`PRD.md` FR-001, FR-002).

use std::path::Path;

use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
use ufc_core::domain::file::{FileInspection, FormatDetection};
use ufc_core::domain::format::{from_extension, FormatId};

/// Inspects a file and reports what is known about it.
///
/// # Errors
/// Returns [`ConversionErrorCode::InvalidSource`] when the file does
/// not exist or is not a regular file.
pub fn inspect(path: &Path) -> DomainResult<FileInspection> {
    let metadata = std::fs::metadata(path).map_err(|source| {
        ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The selected file could not be read.",
        )
        .with_detail(source.to_string())
    })?;

    if !metadata.is_file() {
        return Err(ConversionError::new(
            ConversionErrorCode::InvalidSource,
            "The selection is not a regular file.",
        ));
    }

    let extension_format = path
        .extension()
        .and_then(|extension| from_extension(&extension.to_string_lossy()));

    let content_format = detect_from_content(path);

    let (format, detection) = match (&extension_format, &content_format) {
        (Some(format), _) => (Some(format.clone()), FormatDetection::Extension),
        (None, Some(format)) => (Some(format.clone()), FormatDetection::Content),
        (None, None) => (None, FormatDetection::Unknown),
    };

    let extension_mismatch = match (&extension_format, &content_format) {
        (Some(extension), Some(content)) => extension != content,
        _ => false,
    };

    Ok(FileInspection {
        path: path.to_string_lossy().to_string(),
        file_name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        parent_directory: path
            .parent()
            .map(|parent| parent.to_string_lossy().to_string())
            .filter(|parent| !parent.is_empty()),
        size_bytes: metadata.len(),
        format,
        detection,
        extension_mismatch,
        readable: is_readable(path),
    })
}

/// Detects the format from the signature of the file.
///
/// Only the signatures that the application knows are recognised. An
/// unknown signature is not an error: the extension may still be
/// enough.
fn detect_from_content(path: &Path) -> Option<FormatId> {
    let kind = infer::get_from_path(path).ok()??;

    let format = match kind.mime_type() {
        "image/png" => "png",
        "image/jpeg" => "jpeg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/webp" => "webp",
        "image/tiff" => "tiff",
        "application/pdf" => "pdf",
        "audio/mpeg" => "mp3",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/flac" => "flac",
        "video/mp4" => "mp4",
        "video/x-matroska" => "mkv",
        "video/webm" => "webm",
        "application/zip" => "zip",
        "application/gzip" => "gz",
        "application/x-7z-compressed" => "7z",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.oasis.opendocument.text" => "odt",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "application/vnd.oasis.opendocument.spreadsheet" => "ods",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "pptx",
        "application/vnd.oasis.opendocument.presentation" => "odp",
        "text/csv" => "csv",
        "text/html" => "html",
        "text/markdown" => "md",
        "text/plain" => "txt",
        _ => return None,
    };

    FormatId::new(format)
}

/// Returns `true` when the file can be opened for reading.
fn is_readable(path: &Path) -> bool {
    std::fs::File::open(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_is_inspected_by_its_extension() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("photo.png");
        let image = image::RgbImage::from_pixel(4, 4, image::Rgb([1, 2, 3]));
        image
            .save_with_format(&path, image::ImageFormat::Png)
            .expect("the file is written");

        let inspection = inspect(&path).expect("the file is inspected");

        assert_eq!(inspection.file_name, "photo.png");
        assert_eq!(
            inspection.format.map(|id| id.to_string()),
            Some("png".into())
        );
        assert_eq!(inspection.detection, FormatDetection::Extension);
        assert!(!inspection.extension_mismatch);
        assert!(inspection.readable);
        assert!(inspection.size_bytes > 0);
    }

    #[test]
    fn a_mismatch_between_extension_and_content_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("actually-a-png.jpg");
        let image = image::RgbImage::from_pixel(4, 4, image::Rgb([1, 2, 3]));
        image
            .save_with_format(&path, image::ImageFormat::Png)
            .expect("the file is written");

        let inspection = inspect(&path).expect("the file is inspected");

        assert_eq!(
            inspection.format.map(|id| id.to_string()),
            Some("jpeg".into())
        );
        assert!(inspection.extension_mismatch);
    }

    #[test]
    fn an_unknown_format_is_reported_as_unknown() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("notes.qqq");
        std::fs::write(&path, b"unknown content").expect("the file is written");

        let inspection = inspect(&path).expect("the file is inspected");

        assert_eq!(inspection.detection, FormatDetection::Unknown);
        assert!(inspection.format.is_none());
    }

    #[test]
    fn a_missing_file_is_reported() {
        let dir = tempfile::tempdir().expect("temp dir");

        let error = inspect(&dir.path().join("missing.png")).expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::InvalidSource);
    }
}
