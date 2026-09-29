//! Screen context packaging — prepares screen data for AI models.
//!
//! When an agent needs to understand the screen, this module:
//! 1. Captures the screen
//! 2. Detects UI elements
//! 3. Packages everything into a structured context message
//! 4. Sends to the AI model

use crate::pointer::models::*;
use crate::vision::capture::*;
use crate::pointer::detector::*;

/// Screen context — everything an AI model needs to understand the current screen.
#[derive(Debug, Clone)]
pub struct ScreenContext {
    /// Screenshot data (JPEG encoded).
    pub screenshot_jpeg: Vec<u8>,
    /// Screenshot dimensions.
    pub screenshot_size: (u32, u32),
    /// Detected UI elements.
    pub elements: Vec<UIElement>,
    /// Current cursor position.
    pub cursor_position: ScreenPoint,
    /// Currently focused window info.
    pub focused_window: Option<String>,
    /// Timestamp.
    pub timestamp_ms: u64,
}

/// Packages screen context for AI model consumption.
pub struct ContextPackager {
    capture: ScreenCapture,
    detector: ElementDetector,
}

impl ContextPackager {
    pub fn new(capture_config: CaptureConfig, detector_config: DetectorConfig) -> Result<Self, ContextError> {
        Ok(Self {
            capture: ScreenCapture::new(capture_config),
            detector: ElementDetector::new(detector_config)
                .map_err(|e| ContextError::DetectionFailed(e.to_string()))?,
        })
    }

    /// Capture and package the full screen context.
    pub fn package_full(&mut self) -> Result<ScreenContext, ContextError> {
        let frame = self.capture.capture_screen()?;
        let elements = self.detector.detect_all()?;

        Ok(ScreenContext {
            screenshot_jpeg: frame.data,
            screenshot_size: (frame.width, frame.height),
            elements,
            cursor_position: Self::get_cursor_position(),
            focused_window: Self::get_focused_window_title(),
            timestamp_ms: frame.timestamp_ms,
        })
    }

    /// Package context around the current cursor position.
    pub fn package_cursor_context(&mut self, radius: u32) -> Result<ScreenContext, ContextError> {
        let cursor = Self::get_cursor_position();
        let frame = self.capture.capture_around(&cursor, radius)?;
        let element = self.detector.detect_at(&cursor)?;

        let elements = match element {
            Some(e) => vec![e],
            None => Vec::new(),
        };

        Ok(ScreenContext {
            screenshot_jpeg: frame.data,
            screenshot_size: (frame.width, frame.height),
            elements,
            cursor_position: cursor,
            focused_window: Self::get_focused_window_title(),
            timestamp_ms: frame.timestamp_ms,
        })
    }

    /// Package context for a specific element.
    pub fn package_element_context(
        &mut self,
        element: &UIElement,
    ) -> Result<ScreenContext, ContextError> {
        let frame = self.capture.capture_region(&element.bounds)?;

        Ok(ScreenContext {
            screenshot_jpeg: frame.data,
            screenshot_size: (frame.width, frame.height),
            elements: vec![element.clone()],
            cursor_position: element.bounds.center(),
            focused_window: Self::get_focused_window_title(),
            timestamp_ms: frame.timestamp_ms,
        })
    }

    /// Build a text description of the screen context (for text-only models).
    pub fn build_text_description(&self, ctx: &ScreenContext) -> String {
        let mut desc = String::new();
        desc.push_str(&format!(
            "Screen: {}x{} pixels\n",
            ctx.screenshot_size.0, ctx.screenshot_size.1
        ));
        desc.push_str(&format!(
            "Cursor position: {}\n",
            ctx.cursor_position
        ));
        if let Some(ref win) = ctx.focused_window {
            desc.push_str(&format!("Focused window: {}\n", win));
        }
        desc.push_str(&format!("Detected {} UI elements:\n", ctx.elements.len()));
        for (i, el) in ctx.elements.iter().enumerate() {
            desc.push_str(&format!(
                "  [{}] {} ({}) at {} - interactable: {}\n",
                i,
                el.label,
                el.entity_type,
                el.bounds.center(),
                el.is_interactable
            ));
            if let Some(ref text) = el.text_content {
                desc.push_str(&format!("      Text: \"{}\"\n", text));
            }
        }
        desc
    }

    // ── Windows API helpers ──

    fn get_cursor_position() -> ScreenPoint {
        // TODO: GetCursorPos
        ScreenPoint::new(0, 0)
    }

    fn get_focused_window_title() -> Option<String> {
        // TODO: GetForegroundWindow + GetWindowText
        None
    }
}

#[derive(Debug, Clone)]
pub enum ContextError {
    CaptureFailed(String),
    DetectionFailed(String),
    Other(String),
}

impl PartialEq for ContextError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ContextError::CaptureFailed(a), ContextError::CaptureFailed(b)) => a == b,
            (ContextError::DetectionFailed(a), ContextError::DetectionFailed(b)) => a == b,
            (ContextError::Other(a), ContextError::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ContextError {}

use std::fmt;

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::CaptureFailed(s) => write!(f, "Capture failed: {}", s),
            ContextError::DetectionFailed(s) => write!(f, "Detection failed: {}", s),
            ContextError::Other(s) => write!(f, "Context error: {}", s),
        }
    }
}

impl std::error::Error for ContextError {}

impl From<crate::vision::capture::CaptureError> for ContextError {
    fn from(err: crate::vision::capture::CaptureError) -> Self {
        ContextError::CaptureFailed(err.to_string())
    }
}

impl From<crate::pointer::detector::DetectorError> for ContextError {
    fn from(err: crate::pointer::detector::DetectorError) -> Self {
        ContextError::DetectionFailed(err.to_string())
    }
}
