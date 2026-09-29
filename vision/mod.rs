//! Kraken Click — Vision / Screen Understanding Layer
//!
//! "See and understand the screen."
//!
//! Three components:
//! - **capture.rs** — Screen capture (Desktop Duplication API)
//! - **analyzer.rs** — Vision model integration (provider-agnostic)
//! - **ocr.rs** — OCR fallback (Windows OCR / Tesseract / Azure)

pub mod capture;
pub mod analyzer;
pub mod ocr;

// Re-exports
pub use capture::*;
pub use analyzer::*;
pub use ocr::*;
