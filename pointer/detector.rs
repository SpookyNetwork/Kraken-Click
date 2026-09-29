//! UI element detector — finds interactive elements on screen.
//!
//! Uses a multi-strategy approach:
//! 1. Windows UI Automation API (primary) — structured element tree
//! 2. OCR fallback (Tesseract/Azure) — text extraction from pixels
//! 3. Vision model (provider-agnostic) — semantic understanding for complex cases
//!
//! The detector is the "eyes" of the pointer system. It answers:
//! - "What UI element is at position (x, y)?"
//! - "Find me a button labeled 'Submit'"
//! - "What text is in this region?"

use crate::pointer::models::*;
use std::collections::HashMap;

/// Detection strategy to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionStrategy {
    /// Use Windows UI Automation API only (fastest, structured).
    UIAutomation,
    /// Use OCR only (for text extraction from images/video).
    OcrOnly,
    /// Use vision model only (most capable, slowest).
    VisionModelOnly,
    /// Try UIAutomation first, fall back to vision model.
    UIAthenVision,
    /// Try all strategies and merge results.
    All,
}

impl Default for DetectionStrategy {
    fn default() -> Self {
        DetectionStrategy::UIAutomation
    }
}

/// Configuration for the element detector.
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Which strategy to use.
    pub strategy: DetectionStrategy,
    /// Minimum confidence threshold for returned elements.
    pub min_confidence: f32,
    /// Maximum number of elements to return per query.
    pub max_results: usize,
    /// Whether to include non-visible elements.
    pub include_hidden: bool,
    /// Vision model endpoint (if using vision strategy).
    pub vision_endpoint: Option<String>,
    /// Vision model API key.
    pub vision_api_key: Option<String>,
    /// OCR engine path or endpoint.
    pub ocr_config: Option<OcrConfig>,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            strategy: DetectionStrategy::default(),
            min_confidence: 0.5,
            max_results: 50,
            include_hidden: false,
            vision_endpoint: None,
            vision_api_key: None,
            ocr_config: None,
        }
    }
}

/// OCR engine configuration.
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// Engine type.
    pub engine: OcrEngine,
    /// Language code (e.g., "en", "en+de").
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OcrEngine {
    /// Tesseract OCR (local).
    Tesseract { data_path: Option<String> },
    /// Azure Computer Vision OCR.
    Azure { endpoint: String, key: String },
    /// Windows OCR API (built into Windows 10+).
    WindowsOcr,
}

/// The element detector — main entry point for finding UI elements.
pub struct ElementDetector {
    config: DetectorConfig,
    /// Cache of recently detected elements (keyed by screen region hash).
    cache: HashMap<u64, Vec<UIElement>>,
    /// UIA client for element detection.
    uia_client: crate::integrations::accessibility::UiaClient,
}

impl ElementDetector {
    pub fn new(config: DetectorConfig) -> Result<Self, DetectorError> {
        let uia_client = crate::integrations::accessibility::UiaClient::new()
            .map_err(|e| DetectorError::Other(format!("Failed to init UIA: {}", e)))?;

        Ok(Self {
            config,
            cache: HashMap::new(),
            uia_client,
        })
    }

    /// Detect all visible UI elements on the primary screen.
    pub fn detect_all(&mut self) -> Result<Vec<UIElement>, DetectorError> {
        match self.config.strategy {
            DetectionStrategy::UIAutomation => self.detect_via_uia(),
            DetectionStrategy::OcrOnly => self.detect_via_ocr(),
            DetectionStrategy::VisionModelOnly => self.detect_via_vision(),
            DetectionStrategy::UIAthenVision => {
                let mut elements = self.detect_via_uia()?;
                if elements.is_empty() {
                    elements = self.detect_via_vision()?;
                }
                Ok(elements)
            }
            DetectionStrategy::All => {
                let mut elements = self.detect_via_uia()?;
                let ocr_elements = self.detect_via_ocr()?;
                elements.extend(ocr_elements);
                Ok(self.deduplicate(elements))
            }
        }
    }

    /// Find the element at a specific screen position.
    /// This is the "what is under the cursor?" query.
    pub fn detect_at(&mut self, point: &ScreenPoint) -> Result<Option<UIElement>, DetectorError> {
        // Strategy 1: UI Automation hit test (fastest)
        if matches!(
            self.config.strategy,
            DetectionStrategy::UIAutomation
                | DetectionStrategy::UIAthenVision
                | DetectionStrategy::All
        ) {
            if let Some(element) = self.hit_test_uia(point)? {
                return Ok(Some(element));
            }
        }

        // Strategy 2: Check cache for nearby elements
        if let Some(element) = self.find_in_cache(point) {
            return Ok(Some(element));
        }

        // Strategy 3: Vision model for semantic understanding
        if matches!(
            self.config.strategy,
            DetectionStrategy::VisionModelOnly
                | DetectionStrategy::UIAthenVision
                | DetectionStrategy::All
        ) {
            if let Some(element) = self.detect_at_via_vision(point)? {
                return Ok(Some(element));
            }
        }

        Ok(None)
    }

    /// Find elements matching a text query (e.g., "find the Submit button").
    pub fn find_by_label(&mut self, label: &str) -> Result<Vec<UIElement>, DetectorError> {
        let all = self.detect_all()?;
        let query_lower = label.to_lowercase();
        let matches: Vec<UIElement> = all
            .into_iter()
            .filter(|e| {
                e.label.to_lowercase().contains(&query_lower)
                    || e.text_content
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || e.automation_id
                        .as_ref()
                        .map(|a| a.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .take(self.config.max_results)
            .collect();
        Ok(matches)
    }

    /// Find elements by entity type (e.g., "find all buttons").
    pub fn find_by_type(&mut self, entity_type: &EntityType) -> Result<Vec<UIElement>, DetectorError> {
        let all = self.detect_all()?;
        Ok(all
            .into_iter()
            .filter(|e| &e.entity_type == entity_type)
            .take(self.config.max_results)
            .collect())
    }

    // ── Private detection strategies ──

    /// Detect elements via Windows UI Automation API.
    fn detect_via_uia(&self) -> Result<Vec<UIElement>, DetectorError> {
        let elements = self.uia_client.enumerate_visible_elements();
        Ok(elements)
    }

    /// Hit-test a specific point via UI Automation.
    fn hit_test_uia(&self, point: &ScreenPoint) -> Result<Option<UIElement>, DetectorError> {
        match self.uia_client.element_from_point(point.x, point.y) {
            Ok(uia_element) => {
                let ui_element = uia_element.to_ui_element()
                    .map_err(|e| DetectorError::Other(format!("UIA conversion failed: {}", e)))?;
                Ok(Some(ui_element))
            }
            Err(crate::integrations::accessibility::UiaError::ElementNotFound) => Ok(None),
            Err(e) => Err(DetectorError::Other(format!("UIA error: {}", e))),
        }
    }

    /// Detect elements via OCR.
    fn detect_via_ocr(&self) -> Result<Vec<UIElement>, DetectorError> {
        // TODO: Capture screen → run OCR → convert text regions to UIElements
        Ok(Vec::new())
    }

    /// Detect elements via vision model.
    fn detect_via_vision(&self) -> Result<Vec<UIElement>, DetectorError> {
        // TODO: Capture screen → send to vision model → parse response
        Ok(Vec::new())
    }

    /// Detect what's at a point via vision model.
    fn detect_at_via_vision(&self, _point: &ScreenPoint) -> Result<Option<UIElement>, DetectorError> {
        // TODO: Crop region around point → send to vision model
        Ok(None)
    }

    // ── Helpers ──

    fn find_in_cache(&self, point: &ScreenPoint) -> Option<UIElement> {
        for elements in self.cache.values() {
            for element in elements {
                if element.bounds.contains(point) && element.is_visible {
                    return Some(element.clone());
                }
            }
        }
        None
    }

    fn deduplicate(&self, elements: Vec<UIElement>) -> Vec<UIElement> {
        let mut result: Vec<UIElement> = Vec::new();
        for candidate in elements {
            let is_duplicate = result.iter().any(|existing| {
                let overlap = Self::overlap_ratio(&existing.bounds, &candidate.bounds);
                overlap > 0.8
            });
            if !is_duplicate {
                result.push(candidate);
            }
        }
        result
    }

    fn overlap_ratio(a: &ScreenRegion, b: &ScreenRegion) -> f64 {
        let x_overlap = (a.x + a.width as i32).min(b.x + b.width as i32) - a.x.max(b.x);
        let y_overlap = (a.y + a.height as i32).min(b.y + b.height as i32) - a.y.max(b.y);
        if x_overlap <= 0 || y_overlap <= 0 {
            return 0.0;
        }
        let intersection = (x_overlap as f64) * (y_overlap as f64);
        let union = a.area() as f64 + b.area() as f64 - intersection;
        if union == 0.0 {
            return 0.0;
        }
        intersection / union
    }

    /// Clear the detection cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Errors that can occur during element detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectorError {
    UIAutomationUnavailable,
    ScreenCaptureDenied,
    VisionApiError(String),
    OcrError(String),
    Timeout,
    Other(String),
}

impl fmt::Display for DetectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectorError::UIAutomationUnavailable => write!(f, "UI Automation API not available"),
            DetectorError::ScreenCaptureDenied => write!(f, "Screen capture permission denied"),
            DetectorError::VisionApiError(e) => write!(f, "Vision API error: {}", e),
            DetectorError::OcrError(e) => write!(f, "OCR error: {}", e),
            DetectorError::Timeout => write!(f, "Detection timed out"),
            DetectorError::Other(e) => write!(f, "Detection error: {}", e),
        }
    }
}

impl std::error::Error for DetectorError {}

use std::fmt;
