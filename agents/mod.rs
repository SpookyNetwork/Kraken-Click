//! Kraken Click — Agent Bridge Layer
//!
//! "Connect pointer to intelligence."
//!
//! Bridges the pointer system to AI models through the provider-agnostic
//! `cognition` layer. For provider implementations, see `providers/`.
//!
//! - context.rs — packages screen data for AI consumption
//! - router.rs — routes queries to the right backend
//! - (providers live in `../providers/`)

pub mod context;
pub mod router;

// Re-exports
pub use context::*;
pub use router::*;
