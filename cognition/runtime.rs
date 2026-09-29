//! CognitiveRuntime — the provider-agnostic inference trait.
//!
//! This is the core abstraction that decouples Kraken Click from any
//! specific AI provider. All inference requests flow through this trait.
//!
//! # Design Principles
//!
//! 1. **Provider-agnostic**: The rest of the system never knows which
//!    provider is handling a request.
//! 2. **Async-first**: All inference is async to support network calls.
//! 3. **Error unification**: All provider-specific errors are mapped
//!    to the unified `InferenceError` type.
//! 4. **Context-driven**: Providers receive a rich `RuntimeContext`
//!    with both structured data and optional screenshots.
//!
//! # Implementing a Provider
//!
//! ```rust,no_run
//! use kraken_click::cognition::{CognitiveRuntime, CognitiveCapabilities, InferenceRequest, InferenceResult, InferenceError, BoxFuture};
//!
//! struct MyProvider {
//!     api_key: String,
//!     model: String,
//! }
//!
//! impl CognitiveRuntime for MyProvider {
//!     fn infer(&self, request: InferenceRequest) -> BoxFuture<'_, Result<InferenceResult, InferenceError>> {
//!         // 1. Convert InferenceRequest to provider-specific format
//!         // 2. Send to provider API
//!         // 3. Convert response to InferenceResult
//!         // 4. Map errors to InferenceError
//!         Box::pin(async { todo!() })
//!     }
//!
//!     fn provider_id(&self) -> &str {
//!         "my-provider"
//!     }
//!
//!     fn model_id(&self) -> &str {
//!         &self.model
//!     }
//! }
//!
//! // Optional: implement provider-specific capabilities
//! impl CognitiveCapabilities for MyProvider {
//!     fn supports_vision(&self) -> bool { true }
//!     fn supports_structured_output(&self) -> bool { true }
//!     fn max_context_tokens(&self) -> u32 { 128_000 }
//!     fn supports_tool_calling(&self) -> bool { true }
//!     fn supports_streaming(&self) -> bool { true }
//! }

use super::types::{InferenceError, InferenceRequest, InferenceResult};

/// The core trait that all cognitive providers must implement.
///
/// This is the single interface through which the entire pointer system
/// interacts with AI models. It is intentionally minimal — providers
/// only need to implement `infer()` and metadata methods.
///
/// # Example
///
/// ```rust,no_run
/// # use kraken_click::cognition::{CognitiveRuntime, InferenceRequest, InferenceResult, InferenceError, BoxFuture};
/// # struct MyProvider;
/// # impl CognitiveRuntime for MyProvider {
/// #     fn infer(&self, request: InferenceRequest) -> BoxFuture<'_, Result<InferenceResult, InferenceError>> {
/// #         Box::pin(async { todo!() })
/// #     }
/// #     fn provider_id(&self) -> &str { "my-provider" }
/// #     fn model_id(&self) -> &str { "my-model" }
/// # }
/// ```
use std::pin::Pin;
use std::future::Future;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait CognitiveRuntime: Send + Sync {
    /// Execute an inference request.
    ///
    /// This is the primary method — it takes a provider-agnostic request
    /// and returns a provider-agnostic result.
    ///
    /// # Errors
    ///
    /// Must return `InferenceError` variants, not provider-specific errors.
    /// Map all provider errors to the unified error type.
    fn infer(
        &self,
        request: InferenceRequest,
    ) -> BoxFuture<'_, Result<InferenceResult, InferenceError>>;

    /// Return the provider identifier (e.g., "anthropic", "openai", "gemini").
    fn provider_id(&self) -> &str;

    /// Return the model identifier (e.g., "claude-sonnet-4-6", "gpt-4o").
    fn model_id(&self) -> &str;

    /// Check if the provider is currently available (API reachable, key valid).
    fn is_available(&self) -> BoxFuture<'_, bool> {
        Box::pin(async { true })
    }
}

/// Optional trait for providers that want to advertise capabilities.
///
/// This allows the system to make intelligent routing decisions:
/// - Send vision queries only to vision-capable providers
/// - Route structured output requests to providers that support it
/// - Respect context window limits
pub trait CognitiveCapabilities: CognitiveRuntime {
    /// Whether this provider supports image/screenshot input.
    fn supports_vision(&self) -> bool;

    /// Whether this provider supports structured JSON output.
    fn supports_structured_output(&self) -> bool;

    /// Maximum context window size in tokens.
    fn max_context_tokens(&self) -> u32;

    /// Whether this provider supports tool/function calling.
    fn supports_tool_calling(&self) -> bool;

    /// Whether this provider supports streaming responses.
    fn supports_streaming(&self) -> bool;
}

/// A provider that can be used as a trait object.
///
/// This allows storing heterogeneous providers in a collection
/// and routing between them dynamically.
pub struct CognitiveProvider {
    inner: Box<dyn CognitiveRuntime>,
}

impl CognitiveProvider {
    pub fn new<P: CognitiveRuntime + 'static>(provider: P) -> Self {
        Self {
            inner: Box::new(provider),
        }
    }

    pub fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    pub fn model_id(&self) -> &str {
        self.inner.model_id()
    }
}

impl CognitiveRuntime for CognitiveProvider {
    fn infer(
        &self,
        request: InferenceRequest,
    ) -> BoxFuture<'_, Result<InferenceResult, InferenceError>> {
        self.inner.infer(request)
    }

    fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    fn model_id(&self) -> &str {
        self.inner.model_id()
    }

    fn is_available(&self) -> BoxFuture<'_, bool> {
        self.inner.is_available()
    }
}
