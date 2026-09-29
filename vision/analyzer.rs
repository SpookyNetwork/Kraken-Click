//! Vision model integration — semantic screen understanding.
//!
//! Connects to vision-language models for "what is this?" queries.
//! Supports multiple backends through the provider-agnostic
//! `cognition` layer:
//! - Anthropic (Claude models, Computer Use API for element detection)
//! - Google Gemini (native multimodal)
//! - OpenAI GPT-4V
//! - Local VLM (llama.cpp / ONNX Runtime)

use crate::pointer::models::*;

/// Vision model provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionProvider {
    Anthropic {
        model: String,
        api_key: String,
    },
    GoogleGemini {
        model: String,
        api_key: String,
    },
    OpenAIGpt4V {
        model: String,
        api_key: String,
    },
    LocalVLM {
        endpoint: String, // e.g., "http://localhost:8080"
    },
    OpenRouter {
        model: String,
        api_key: String,
    },
}

/// Configuration for the vision analyzer.
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Primary vision provider.
    pub primary: VisionProvider,
    /// Fallback provider (if primary fails).
    pub fallback: Option<VisionProvider>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Max image dimension (images are resized to this before sending).
    pub max_image_dim: u32,
    /// JPEG quality for image encoding.
    pub jpeg_quality: u8,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            primary: VisionProvider::Anthropic {
                model: "claude-sonnet-4-6".into(),
                api_key: String::new(),
            },
            fallback: None,
            timeout_secs: 15,
            max_image_dim: 1280,
            jpeg_quality: 80,
        }
    }
}

/// Vision analyzer — sends screenshots to AI models for understanding.
pub struct VisionAnalyzer {
    config: AnalyzerConfig,
}

impl VisionAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    /// Analyze a screenshot and return all detected elements with semantic labels.
    pub fn analyze_screenshot(
        &self,
        _image_data: &[u8],
    ) -> Result<Vec<SemanticLabel>, VisionError> {
        // TODO: Send image to vision model with prompt:
        // "Analyze this screenshot. For each visible UI element, provide:
        //  - bounding_box: [x, y, width, height] in pixels
        //  - element_type: button|text_field|menu|image|text|etc
        //  - label: human-readable description
        //  - text_content: any text in the element (OCR)
        //  - interactable: true/false
        //  - confidence: 0.0-1.0
        // Return as JSON array."
        Err(VisionError::NotImplemented)
    }

    /// "What is this?" — analyze a specific region around a point.
    pub fn analyze_region(
        &self,
        _image_data: &[u8],
        _region: &ScreenRegion,
    ) -> Result<Option<SemanticLabel>, VisionError> {
        // TODO: Crop to region + send to vision model
        Err(VisionError::NotImplemented)
    }

    /// "Find me X" — locate elements matching a description.
    pub fn find_elements(
        &self,
        _image_data: &[u8],
        _description: &str,
    ) -> Result<Vec<UIElement>, VisionError> {
        // TODO: Send image + find instruction to vision model
        // Uses provider's Computer Use / tool-calling API for pixel-accurate coordinates
        Err(VisionError::NotImplemented)
    }

    /// "Is this text readable?" — extract text from a region via vision model.
    pub fn extract_text(
        &self,
        _image_data: &[u8],
        _region: Option<&ScreenRegion>,
    ) -> Result<String, VisionError> {
        // TODO: Vision-based OCR (better than Tesseract for complex layouts)
        Err(VisionError::NotImplemented)
    }
}

/// Vision analysis errors.
#[derive(Debug, Clone, PartialEq, Eq)]
 pub enum VisionError {
    NotImplemented,
    ApiError(String),
    Timeout,
    RateLimited,
    ImageTooLarge,
    ModelUnavailable,
    ParseError(String),
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisionError::NotImplemented => write!(f, "Vision analysis not yet implemented"),
            VisionError::ApiError(s) => write!(f, "API error: {}", s),
            VisionError::Timeout => write!(f, "Vision analysis timed out"),
            VisionError::RateLimited => write!(f, "Rate limited by vision provider"),
            VisionError::ImageTooLarge => write!(f, "Image too large for vision API"),
            VisionError::ModelUnavailable => write!(f, "Vision model unavailable"),
            VisionError::ParseError(s) => write!(f, "Failed to parse vision response: {}", s),
        }
    }
}

impl std::error::Error for VisionError {}

use std::fmt;
