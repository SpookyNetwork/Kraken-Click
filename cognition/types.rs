//! Cognition layer types — provider-agnostic data structures.
//!
//! These types define the interface between the pointer system and
//! any cognitive provider. No provider-specific types leak into this layer.

use crate::pointer::models::{ScreenPoint, ScreenRegion, UIElement, SemanticLabel};

/// The context sent to a cognitive provider for inference.
///
/// Packages everything the provider needs to reason about the screen:
/// - screenshot data (optional, for vision-capable providers)
/// - detected UI elements (from fast local detection)
/// - cursor position and query type
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    /// Screenshot of the relevant screen region (JPEG-encoded).
    /// `None` if only structured data is being sent.
    pub screenshot: Option<Vec<u8>>,
    /// Pre-detected UI elements from local detection (UIA/OCR).
    pub detected_elements: Vec<UIElement>,
    /// The screen region of interest.
    pub region: ScreenRegion,
    /// Cursor position at time of query.
    pub cursor_position: ScreenPoint,
    /// Human-readable description of the screen context.
    pub text_description: String,
    /// Active application name (if known).
    pub active_app: Option<String>,
    /// Window title (if known).
    pub window_title: Option<String>,
}

impl RuntimeContext {
    /// Build a minimal context from just a screen region and cursor position.
    pub fn minimal(region: ScreenRegion, cursor: ScreenPoint) -> Self {
        Self {
            screenshot: None,
            detected_elements: Vec::new(),
            region,
            cursor_position: cursor,
            text_description: String::new(),
            active_app: None,
            window_title: None,
        }
    }

    /// Attach a screenshot to the context.
    pub fn with_screenshot(mut self, data: Vec<u8>) -> Self {
        self.screenshot = Some(data);
        self
    }

    /// Attach pre-detected elements.
    pub fn with_elements(mut self, elements: Vec<UIElement>) -> Self {
        self.detected_elements = elements;
        self
    }

    /// Attach a text description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.text_description = desc.into();
        self
    }
}

/// The type of query being sent to the cognitive provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    /// "What is this?" — identify what's at a specific position.
    WhatIsThis,
    /// "Find X" — locate an element by natural language description.
    FindElement { description: String },
    /// "Click X" — find and click an element.
    ClickElement { description: String },
    /// "Type X in Y" — find a field and type text.
    TypeInField { text: String, field: String },
    /// "Summarize this" — summarize screen content.
    SummarizeScreen,
    /// "What can I click?" — list interactable elements.
    ListInteractable,
    /// "Is this correct?" — validate a proposed action.
    ValidateAction { action_description: String },
    /// Custom query with a free-form prompt.
    Custom { prompt: String },
}

/// A request sent to a cognitive provider.
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// The type of query.
    pub query: QueryType,
    /// The screen context for the query.
    pub context: RuntimeContext,
    /// Maximum response length (in tokens, if applicable).
    pub max_response_tokens: Option<u32>,
    /// Whether to include a screenshot in the request.
    pub include_screenshot: bool,
}

impl InferenceRequest {
    /// Create a "What is this?" request.
    pub fn what_is_this(position: ScreenPoint, context: RuntimeContext) -> Self {
        Self {
            query: QueryType::WhatIsThis,
            context,
            max_response_tokens: Some(256),
            include_screenshot: true,
        }
    }

    /// Create a "Find element" request.
    pub fn find_element(description: impl Into<String>, context: RuntimeContext) -> Self {
        Self {
            query: QueryType::FindElement {
                description: description.into(),
            },
            context,
            max_response_tokens: Some(512),
            include_screenshot: true,
        }
    }

    /// Create a "Summarize screen" request.
    pub fn summarize(context: RuntimeContext) -> Self {
        Self {
            query: QueryType::SummarizeScreen,
            context,
            max_response_tokens: Some(1024),
            include_screenshot: true,
        }
    }

    /// Create a custom prompt request.
    pub fn custom(prompt: impl Into<String>, context: RuntimeContext) -> Self {
        Self {
            query: QueryType::Custom {
                prompt: prompt.into(),
            },
            context,
            max_response_tokens: Some(2048),
            include_screenshot: false,
        }
    }
}

/// The result returned by a cognitive provider.
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// The provider that generated this result.
    pub provider_id: String,
    /// The model that generated this result.
    pub model_id: String,
    /// Text response from the provider.
    pub text: String,
    /// Structured data extracted from the response (if any).
    pub structured_data: Option<serde_json::Value>,
    /// Token usage statistics.
    pub usage: TokenUsage,
    /// Whether this result was served from cache.
    pub cached: bool,
}

impl InferenceResult {
    /// Create a simple text result.
    pub fn text(provider: impl Into<String>, model: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            provider_id: provider.into(),
            model_id: model.into(),
            text: text.into(),
            structured_data: None,
            usage: TokenUsage::default(),
            cached: false,
        }
    }

    /// Parse the response text as JSON.
    pub fn parse_json(&self) -> Result<serde_json::Value, InferenceError> {
        self.structured_data.clone().ok_or_else(|| {
            InferenceError::ParseError(
                "No structured data in response".into()
            )
        })
    }
}

/// Token usage statistics from a provider.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    /// Tokens in the prompt/input.
    pub input_tokens: u32,
    /// Tokens in the response/output.
    pub output_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
    /// Estimated cost in USD (if available).
    pub estimated_cost_usd: Option<f64>,
}

impl TokenUsage {
    pub fn new(input: u32, output: u32) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
            estimated_cost_usd: None,
        }
    }
}

/// Errors that can occur during inference.
#[derive(Debug, Clone, thiserror::Error)]
pub enum InferenceError {
    #[error("Provider not available: {0}")]
    ProviderUnavailable(String),

    #[error("API error from {provider}: {message}")]
    ApiError { provider: String, message: String },

    #[error("Request timed out after {0}s")]
    Timeout(u64),

    #[error("Rate limited by provider: {0}")]
    RateLimited(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Authentication failed for provider: {0}")]
    AuthenticationFailed(String),

    #[error("Context too large: {0} tokens (max {1})")]
    ContextTooLarge(u32, u32),

    #[error("Provider returned no content")]
    EmptyResponse,

    #[error("Internal error: {0}")]
    Internal(String),
}
