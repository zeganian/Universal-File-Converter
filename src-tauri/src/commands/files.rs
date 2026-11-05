//! Inspecting the files that the user selected.

use tauri::State;
use ufc_core::domain::file::{FileInspection, FormatDetection};
use ufc_core::domain::format::from_extension;

use crate::commands::app::CommandError;
use crate::services::state::AppState;

/// Inspects a list of files.
///
/// The paths come from the file dialog of the operating system. The
/// command never reads the content of a file beyond its signature, so
/// it stays fast for large files (`PRD.md` FR-001, FR-002).
///
/// A file that cannot be inspected is not dropped: it is reported with
/// an unknown format and `readable: false`, so that the interface can
/// tell the user which selection is the problem.
///
/// # Errors
/// Never fails.
#[tauri::command]
pub fn inspect_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> Result<Vec<FileInspection>, CommandError> {
    let _ = &state;

    let inspections = paths
        .iter()
        .map(|path| {
            let path = std::path::Path::new(path);
            ufc_engines::inspection::inspect(path).unwrap_or_else(|_| unreadable(path))
        })
        .collect();

    Ok(inspections)
}

/// Describes a file that could not be inspected.
///
/// The name and the extension are still read from the path, so that the
/// interface can show what the user selected. The content was never
/// read, so the format stays unknown.
fn unreadable(path: &std::path::Path) -> FileInspection {
    let format = path
        .extension()
        .and_then(|extension| from_extension(&extension.to_string_lossy()));

    FileInspection {
        path: path.to_string_lossy().to_string(),
        file_name: path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        parent_directory: path
            .parent()
            .map(|parent| parent.to_string_lossy().to_string())
            .filter(|parent| !parent.is_empty()),
        size_bytes: 0,
        format,
        detection: FormatDetection::Unknown,
        extension_mismatch: false,
        readable: false,
    }
}
