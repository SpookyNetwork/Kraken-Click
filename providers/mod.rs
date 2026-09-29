//! Kraken Click — Cognitive Provider Registry
//!
//! "Pluggable intelligence backends."
//!
//! All cognitive providers live here. Each provider implements the
//! `CognitiveRuntime` trait from the `cognition` layer, making them
//! interchangeable at runtime.
//!
//! # Available Providers
//!
//! | Provider | Module | Vision | Structured Output | Tool Calling |
//! |----------|--------|--------|-------------------|--------------|
//! | Anthropic | `anthropic` | ✅ | ✅ | ✅ |
//! | OpenAI | `openai` | ✅ | ✅ | ✅ |
//! | Google Gemini | `gemini` | ✅ | ✅ | ✅ |
//! | Local VLM | `local` | ✅ | ❌ | ❌ |
//!
//! # Adding a New Provider
//!
//! 1. Create a new file: `providers/<name>.rs`
//! 2. Implement `CognitiveRuntime` (+ optionally `CognitiveCapabilities`)
//! 3. Add `pub mod <name>;` below
//! 4. Re-export key types if needed

pub mod anthropic;
// pub mod openai;
// pub mod gemini;
// pub mod local;
