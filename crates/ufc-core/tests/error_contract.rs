//! Integration tests for the structured error contract.
//!
//! The frontend branches on the serialised `code` value, so the wire
//! format is part of the public contract between Rust and TypeScript
//! (`PRD.md` FR-005, FR-012).

use ufc_core::domain::error::{ConversionError, ConversionErrorCode};

/// Every error code with the exact value the frontend expects.
///
/// If a code is renamed or removed, this test fails and the frontend
/// mapping in `src/lib/ipc.ts` has to be updated in the same change.
const WIRE_CONTRACT: &[(ConversionErrorCode, &str)] = &[
    (ConversionErrorCode::UnsupportedPair, "unsupportedPair"),
    (ConversionErrorCode::MissingDependency, "missingDependency"),
    (
        ConversionErrorCode::IncompatibleEngineVersion,
        "incompatibleEngineVersion",
    ),
    (ConversionErrorCode::InvalidSource, "invalidSource"),
    (ConversionErrorCode::UndetectedFormat, "undetectedFormat"),
    (ConversionErrorCode::FormatMismatch, "formatMismatch"),
    (ConversionErrorCode::InvalidOptions, "invalidOptions"),
    (
        ConversionErrorCode::InvalidOutputLocation,
        "invalidOutputLocation",
    ),
    (ConversionErrorCode::OutputExists, "outputExists"),
    (ConversionErrorCode::EngineFailed, "engineFailed"),
    (ConversionErrorCode::MissingOutput, "missingOutput"),
    (
        ConversionErrorCode::OutputValidationFailed,
        "outputValidationFailed",
    ),
    (ConversionErrorCode::Cancelled, "cancelled"),
    (ConversionErrorCode::Timeout, "timeout"),
    (
        ConversionErrorCode::InsufficientDiskSpace,
        "insufficientDiskSpace",
    ),
    (ConversionErrorCode::IoError, "ioError"),
    (ConversionErrorCode::EngineStartFailed, "engineStartFailed"),
    (ConversionErrorCode::Internal, "internal"),
];

#[test]
fn every_error_code_has_the_expected_wire_value() {
    for (code, expected) in WIRE_CONTRACT {
        let json = serde_json::to_value(code).expect("serializable");
        assert_eq!(json, serde_json::Value::String((*expected).to_owned()));
    }
}

#[test]
fn the_wire_contract_covers_every_error_code() {
    // The list above must not drift away from the enum. The enum has
    // 18 variants; a new variant has to be added here deliberately.
    assert_eq!(WIRE_CONTRACT.len(), 18);
}

#[test]
fn log_identifiers_are_unique_and_snake_case() {
    let mut seen = std::collections::HashSet::new();

    for (code, _) in WIRE_CONTRACT {
        let identifier = code.as_str();
        assert!(
            seen.insert(identifier),
            "duplicate identifier: {identifier}"
        );
        assert!(
            identifier
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_'),
            "identifier must be snake_case: {identifier}"
        );
    }
}

#[test]
fn optional_fields_are_absent_from_the_payload() {
    let error = ConversionError::new(ConversionErrorCode::InvalidSource, "missing file");

    let json = serde_json::to_value(&error).expect("serializable");
    assert_eq!(json["code"], "invalidSource");
    assert_eq!(json["message"], "missing file");
    assert!(json.get("remediation").is_none());
    assert!(json.get("detail").is_none());
}

#[test]
fn optional_fields_are_included_when_present() {
    let error = ConversionError::new(ConversionErrorCode::MissingDependency, "ffmpeg not found")
        .with_remediation("Install FFmpeg")
        .with_detail("probe exited with code 1");

    let json = serde_json::to_value(&error).expect("serializable");
    assert_eq!(json["remediation"], "Install FFmpeg");
    assert_eq!(json["detail"], "probe exited with code 1");
}

#[test]
fn an_error_describes_itself_without_leaking_details() {
    let error = ConversionError::new(ConversionErrorCode::EngineFailed, "Conversion failed.");
    let displayed = error.to_string();

    assert_eq!(displayed, "Conversion failed.");
    assert!(!displayed.contains("detail"));
}
