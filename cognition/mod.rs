//! Kraken Click — Cognition Layer
//!
//! "Provider-agnostic intelligence interface."
//!
//! This module defines the abstraction that decouples the pointer system
//! from any specific AI provider. All inference requests flow through
//! the `CognitiveRuntime` trait, making providers interchangeable.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                  Kraken Click                       │
//! │                                                     │
//! │  pointer/  vision/  overlay/  actions/              │
//! │      │        │        │         │                  │
//! │      └────────┴────────┴─────────┘                  │
//! │                      │                              │
//! │              CognitiveRuntime (trait)               │
//! │                      │                              │
//! │         ┌────────────┼────────────┐                 │
//! │         ▼            ▼            ▼                 │
//! │   providers::   providers::   providers::          │
//! │   anthropic     openai        gemini              │
//! │                                                      │
//! └─────────────────────────────────────────────────────┘
//!
//! # Usage
//!
//! ```rust,no_run
//! use kraken_click::cognition::{CognitiveRuntime, RuntimeContext, InferenceRequest, InferenceResult};
//! use kraken_click::providers::anthropic::AnthropicProvider;
//!
//! // Initialize any provider
//! let provider = AnthropicProvider::new(config);
//!
//! // Use through the trait — provider-agnostic
//! let ctx = RuntimeContext::from_screen(screen_data);
//! let request = InferenceRequest::what_is_this(cursor_position, ctx);
//! let result = provider.infer(request).await?;
//! ```
//!
//! # Adding a New Provider
//!
//! 1. Create `providers/<name>.rs`
//! 2. Implement `CognitiveRuntime` for your provider struct
//! 3. Register in `providers/mod.rs`
//! 4. No other code changes needed — the rest of the system
//!    only interacts with the trait.

pub mod types;
pub mod runtime;

// Core types
pub use types::{
    RuntimeContext,
    InferenceRequest,
    InferenceResult,
    InferenceError,
    QueryType,
    TokenUsage,
};

// Agent screen context (packaged for inference)
pub use crate::agents::ScreenContext;

// Core traits and provider wrapper
pub use runtime::{CognitiveRuntime, CognitiveCapabilities, CognitiveProvider, BoxFuture};
