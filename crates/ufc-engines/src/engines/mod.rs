//! Engine adapters.
//!
//! Every adapter implements the contract of `ufc-core`. The adapters
//! are the only place that knows how a specific program is called.

pub mod detection;
pub mod imagemagick;

pub use ufc_core::engines::adapter::{
    CancellationSignal, EngineAdapter, EngineInvocation, EngineOutcome, NeverCancelled,
    ValidatedConversion,
};

use std::sync::Arc;

use ufc_core::domain::conversion::EngineStatus;

/// Returns every adapter of the application.
///
/// The adapters detect their installation on this device, so the list
/// already reflects what is actually available.
pub fn adapters() -> Vec<Arc<dyn EngineAdapter>> {
    vec![Arc::new(imagemagick::ImageMagickAdapter::detect())]
}

/// Returns the status of every adapter.
pub fn engine_statuses(adapters: &[Arc<dyn EngineAdapter>]) -> Vec<EngineStatus> {
    adapters
        .iter()
        .map(|adapter| adapter.detect_installation())
        .collect()
}
