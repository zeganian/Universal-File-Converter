//! The conversion capability registry.
//!
//! The registry is the single source of truth for which source-to-target
//! pairs the application may offer (`SUPPORTED_FORMATS.md`, "Registry
//! rules"). The frontend never maintains its own hardcoded list.
//!
//! A pair only becomes *offerable* when three independent conditions
//! hold:
//!
//! 1. the pair is [`PairStatus::Stable`] or [`PairStatus::Experimental`],
//! 2. the pair has been implemented, and
//! 3. the engine required by the pair is available on this machine.
//!
//! Until an engine adapter is implemented, the registry stays empty and
//! the application reports that no conversion is available yet. This is
//! deliberate: the product must never advertise a conversion it cannot
//! perform (`RULES.md` sections 1 and 6).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domain::conversion::{EngineAvailability, EngineId};
use crate::domain::format::FormatId;

/// Maturity of a source-to-target conversion pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PairStatus {
    /// Not implemented yet.
    Planned,
    /// Implemented, but quality or platform coverage is incomplete.
    Experimental,
    /// Release gated tests pass on the listed platforms.
    Stable,
    /// Intentionally not supported.
    NotSupported,
}

impl PairStatus {
    /// Returns `true` when a pair in this state may be offered to the
    /// user, provided its engine is available.
    pub fn is_offerable(self) -> bool {
        matches!(self, PairStatus::Stable | PairStatus::Experimental)
    }

    /// Returns a short, stable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            PairStatus::Planned => "Planned",
            PairStatus::Experimental => "Experimental",
            PairStatus::Stable => "Stable",
            PairStatus::NotSupported => "Not supported",
        }
    }
}

/// A version range of an engine that has been tested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EngineVersionRange {
    /// Lowest tested version, inclusive.
    pub min: String,
    /// Highest tested version, inclusive.
    pub max: String,
}

/// The options a conversion pair accepts.
///
/// The options are part of the pair, not of the engine: a lossy target
/// has an encoder quality, while a lossless target has none
/// (`SUPPORTED_FORMATS.md`, "Images"). The interface uses this
/// information to offer only the options that have an effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityOptions {
    /// The pair accepts an encoder quality between 1 and 100.
    pub quality: bool,
    /// The pair accepts a background colour, for a target that cannot
    /// store transparency.
    pub background: bool,
}

impl CapabilityOptions {
    /// Returns options that the pair does not accept.
    pub const fn none() -> Self {
        Self {
            quality: false,
            background: false,
        }
    }
}

/// A single source-to-target conversion capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ConversionCapability {
    /// Source format.
    pub source_format: FormatId,
    /// Target format.
    pub target_format: FormatId,
    /// Engine that performs the conversion.
    pub engine_id: EngineId,
    /// Tested version range of the engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version_range: Option<EngineVersionRange>,
    /// Maturity of the pair.
    pub status: PairStatus,
    /// Identifier of the validation strategy used for the output.
    pub validation_strategy: String,
    /// The options that the pair accepts.
    pub options: CapabilityOptions,
    /// User facing limitations of this pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Whether the pair is currently offered to the user.
    ///
    /// This is `false` unless the pair is offerable *and* the engine is
    /// available on this machine.
    pub available: bool,
}

impl ConversionCapability {
    /// Creates a planned, unavailable capability.
    pub fn planned(source_format: FormatId, target_format: FormatId, engine_id: EngineId) -> Self {
        Self {
            source_format,
            target_format,
            engine_id,
            engine_version_range: None,
            status: PairStatus::Planned,
            validation_strategy: "none".to_owned(),
            options: CapabilityOptions::none(),
            limitations: Vec::new(),
            available: false,
        }
    }
}

/// Registry of conversion capabilities.
///
/// The registry is immutable once built; runtime availability is applied
/// as a separate step by [`CapabilityRegistry::apply_availability`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRegistry {
    capabilities: Vec<ConversionCapability>,
}

impl CapabilityRegistry {
    /// Creates an empty registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a registry from a list of capabilities.
    pub fn new(capabilities: Vec<ConversionCapability>) -> Self {
        Self { capabilities }
    }

    /// Adds a capability to the registry.
    pub fn insert(&mut self, capability: ConversionCapability) {
        self.capabilities.push(capability);
    }

    /// Returns every registered capability.
    pub fn all(&self) -> &[ConversionCapability] {
        &self.capabilities
    }

    /// Returns the number of registered capabilities.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Returns `true` when no capability is registered.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Looks up a pair by source and target format.
    pub fn find(&self, source: &FormatId, target: &FormatId) -> Option<&ConversionCapability> {
        self.capabilities
            .iter()
            .find(|c| &c.source_format == source && &c.target_format == target)
    }

    /// Returns every target format that may be offered for `source`.
    ///
    /// Only pairs whose status is offerable and whose engine is
    /// available are returned.
    pub fn available_targets(&self, source: &FormatId) -> Vec<&FormatId> {
        self.capabilities
            .iter()
            .filter(|c| &c.source_format == source && c.available)
            .map(|c| &c.target_format)
            .collect()
    }

    /// Returns every source format that can be converted right now.
    pub fn available_sources(&self) -> Vec<FormatId> {
        let mut seen = BTreeMap::new();
        for capability in self.capabilities.iter().filter(|c| c.available) {
            seen.insert(capability.source_format.clone(), ());
        }
        seen.into_keys().collect()
    }

    /// Recomputes the `available` flag of every capability against the
    /// current engine availability.
    ///
    /// A capability is available when its status is offerable and its
    /// engine is [`EngineAvailability::Available`].
    pub fn apply_availability(&mut self, engines: &BTreeMap<EngineId, EngineAvailability>) {
        for capability in &mut self.capabilities {
            let engine_ok = engines
                .get(&capability.engine_id)
                .is_some_and(|availability| availability.is_usable());
            capability.available = capability.status.is_offerable() && engine_ok;
        }
    }

    /// Returns the number of capabilities that may currently be offered.
    pub fn available_count(&self) -> usize {
        self.capabilities.iter().filter(|c| c.available).count()
    }
}

/// The registry used by the application.
///
/// No engine adapter is implemented yet, therefore this registry holds
/// no capabilities and the application correctly reports that no
/// conversion is available.
pub fn default_registry() -> CapabilityRegistry {
    CapabilityRegistry::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(source: &str, target: &str, status: PairStatus) -> ConversionCapability {
        let mut capability = ConversionCapability::planned(
            FormatId::new(source).expect("valid source"),
            FormatId::new(target).expect("valid target"),
            EngineId::new("test-engine").expect("valid engine"),
        );
        capability.status = status;
        capability
    }

    fn engines_with(
        entries: &[(&str, EngineAvailability)],
    ) -> BTreeMap<EngineId, EngineAvailability> {
        entries
            .iter()
            .map(|(id, availability)| (EngineId::new(*id).expect("valid"), *availability))
            .collect()
    }

    #[test]
    fn empty_registry_offers_nothing() {
        let registry = CapabilityRegistry::empty();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.available_count(), 0);
        assert!(registry.available_sources().is_empty());
    }

    #[test]
    fn default_registry_is_empty() {
        assert!(default_registry().is_empty());
    }

    #[test]
    fn planned_pair_is_never_offered_even_when_engine_is_present() {
        let mut registry =
            CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::Planned)]);
        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Available,
        )]));
        assert_eq!(registry.available_count(), 0);
        assert!(registry
            .available_targets(&FormatId::new("png").expect("valid"))
            .is_empty());
    }

    #[test]
    fn stable_pair_requires_engine_availability() {
        let mut registry =
            CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::Stable)]);

        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Missing,
        )]));
        assert_eq!(
            registry.available_count(),
            0,
            "missing engine must not offer the pair"
        );

        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::IncompatibleVersion,
        )]));
        assert_eq!(
            registry.available_count(),
            0,
            "incompatible engine must not offer the pair"
        );

        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Available,
        )]));
        assert_eq!(
            registry.available_count(),
            1,
            "available engine must offer the pair"
        );
    }

    #[test]
    fn unknown_engine_never_offers_a_pair() {
        let mut registry =
            CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::Stable)]);
        registry.apply_availability(&BTreeMap::new());
        assert_eq!(registry.available_count(), 0);
    }

    #[test]
    fn experimental_pair_is_offerable() {
        let mut registry =
            CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::Experimental)]);
        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Available,
        )]));
        assert_eq!(registry.available_count(), 1);
    }

    #[test]
    fn not_supported_pair_is_never_offered() {
        let mut registry =
            CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::NotSupported)]);
        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Available,
        )]));
        assert_eq!(registry.available_count(), 0);
    }

    #[test]
    fn available_targets_are_scoped_to_the_source_format() {
        let mut registry = CapabilityRegistry::new(vec![
            capability("png", "pdf", PairStatus::Stable),
            capability("png", "webp", PairStatus::Stable),
            capability("jpg", "pdf", PairStatus::Stable),
        ]);
        registry.apply_availability(&engines_with(&[(
            "test-engine",
            EngineAvailability::Available,
        )]));

        let png = FormatId::new("png").expect("valid");
        let targets = registry.available_targets(&png);
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&&FormatId::new("pdf").expect("valid")));
        assert!(targets.contains(&&FormatId::new("webp").expect("valid")));
    }

    #[test]
    fn find_locates_a_pair_by_formats() {
        let registry = CapabilityRegistry::new(vec![capability("png", "pdf", PairStatus::Stable)]);
        let png = FormatId::new("png").expect("valid");
        let pdf = FormatId::new("pdf").expect("valid");
        let other = FormatId::new("gif").expect("valid");

        assert!(registry.find(&png, &pdf).is_some());
        assert!(registry.find(&png, &other).is_none());
    }

    #[test]
    fn a_planned_pair_declares_no_options() {
        let capability = capability("png", "pdf", PairStatus::Planned);

        assert!(!capability.options.quality);
        assert!(!capability.options.background);
    }

    #[test]
    fn the_options_are_part_of_the_wire_format() {
        let mut capability = capability("png", "jpeg", PairStatus::Stable);
        capability.options = CapabilityOptions {
            quality: true,
            background: true,
        };

        let json = serde_json::to_value(&capability).expect("serializable");

        assert_eq!(
            json.get("options")
                .and_then(|options| options.get("quality")),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            json.get("options")
                .and_then(|options| options.get("background")),
            Some(&serde_json::Value::Bool(true))
        );
    }
}
