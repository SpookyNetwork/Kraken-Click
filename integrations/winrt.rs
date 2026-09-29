//! Windows Runtime API bindings.
//!
//! Provides Rust FFI bindings for Windows APIs used by Kraken Click:
//! - DXGI Desktop Duplication (screen capture)
//! - Windows UI Automation (element detection)
//! - Windows Graphics Capture (alternative capture)
//! - Direct2D (overlay rendering)
//! - SendInput (mouse/keyboard injection)
//!
//! Uses the `windows-rs` crate for type-safe WinRT bindings.

/// Windows API module — re-exports from windows-rs.
pub mod windows_api {
    // TODO: Add windows-rs dependency to Cargo.toml
    // [dependencies]
    // windows = { version = "0.58", features = [
    //     "Win32_Foundation",
    //     "Win32_Graphics_Dxgi",
    //     "Win32_Graphics_Dxgi_Common",
    //     "Win32_Graphics_Direct2D",
    //     "Win32_Graphics_Direct2D_Common",
    //     "Win32_Graphics_DirectWrite",
    //     "Win32_Graphics_Gdi",
    //     "Win32_UI_WindowsAndMessaging",
    //     "Win32_UI_Input_KeyboardAndMouse",
    //     "Win32_UI_Accessibility",
    //     "Win32_System_Com",
    //     "Win32_System_LibraryLoader",
    //     "Win32_System_Threading",
    //     "Win32_Security",
    //     "UI_Xaml",
    //     "Graphics_Capture",
    //     "Media_Ocr",
    // ] }
}

/// Screen capture via DXGI Desktop Duplication.
pub mod dda {
    // TODO: IDXGIOutputDuplication implementation
    // Key interfaces:
    // - IDXGIOutput1::DuplicateOutput
    // - IDXGIOutputDuplication::AcquireNextFrame
    // - IDXGIResource::GetSharedHandle
}

/// Screen capture via Windows Graphics Capture API.
pub mod graphics_capture {
    // TODO: Windows.Graphics.Capture interop
    // Key interfaces:
    // - GraphicsCaptureItem
    // - Direct3D11CaptureFramePool
    // - GraphicsCaptureSession
}

/// UI Automation for element detection.
pub mod uia {
    // TODO: IUIAutomation implementation
    // Key interfaces:
    // - IUIAutomation::ElementFromPoint
    // - IUIAutomation::CreateTreeWalker
    // - IUIAutomationElement::GetCurrentPropertyValue
    // - IUIAutomationElement::GetCurrentPattern
}

/// Direct2D rendering.
pub mod d2d {
    // TODO: ID2D1Factory + ID2D1HwndRenderTarget
    // Key interfaces:
    // - D2D1CreateFactory
    // - ID2D1Factory::CreateHwndRenderTarget
    // - ID2D1RenderTarget::DrawRectangle, DrawText, etc.
}

/// DirectWrite text rendering.
pub mod dwrite {
    // TODO: IDWriteFactory + IDWriteTextLayout
    // Key interfaces:
    // - DWriteCreateFactory
    // - IDWriteFactory::CreateTextLayout
}

/// SendInput for mouse/keyboard.
pub mod input {
    // TODO: SendInput wrapper
    // Key functions:
    // - SetCursorPos
    // - SendInput (INPUT structure with MOUSEINPUT / KEYBDINPUT / HARDWAREINPUT)
}
