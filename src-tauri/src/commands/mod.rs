//! Typed command boundary exposed to the frontend.
//!
//! Every command is narrow, typed and validated in Rust. The frontend
//! never executes a process, never passes an executable path and never
//! passes a raw argument vector (`ARCHITECTURE.md` section 11).

pub mod app;
pub mod capabilities;
pub mod engines;

pub use app::CommandError;
