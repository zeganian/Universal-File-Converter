//! Engine adapters, process infrastructure and the conversion service.
//!
//! This crate contains everything that talks to the outside world:
//! the process runner, the engine adapters and the orchestration of a
//! conversion. It does not depend on Tauri, so every part of it can be
//! tested with a plain `cargo test`.
//!
//! The business rules themselves live in `ufc-core`. This crate only
//! implements them.

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod conversion;
pub mod engines;
pub mod inspection;
pub mod output;
pub mod process;
pub mod validation;

pub use conversion::{ConversionService, JobObserver};
pub use engines::{adapters, engine_statuses, EngineAdapter, NeverCancelled};
pub use process::{ProcessOutcome, ProcessRequest, ProcessRunner};
