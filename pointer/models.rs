//! Pointer state models and data types.
//!
//! Defines the core data structures for the pointer intelligence layer:
//! - Screen regions and coordinates
//! - UI element descriptors
//! - Semantic labels
//! - Pointer state machine

use std::fmt;

/// A point in screen space (physical pixels, origin top-left).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl ScreenPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another point.
    pub fn distance_to(&self, other: &ScreenPoint) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

impl fmt::Display for ScreenPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A rectangular region in screen space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ScreenRegion {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// Center point of the region.
    pub fn center(&self) -> ScreenPoint {
        ScreenPoint::new(
            self.x + (self.width as i32) / 2,
            self.y + (self.height as i32) / 2,
        )
    }

    /// Check if a point is inside this region.
    pub fn contains(&self, point: &ScreenPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width as i32
            && point.y >= self.y
            && point.y < self.y + self.height as i32
    }

    /// Area in square pixels.
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// Semantic entity type — what the AI understands a screen region to be.
/// This is the DeepMind AI Pointer "turn pixels into actionable entities" principle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    // Text
    Word,
    Sentence,
    Paragraph,
    Heading,
    Label,
    // UI Elements
    Button,
    TextField,
    Checkbox,
    Dropdown,
    Menu,
    MenuItem,
    Tab,
    ScrollBar,
    Slider,
    Toggle,
    // Content
    Image,
    Video,
    Table,
    Chart,
    CodeBlock,
    Link,
    // Structural
    Window,
    Dialog,
    Toolbar,
    Sidebar,
    StatusBar,
    // Domain-specific
    Place,
    Date,
    Email,
    Phone,
    Price,
    // Fallback
    Unknown(String),
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Unknown(s) => write!(f, "Unknown({})", s),
            other => write!(f, "{:?}", other),
        }
    }
}

/// A detected UI element on screen.
#[derive(Debug, Clone)]
pub struct UIElement {
    /// Unique identifier for this detection.
    pub id: u64,
    /// Bounding box in screen coordinates.
    pub bounds: ScreenRegion,
    /// What kind of element this is.
    pub entity_type: EntityType,
    /// Human-readable label (e.g., "Submit button", "Search field").
    pub label: String,
    /// Element name from accessibility API.
    pub name: String,
    /// Extracted text content, if any.
    pub text_content: Option<String>,
    /// Value (e.g., text field content).
    pub value: String,
    /// Confidence score from the detection model (0.0–1.0).
    pub confidence: f32,
    /// Whether the element is currently visible on screen.
    pub is_visible: bool,
    /// Whether the element is interactable (clickable, typeable, etc.).
    pub is_interactable: bool,
    /// Whether the element has keyboard focus.
    pub is_focused: bool,
    /// Automation ID from UI Automation API, if available.
    pub automation_id: Option<String>,
    /// Control type from UI Automation API.
    pub control_type: Option<String>,
    /// Semantic labels.
    pub semantic_labels: Vec<SemanticLabel>,
    /// Child elements.
    pub children: Vec<UIElement>,
}

impl UIElement {
    pub fn new(id: u64, bounds: ScreenRegion, entity_type: EntityType) -> Self {
        Self {
            id,
            bounds,
            entity_type,
            label: String::new(),
            name: String::new(),
            text_content: None,
            value: String::new(),
            confidence: 1.0,
            is_visible: true,
            is_interactable: false,
            is_focused: false,
            automation_id: None,
            control_type: None,
            semantic_labels: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Builder: set label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Builder: set text content.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_content = Some(text.into());
        self
    }

    /// Builder: set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Builder: set interactable.
    pub fn with_interactable(mut self, interactable: bool) -> Self {
        self.is_interactable = interactable;
        self
    }
}

/// Semantic label for a screen region — the "what is this?" answer.
#[derive(Debug, Clone)]
pub struct SemanticLabel {
    /// Human-readable label (e.g., "push button", "text field").
    pub label: String,
    /// Confidence score.
    pub confidence: f32,
}

/// Pointer mode — what the pointer is currently doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerMode {
    /// Idle — just tracking position.
    Idle,
    /// Hovering — user is hovering over an element, showing context.
    Hovering,
    /// Pointing — actively pointing at something (triangle animation).
    Pointing,
    /// Selecting — user is selecting a region.
    Selecting,
    /// Captioning — showing a caption bubble.
    Captioning,
    /// Multi-pointing — showing multiple markers simultaneously.
    MultiPointing,
}

/// Current pointer state.
#[derive(Debug, Clone)]
pub struct PointerState {
    /// Current pointer mode.
    pub mode: PointerMode,
    /// Current cursor position in screen coordinates.
    pub cursor_position: ScreenPoint,
    /// Element currently under the cursor, if any.
    pub hovered_element: Option<UIElement>,
    /// Element currently being pointed at, if any.
    pub pointed_element: Option<UIElement>,
    /// Active caption text, if any.
    pub active_caption: Option<String>,
    /// Multi-marker state.
    pub active_markers: Vec<PointerMarker>,
    /// Timestamp of last state change.
    pub last_update_ms: u64,
}

impl Default for PointerState {
    fn default() -> Self {
        Self {
            mode: PointerMode::Idle,
            cursor_position: ScreenPoint::new(0, 0),
            hovered_element: None,
            pointed_element: None,
            active_caption: None,
            active_markers: Vec::new(),
            last_update_ms: 0,
        }
    }
}

/// A pointer marker — a visual indicator at a screen position with optional caption.
#[derive(Debug, Clone)]
pub struct PointerMarker {
    pub id: String,
    pub position: ScreenPoint,
    pub caption: Option<String>,
    pub accent_color: Option<String>,  // hex color like "#2563EB"
    pub duration_ms: Option<u64>,      // None = persistent until cleared
    pub created_at_ms: u64,
}

impl PointerMarker {
    pub fn new(id: impl Into<String>, position: ScreenPoint) -> Self {
        Self {
            id: id.into(),
            position,
            caption: None,
            accent_color: None,
            duration_ms: Some(4500),
            created_at_ms: 0, // set by overlay system
        }
    }

    pub fn with_caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.accent_color = Some(color.into());
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn persistent(mut self) -> Self {
        self.duration_ms = None;
        self
    }

    /// Check if this marker has expired.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        match self.duration_ms {
            Some(duration) => now_ms - self.created_at_ms > duration,
            None => false,
        }
    }
}

/// A "this" or "that" reference — natural language shorthand tied to a screen location.
/// From DeepMind AI Pointer: "Fix this", "Move that here", "What does this mean?"
#[derive(Debug, Clone)]
pub struct DeicticReference {
    /// The word used ("this", "that", "here", "there", etc.).
    pub word: String,
    /// The screen location being referred to.
    pub location: ScreenPoint,
    /// The element at that location, if identified.
    pub resolved_element: Option<UIElement>,
    /// The user's full utterance for context.
    pub utterance: String,
}
