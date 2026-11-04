//! Canonical format identifiers.
//!
//! A [`FormatId`] is the stable, machine-readable name of a file format.
//! It is intentionally a plain lowercase slug so that it can be used
//! interchangeably in Rust, TypeScript, JSON and log output without
//! ambiguity.
//!
//! Human readable names are presentation concerns and belong to the UI
//! layer, not to the domain.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A canonical format identifier such as `pdf` or `docx`.
///
/// The value is always lowercase ASCII and never contains whitespace or
/// path separators.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FormatId(String);

impl FormatId {
    /// Creates a format identifier from a string.
    ///
    /// Returns `None` when the value is not a valid format slug.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if is_valid_slug(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier and returns the inner string.
    pub fn into_string(self) -> String {
        self.0
    }
}

fn is_valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

impl fmt::Display for FormatId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for FormatId {
    type Err = InvalidFormatId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FormatId::new(s).ok_or(InvalidFormatId(s.to_owned()))
    }
}

/// Error returned when a string cannot be parsed into a [`FormatId`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a valid format identifier")]
pub struct InvalidFormatId(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_lowercase_slugs() {
        assert_eq!(FormatId::new("pdf"), Some(FormatId("pdf".to_owned())));
        assert_eq!(FormatId::new("docx"), Some(FormatId("docx".to_owned())));
        assert_eq!(FormatId::new("mpeg-4"), Some(FormatId("mpeg-4".to_owned())));
    }

    #[test]
    fn rejects_invalid_slugs() {
        assert_eq!(FormatId::new(""), None);
        assert_eq!(FormatId::new("PDF"), None);
        assert_eq!(FormatId::new("pdf "), None);
        assert_eq!(FormatId::new("../pdf"), None);
        assert_eq!(FormatId::new("pdf/../etc"), None);
        assert_eq!(FormatId::new("-pdf"), None);
        assert_eq!(FormatId::new("pdf-"), None);
        assert_eq!(FormatId::new("a".repeat(33)), None);
    }

    #[test]
    fn round_trips_through_str() {
        let id = FormatId::new("png").expect("valid");
        assert_eq!(id.to_string(), "png");
        assert_eq!(FormatId::from_str("png"), Ok(id));
    }

    #[test]
    fn parses_invalid_value_into_error() {
        let err = FormatId::from_str("PNG").expect_err("must fail");
        assert_eq!(err.0, "PNG");
    }
}
