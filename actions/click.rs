//! Mouse click and drag operations.
//!
//! Uses SendInput on Windows for reliable mouse event injection.
//! On Windows ARM64, SendInput works identically to x64.

use super::gate::*;
use crate::pointer::models::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Mouse action executor.
pub struct ClickExecutor {
    gate: PscGate,
}

impl ClickExecutor {
    pub fn new(gate: PscGate) -> Self {
        Self { gate }
    }

    /// Perform a mouse click at the given position.
    pub fn click(
        &mut self,
        position: &ScreenPoint,
        button: MouseButton,
        click_type: ClickType,
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::Click {
            position: *position,
            button,
            click_type,
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_click(position, button, click_type),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    /// Perform a drag operation.
    pub fn drag(
        &mut self,
        from: &ScreenPoint,
        to: &ScreenPoint,
        button: MouseButton,
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::Drag {
            from: *from,
            to: *to,
            button,
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_drag(from, to, button),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    fn execute_click(
        &self,
        position: &ScreenPoint,
        button: MouseButton,
        click_type: ClickType,
    ) -> Result<(), ActionError> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        // Move cursor to position first
        self.set_cursor_pos(position)?;

        let click_count = match click_type {
            ClickType::Single => 1,
            ClickType::Double => 2,
            ClickType::Triple => 3,
        };

        for _ in 0..click_count {
            let (down_flag, up_flag) = mouse_button_flags(button, false);
            let (down_flag_up, up_flag_up) = mouse_button_flags(button, true);

            let down = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: down_flag,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let up = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: up_flag_up,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            unsafe {
                let sent = SendInput(
                    &[down, up],
                    std::mem::size_of::<INPUT>() as i32,
                );
                if sent == 0 {
                    return Err(ActionError::Other("SendInput failed".into()));
                }
            }

            // Small delay between multi-clicks
            if click_count > 1 {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        Ok(())
    }

    fn execute_drag(
        &self,
        from: &ScreenPoint,
        to: &ScreenPoint,
        button: MouseButton,
    ) -> Result<(), ActionError> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        // Move to start position
        self.set_cursor_pos(from)?;

        // Mouse down
        let (down_flag, _) = mouse_button_flags(button, false);
        let (_, up_flag) = mouse_button_flags(button, true);

        let down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: down_flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            let sent = SendInput(&[down], std::mem::size_of::<INPUT>() as i32);
            if sent == 0 {
                return Err(ActionError::Other("SendInput failed".into()));
            }
        }

        // Small delay before moving
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Move to end position
        self.set_cursor_pos(to)?;

        // Small delay before releasing
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Mouse up
        let up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: up_flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        unsafe {
            let sent = SendInput(&[up], std::mem::size_of::<INPUT>() as i32);
            if sent == 0 {
                return Err(ActionError::Other("SendInput failed".into()));
            }
        }

        Ok(())
    }

    fn set_cursor_pos(&self, position: &ScreenPoint) -> Result<(), ActionError> {
        use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
        unsafe {
            if SetCursorPos(position.x, position.y).is_err() {
                return Err(ActionError::Other("SetCursorPos failed".into()));
            }
        }
        Ok(())
    }
}

fn mouse_button_flags(button: MouseButton, key_up: bool) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    match button {
        MouseButton::Left => {
            if key_up {
                (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
            } else {
                (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
            }
        }
        MouseButton::Right => {
            if key_up {
                (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
            } else {
                (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
            }
        }
        MouseButton::Middle => {
            if key_up {
                (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP)
            } else {
                (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    NotImplemented,
    Denied(String),
    NeedsConfirmation(String),
    RateLimited(u64),
    Other(String),
}

use std::fmt;

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::NotImplemented => write!(f, "Action not yet implemented"),
            ActionError::Denied(s) => write!(f, "Action denied: {}", s),
            ActionError::NeedsConfirmation(s) => write!(f, "Needs confirmation: {}", s),
            ActionError::RateLimited(ms) => write!(f, "Rate limited, retry after {}ms", ms),
            ActionError::Other(s) => write!(f, "Action error: {}", s),
        }
    }
}

impl std::error::Error for ActionError {}
