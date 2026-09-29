//! Agent router — routes pointer events to AI models.
//!
//! Decides which AI model to use for different types of pointer queries:
//! - "What is this?" → Vision model (provider-agnostic)
//! - "Click the Submit button" → Element detector + action executor
//! - "Summarize this page" → Vision model with full screenshot
//! - "What's under the cursor?" → UI Automation (fast, no AI needed)

use crate::pointer::models::*;
use crate::vision::analyzer::*;
use crate::pointer::detector::*;

/// Query types the pointer system can handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerQuery {
    /// "What is this?" — identify what's at a position.
    WhatIsThis { position: ScreenPoint },
    /// "Find X" — locate an element by description.
    FindElement { description: String },
    /// "Click X" — find and click an element.
    ClickElement { description: String },
    /// "Type X in Y" — find a field and type text.
    TypeInField { text: String, field_description: String },
    /// "Summarize this" — summarize screen content.
    SummarizeScreen,
    /// "What can I click?" — list interactable elements.
    ListInteractable,
    /// "Show me X" — highlight an element.
    ShowMe { description: String },
}

/// Query result.
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Element was found.
    ElementFound(UIElement),
    /// Multiple elements found.
    MultipleFound(Vec<UIElement>),
    /// Text answer (from vision model).
    TextAnswer(String),
    /// Action was performed.
    ActionPerformed(String),
    /// Nothing found.
    NotFound,
    /// Error.
    Error(String),
}

/// Routes pointer queries to the appropriate backend.
pub struct AgentRouter {
    detector: ElementDetector,
    analyzer: VisionAnalyzer,
}

impl AgentRouter {
    pub fn new(detector_config: DetectorConfig, analyzer_config: AnalyzerConfig) -> Result<Self, RouterError> {
        Ok(Self {
            detector: ElementDetector::new(detector_config)
                .map_err(|e| RouterError::DetectorError(e.to_string()))?,
            analyzer: VisionAnalyzer::new(analyzer_config),
        })
    }

    /// Route and execute a pointer query.
    pub fn query(&mut self, query: &PointerQuery) -> Result<QueryResult, RouterError> {
        match query {
            PointerQuery::WhatIsThis { position } => self.handle_what_is_this(position),
            PointerQuery::FindElement { description } => self.handle_find(description),
            PointerQuery::ClickElement { description } => self.handle_click(description),
            PointerQuery::TypeInField {
                text,
                field_description,
            } => self.handle_type(text, field_description),
            PointerQuery::SummarizeScreen => self.handle_summarize(),
            PointerQuery::ListInteractable => self.handle_list_interactable(),
            PointerQuery::ShowMe { description } => self.handle_show_me(description),
        }
    }

    fn handle_what_is_this(
        &mut self,
        position: &ScreenPoint,
    ) -> Result<QueryResult, RouterError> {
        // Fast path: UI Automation hit test
        if let Some(element) = self.detector.detect_at(position)? {
            return Ok(QueryResult::ElementFound(element));
        }

        // Slow path: Vision model
        // TODO: Capture region → send to vision model
        Ok(QueryResult::NotFound)
    }

    fn handle_find(&mut self, description: &str) -> Result<QueryResult, RouterError> {
        let elements = self.detector.find_by_label(description)?;
        match elements.len() {
            0 => Ok(QueryResult::NotFound),
            1 => Ok(QueryResult::ElementFound(elements.into_iter().next().unwrap())),
            _ => Ok(QueryResult::MultipleFound(elements)),
        }
    }

    fn handle_click(&mut self, description: &str) -> Result<QueryResult, RouterError> {
        let elements = self.detector.find_by_label(description)?;
        if elements.is_empty() {
            return Ok(QueryResult::NotFound);
        }
        let target = &elements[0];
        if !target.is_interactable {
            return Ok(QueryResult::Error(format!(
                "{} is not interactable",
                target.label
            )));
        }
        // TODO: Execute click via action layer
        Ok(QueryResult::ActionPerformed(format!(
            "Clicked {} at {}",
            target.label,
            target.bounds.center()
        )))
    }

    fn handle_type(
        &mut self,
        text: &str,
        field_description: &str,
    ) -> Result<QueryResult, RouterError> {
        let elements = self.detector.find_by_label(field_description)?;
        if elements.is_empty() {
            return Ok(QueryResult::NotFound);
        }
        let target = &elements[0];
        // TODO: Focus field + type text via action layer
        Ok(QueryResult::ActionPerformed(format!(
            "Typed '{}' in {} at {}",
            text,
            target.label,
            target.bounds.center()
        )))
    }

    fn handle_summarize(&mut self) -> Result<QueryResult, RouterError> {
        // TODO: Capture full screen → send to vision model for summarization
        Ok(QueryResult::Error("Summarize not yet implemented".into()))
    }

    fn handle_list_interactable(&mut self) -> Result<QueryResult, RouterError> {
        let all = self.detector.detect_all()?;
        let interactable: Vec<UIElement> = all
            .into_iter()
            .filter(|e| e.is_interactable)
            .collect();
        Ok(QueryResult::MultipleFound(interactable))
    }

    fn handle_show_me(&mut self, description: &str) -> Result<QueryResult, RouterError> {
        let elements = self.detector.find_by_label(description)?;
        if elements.is_empty() {
            return Ok(QueryResult::NotFound);
        }
        // TODO: Trigger overlay animation to highlight the element
        Ok(QueryResult::ElementFound(elements.into_iter().next().unwrap()))
    }
}

#[derive(Debug, Clone)]
pub enum RouterError {
    DetectorError(String),
    VisionError(String),
    ActionError(String),
    Other(String),
}

impl PartialEq for RouterError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RouterError::DetectorError(a), RouterError::DetectorError(b)) => a == b,
            (RouterError::VisionError(a), RouterError::VisionError(b)) => a == b,
            (RouterError::ActionError(a), RouterError::ActionError(b)) => a == b,
            (RouterError::Other(a), RouterError::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for RouterError {}

use std::fmt;

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::DetectorError(s) => write!(f, "Detector error: {}", s),
            RouterError::VisionError(s) => write!(f, "Vision error: {}", s),
            RouterError::ActionError(s) => write!(f, "Action error: {}", s),
            RouterError::Other(s) => write!(f, "Router error: {}", s),
        }
    }
}

impl std::error::Error for RouterError {}

impl From<crate::pointer::detector::DetectorError> for RouterError {
    fn from(err: crate::pointer::detector::DetectorError) -> Self {
        RouterError::DetectorError(err.to_string())
    }
}
