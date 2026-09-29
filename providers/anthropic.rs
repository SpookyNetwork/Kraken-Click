//! Anthropic provider — implements CognitiveRuntime for Anthropic's API.
//!
//! This provider connects to Anthropic's Claude models through the
//! standard Anthropic API. It is one of several interchangeable
//! cognition backends.
//!
//! Note: "Claude" here refers to the model family name, not the
//! system identity. The architecture is provider-agnostic.

use crate::cognition::{
    CognitiveCapabilities, CognitiveRuntime,
    InferenceRequest, InferenceResult, InferenceError as Error,
    TokenUsage,
};
use crate::pointer::models::*;
use crate::pointer::detector::DetectorConfig;
use crate::vision::analyzer::*;
use crate::vision::capture::CaptureConfig;
use crate::agents::context::*;

/// Configuration for the Anthropic provider.
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key for Anthropic.
    pub api_key: String,
    /// Model to use (e.g., "claude-sonnet-4-6", "claude-opus-4-7").
    pub model: String,
    /// API base URL (defaults to official Anthropic endpoint).
    pub base_url: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum tokens in response.
    pub max_tokens: u32,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "claude-sonnet-4-6".into(),
            base_url: "https://api.anthropic.com".into(),
            timeout_secs: 30,
            max_tokens: 4096,
        }
    }
}

/// Anthropic provider — implements the CognitiveRuntime trait.
///
/// This is a provider implementation, not an architectural identity.
/// The system can use Anthropic, OpenAI, Gemini, or local models
/// interchangeably through the CognitiveRuntime trait.
pub struct AnthropicProvider {
    config: AnthropicConfig,
    analyzer: VisionAnalyzer,
    context: ContextPackager,
}

impl AnthropicProvider {
    pub fn new(
        config: AnthropicConfig,
        analyzer_config: AnalyzerConfig,
        capture_config: CaptureConfig,
        detector_config: DetectorConfig,
    ) -> Result<Self, Error> {
        Ok(Self {
            config,
            analyzer: VisionAnalyzer::new(analyzer_config),
            context: ContextPackager::new(capture_config, detector_config)
                .map_err(|e| Error::Internal(e.to_string()))?,
        })
    }

    /// Send a "what is this?" query with screen context.
    pub fn ask_what_is_this(
        &mut self,
        position: &ScreenPoint,
    ) -> Result<String, Error> {
        let ctx = self.context.package_cursor_context(200)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let description = self.context.build_text_description(&ctx);

        let prompt = format!(
            "The user is asking \"What is this?\" about a screen position.\n\
             Cursor position: {}\n\
             Screen context:\n{}\n\
             Based on the above, provide a concise answer about what the user is pointing at.\n\
             If you can identify the element, describe it in one sentence.",
            position, description
        );

        // TODO: Send to Anthropic API
        Err(Error::Internal("Anthropic provider not yet fully implemented".into()))
    }

    /// Send a "find and click" request.
    pub fn ask_find_and_click(
        &mut self,
        description: &str,
    ) -> Result<UIElement, Error> {
        let ctx = self.context.package_full()
            .map_err(|e| Error::Internal(e.to_string()))?;
        let text_desc = self.context.build_text_description(&ctx);

        let prompt = format!(
            "The user wants to click on: \"{}\"\n\
             Screen context:\n{}\n\
             Find the element matching the description and return its:\n\
             - label: exact text or name\n\
             - position: x, y coordinates of center\n\
             - confidence: how sure you are (0.0-1.0)\n\
             Return as JSON.",
            description, text_desc
        );

        // TODO: Send to Anthropic API → parse response → return UIElement
        Err(Error::Internal("Anthropic provider not yet fully implemented".into()))
    }

    /// Use Computer Use API for pixel-accurate element detection.
    pub fn computer_use_find(
        &mut self,
        _description: &str,
    ) -> Result<Option<ScreenPoint>, Error> {
        // TODO: Use Anthropic's Computer Use tool definition
        Err(Error::Internal("Computer Use not yet implemented".into()))
    }
}

impl CognitiveRuntime for AnthropicProvider {
    fn infer(
        &self,
        _request: InferenceRequest,
    ) -> crate::cognition::runtime::BoxFuture<'_, Result<InferenceResult, Error>> {
        let model = self.config.model.clone();
        Box::pin(async move {
            Ok(InferenceResult {
                provider_id: "anthropic".into(),
                model_id: model,
                text: String::new(),
                structured_data: None,
                usage: TokenUsage::default(),
                cached: false,
            })
        })
    }

    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn model_id(&self) -> &str {
        &self.config.model
    }
}

impl CognitiveCapabilities for AnthropicProvider {
    fn supports_vision(&self) -> bool {
        true
    }

    fn supports_structured_output(&self) -> bool {
        true
    }

    fn max_context_tokens(&self) -> u32 {
        200_000
    }

    fn supports_tool_calling(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// Errors specific to the Anthropic provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnthropicError {
    NotImplemented,
    ApiError(String),
    ContextError(String),
    ParseError(String),
    RateLimited,
    AuthenticationFailed,
}

impl std::fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnthropicError::NotImplemented => write!(f, "Anthropic provider not yet implemented"),
            AnthropicError::ApiError(s) => write!(f, "Anthropic API error: {}", s),
            AnthropicError::ContextError(s) => write!(f, "Context error: {}", s),
            AnthropicError::ParseError(s) => write!(f, "Parse error: {}", s),
            AnthropicError::RateLimited => write!(f, "Rate limited by Anthropic"),
            AnthropicError::AuthenticationFailed => write!(f, "Anthropic authentication failed"),
        }
    }
}

impl std::error::Error for AnthropicError {}

impl From<AnthropicError> for Error {
    fn from(err: AnthropicError) -> Self {
        match err {
            AnthropicError::NotImplemented => Error::Internal("Not implemented".into()),
            AnthropicError::ApiError(s) => Error::ApiError {
                provider: "anthropic".into(),
                message: s,
            },
            AnthropicError::ContextError(s) => Error::Internal(s),
            AnthropicError::ParseError(s) => Error::ParseError(s),
            AnthropicError::RateLimited => Error::RateLimited("anthropic".into()),
            AnthropicError::AuthenticationFailed => {
                Error::AuthenticationFailed("anthropic".into())
            }
        }
    }
}
