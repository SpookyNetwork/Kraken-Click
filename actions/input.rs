//! Keyboard input — key presses and text typing.
//!
//! Uses SendInput for keyboard event injection.
//! Supports:
//! - Single key presses with modifiers
//! - Text typing (Unicode character by character)
//! - Special keys (Enter, Tab, Escape, Arrow keys, etc.)

use super::gate::*;
use crate::pointer::models::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Keyboard action executor.
pub struct InputExecutor {
    gate: PscGate,
    /// Delay between keystrokes in ms (for text typing).
    pub key_delay_ms: u32,
}

impl InputExecutor {
    pub fn new(gate: PscGate) -> Self {
        Self {
            gate,
            key_delay_ms: 30,
        }
    }

    /// Press a key (with optional modifiers).
    pub fn press_key(
        &mut self,
        key: &str,
        modifiers: &[ModifierKey],
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::KeyPress {
            key: key.into(),
            modifiers: modifiers.to_vec(),
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_key_press(key, modifiers),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    /// Type text (character by character).
    pub fn type_text(
        &mut self,
        text: &str,
        target: Option<&ScreenPoint>,
        now_ms: u64,
    ) -> Result<(), ActionError> {
        let action = PointerAction::TypeText {
            text: text.into(),
            target: target.copied(),
        };

        match self.gate.evaluate(&action, now_ms) {
            GateDecision::Allow => self.execute_type_text(text),
            GateDecision::Deny { reason } => Err(ActionError::Denied(reason)),
            GateDecision::RequireConfirmation { prompt } => {
                Err(ActionError::NeedsConfirmation(prompt))
            }
            GateDecision::RateLimited { retry_after_ms } => {
                Err(ActionError::RateLimited(retry_after_ms))
            }
        }
    }

    fn execute_key_press(
        &self,
        key: &str,
        modifiers: &[ModifierKey],
    ) -> Result<(), ActionError> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        let mut inputs: Vec<INPUT> = Vec::new();

        // Press modifiers first
        for modifier in modifiers {
            let vk = match modifier {
                ModifierKey::Ctrl => VK_CONTROL,
                ModifierKey::Alt => VK_MENU,
                ModifierKey::Shift => VK_SHIFT,
                ModifierKey::Win => VK_LWIN,
            };
            inputs.push(create_key_input(vk.0 as u16, false));
        }

        // Press and release the main key
        let vk = key_to_virtual_key(key)?;
        inputs.push(create_key_input(vk, false)); // keydown
        inputs.push(create_key_input(vk, true));  // keyup

        // Release modifiers in reverse order
        for modifier in modifiers.iter().rev() {
            let vk = match modifier {
                ModifierKey::Ctrl => VK_CONTROL,
                ModifierKey::Alt => VK_MENU,
                ModifierKey::Shift => VK_SHIFT,
                ModifierKey::Win => VK_LWIN,
            };
            inputs.push(create_key_input(vk.0 as u16, true));
        }

        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent == 0 {
                return Err(ActionError::Other("SendInput failed".into()));
            }
        }

        Ok(())
    }

    fn execute_type_text(&self, text: &str) -> Result<(), ActionError> {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;

        let mut inputs: Vec<INPUT> = Vec::new();

        for ch in text.chars() {
            // Use KEYEVENTF_UNICODE for reliable text input
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            inputs.push(input);

            // Also send keyup
            let input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            inputs.push(input_up);
        }

        if !inputs.is_empty() {
            let sent = unsafe {
                SendInput(&inputs, std::mem::size_of::<INPUT>() as i32)
            };
            if sent == 0 {
                return Err(ActionError::Other("SendInput failed".into()));
            }
        }

        Ok(())
    }
}

fn create_key_input(vk: u16, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Map a key name string to a virtual key code.
fn key_to_virtual_key(key: &str) -> Result<u16, ActionError> {
    let vk = match key.to_lowercase().as_str() {
        "enter" | "return" => VK_RETURN.0,
        "tab" => VK_TAB.0,
        "escape" | "esc" => VK_ESCAPE.0,
        "space" => VK_SPACE.0,
        "backspace" | "bksp" => VK_BACK.0,
        "delete" | "del" => VK_DELETE.0,
        "up" => VK_UP.0,
        "down" => VK_DOWN.0,
        "left" => VK_LEFT.0,
        "right" => VK_RIGHT.0,
        "home" => VK_HOME.0,
        "end" => VK_END.0,
        "pageup" | "pgup" => VK_PRIOR.0,
        "pagedown" | "pgdn" => VK_NEXT.0,
        "f1" => VK_F1.0,
        "f2" => VK_F2.0,
        "f3" => VK_F3.0,
        "f4" => VK_F4.0,
        "f5" => VK_F5.0,
        "f6" => VK_F6.0,
        "f7" => VK_F7.0,
        "f8" => VK_F8.0,
        "f9" => VK_F9.0,
        "f10" => VK_F10.0,
        "f11" => VK_F11.0,
        "f12" => VK_F12.0,
        // Single character keys
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                // A-Z: virtual key codes 0x41-0x5A
                ch.to_ascii_uppercase() as u16
            } else if ch.is_ascii_digit() {
                // 0-9: virtual key codes 0x30-0x39
                ch as u16
            } else {
                return Err(ActionError::Other(format!("Unsupported key character: {}", ch)));
            }
        }
        _ => return Err(ActionError::Other(format!("Unknown key: {}", key))),
    };
    Ok(vk)
}

// Re-export ActionError from click.rs
pub use super::click::ActionError;
