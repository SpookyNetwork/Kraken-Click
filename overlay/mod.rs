//! Kraken Click — Cursor Overlay + Caption System
//!
//! "Show, don't tell."
//!
//! Renders transparent overlay windows on each monitor for:
//! - Cursor companion (triangle that zips to targets)
//! - Caption bubbles with text
//! - Multi-marker support
//! - Animated pointing choreography
//!
//! Uses Direct2D for hardware-accelerated rendering on Windows ARM64.

pub mod window;
pub mod cursor;
pub mod caption;
pub mod renderer;

// Re-exports
pub use window::*;
pub use cursor::*;
pub use caption::*;
pub use renderer::*;
