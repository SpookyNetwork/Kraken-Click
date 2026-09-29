//! Window management — focus, app switching, window positioning.
//!
//! Uses Windows API:
//! - SetForegroundWindow (focus)
//! - ShellExecute / CreateProcess (launch app)
//! - EnumWindows (find windows)
//! - GetWindowText, GetClassName (identify windows)
//! - SetWindowPos (position/resize)

use super::gate::*;
use crate::pointer::models::*;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::ShellExecuteW;

/// Window information.
#[derive(Debug, Clone)]
pub struct WindowInfo {
    /// Window handle (HWND as usize).
    pub handle: usize,
    /// Window title.
    pub title: String,
    /// Process name.
    pub process_name: String,
    /// Window bounds.
    pub bounds: ScreenRegion,
    /// Whether the window is visible.
    pub is_visible: bool,
    /// Whether the window is minimized.
    pub is_minimized: bool,
    /// Whether the window is maximized.
    pub is_maximized: bool,
}

/// Window action executor.
pub struct WindowExecutor {
    gate: PscGate,
}

impl WindowExecutor {
    pub fn new(gate: PscGate) -> Self {
        Self { gate }
    }

    /// Focus a window by title or process name.
    pub fn focus_window(
        &mut self,
        title: Option<&str>,
        process_name: Option<&str>,
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::FocusWindow {
            window_title: title.map(String::from),
            process_name: process_name.map(String::from),
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_focus(title, process_name),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    /// Switch to an app by name.
    pub fn switch_app(
        &mut self,
        app_name: &str,
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::SwitchApp {
            app_name: app_name.into(),
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_switch(app_name),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    /// Enumerate all visible windows.
    pub fn enumerate_windows(&self) -> Vec<WindowInfo> {
        use windows::Win32::UI::WindowsAndMessaging::*;

        let mut windows: Vec<WindowInfo> = Vec::new();

        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut _ as isize),
            );
        }

        windows
    }

    /// Get the currently focused window.
    pub fn get_focused_window(&self) -> Option<WindowInfo> {
        use windows::Win32::UI::WindowsAndMessaging::*;

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == HWND::default() {
                return None;
            }
            window_info_from_hwnd(hwnd)
        }
    }

    fn execute_focus(
        &self,
        title: Option<&str>,
        process_name: Option<&str>,
    ) -> Result<(), ActionError> {
        use windows::Win32::UI::WindowsAndMessaging::*;

        let windows = self.enumerate_windows();

        let target = windows.iter().find(|w| {
            if let Some(t) = title {
                w.title.to_lowercase().contains(&t.to_lowercase())
            } else if let Some(p) = process_name {
                w.process_name.to_lowercase().contains(&p.to_lowercase())
            } else {
                false
            }
        });

        if let Some(win) = target {
            unsafe {
                let hwnd = HWND(win.handle as *mut std::ffi::c_void);
                let _ = ShowWindow(hwnd, SW_RESTORE);
                if !SetForegroundWindow(hwnd).as_bool() {
                    return Err(ActionError::Other("SetForegroundWindow failed".into()));
                }
            }
            Ok(())
        } else {
            Err(ActionError::Other(format!(
                "Window not found: title={:?}, process={:?}",
                title, process_name
            )))
        }
    }

    fn execute_switch(&self, app_name: &str) -> Result<(), ActionError> {
        use windows::Win32::UI::Shell::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Try to find the window first
        let windows = self.enumerate_windows();
        let target = windows.iter().find(|w| {
            w.title.to_lowercase().contains(&app_name.to_lowercase())
                || w.process_name.to_lowercase().contains(&app_name.to_lowercase())
        });

        if let Some(win) = target {
            return self.execute_focus(Some(&win.title), None);
        }

        // If not found, try to launch via ShellExecute
        let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(Some(0)).collect();
        let file: Vec<u16> = OsStr::new(app_name).encode_wide().chain(Some(0)).collect();

        unsafe {
            let result = ShellExecuteW(
                HWND::default(),
                windows::core::PCWSTR(operation.as_ptr()),
                windows::core::PCWSTR(file.as_ptr()),
                windows::core::PCWSTR::null(),
                windows::core::PCWSTR::null(),
                SW_SHOWNORMAL,
            );

            if (result.0 as isize) > 32 {
                Ok(())
            } else {
                Err(ActionError::Other(format!(
                    "ShellExecute failed for: {}",
                    app_name
                )))
            }
        }
    }
}

unsafe extern "system" fn enum_windows_callback(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::*;

    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    if !IsWindowVisible(hwnd).as_bool() {
        return windows::Win32::Foundation::TRUE;
    }

    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    if (ex_style & WS_EX_TOOLWINDOW.0 as i32) != 0 {
        return windows::Win32::Foundation::TRUE;
    }

    if let Some(info) = window_info_from_hwnd(hwnd) {
        windows.push(info);
    }

    windows::Win32::Foundation::TRUE
}

fn window_info_from_hwnd(hwnd: windows::Win32::Foundation::HWND) -> Option<WindowInfo> {
    use windows::Win32::UI::WindowsAndMessaging::*;

    unsafe {
        let title_length = GetWindowTextLengthW(hwnd);
        if title_length == 0 {
            return None;
        }
        let mut title_buf = vec![0u16; (title_length + 1) as usize];
        GetWindowTextW(hwnd, &mut title_buf);
        let title = OsString::from_wide(&title_buf[..title_length as usize])
            .to_string_lossy()
            .to_string();

        if title.is_empty() {
            return None;
        }

        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }

        let bounds = ScreenRegion::new(
            rect.left,
            rect.top,
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
        );

        let is_minimized = IsIconic(hwnd).as_bool();
        let is_maximized = IsZoomed(hwnd).as_bool();
        let process_name = title.clone();

        Some(WindowInfo {
            handle: hwnd.0 as usize,
            title,
            process_name,
            bounds,
            is_visible: true,
            is_minimized,
            is_maximized,
        })
    }
}

pub use super::click::ActionError;
