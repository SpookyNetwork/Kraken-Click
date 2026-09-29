//! 🦑 Kraken Click — KRK Pointer Control Subsystem
//!
//! A real-time perception + interaction layer for agents.
//! Plugs into the KRK-OS control plane via Kraken Code.
//!
//! # Architecture
//!
//! ```text
//! KRK-OS CORE → Kraken CLI → Kraken Click → Windows Host
//!                                    │
//!                    ┌───────────────┼───────────────┐
//!                    ▼               ▼               ▼
//!               pointer/        vision/         overlay/
//!               (detection)     (capture)       (rendering)
//!                    │               │               │
//!                    └───────────────┼───────────────┘
//!                                    ▼
//!                               actions/
//!                           (PSC-gated execution)
//!                                    │
//!                                    ▼
//!                             cognition/
//!                      (provider-agnostic trait)
//!                                    │
//!                    ┌───────────────┼───────────────┐
//!                    ▼               ▼               ▼
//!              providers::     providers::     providers::
//!              anthropic       openai          gemini
//! ```
//!
//! # Core Principles (DeepMind AI Pointer)
//!
//! 1. **Maintain the flow** — AI works across all apps, no "AI detours"
//! 2. **Show and tell** — Pointer captures visual/semantic context
//! 3. **"This" and "That"** — Natural language shorthand + pointing
//! 4. **Turn pixels into actionable entities** — AI understands what you're pointing at
//!
//! # Platform
//!
//! - Windows 11 ARM64 (Surface)
//! - Rust core + TypeScript bridge layers
//! - All actions gated by KRK-PSC

pub mod pointer;
pub mod vision;
pub mod overlay;
pub mod actions;
pub mod agents;
pub mod integrations;
pub mod cognition;
pub mod providers;

// ── Re-exports for convenience ──

// Pointer types
pub use pointer::{
    ScreenPoint, ScreenRegion,
    UIElement, EntityType, SemanticLabel,
    PointerState, PointerMode, PointerMarker,
    DeicticReference,
    ElementDetector, DetectorConfig, DetectorError,
    DetectionStrategy,
};

// Vision types
pub use vision::{
    ScreenFrame,
    ScreenCapture, CaptureConfig, CaptureError, CaptureBackend,
    VisionAnalyzer, AnalyzerConfig, VisionError, VisionProvider,
    OcrEngineWrapper, OcrConfig, OcrEngine, OcrError, OcrResult,
};

// Overlay types
pub use overlay::{
    OverlayConfig, OverlayWindow, OverlayManager, OverlayError,
    CursorConfig, CursorRenderer, CursorAnimation, AnimationEasing,
    CaptionConfig, CaptionBubble, MarkerLayout,
    D2DRenderer, RendererConfig, RenderError,
};

// Action types
pub use actions::{
    PointerAction, MouseButton, ClickType, ModifierKey,
    GateConfig, GateDecision, PscGate,
    ClickExecutor, InputExecutor, WindowExecutor,
    ActionError,
};

// Agent types
pub use agents::{
    ScreenContext, ContextPackager, ContextError,
    PointerQuery, QueryResult, AgentRouter, RouterError,
};

// Cognition types
pub use cognition::{
    CognitiveRuntime, CognitiveCapabilities, CognitiveProvider,
    RuntimeContext, InferenceRequest, InferenceResult, InferenceError,
    TokenUsage, QueryType,
};
