//! Re-exports of the framework-agnostic domain types.
//!
//! The Tauri layer re-exports `ufc-core` types so that command
//! signatures stay readable without deep paths. The domain logic itself
//! remains in `ufc-core`.

pub use ufc_core::prelude::*;
pub use ufc_core::CORE_VERSION;
