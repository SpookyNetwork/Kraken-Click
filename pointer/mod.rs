//! Kraken Click — Pointer Intelligence Layer
//!
//! "What is under the cursor?"
//!
//! Provides UI element detection, semantic labeling, and deictic reference
//! resolution for the Kraken Click pointer control subsystem.
//!
//! # Architecture
//!
//! Three detection strategies:
//! 1. **Windows UI Automation** — structured element tree (fast, no ML)
//! 2. **OCR** — text extraction from pixels (Tesseract/Azure/Windows OCR)
//! 3. **Vision Model** — semantic understanding (provider-agnostic)
//!
//! # Example
//!
//! ```rust,no_run
//! use kraken_click::pointer::{ElementDetector, DetectorConfig, ScreenPoint};
//!
//! let config = DetectorConfig::default();
//! let mut detector = ElementDetector::new(config).unwrap();
//!
//! // "What's under the cursor?"
//! let cursor_pos = ScreenPoint::new(960, 540);
//! if let Some(element) = detector.detect_at(&cursor_pos).unwrap() {
//!     println!("Hovering over: {} ({})", element.label, element.entity_type);
//! }
//!
//! // "Find the Submit button"
//! let buttons = detector.find_by_label("Submit").unwrap();
//! for btn in &buttons {
//!     println!("Found: {} at {}", btn.label, btn.bounds.center());
//! }
//! ```

pub mod models;
pub mod detector;
pub mod semantic;

// Re-exports
pub use models::*;
pub use detector::*;
pub use semantic::*;
