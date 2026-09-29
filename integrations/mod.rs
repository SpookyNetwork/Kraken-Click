//! Kraken Click — Windows Integration Layer
//!
//! FFI bindings and interop with Windows APIs.

pub mod winrt;
pub mod accessibility;

// Re-exports
pub use winrt::*;
pub use accessibility::*;
