//! Information about the files that the user selected.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::format::FormatId;

/// How the format of a file was determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum FormatDetection {
    /// The format was taken from the file extension.
    Extension,
    /// The format was taken from the content of the file.
    Content,
    /// The format could not be determined.
    Unknown,
}

/// The result of inspecting a file.
///
/// The inspection never reads the content of the file beyond its
/// signature, so it stays cheap for large files (`PRD.md` FR-001).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FileInspection {
    /// Absolute path of the file.
    pub path: String,
    /// Name of the file, without its directory.
    pub file_name: String,
    /// The directory that contains the file.
    pub parent_directory: Option<String>,
    /// Size of the file in bytes.
    ///
    /// The value is a plain number in the payload. The Rust type stays
    /// a `u64`, because a file can be larger than four gigabytes.
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// The detected format, when it is known.
    pub format: Option<FormatId>,
    /// How the format was determined.
    pub detection: FormatDetection,
    /// Whether the extension and the content of the file disagree.
    pub extension_mismatch: bool,
    /// Whether the file can be read.
    pub readable: bool,
}
