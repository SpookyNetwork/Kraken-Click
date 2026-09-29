//! Kraken Click — Action Layer (PSC-Gated)
//!
//! "Act on the screen, safely."
//!
//! All actions pass through the KRK-PSC gate before execution:
//! - Click/drag operations
//! - Keyboard input + text typing
//! - Window focus + app switching
//!
//! Every action is:
//! 1. Evaluated by PSC gate (rate limit, permissions, destructive check)
//! 2. Logged to audit trail
//! 3. Executed via Windows SendInput / WinAPI

pub mod gate;
pub mod click;
pub mod input;
pub mod window;

// Re-exports
pub use gate::*;
pub use click::*;
pub use input::*;
pub use window::*;
