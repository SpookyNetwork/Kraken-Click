//! Direct2D rendering backend.
//!
//! Hardware-accelerated rendering for overlay windows.
//! Uses Direct2D + DirectWrite for text and shapes.
//!
//! On Windows ARM64, Direct2D is hardware-accelerated on Qualcomm Adreno GPU.

use crate::pointer::{ScreenPoint, ScreenRegion};

/// Renderer configuration.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Enable vsync.
    pub vsync: bool,
    /// Anti-aliasing mode.
    pub antialias: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            antialias: true,
        }
    }
}

/// The Direct2D renderer.
pub struct D2DRenderer {
    config: RendererConfig,
}

impl D2DRenderer {
    pub fn new(config: RendererConfig) -> Self {
        Self { config }
    }

    /// Initialize Direct2D resources.
    pub fn init(&self) -> Result<(), RenderError> {
        // TODO: D2D1CreateFactory, create render target
        Err(RenderError::NotImplemented)
    }

    /// Begin a frame.
    pub fn begin_draw(&self) {
        // TODO: BeginDraw
    }

    /// End a frame and present.
    pub fn end_draw(&self) {
        // TODO: EndDraw
    }

    /// Clear the render target.
    pub fn clear(&self, _color: (f32, f32, f32, f32)) {
        // TODO: Clear
    }

    /// Draw a triangle (cursor companion).
    pub fn draw_triangle(
        &self,
        _center: &ScreenPoint,
        _size: f32,
        _color: (f32, f32, f32, f32),
        _rotation_degrees: f32,
    ) {
        // TODO: Draw triangle geometry
    }

    /// Draw a circle (cursor highlight).
    pub fn draw_circle(
        &self,
        _center: &ScreenPoint,
        _radius: f32,
        _thickness: f32,
        _color: (f32, f32, f32, f32),
    ) {
        // TODO: Draw circle geometry
    }

    /// Draw a rounded rectangle (caption background).
    pub fn draw_rounded_rect(
        &self,
        _bounds: &ScreenRegion,
        _radius: f32,
        _fill: (f32, f32, f32, f32),
        _stroke: (f32, f32, f32, f32),
        _stroke_width: f32,
    ) {
        // TODO: Draw rounded rect
    }

    /// Draw text (caption content).
    pub fn draw_text(
        &self,
        _text: &str,
        _bounds: &ScreenRegion,
        _color: (f32, f32, f32, f32),
        _font_size: f32,
        _font_family: &str,
    ) {
        // TODO: DirectWrite text layout + draw
    }

    /// Draw a line (pointer line from cursor to target).
    pub fn draw_line(
        &self,
        _from: &ScreenPoint,
        _to: &ScreenPoint,
        _thickness: f32,
        _color: (f32, f32, f32, f32),
    ) {
        // TODO: Draw line
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    NotImplemented,
    DeviceLost,
    Other(String),
}

use std::fmt;

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::NotImplemented => write!(f, "Renderer not yet implemented"),
            RenderError::DeviceLost => write!(f, "D2D device lost"),
            RenderError::Other(s) => write!(f, "Render error: {}", s),
        }
    }
}

impl std::error::Error for RenderError {}
