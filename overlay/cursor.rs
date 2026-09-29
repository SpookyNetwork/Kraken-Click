//! Cursor companion — animated pointing choreography.
//!
//! Renders a triangle that zips from cursor to target element,
//! shows a caption, then returns.
//!
//! Animation timeline:
//! 1. Triangle appears at cursor (0ms)
//! 2. Triangle zips to target (400ms)
//! 3. Caption fades in at target (200ms)
//! 4. Hold (2000ms)
//! 5. Caption fades out (300ms)
//! 6. Triangle returns to cursor (400ms)

use crate::pointer::models::*;

/// Cursor rendering configuration.
#[derive(Debug, Clone)]
pub struct CursorConfig {
    /// Triangle size in pixels.
    pub triangle_size: f32,
    /// Accent color (hex).
    pub accent_color: String,
    /// Animation easing function.
    pub easing: AnimationEasing,
    /// Travel duration in ms.
    pub travel_duration_ms: u64,
    /// Hold duration in ms.
    pub hold_duration_ms: u64,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            triangle_size: 12.0,
            accent_color: "#2563EB".into(),
            easing: AnimationEasing::EaseInOutCubic,
            travel_duration_ms: 400,
            hold_duration_ms: 2000,
        }
    }
}

/// Animation easing functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationEasing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutCubic,
    EaseOutElastic,
}

impl AnimationEasing {
    /// Evaluate the easing function for a given progress (0.0–1.0).
    pub fn eval(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            AnimationEasing::Linear => t,
            AnimationEasing::EaseInQuad => t * t,
            AnimationEasing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            AnimationEasing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            AnimationEasing::EaseOutElastic => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
        }
    }
}

/// An in-flight cursor animation.
#[derive(Debug, Clone)]
pub struct CursorAnimation {
    /// Start position (cursor).
    pub from: ScreenPoint,
    /// End position (target element).
    pub to: ScreenPoint,
    /// Animation start time (ms).
    pub start_ms: u64,
    /// Total duration (ms).
    pub duration_ms: u64,
    /// Easing function.
    pub easing: AnimationEasing,
    /// Whether the animation is complete.
    pub is_complete: bool,
}

impl CursorAnimation {
    pub fn new(from: ScreenPoint, to: ScreenPoint, start_ms: u64, duration_ms: u64, easing: AnimationEasing) -> Self {
        Self {
            from,
            to,
            start_ms,
            duration_ms,
            easing,
            is_complete: false,
        }
    }

    /// Get the current interpolated position.
    pub fn current_position(&mut self, now_ms: u64) -> ScreenPoint {
        let elapsed = now_ms.saturating_sub(self.start_ms);
        if elapsed >= self.duration_ms {
            self.is_complete = true;
            return self.to;
        }
        let progress = elapsed as f32 / self.duration_ms as f32;
        let eased = self.easing.eval(progress);
        ScreenPoint::new(
            (self.from.x as f32 + (self.to.x as f32 - self.from.x as f32) * eased) as i32,
            (self.from.y as f32 + (self.to.y as f32 - self.from.y as f32) * eased) as i32,
        )
    }
}

/// Cursor renderer — draws the cursor companion triangle.
pub struct CursorRenderer {
    pub config: CursorConfig,
    /// Current animation, if any.
    pub current_animation: Option<CursorAnimation>,
}

impl CursorRenderer {
    pub fn new(config: CursorConfig) -> Self {
        Self {
            config,
            current_animation: None,
        }
    }

    /// Start a new pointing animation.
    pub fn start_animation(&mut self, from: ScreenPoint, to: ScreenPoint, now_ms: u64) {
        self.current_animation = Some(CursorAnimation::new(
            from,
            to,
            now_ms,
            self.config.travel_duration_ms,
            self.config.easing.clone(),
        ));
    }

    /// Clear the current animation.
    pub fn clear(&mut self) {
        self.current_animation = None;
    }

    /// Check if there's an active animation.
    pub fn is_animating(&self) -> bool {
        self.current_animation
            .as_ref()
            .map(|a| !a.is_complete)
            .unwrap_or(false)
    }
}
