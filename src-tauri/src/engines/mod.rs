//! Engine integration layer.
//!
//! The adapter trait lives in `ufc-core`. This module hosts the concrete
//! adapters, none of which exist yet.

pub use ufc_core::engines::adapter::{
    CancellationSignal, EngineAdapter, EngineInvocation, EngineOutcome, NeverCancelled,
    ValidatedConversion,
};
