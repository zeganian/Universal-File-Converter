//! Framework-agnostic domain logic for Universal File Converter.
//!
//! This crate contains the business rules of the application: the job
//! state machine, the conversion capability registry, the structured
//! error model and the engine adapter contract.
//!
//! It deliberately has **no dependency on Tauri**, on any I/O framework
//! and on any GUI toolkit. That keeps the domain testable in isolation
//! (`cargo test -p ufc-core` runs in seconds and needs no system web
//! toolkit) and keeps the boundary between "what the app means" and
//! "how the app is displayed" explicit.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod domain;
pub mod engines;

/// Version of the domain core crate.
///
/// Exposed to the UI so that the application can report exactly which
/// domain logic it is running.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-export of the most frequently used domain types.
pub mod prelude {
    pub use crate::domain::conversion::{
        ConversionOptions, ConversionRequest, ConversionResult, EngineAvailability, EngineId,
        EngineSource, EngineStatus, OverwritePolicy, ValidationLevel,
    };
    pub use crate::domain::error::{ConversionError, ConversionErrorCode, DomainResult};
    pub use crate::domain::format::FormatId;
    pub use crate::domain::job::{JobStatus, Progress};
    pub use crate::domain::registry::{CapabilityRegistry, ConversionCapability, PairStatus};
    pub use crate::engines::adapter::{EngineAdapter, EngineInvocation, ValidatedConversion};
}
