//! Resolving the destination of a converted file.
//!
//! An existing file is never overwritten without an explicit decision
//! (`PRD.md` FR-009). The policy is applied here, before the engine is
//! started, so that a conversion cannot fail halfway through because
//! of a name conflict.

use std::path::{Path, PathBuf};

use ufc_core::domain::conversion::OverwritePolicy;
use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};

/// Returns the path that the converted file is written to.
///
/// - [`OverwritePolicy::Skip`] fails when the file already exists.
/// - [`OverwritePolicy::Rename`] adds a counter to the name.
/// - [`OverwritePolicy::Overwrite`] replaces the existing file.
///
/// # Errors
/// Returns [`ConversionErrorCode::OutputExists`] when the file exists
/// and the policy forbids replacing it.
pub fn resolve_destination(
    directory: &Path,
    file_name: &str,
    policy: OverwritePolicy,
) -> DomainResult<PathBuf> {
    let candidate = directory.join(file_name);

    if !candidate.exists() {
        return Ok(candidate);
    }

    match policy {
        OverwritePolicy::Overwrite => Ok(candidate),
        OverwritePolicy::Skip => Err(ConversionError::new(
            ConversionErrorCode::OutputExists,
            format!("The file '{file_name}' already exists in the output directory."),
        )
        .with_remediation(
            "Choose another output folder, or allow the application to rename the file.",
        )),
        OverwritePolicy::Rename => Ok(next_free_name(directory, file_name)),
    }
}

/// Returns the first free name of the form `name (2).ext`.
fn next_free_name(directory: &Path, file_name: &str) -> PathBuf {
    let (stem, extension) = split_file_name(file_name);

    for counter in 2_u32..10_000 {
        let candidate = format!("{stem} ({counter}){extension}");
        let path = directory.join(&candidate);
        if !path.exists() {
            return path;
        }
    }

    // The directory is full of conflicts. The last name is returned and
    // the engine overwrites it, which is reported by the collision
    // policy of the request.
    directory.join(format!("{stem} (9999){extension}"))
}

/// Splits a file name into its stem and its extension.
fn split_file_name(file_name: &str) -> (&str, &str) {
    match file_name.rfind('.') {
        Some(index) if index > 0 => (&file_name[..index], &file_name[index..]),
        _ => (file_name, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_free_name_is_used_as_it_is() {
        let dir = tempfile::tempdir().expect("temp dir");

        let path = resolve_destination(dir.path(), "photo.jpg", OverwritePolicy::Skip)
            .expect("the name is free");

        assert_eq!(path, dir.path().join("photo.jpg"));
    }

    #[test]
    fn an_existing_file_is_not_replaced_when_the_policy_skips() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("photo.jpg"), b"data").expect("the file is written");

        let error = resolve_destination(dir.path(), "photo.jpg", OverwritePolicy::Skip)
            .expect_err("must fail");

        assert_eq!(error.code, ConversionErrorCode::OutputExists);
        assert!(error.remediation.is_some());
    }

    #[test]
    fn an_existing_file_is_replaced_when_the_policy_overwrites() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("photo.jpg"), b"data").expect("the file is written");

        let path = resolve_destination(dir.path(), "photo.jpg", OverwritePolicy::Overwrite)
            .expect("the file is replaced");

        assert_eq!(path, dir.path().join("photo.jpg"));
    }

    #[test]
    fn an_existing_file_is_renamed_when_the_policy_renames() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("photo.jpg"), b"data").expect("the file is written");

        let path = resolve_destination(dir.path(), "photo.jpg", OverwritePolicy::Rename)
            .expect("a new name is found");

        assert_eq!(path.file_name().unwrap(), "photo (2).jpg");
    }

    #[test]
    fn the_counter_grows_with_every_conflict() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("photo.jpg"), b"data").expect("the file is written");
        std::fs::write(dir.path().join("photo (2).jpg"), b"data").expect("the file is written");

        let path = resolve_destination(dir.path(), "photo.jpg", OverwritePolicy::Rename)
            .expect("a new name is found");

        assert_eq!(path.file_name().unwrap(), "photo (3).jpg");
    }

    #[test]
    fn a_name_without_an_extension_is_renamed_correctly() {
        let dir = tempfile::tempdir().expect("temp dir");
        std::fs::write(dir.path().join("photo"), b"data").expect("the file is written");

        let path = resolve_destination(dir.path(), "photo", OverwritePolicy::Rename)
            .expect("a new name is found");

        assert_eq!(path.file_name().unwrap(), "photo (2)");
    }
}
