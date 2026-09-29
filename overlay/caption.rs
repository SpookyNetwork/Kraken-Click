//! Caption bubbles + multi-marker rendering.
//!
//! Renders text captions near screen positions with:
//! - Rounded rectangle background
//! - Arrow pointing to target
//! - Auto-sizing text
//! - Fade in/out animation
//! - Multi-marker layout (avoids overlap)

use crate::pointer::models::*;

/// Caption style configuration.
#[derive(Debug, Clone)]
pub struct CaptionConfig {
    /// Background color (RGBA).
    pub background: (u8, u8, u8, u8),
    /// Text color (RGBA).
    pub text_color: (u8, u8, u8, u8),
    /// Border color (RGBA).
    pub border_color: (u8, u8, u8, u8),
    /// Border radius in pixels.
    pub border_radius: f32,
    /// Padding inside the bubble (pixels).
    pub padding: f32,
    /// Font size in points.
    pub font_size: f32,
    /// Font family.
    pub font_family: String,
    /// Max width in pixels.
    pub max_width: f32,
    /// Fade-in duration (ms).
    pub fade_in_ms: u64,
    /// Fade-out duration (ms).
    pub fade_out_ms: u64,
}

impl Default for CaptionConfig {
    fn default() -> Self {
        Self {
            background: (30, 30, 30, 230),
            text_color: (255, 255, 255, 255),
            border_color: (37, 99, 235, 255),
            border_radius: 8.0,
            padding: 10.0,
            font_size: 13.0,
            font_family: "Segoe UI".into(),
            max_width: 300.0,
            fade_in_ms: 200,
            fade_out_ms: 300,
        }
    }
}

/// A caption bubble to render.
#[derive(Debug, Clone)]
pub struct CaptionBubble {
    /// The text to display.
    pub text: String,
    /// Screen position to point at.
    pub target: ScreenPoint,
    /// Computed bounding box (set during layout).
    pub bounds: Option<ScreenRegion>,
    /// Opacity (0.0–1.0) for fade animation.
    pub opacity: f32,
    /// Accent color override.
    pub accent_color: Option<String>,
}

impl CaptionBubble {
    pub fn new(text: impl Into<String>, target: ScreenPoint) -> Self {
        Self {
            text: text.into(),
            target,
            bounds: None,
            opacity: 1.0,
            accent_color: None,
        }
    }
}

/// Multi-marker layout engine.
/// Positions multiple caption bubbles to avoid overlap.
pub struct MarkerLayout {
    pub config: CaptionConfig,
}

impl MarkerLayout {
    pub fn new(config: CaptionConfig) -> Self {
        Self { config }
    }

    /// Layout multiple markers, adjusting positions to avoid overlap.
    pub fn layout(&self, markers: &[PointerMarker]) -> Vec<CaptionBubble> {
        let mut bubbles: Vec<CaptionBubble> = markers
            .iter()
            .filter_map(|m| {
                m.caption.as_ref().map(|text| {
                    let mut bubble = CaptionBubble::new(text.clone(), m.position);
                    bubble.accent_color = m.accent_color.clone();
                    bubble
                })
            })
            .collect();

        // Simple layout: offset overlapping bubbles vertically
        for i in 0..bubbles.len() {
            for j in (i + 1)..bubbles.len() {
                // TODO: Check overlap and offset
            }
        }

        bubbles
    }
}
