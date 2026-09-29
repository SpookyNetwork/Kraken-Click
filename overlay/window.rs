//! Transparent overlay windows — the visual layer.
//!
//! Creates per-monitor transparent, topmost, click-through windows
//! for rendering cursor highlights, captions, and markers.
//!
//! On Windows, this uses:
//! - WS_EX_LAYERED + WS_EX_TRANSPARENT + WS_EX_TOPMOST for the overlay window
//! - UpdateLayeredWindow for per-pixel alpha rendering
//! - Direct2D for hardware-accelerated drawing

use crate::pointer::models::*;

/// Overlay window configuration.
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// Background color (usually transparent).
    pub background_color: (u8, u8, u8, u8),
    /// Whether the window is click-through.
    pub click_through: bool,
    /// Whether the window appears in the taskbar.
    pub show_in_taskbar: bool,
    /// Accent color for cursor/markers (hex).
    pub accent_color: String,
    /// Caption font size in points.
    pub caption_font_size: f32,
    /// Caption font family.
    pub caption_font_family: String,
    /// Animation duration in milliseconds.
    pub animation_duration_ms: u64,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            background_color: (0, 0, 0, 0),
            click_through: true,
            show_in_taskbar: false,
            accent_color: "#2563EB".into(),
            caption_font_size: 14.0,
            caption_font_family: "Segoe UI".into(),
            animation_duration_ms: 300,
        }
    }
}

/// An overlay window — one per monitor.
pub struct OverlayWindow {
    config: OverlayConfig,
    /// The pointer state this overlay renders.
    state: PointerState,
    /// Monitor index this overlay covers.
    monitor_index: u32,
    /// Monitor bounds in screen coordinates.
    monitor_bounds: ScreenRegion,
}

impl OverlayWindow {
    pub fn new(config: OverlayConfig, monitor_index: u32, monitor_bounds: ScreenRegion) -> Self {
        Self {
            config,
            state: PointerState::default(),
            monitor_index,
            monitor_bounds,
        }
    }

    /// Create the native window.
    pub fn create(&self) -> Result<(), OverlayError> {
        // TODO: CreateWindowExW with:
        // - WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_NOACTIVATE
        // - WS_POPUP style
        // - Position covering the full monitor
        Err(OverlayError::NotImplemented)
    }

    /// Update the pointer state and trigger a redraw.
    pub fn update_state(&mut self, state: PointerState) {
        self.state = state;
        self.redraw();
    }

    /// Redraw the overlay.
    fn redraw(&self) {
        // TODO: Render via Direct2D:
        // 1. Clear to transparent
        // 2. Draw cursor highlight (if mode is Pointing)
        // 3. Draw caption bubble (if caption is active)
        // 4. Draw markers (if any)
        // 5. UpdateLayeredWindow
    }

    /// Destroy the native window.
    pub fn destroy(&self) -> Result<(), OverlayError> {
        // TODO: DestroyWindow
        Err(OverlayError::NotImplemented)
    }
}

/// Overlay manager — manages one overlay window per monitor.
pub struct OverlayManager {
    config: OverlayConfig,
    overlays: Vec<OverlayWindow>,
}

impl OverlayManager {
    pub fn new(config: OverlayConfig) -> Self {
        Self {
            config,
            overlays: Vec::new(),
        }
    }

    /// Initialize overlays for all connected monitors.
    pub fn init(&mut self) -> Result<(), OverlayError> {
        // TODO: EnumDisplayMonitors → create one OverlayWindow per monitor
        Err(OverlayError::NotImplemented)
    }

    /// Update pointer state across all overlays.
    pub fn update(&mut self, state: PointerState) {
        for overlay in &mut self.overlays {
            overlay.update_state(state.clone());
        }
    }

    /// Show a caption at a screen position.
    pub fn show_caption(&mut self, position: &ScreenPoint, text: &str, duration_ms: u64) {
        // TODO: Update state with caption, set auto-clear timer
    }

    /// Show multiple markers simultaneously (multi-point).
    pub fn show_markers(&mut self, markers: &[PointerMarker]) {
        // TODO: Update state with all markers
    }

    /// Clear all overlay elements.
    pub fn clear(&mut self) {
        for overlay in &mut self.overlays {
            overlay.update_state(PointerState::default());
        }
    }

    /// Destroy all overlay windows.
    pub fn shutdown(&mut self) {
        for overlay in &self.overlays {
            let _ = overlay.destroy();
        }
        self.overlays.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayError {
    NotImplemented,
    WindowCreationFailed,
    Direct2DUnavailable,
    MonitorNotFound(u32),
    Other(String),
}

use std::fmt;

impl fmt::Display for OverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OverlayError::NotImplemented => write!(f, "Overlay not yet implemented"),
            OverlayError::WindowCreationFailed => write!(f, "Failed to create overlay window"),
            OverlayError::Direct2DUnavailable => write!(f, "Direct2D not available"),
            OverlayError::MonitorNotFound(i) => write!(f, "Monitor {} not found", i),
            OverlayError::Other(s) => write!(f, "Overlay error: {}", s),
        }
    }
}

impl std::error::Error for OverlayError {}
