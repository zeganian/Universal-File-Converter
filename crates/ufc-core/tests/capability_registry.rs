//! Integration tests for the capability registry.
//!
//! These tests lock the rule that matters most for the product: the
//! application must never offer a conversion it cannot perform.

use std::collections::BTreeMap;

use ufc_core::domain::conversion::{EngineAvailability, EngineId};
use ufc_core::domain::format::FormatId;
use ufc_core::domain::registry::{
    default_registry, CapabilityRegistry, ConversionCapability, PairStatus,
};

fn format(value: &str) -> FormatId {
    FormatId::new(value).expect("valid format identifier")
}

fn engine(value: &str) -> EngineId {
    EngineId::new(value).expect("valid engine identifier")
}

fn capability(
    source: &str,
    target: &str,
    engine_id: &str,
    status: PairStatus,
) -> ConversionCapability {
    let mut capability =
        ConversionCapability::planned(format(source), format(target), engine(engine_id));
    capability.status = status;
    capability
}

fn availability(entries: &[(&str, EngineAvailability)]) -> BTreeMap<EngineId, EngineAvailability> {
    entries
        .iter()
        .map(|(id, state)| (engine(id), *state))
        .collect()
}

#[test]
fn the_shipped_registry_advertises_nothing() {
    let registry = default_registry();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert_eq!(registry.available_count(), 0);
    assert!(registry.available_sources().is_empty());
    assert!(registry.available_targets(&format("png")).is_empty());
}

#[test]
fn a_planned_pair_is_never_offered() {
    let mut registry = CapabilityRegistry::new(vec![capability(
        "docx",
        "pdf",
        "libreoffice",
        PairStatus::Planned,
    )]);

    registry.apply_availability(&availability(&[(
        "libreoffice",
        EngineAvailability::Available,
    )]));

    assert_eq!(registry.available_count(), 0);
    assert!(registry.find(&format("docx"), &format("pdf")).is_some());
    assert!(
        !registry
            .find(&format("docx"), &format("pdf"))
            .expect("pair exists")
            .available
    );
}

#[test]
fn a_pair_needs_both_an_implementation_and_an_engine() {
    let mut registry = CapabilityRegistry::new(vec![capability(
        "docx",
        "pdf",
        "libreoffice",
        PairStatus::Stable,
    )]);

    // Engine missing: the pair stays unavailable.
    registry.apply_availability(&availability(&[(
        "libreoffice",
        EngineAvailability::Missing,
    )]));
    assert_eq!(registry.available_count(), 0);

    // Engine present but incompatible: still unavailable.
    registry.apply_availability(&availability(&[(
        "libreoffice",
        EngineAvailability::IncompatibleVersion,
    )]));
    assert_eq!(registry.available_count(), 0);

    // Engine usable and the pair implemented: now it may be offered.
    registry.apply_availability(&availability(&[(
        "libreoffice",
        EngineAvailability::Available,
    )]));
    assert_eq!(registry.available_count(), 1);
    assert_eq!(
        registry.available_targets(&format("docx")),
        vec![&format("pdf")]
    );
    assert_eq!(registry.available_sources(), vec![format("docx")]);
}

#[test]
fn a_pair_for_an_unknown_engine_is_never_offered() {
    let mut registry = CapabilityRegistry::new(vec![capability(
        "png",
        "webp",
        "imagemagick",
        PairStatus::Stable,
    )]);

    registry.apply_availability(&BTreeMap::new());

    assert_eq!(registry.available_count(), 0);
}

#[test]
fn an_experimental_pair_may_be_offered() {
    let mut registry = CapabilityRegistry::new(vec![capability(
        "pdf",
        "docx",
        "libreoffice",
        PairStatus::Experimental,
    )]);

    registry.apply_availability(&availability(&[(
        "libreoffice",
        EngineAvailability::Available,
    )]));

    assert_eq!(registry.available_count(), 1);
    let pair = registry
        .find(&format("pdf"), &format("docx"))
        .expect("pair");
    assert_eq!(pair.status, PairStatus::Experimental);
    assert!(pair.status.is_offerable());
}

#[test]
fn a_not_supported_pair_is_never_offered() {
    let mut registry = CapabilityRegistry::new(vec![capability(
        "png",
        "pdf",
        "imagemagick",
        PairStatus::NotSupported,
    )]);

    registry.apply_availability(&availability(&[(
        "imagemagick",
        EngineAvailability::Available,
    )]));

    assert_eq!(registry.available_count(), 0);
    assert!(!PairStatus::NotSupported.is_offerable());
}

#[test]
fn targets_are_scoped_to_their_source_format() {
    let mut registry = CapabilityRegistry::new(vec![
        capability("png", "webp", "imagemagick", PairStatus::Stable),
        capability("png", "jpeg", "imagemagick", PairStatus::Stable),
        capability("jpg", "png", "imagemagick", PairStatus::Stable),
    ]);

    registry.apply_availability(&availability(&[(
        "imagemagick",
        EngineAvailability::Available,
    )]));

    let png_targets = registry.available_targets(&format("png"));
    assert_eq!(png_targets.len(), 2);
    assert!(png_targets.contains(&&format("webp")));
    assert!(png_targets.contains(&&format("jpeg")));

    let jpg_targets = registry.available_targets(&format("jpg"));
    assert_eq!(jpg_targets.len(), 1);
    assert!(jpg_targets.contains(&&format("png")));
}

#[test]
fn one_missing_engine_only_disables_its_own_pairs() {
    let mut registry = CapabilityRegistry::new(vec![
        capability("png", "webp", "imagemagick", PairStatus::Stable),
        capability("mp4", "webm", "ffmpeg", PairStatus::Stable),
    ]);

    registry.apply_availability(&availability(&[
        ("imagemagick", EngineAvailability::Available),
        ("ffmpeg", EngineAvailability::Missing),
    ]));

    assert_eq!(registry.available_count(), 1);
    assert!(registry
        .available_targets(&format("png"))
        .contains(&&format("webp")));
    assert!(registry.available_targets(&format("mp4")).is_empty());
}
