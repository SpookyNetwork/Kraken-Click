//! OCR fallback layer — text extraction from pixels.
//!
//! Multiple OCR engines supported:
//! 1. Windows OCR API (built-in, fastest on Windows 11 ARM64)
//! 2. Tesseract (local, open-source)
//! 3. Azure Computer Vision OCR (cloud, highest quality)

use crate::pointer::models::*;

/// OCR engine selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrEngine {
    /// Windows built-in OCR (fastest, works offline).
    WindowsOcr,
    /// Tesseract OCR.
    Tesseract {
        executable_path: Option<String>,
        data_path: Option<String>,
    },
    /// Azure Computer Vision.
    AzureOcr {
        endpoint: String,
        api_key: String,
    },
}

impl Default for OcrEngine {
    fn default() -> Self {
        OcrEngine::WindowsOcr
    }
}

/// OCR configuration.
#[derive(Debug, Clone)]
pub struct OcrConfig {
    pub engine: OcrEngine,
    pub language: String,
    pub timeout_secs: u64,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            engine: OcrEngine::default(),
            language: "en".into(),
            timeout_secs: 10,
        }
    }
}

/// A recognized text region.
#[derive(Debug, Clone)]
pub struct OcrResult {
    /// Recognized text.
    pub text: String,
    /// Bounding box of the text region.
    pub bounds: ScreenRegion,
    /// Confidence (0.0–1.0).
    pub confidence: f32,
}

/// OCR engine.
pub struct OcrEngineWrapper {
    config: OcrConfig,
}

impl OcrEngineWrapper {
    pub fn new(config: OcrConfig) -> Self {
        Self { config }
    }

    /// Extract all text from an image.
    pub fn extract_text(&self, _image_data: &[u8]) -> Result<Vec<OcrResult>, OcrError> {
        match &self.config.engine {
            OcrEngine::WindowsOcr => self.extract_via_windows_ocr(),
            OcrEngine::Tesseract { .. } => self.extract_via_tesseract(),
            OcrEngine::AzureOcr { .. } => self.extract_via_azure(),
        }
    }

    fn extract_via_windows_ocr(&self) -> Result<Vec<OcrResult>, OcrError> {
        // TODO: Windows.Media.Ocr via windows-rs
        Err(OcrError::EngineUnavailable("Windows OCR not yet implemented".into()))
    }

    fn extract_via_tesseract(&self) -> Result<Vec<OcrResult>, OcrError> {
        // TODO: Shell out to tesseract CLI
        Err(OcrError::EngineUnavailable("Tesseract not yet implemented".into()))
    }

    fn extract_via_azure(&self) -> Result<Vec<OcrResult>, OcrError> {
        // TODO: REST API call to Azure Computer Vision
        Err(OcrError::EngineUnavailable("Azure OCR not yet implemented".into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrError {
    EngineUnavailable(String),
    ImageDecodeFailed,
    Timeout,
    Other(String),
}

use std::fmt;

impl fmt::Display for OcrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OcrError::EngineUnavailable(s) => write!(f, "OCR engine unavailable: {}", s),
            OcrError::ImageDecodeFailed => write!(f, "Failed to decode image for OCR"),
            OcrError::Timeout => write!(f, "OCR timed out"),
            OcrError::Other(s) => write!(f, "OCR error: {}", s),
        }
    }
}

impl std::error::Error for OcrError {}
