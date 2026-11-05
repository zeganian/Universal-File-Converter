//! Locating and inspecting external programs.
//!
//! The application never accepts an executable path from the frontend.
//! An engine is found by searching a fixed list of trusted locations
//! and is then asked for its version (`ARCHITECTURE.md` section 5).

use std::path::{Path, PathBuf};

use ufc_core::domain::error::{ConversionError, ConversionErrorCode, DomainResult};

/// Directories that are searched in addition to `PATH`.
const EXTRA_DIRECTORIES: &[&str] = &[
    "/usr/local/bin",
    "/usr/bin",
    "/opt/homebrew/bin",
    "/usr/local/opt/imagemagick/bin",
    "/snap/bin",
];

/// Finds an executable by name.
///
/// The name is looked up in `PATH` first and then in a fixed list of
/// trusted installation directories. The frontend can never influence
/// the result beyond the program names that the adapter itself defines.
pub fn find_executable(names: &[&str]) -> Option<PathBuf> {
    for directory in search_directories() {
        for name in names {
            let candidate = directory.join(name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

/// Returns the directories that are searched for an executable.
fn search_directories() -> Vec<PathBuf> {
    let mut directories: Vec<PathBuf> = Vec::new();

    if let Some(path) = std::env::var_os("PATH") {
        directories.extend(std::env::split_paths(&path));
    }

    for extra in EXTRA_DIRECTORIES {
        directories.push(PathBuf::from(extra));
    }

    directories.dedup();
    directories
}

/// Returns `true` when the path is a file that can be executed.
fn is_executable(path: &Path) -> bool {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

/// Parses the version out of the output of `<program> -version`.
///
/// ImageMagick prints a first line such as
/// `Version: ImageMagick 7.1.2-34 Q16-HDRI x86_64 ...`.
pub fn parse_imagemagick_version(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("Version: ImageMagick ") else {
            continue;
        };

        let version = rest.split_whitespace().next()?;
        if version.is_empty() {
            continue;
        }

        return Some(version.to_owned());
    }

    None
}

/// Returns the major version of an ImageMagick version string.
pub fn imagemagick_major(version: &str) -> Option<u32> {
    version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
}

/// Builds the error that explains a missing engine.
pub fn missing_engine_error(engine_name: &str, hint: &str) -> ConversionError {
    ConversionError::new(
        ConversionErrorCode::MissingDependency,
        format!("{engine_name} was not found on this device."),
    )
    .with_remediation(hint.to_owned())
}

/// Builds the error that explains an unsupported engine version.
pub fn incompatible_version_error(
    engine_name: &str,
    found: &str,
    required: &str,
    hint: &str,
) -> ConversionError {
    ConversionError::new(
        ConversionErrorCode::IncompatibleEngineVersion,
        format!("{engine_name} {found} is installed, but {required} is required."),
    )
    .with_remediation(hint.to_owned())
}

/// Returns the installation hint for the current platform.
pub fn imagemagick_install_hint() -> String {
    #[cfg(target_os = "linux")]
    {
        "Install ImageMagick 7 with your package manager, for example \
         'sudo apt install imagemagick' on Debian and Ubuntu, \
         'sudo dnf install ImageMagick' on Fedora, or \
         'sudo pacman -S imagemagick' on Arch Linux."
            .to_owned()
    }

    #[cfg(target_os = "macos")]
    {
        "Install ImageMagick 7 with Homebrew: 'brew install imagemagick'.".to_owned()
    }

    #[cfg(target_os = "windows")]
    {
        "Install ImageMagick 7 from the official installer at \
         https://imagemagick.org/script/download.php and add its \
         directory to the system path."
            .to_owned()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Install ImageMagick 7 and make sure that 'magick' is on the \
         system path."
            .to_owned()
    }
}

/// Returns a domain error for an engine that is not usable.
pub fn engine_unusable(engine_name: &str) -> DomainResult<ConversionError> {
    Err(ConversionError::new(
        ConversionErrorCode::MissingDependency,
        format!("{engine_name} is not available on this device."),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_imagemagick_version_is_parsed() {
        let output = "Version: ImageMagick 7.1.2-34 Q16-HDRI x86_64 2026-10-14 https://imagemagick.org\nCopyright: ...";
        assert_eq!(
            parse_imagemagick_version(output),
            Some("7.1.2-34".to_owned())
        );
    }

    #[test]
    fn a_version_without_the_marker_is_not_parsed() {
        assert_eq!(parse_imagemagick_version("convert: not authorized"), None);
        assert_eq!(parse_imagemagick_version(""), None);
    }

    #[test]
    fn the_major_version_is_extracted() {
        assert_eq!(imagemagick_major("7.1.2-34"), Some(7));
        assert_eq!(imagemagick_major("6.9.11-60"), Some(6));
        assert_eq!(imagemagick_major("unknown"), None);
    }

    #[test]
    fn an_executable_is_found_by_name() {
        let found = find_executable(&["sh"]);
        assert!(found.is_some(), "sh must be on the path");
    }

    #[test]
    fn an_unknown_executable_is_not_found() {
        assert!(find_executable(&["definitely-not-an-executable-9f3a"]).is_none());
    }

    #[test]
    fn a_directory_is_not_an_executable() {
        assert!(find_executable(&[""]).is_none());
    }
}
