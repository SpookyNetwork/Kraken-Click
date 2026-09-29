//! MSAA / UI Automation bindings.
//!
//! Windows Accessibility APIs for UI element detection:
//! - Microsoft Active Accessibility (MSAA) — legacy but widely supported
//! - UI Automation (UIA) — modern, preferred
//!
//! On Windows 11 ARM64, both are fully supported.

use crate::pointer::models::*;

/// UI Automation element properties we care about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiaProperty {
    AutomationId,
    Name,
    ControlType,
    BoundingRectangle,
    IsEnabled,
    IsOffscreen,
    HasKeyboardFocus,
    Value,
}

/// UI Automation control types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiaControlType {
    Button,
    CheckBox,
    ComboBox,
    Edit,
    Hyperlink,
    Image,
    ListItem,
    Menu,
    MenuBar,
    MenuItem,
    ProgressBar,
    RadioButton,
    ScrollBar,
    Slider,
    Spinner,
    Tab,
    TabItem,
    Text,
    ToolBar,
    Tree,
    TreeItem,
    Window,
    Custom,
    Group,
    DataItem,
    Header,
    HeaderItem,
    Table,
    Unknown(u32),
}

impl UiaControlType {
    /// Map to our EntityType.
    pub fn to_entity_type(&self) -> EntityType {
        match self {
            UiaControlType::Button => EntityType::Button,
            UiaControlType::CheckBox => EntityType::Checkbox,
            UiaControlType::ComboBox => EntityType::Dropdown,
            UiaControlType::Edit => EntityType::TextField,
            UiaControlType::Hyperlink => EntityType::Link,
            UiaControlType::Image => EntityType::Image,
            UiaControlType::Menu | UiaControlType::MenuBar => EntityType::Menu,
            UiaControlType::MenuItem => EntityType::MenuItem,
            UiaControlType::ScrollBar => EntityType::ScrollBar,
            UiaControlType::Slider => EntityType::Slider,
            UiaControlType::Tab => EntityType::Tab,
            UiaControlType::Text | UiaControlType::Header => EntityType::Label,
            UiaControlType::ToolBar => EntityType::Toolbar,
            UiaControlType::Window => EntityType::Window,
            UiaControlType::Table => EntityType::Table,
            UiaControlType::ListItem | UiaControlType::TreeItem | UiaControlType::DataItem => {
                EntityType::Label
            }
            _ => EntityType::Unknown(format!("{:?}", self)),
        }
    }
}

/// UI Automation client.
pub struct UiaClient {
    _private: (),
}

impl UiaClient {
    /// Create a new UI Automation client.
    pub fn new() -> Result<Self, UiaError> {
        // Initialize COM if needed
        unsafe {
            use windows::Win32::System::Com::CoInitializeEx;
            use windows::Win32::System::Com::COINIT_MULTITHREADED;
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        Ok(UiaClient { _private: () })
    }

    /// Get the element at a screen point using MSAA (AccessibleObjectFromPoint).
    pub fn element_from_point(&self, x: i32, y: i32) -> Result<UiaElement, UiaError> {
        use windows::Win32::UI::Accessibility::*;
        use windows::Win32::Foundation::*;

        unsafe {
            let pt = POINT { x, y };
            let mut acc: Option<IAccessible> = None;
            let mut child_id = windows::core::VARIANT::default();

            AccessibleObjectFromPoint(pt, &mut acc, &mut child_id)
                .map_err(|e| UiaError::ComError(format!("AccessibleObjectFromPoint: {}", e)))?;

            match acc {
                Some(accessible) => Ok(UiaElement {
                    accessible,
                    child_id,
                }),
                None => Err(UiaError::ElementNotFound),
            }
        }
    }

    /// Enumerate visible elements (simplified — returns empty for now).
    pub fn enumerate_visible_elements(&self) -> Vec<UIElement> {
        Vec::new()
    }
}

/// A UI Automation element (MSAA wrapper).
pub struct UiaElement {
    accessible: windows::Win32::UI::Accessibility::IAccessible,
    child_id: windows::core::VARIANT,
}

impl UiaElement {
    /// Get the name (label).
    pub fn get_name(&self) -> Result<String, UiaError> {
        unsafe {
            let name = self.accessible.get_accName(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accName: {}", e)))?;
            Ok(name.to_string())
        }
    }

    /// Get the role as a string.
    pub fn get_role(&self) -> Result<String, UiaError> {
        unsafe {
            let role = self.accessible.get_accRole(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accRole: {}", e)))?;
            let role_id = role.to_string().parse::<i32>().unwrap_or(0);
            Ok(uia_role_to_string(role_id))
        }
    }

    /// Get the bounding rectangle in screen coordinates.
    pub fn get_bounding_rect(&self) -> Result<(i32, i32, i32, i32), UiaError> {
        unsafe {
            let mut left: i32 = 0;
            let mut top: i32 = 0;
            let mut width: i32 = 0;
            let mut height: i32 = 0;

            self.accessible.accLocation(
                &mut left,
                &mut top,
                &mut width,
                &mut height,
                &self.child_id,
            ).map_err(|e| UiaError::ComError(format!("accLocation: {}", e)))?;

            Ok((left, top, left + width, top + height))
        }
    }

    /// Get the value (text content for edit controls).
    pub fn get_value(&self) -> Result<String, UiaError> {
        unsafe {
            let value = self.accessible.get_accValue(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accValue: {}", e)))?;
            Ok(value.to_string())
        }
    }

    /// Check if the element is focused.
    pub fn is_focused(&self) -> Result<bool, UiaError> {
        unsafe {
            let state = self.accessible.get_accState(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accState: {}", e)))?;
            let state_val = state.to_string().parse::<i32>().unwrap_or(0);
            // STATE_SYSTEM_FOCUSED = 0x00000004
            Ok((state_val & 0x00000004) != 0)
        }
    }

    /// Check if the element is visible.
    pub fn is_visible(&self) -> Result<bool, UiaError> {
        unsafe {
            let state = self.accessible.get_accState(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accState: {}", e)))?;
            let state_val = state.to_string().parse::<i32>().unwrap_or(0);
            // STATE_SYSTEM_INVISIBLE = 0x00008000, STATE_SYSTEM_OFFSCREEN = 0x00010000
            Ok((state_val & 0x00018000) == 0)
        }
    }

    /// Get the default action.
    pub fn get_default_action(&self) -> Result<String, UiaError> {
        unsafe {
            let action = self.accessible.get_accDefaultAction(&self.child_id)
                .map_err(|e| UiaError::ComError(format!("accDefaultAction: {}", e)))?;
            Ok(action.to_string())
        }
    }

    /// Convert to our UIElement model.
    pub fn to_ui_element(&self) -> Result<UIElement, UiaError> {
        let name = self.get_name().unwrap_or_default();
        let role = self.get_role().unwrap_or_default();
        let (left, top, right, bottom) = self.get_bounding_rect()?;
        let value = self.get_value().unwrap_or_default();
        let is_focused = self.is_focused().unwrap_or(false);
        let is_visible = self.is_visible().unwrap_or(true);
        let default_action = self.get_default_action().unwrap_or_default();

        let entity_type = role_to_entity_type(&role);
        let label = if name.is_empty() { role.clone() } else { name.clone() };

        Ok(UIElement {
            id: 0,
            name,
            entity_type,
            label,
            text_content: None,
            value,
            bounds: ScreenRegion::new(left, top, (right - left).max(0) as u32, (bottom - top).max(0) as u32),
            confidence: 1.0,
            is_visible,
            is_interactable: !default_action.is_empty(),
            is_focused,
            automation_id: None,
            control_type: Some(role.clone()),
            semantic_labels: vec![SemanticLabel { label: role, confidence: 1.0 }],
            children: Vec::new(),
        })
    }
}

fn uia_role_to_string(role_id: i32) -> String {
    match role_id {
        1 => "title bar",
        2 => "menu bar",
        3 => "scroll bar",
        4 => "grip",
        5 => "sound",
        6 => "cursor",
        7 => "caret",
        8 => "alert",
        9 => "window",
        10 => "client",
        11 => "menu popup",
        12 => "menu item",
        13 => "tooltip",
        14 => "application",
        15 => "document",
        16 => "pane",
        17 => "chart",
        18 => "dialog",
        19 => "border",
        20 => "grouping",
        21 => "separator",
        22 => "toolbar",
        23 => "status bar",
        24 => "table",
        25 => "column header",
        26 => "row header",
        27 => "column",
        28 => "row",
        29 => "cell",
        30 => "link",
        31 => "help balloon",
        32 => "character",
        33 => "list",
        34 => "list item",
        35 => "outline",
        36 => "outline item",
        37 => "page tab",
        38 => "property page",
        39 => "indicator",
        40 => "graphic",
        41 => "static text",
        42 => "text",
        43 => "push button",
        44 => "check box",
        45 => "radio button",
        46 => "combo box",
        47 => "drop list",
        48 => "progress bar",
        49 => "dial",
        50 => "hot key field",
        51 => "slider",
        52 => "spin box",
        53 => "diagram",
        54 => "animation",
        55 => "equation",
        56 => "button dropdown",
        57 => "button menu",
        58 => "button dropdown grid",
        59 => "whitespace",
        60 => "page tab list",
        61 => "clock",
        62 => "split button",
        63 => "ip address",
        64 => "outline button",
        _ => "unknown",
    }
    .into()
}

fn role_to_entity_type(role: &str) -> EntityType {
    match role.to_lowercase().as_str() {
        "push button" | "split button" | "outline button" => EntityType::Button,
        "check box" => EntityType::Checkbox,
        "radio button" => EntityType::Checkbox,
        "combo box" | "drop list" => EntityType::Dropdown,
        "text" | "hot key field" | "ip address" => EntityType::TextField,
        "link" => EntityType::Link,
        "graphic" => EntityType::Image,
        "menu bar" => EntityType::Menu,
        "menu item" | "button menu" | "button dropdown" => EntityType::MenuItem,
        "scroll bar" => EntityType::ScrollBar,
        "slider" | "spin box" => EntityType::Slider,
        "page tab" => EntityType::Tab,
        "static text" | "row header" | "column header" => EntityType::Label,
        "toolbar" => EntityType::Toolbar,
        "window" | "dialog" | "pane" | "client" => EntityType::Window,
        "table" => EntityType::Table,
        "list item" | "outline item" => EntityType::Label,
        "progress bar" => EntityType::Unknown("progress".into()),
        _ => EntityType::Unknown(role.into()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiaError {
    NotImplemented,
    ComError(String),
    ElementNotFound,
    PropertyNotFound(u32),
    Other(String),
}

use std::fmt;

impl fmt::Display for UiaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiaError::NotImplemented => write!(f, "UI Automation not yet implemented"),
            UiaError::ComError(s) => write!(f, "COM error: {}", s),
            UiaError::ElementNotFound => write!(f, "Element not found"),
            UiaError::PropertyNotFound(id) => write!(f, "Property {} not found", id),
            UiaError::Other(s) => write!(f, "UIA error: {}", s),
        }
    }
}

impl std::error::Error for UiaError {}
