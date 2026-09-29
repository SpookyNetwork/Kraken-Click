//! Screen capture — Desktop Duplication API integration.
//!
//! Uses Windows Desktop Duplication API (DXGI) for efficient screen capture.
//! Falls back to GDI BitBlt if DDA is unavailable.
//!
//! On Windows ARM64, DXGI Desktop Duplication is the preferred method because:
//! - Zero-copy GPU → CPU path
//! - Hardware-accelerated on Qualcomm Adreno GPU
//! - Can capture per-display, excluding specific windows

use crate::pointer::models::*;

/// Screen capture backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureBackend {
    /// DXGI Desktop Duplication API (preferred).
    DesktopDuplication,
    /// GDI BitBlt (fallback).
    GdiBitBlt,
    /// Windows Graphics Capture API (Win10 1809+, supports window capture).
    GraphicsCapture,
}

/// Capture configuration.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Which backend to use.
    pub backend: CaptureBackend,
    /// Which display to capture (None = primary).
    pub display_index: Option<u32>,
    /// Whether to exclude the Kraken Click overlay from capture.
    pub exclude_overlay: bool,
    /// Target resolution (None = native resolution).
    pub target_resolution: Option<(u32, u32)>,
    /// JPEG quality for compressed output (1-100).
    pub jpeg_quality: u8,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            backend: CaptureBackend::GdiBitBlt, // Start with GDI as it's most reliable
            display_index: None,
            exclude_overlay: true,
            target_resolution: None,
            jpeg_quality: 85,
        }
    }
}

/// A captured screen frame.
#[derive(Debug, Clone)]
pub struct ScreenFrame {
    /// Frame data (BGRA32 for raw, JPEG for compressed).
    pub data: Vec<u8>,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Bytes per pixel (4 for BGRA32).
    pub bpp: u8,
    /// Whether the data is compressed (JPEG).
    pub is_compressed: bool,
    /// Timestamp when the frame was captured.
    pub timestamp_ms: u64,
    /// Display index this frame was captured from.
    pub display_index: u32,
}

/// Screen capture engine.
pub struct ScreenCapture {
    config: CaptureConfig,
}

impl ScreenCapture {
    pub fn new(config: CaptureConfig) -> Self {
        Self { config }
    }

    /// Capture the full screen.
    pub fn capture_screen(&self) -> Result<ScreenFrame, CaptureError> {
        match self.config.backend {
            CaptureBackend::DesktopDuplication => self.capture_via_dda(),
            CaptureBackend::GdiBitBlt => self.capture_via_gdi(),
            CaptureBackend::GraphicsCapture => self.capture_via_gc(),
        }
    }

    /// Capture a specific region of the screen.
    pub fn capture_region(&self, region: &ScreenRegion) -> Result<ScreenFrame, CaptureError> {
        let full = self.capture_screen()?;
        self.crop_frame(&full, region)
    }

    /// Capture the region around a point (for pointer hover context).
    /// Captures a square region of `radius` pixels around the point.
    pub fn capture_around(
        &self,
        point: &ScreenPoint,
        radius: u32,
    ) -> Result<ScreenFrame, CaptureError> {
        let x = (point.x as i32 - radius as i32).max(0);
        let y = (point.y as i32 - radius as i32).max(0);
        let region = ScreenRegion::new(x, y, radius * 2, radius * 2);
        self.capture_region(&region)
    }

    // ── GDI BitBlt implementation ──

    fn capture_via_gdi(&self) -> Result<ScreenFrame, CaptureError> {
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::Foundation::*;

        unsafe {
            // Get the full screen DC
            let screen_dc = GetDC(HWND::default());
            if screen_dc.is_invalid() {
                return Err(CaptureError::Other("GetDC failed".into()));
            }

            // Get screen dimensions
            let screen_width = GetDeviceCaps(screen_dc, HORZRES);
            let screen_height = GetDeviceCaps(screen_dc, VERTRES);

            // Create compatible DC and bitmap
            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_invalid() {
                ReleaseDC(HWND::default(), screen_dc);
                return Err(CaptureError::Other("CreateCompatibleDC failed".into()));
            }

            let bitmap = CreateCompatibleBitmap(screen_dc, screen_width, screen_height);
            if bitmap.is_invalid() {
                DeleteDC(mem_dc);
                ReleaseDC(HWND::default(), screen_dc);
                return Err(CaptureError::Other("CreateCompatibleBitmap failed".into()));
            }

            let old_bitmap = SelectObject(mem_dc, bitmap);

            // Copy screen to bitmap
            let result = BitBlt(
                mem_dc,
                0,
                0,
                screen_width,
                screen_height,
                screen_dc,
                0,
                0,
                SRCCOPY,
            );

            if result.is_err() {
                SelectObject(mem_dc, old_bitmap);
                DeleteObject(bitmap);
                DeleteDC(mem_dc);
                ReleaseDC(HWND::default(), screen_dc);
                return Err(CaptureError::Other("BitBlt failed".into()));
            }

            // Prepare bitmap info for GetDIBits
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: screen_width,
                    biHeight: -screen_height, // Negative = top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }; 1],
            };

            let row_size = ((screen_width * 32 + 31) / 32) * 4;
            let buffer_size = (row_size * screen_height) as usize;
            let mut buffer: Vec<u8> = vec![0u8; buffer_size];

            let lines = GetDIBits(
                mem_dc,
                bitmap,
                0,
                screen_height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Cleanup GDI objects
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(mem_dc);
            ReleaseDC(HWND::default(), screen_dc);

            if lines == 0 {
                return Err(CaptureError::Other("GetDIBits failed".into()));
            }

            // Convert BGRA to RGBA
            for chunk in buffer.chunks_exact_mut(4) {
                chunk.swap(0, 2); // Swap B and R
            }

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            Ok(ScreenFrame {
                data: buffer,
                width: screen_width as u32,
                height: screen_height as u32,
                bpp: 4,
                is_compressed: false,
                timestamp_ms: timestamp,
                display_index: 0,
            })
        }
    }

    fn capture_via_dda(&self) -> Result<ScreenFrame, CaptureError> {
        // TODO: Implement DXGI Desktop Duplication
        // For now, fall back to GDI
        self.capture_via_gdi()
    }

    fn capture_via_gc(&self) -> Result<ScreenFrame, CaptureError> {
        // TODO: Implement Windows Graphics Capture API
        // For now, fall back to GDI
        self.capture_via_gdi()
    }

    fn crop_frame(&self, frame: &ScreenFrame, region: &ScreenRegion) -> Result<ScreenFrame, CaptureError> {
        let x = region.x.max(0) as u32;
        let y = region.y.max(0) as u32;
        let crop_w = (region.width as u32).min(frame.width - x);
        let crop_h = (region.height as u32).min(frame.height - y);

        if crop_w == 0 || crop_h == 0 {
            return Err(CaptureError::Other("Invalid crop region".into()));
        }

        let bpp = frame.bpp as usize;
        let src_stride = frame.width as usize * bpp;
        let dst_stride = crop_w as usize * bpp;
        let mut cropped = vec![0u8; (crop_h as usize) * dst_stride];

        for row in 0..crop_h as usize {
            let src_offset = ((y as usize + row) * src_stride) + (x as usize * bpp);
            let dst_offset = row * dst_stride;
            cropped[dst_offset..dst_offset + dst_stride]
                .copy_from_slice(&frame.data[src_offset..src_offset + dst_stride]);
        }

        Ok(ScreenFrame {
            data: cropped,
            width: crop_w,
            height: crop_h,
            bpp: frame.bpp,
            is_compressed: false,
            timestamp_ms: frame.timestamp_ms,
            display_index: frame.display_index,
        })
    }
}

/// Capture errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    BackendUnavailable(String),
    ScreenCaptureDenied,
    DisplayNotFound(u32),
    Timeout,
    OutOfMemory,
    Other(String),
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaptureError::BackendUnavailable(s) => write!(f, "Capture backend unavailable: {}", s),
            CaptureError::ScreenCaptureDenied => write!(f, "Screen capture permission denied"),
            CaptureError::DisplayNotFound(i) => write!(f, "Display {} not found", i),
            CaptureError::Timeout => write!(f, "Capture timed out"),
            CaptureError::OutOfMemory => write!(f, "Out of memory during capture"),
            CaptureError::Other(s) => write!(f, "Capture error: {}", s),
        }
    }
}

impl std::error::Error for CaptureError {}

use std::fmt;
