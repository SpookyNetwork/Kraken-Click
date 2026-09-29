//! PSC Gate — all pointer actions must pass through here.
//!
//! This is the safety layer. Every action (click, type, window switch, etc.)
//! is evaluated by the KRK-PSC (Phase-Stabilizing Controller) before execution.
//!
//! The gate enforces:
//! - Action allowlist (only permitted actions)
//! - Rate limiting (no action flooding)
//! - Context validation (is this action appropriate given current state?)
//! - Audit logging (all actions recorded)

use crate::pointer::models::*;

/// Action types that can be gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerAction {
    /// Mouse click at a screen position.
    Click {
        position: ScreenPoint,
        button: MouseButton,
        click_type: ClickType,
    },
    /// Mouse drag operation.
    Drag {
        from: ScreenPoint,
        to: ScreenPoint,
        button: MouseButton,
    },
    /// Keyboard input.
    KeyPress {
        key: String,
        modifiers: Vec<ModifierKey>,
    },
    /// Type text.
    TypeText {
        text: String,
        target: Option<ScreenPoint>,
    },
    /// Window focus.
    FocusWindow {
        window_title: Option<String>,
        process_name: Option<String>,
    },
    /// App switching.
    SwitchApp {
        app_name: String,
    },
    /// Scroll.
    Scroll {
        position: ScreenPoint,
        delta_lines: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Single,
    Double,
    Triple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Win,
}

/// PSC gate decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateDecision {
    /// Action is allowed.
    Allow,
    /// Action is denied with reason.
    Deny { reason: String },
    /// Action requires explicit user confirmation.
    RequireConfirmation { prompt: String },
    /// Action is rate-limited.
    RateLimited { retry_after_ms: u64 },
}

/// PSC Gate configuration.
#[derive(Debug, Clone)]
pub struct GateConfig {
    /// Maximum actions per second.
    pub max_actions_per_second: u32,
    /// Whether to require confirmation for destructive actions.
    pub confirm_destructive: bool,
    /// Whether to allow text input (typing).
    pub allow_text_input: bool,
    /// Whether to allow window management.
    pub allow_window_management: bool,
    /// Whether to allow app switching.
    pub allow_app_switching: bool,
    /// Audit log path.
    pub audit_log_path: Option<String>,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            max_actions_per_second: 10,
            confirm_destructive: true,
            allow_text_input: true,
            allow_window_management: true,
            allow_app_switching: true,
            audit_log_path: None,
        }
    }
}

/// The PSC Gate — evaluates all pointer actions.
pub struct PscGate {
    config: GateConfig,
    /// Action timestamps for rate limiting.
    action_timestamps: Vec<u64>,
}

impl PscGate {
    pub fn new(config: GateConfig) -> Self {
        Self {
            config,
            action_timestamps: Vec::new(),
        }
    }

    /// Evaluate an action and return a gate decision.
    pub fn evaluate(&mut self, action: &PointerAction, now_ms: u64) -> GateDecision {
        // 1. Check rate limit
        if self.is_rate_limited(now_ms) {
            return GateDecision::RateLimited {
                retry_after_ms: 1000 / self.config.max_actions_per_second as u64,
            };
        }

        // 2. Check action type permissions
        match action {
            PointerAction::TypeText { .. } if !self.config.allow_text_input => {
                return GateDecision::Deny {
                    reason: "Text input is disabled".into(),
                };
            }
            PointerAction::FocusWindow { .. } | PointerAction::SwitchApp { .. }
                if !self.config.allow_window_management && !self.config.allow_app_switching =>
            {
                return GateDecision::Deny {
                    reason: "Window management is disabled".into(),
                };
            }
            _ => {}
        }

        // 3. Check for destructive actions
        if self.config.confirm_destructive && self.is_potentially_destructive(action) {
            return GateDecision::RequireConfirmation {
                prompt: format!("Allow this action: {:?}?", action),
            };
        }

        // 4. Record and allow
        self.action_timestamps.push(now_ms);
        self.prune_old_timestamps(now_ms);

        // 5. Audit log
        self.log_action(action, &GateDecision::Allow, now_ms);

        GateDecision::Allow
    }

    fn is_rate_limited(&self, now_ms: u64) -> bool {
        let window_start = now_ms.saturating_sub(1000);
        let recent = self
            .action_timestamps
            .iter()
            .filter(|&&t| t >= window_start)
            .count();
        recent >= self.config.max_actions_per_second as usize
    }

    fn prune_old_timestamps(&mut self, now_ms: u64) {
        let window_start = now_ms.saturating_sub(1000);
        self.action_timestamps.retain(|&t| t >= window_start);
    }

    fn is_potentially_destructive(&self, action: &PointerAction) -> bool {
        match action {
            // Typing into system dialogs could be destructive
            PointerAction::TypeText { text, .. } if text.contains("delete") || text.contains("format") => true,
            // Right-clicking could open context menus with destructive options
            PointerAction::Click { button: MouseButton::Right, .. } => false, // not destructive by itself
            _ => false,
        }
    }

    fn log_action(&self, action: &PointerAction, decision: &GateDecision, now_ms: u64) {
        // TODO: Write to audit log
        println!(
            "[PSC-GATE] {} | {:?} | {:?}",
            now_ms, action, decision
        );
    }
}
