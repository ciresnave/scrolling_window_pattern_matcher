//! # Scrolling Window Pattern Matcher
//!
//! A high-performance, unified pattern matching library for Rust that processes streaming data
//! with configurable window sizes and custom data extractors. This library provides a single
//! `Matcher` type that handles both simple pattern matching and advanced stateful operations
//! with optional context management.
//!
//! ## Features
//!
//! - **Unified Architecture** - Single `Matcher<T, Context>` type for all pattern matching scenarios
//! - **Context Support** - Optional context parameter for stateful operations and data capture
//! - **Pattern Elements** - Exact matching, predicate functions, and range matching
//! - **Extractor System** - Powerful extractors that can continue, extract, or restart pattern matching
//! - **Optional Elements** - Flexible patterns with optional components
//! - **Error Handling** - Comprehensive error types with proper propagation
//! - **Memory Safe** - Zero-copy operations where possible with Rust's ownership guarantees
//!
//! ## Quick Start
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{Matcher, PatternElement};
//!
//! // Create a matcher with window size 10
//! let mut matcher = Matcher::<i32, ()>::new(10);
//!
//! // Add patterns to find sequence 1, 2, 3
//! matcher.add_pattern(PatternElement::exact(1));
//! matcher.add_pattern(PatternElement::exact(2));
//! matcher.add_pattern(PatternElement::exact(3));
//!
//! // Process data items
//! assert_eq!(matcher.process_item(1).unwrap(), None);
//! assert_eq!(matcher.process_item(2).unwrap(), None);
//! assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Pattern complete!
//! ```
//!
//! ## Pattern Elements
//!
//! The library supports three main types of pattern elements:
//!
//! - **Exact Match**: `PatternElement::exact(value)` - matches specific values
//! - **Predicate**: `PatternElement::predicate(|x| condition)` - matches based on custom logic
//! - **Range**: `PatternElement::range(min, max)` - matches values within inclusive range
//!
//! Each pattern element can be configured with `ElementSettings` for advanced behavior
//! including optional elements and custom extractors.
//!
//! ## Extractors
//!
//! Extractors allow you to modify the matching flow and extract custom data:
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{Matcher, PatternElement, ElementSettings, ExtractorAction};
//!
//! let mut matcher = Matcher::<i32, ()>::new(10);
//!
//! // Register an extractor that doubles the matched value
//! matcher.register_extractor(1, |state| {
//!     Ok(ExtractorAction::Extract(state.current_item * 2))
//! });
//!
//! let mut settings = ElementSettings::default();
//! settings.extractor_id = Some(1);
//! matcher.add_pattern(PatternElement::exact_with_settings(5, settings));
//!
//! assert_eq!(matcher.process_item(5).unwrap(), Some(10)); // 5 * 2 = 10
//! ```

use std::collections::HashMap;
use std::fmt;

pub type ExtractorId = u32;

/// Represents the result of running a pattern element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    /// The element matched the current item.
    Match,
    /// The element did not match the current item.
    NoMatch,
    /// There was an error during matching.
    Error,
}

/// Represents the current state during pattern matching.
#[derive(Debug, Clone)]
pub struct MatchState<T> {
    /// The current item being matched.
    pub current_item: T,
    /// The position in the current match sequence.
    pub position: usize,
    /// The total number of items processed.
    pub total_processed: usize,
}

/// Error types for extractors.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractorError {
    /// Extractor failed to process the current state.
    ProcessingFailed(String),
    /// Invalid extractor configuration.
    InvalidConfiguration(String),
}

impl fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractorError::ProcessingFailed(msg) => write!(f, "Processing failed: {}", msg),
            ExtractorError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for ExtractorError {}

/// Action to take after an extractor runs.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractorAction<T> {
    /// Continue with pattern matching.
    Continue,
    /// Stop processing and return the extracted data.
    Extract(T),
    /// Restart the pattern matching process.
    Restart,
}

/// Type alias for extractor functions.
pub type Extractor<T> = Box<dyn Fn(&MatchState<T>) -> Result<ExtractorAction<T>, ExtractorError>>;

/// Error types for the pattern matcher.
#[derive(Debug, Clone, PartialEq)]
pub enum MatcherError {
    /// No patterns have been configured.
    NoPatterns,
    /// Pattern configuration is invalid.
    InvalidPattern(String),
    /// Extractor execution failed.
    ExtractorFailed(ExtractorError),
}

impl fmt::Display for MatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatcherError::NoPatterns => write!(f, "No patterns configured"),
            MatcherError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
            MatcherError::ExtractorFailed(err) => write!(f, "Extractor failed: {}", err),
        }
    }
}

impl std::error::Error for MatcherError {}

/// Configuration settings for pattern elements.
#[derive(Debug)]
pub struct ElementSettings<Context>
where
    Context: Clone + fmt::Debug,
{
    /// Maximum number of retries for this element.
    pub max_retries: usize,
    /// Whether this element is optional in the pattern.
    pub optional: bool,
    /// Custom timeout for this element.
    pub timeout_ms: Option<u64>,
    /// Custom context data for this element.
    pub context: Option<Context>,
    /// Associated extractor ID.
    pub extractor_id: Option<ExtractorId>,
}

impl<Context> Clone for ElementSettings<Context>
where
    Context: Clone + fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            max_retries: self.max_retries,
            optional: self.optional,
            timeout_ms: self.timeout_ms,
            context: self.context.clone(),
            extractor_id: self.extractor_id,
        }
    }
}

impl<Context> Default for ElementSettings<Context>
where
    Context: Clone + fmt::Debug,
{
    fn default() -> Self {
        Self {
            max_retries: 0,
            optional: false,
            timeout_ms: None,
            context: None,
            extractor_id: None,
        }
    }
}

/// A pattern element that can match against items of type T.
pub enum PatternElement<T, Context>
where
    T: Clone + PartialEq + fmt::Debug,
    Context: Clone + fmt::Debug,
{
    /// Matches a specific value.
    Exact {
        value: T,
        settings: Option<ElementSettings<Context>>,
    },
    /// Matches using a custom function.
    Predicate {
        function: Box<dyn Fn(&T) -> bool>,
        settings: Option<ElementSettings<Context>>,
    },
    /// Matches a range of values.
    Range {
        min: T,
        max: T,
        settings: Option<ElementSettings<Context>>,
    },
}

impl<T, Context> Clone for PatternElement<T, Context>
where
    T: Clone + PartialEq + fmt::Debug,
    Context: Clone + fmt::Debug,
{
    fn clone(&self) -> Self {
        match self {
            PatternElement::Exact { value, settings } => PatternElement::Exact {
                value: value.clone(),
                settings: settings.clone(),
            },
            PatternElement::Predicate { settings, .. } => {
                // Note: Functions cannot be cloned, so we create a dummy predicate
                PatternElement::Predicate {
                    function: Box::new(|_| false),
                    settings: settings.clone(),
                }
            }
            PatternElement::Range { min, max, settings } => PatternElement::Range {
                min: min.clone(),
                max: max.clone(),
                settings: settings.clone(),
            },
        }
    }
}

impl<T, Context> fmt::Debug for PatternElement<T, Context>
where
    T: Clone + PartialEq + fmt::Debug,
    Context: Clone + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternElement::Exact { value, settings } => f
                .debug_struct("Exact")
                .field("value", value)
                .field("settings", settings)
                .finish(),
            PatternElement::Predicate { settings, .. } => f
                .debug_struct("Predicate")
                .field("function", &"<function>")
                .field("settings", settings)
                .finish(),
            PatternElement::Range { min, max, settings } => f
                .debug_struct("Range")
                .field("min", min)
                .field("max", max)
                .field("settings", settings)
                .finish(),
        }
    }
}

impl<T, Context> fmt::Display for PatternElement<T, Context>
where
    T: Clone + PartialEq + fmt::Debug,
    Context: Clone + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternElement::Exact { value, .. } => write!(f, "Exact({:?})", value),
            PatternElement::Predicate { .. } => write!(f, "Predicate(<function>)"),
            PatternElement::Range { min, max, .. } => write!(f, "Range({:?}..{:?})", min, max),
        }
    }
}

impl<T, Context> PatternElement<T, Context>
where
    T: Clone + PartialEq + fmt::Debug + std::cmp::PartialOrd,
    Context: Clone + fmt::Debug,
{
    /// Get the settings for this pattern element.
    pub fn settings(&self) -> ElementSettings<Context> {
        match self {
            PatternElement::Exact { settings, .. } => settings.clone().unwrap_or_default(),
            PatternElement::Predicate { settings, .. } => settings.clone().unwrap_or_default(),
            PatternElement::Range { settings, .. } => settings.clone().unwrap_or_default(),
        }
    }

    /// Check if this pattern element matches the given item.
    pub fn matches(&self, item: &T) -> Result<bool, MatcherError> {
        match self {
            PatternElement::Exact { value, .. } => Ok(item == value),
            PatternElement::Predicate { function, .. } => Ok(function(item)),
            PatternElement::Range { min, max, .. } => Ok(item >= min && item <= max),
        }
    }

    /// Create a new exact match pattern element.
    pub fn exact(value: T) -> Self {
        PatternElement::Exact {
            value,
            settings: None,
        }
    }

    /// Create a new exact match pattern element with settings.
    pub fn exact_with_settings(value: T, settings: ElementSettings<Context>) -> Self {
        PatternElement::Exact {
            value,
            settings: Some(settings),
        }
    }

    /// Create a new predicate pattern element.
    pub fn predicate<F>(function: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        PatternElement::Predicate {
            function: Box::new(function),
            settings: None,
        }
    }

    /// Create a new predicate pattern element with settings.
    pub fn predicate_with_settings<F>(function: F, settings: ElementSettings<Context>) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        PatternElement::Predicate {
            function: Box::new(function),
            settings: Some(settings),
        }
    }

    /// Create a new range pattern element.
    pub fn range(min: T, max: T) -> Self {
        PatternElement::Range {
            min,
            max,
            settings: None,
        }
    }

    /// Create a new range pattern element with settings.
    pub fn range_with_settings(min: T, max: T, settings: ElementSettings<Context>) -> Self {
        PatternElement::Range {
            min,
            max,
            settings: Some(settings),
        }
    }
}

/// The main pattern matcher that processes streaming data.
pub struct Matcher<T, Context>
where
    T: Clone + PartialEq + fmt::Debug + std::cmp::PartialOrd,
    Context: Clone + fmt::Debug,
{
    patterns: Vec<PatternElement<T, Context>>,
    current_position: usize,
    total_processed: usize,
    window_size: usize,
    extractors: HashMap<ExtractorId, Extractor<T>>,
    context: Option<Context>,
}

impl<T, Context> Matcher<T, Context>
where
    T: Clone + PartialEq + fmt::Debug + std::cmp::PartialOrd,
    Context: Clone + fmt::Debug,
{
    /// Create a new matcher with the specified window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            patterns: Vec::new(),
            current_position: 0,
            total_processed: 0,
            window_size,
            extractors: HashMap::new(),
            context: None,
        }
    }

    /// Create a new matcher with patterns and window size.
    pub fn with_patterns(patterns: Vec<PatternElement<T, Context>>, window_size: usize) -> Self {
        Self {
            patterns,
            current_position: 0,
            total_processed: 0,
            window_size,
            extractors: HashMap::new(),
            context: None,
        }
    }

    /// Add a pattern element to the matcher.
    pub fn add_pattern(&mut self, pattern: PatternElement<T, Context>) {
        self.patterns.push(pattern);
    }

    /// Register an extractor with the given ID.
    pub fn register_extractor<F>(&mut self, id: ExtractorId, extractor: F)
    where
        F: Fn(&MatchState<T>) -> Result<ExtractorAction<T>, ExtractorError> + 'static,
    {
        self.extractors.insert(id, Box::new(extractor));
    }

    /// Set the context for this matcher.
    pub fn set_context(&mut self, context: Context) {
        self.context = Some(context);
    }

    /// Get the current context.
    pub fn context(&self) -> Option<&Context> {
        self.context.as_ref()
    }

    /// Process a single item and return any extracted data.
    pub fn process_item(&mut self, item: T) -> Result<Option<T>, MatcherError> {
        if self.patterns.is_empty() {
            return Err(MatcherError::NoPatterns);
        }

        self.total_processed += 1;

        let state = MatchState {
            current_item: item.clone(),
            position: self.current_position,
            total_processed: self.total_processed,
        };

        let mut had_any_match = false;

        loop {
            // Check if we're at the end of patterns
            if self.current_position >= self.patterns.len() {
                self.current_position = 0;
                // Only return the item if we had at least one actual match
                return Ok(if had_any_match { Some(item) } else { None });
            }

            let pattern = &self.patterns[self.current_position];
            let matches = pattern.matches(&item)?;

            if matches {
                had_any_match = true;

                // Run any associated extractor before advancing position
                let settings = pattern.settings();
                if let Some(extractor_id) = settings.extractor_id {
                    if let Some(extractor) = self.extractors.get(&extractor_id) {
                        match extractor(&state).map_err(MatcherError::ExtractorFailed)? {
                            ExtractorAction::Continue => {
                                // Continue normal processing
                            }
                            ExtractorAction::Extract(data) => {
                                self.current_position = 0;
                                return Ok(Some(data));
                            }
                            ExtractorAction::Restart => {
                                self.current_position = 0;
                                return Ok(None);
                            }
                        }
                    }
                }

                self.current_position += 1;

                // Check if we've completed the pattern
                if self.current_position >= self.patterns.len() {
                    self.current_position = 0;
                    return Ok(Some(item));
                }

                // Pattern element matched, exit loop
                break;
            } else {
                // No match, check if element is optional
                let settings = pattern.settings();
                if settings.optional {
                    self.current_position += 1;
                    // Continue loop to check next pattern element against same item
                } else {
                    self.current_position = 0;
                    break;
                }
            }
        }

        Ok(None)
    }

    /// Process multiple items and return all extracted data.
    pub fn process_items(&mut self, items: Vec<T>) -> Result<Vec<T>, MatcherError> {
        let mut results = Vec::new();
        for item in items {
            if let Some(extracted) = self.process_item(item)? {
                results.push(extracted);
            }
        }
        Ok(results)
    }

    /// Reset the matcher state.
    pub fn reset(&mut self) {
        self.current_position = 0;
        self.total_processed = 0;
    }

    /// Get the current position in the pattern.
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Get the total number of items processed.
    pub fn total_processed(&self) -> usize {
        self.total_processed
    }

    /// Get the window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Set the window size.
    pub fn set_window_size(&mut self, size: usize) {
        self.window_size = size;
    }

    /// Get the number of patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get a reference to the patterns.
    pub fn patterns(&self) -> &[PatternElement<T, Context>] {
        &self.patterns
    }

    /// Check if the matcher is currently in a matching state.
    pub fn is_matching(&self) -> bool {
        self.current_position > 0
    }
}

impl<T, Context> fmt::Debug for Matcher<T, Context>
where
    T: Clone + PartialEq + fmt::Debug + std::cmp::PartialOrd,
    Context: Clone + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matcher")
            .field("pattern_count", &self.patterns.len())
            .field("current_position", &self.current_position)
            .field("total_processed", &self.total_processed)
            .field("window_size", &self.window_size)
            .field("extractor_count", &self.extractors.len())
            .field("has_context", &self.context.is_some())
            .finish()
    }
}

impl<T, Context> Default for Matcher<T, Context>
where
    T: Clone + PartialEq + fmt::Debug + std::cmp::PartialOrd,
    Context: Clone + fmt::Debug,
{
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    struct TestContext {
        name: String,
        value: i32,
        captured_values: Vec<i32>,
        counters: HashMap<String, usize>,
    }

    impl Default for TestContext {
        fn default() -> Self {
            Self {
                name: "test".to_string(),
                value: 0,
                captured_values: Vec::new(),
                counters: HashMap::new(),
            }
        }
    }

    // === Basic Pattern Matching Tests ===

    #[test]
    fn test_exact_match_simple() {
        let mut matcher = Matcher::<i32, ()>::new(5);
        matcher.add_pattern(PatternElement::exact(42));

        assert_eq!(matcher.process_item(41).unwrap(), None);
        assert_eq!(matcher.process_item(42).unwrap(), Some(42));
    }

    #[test]
    fn test_exact_match_sequence() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));
        matcher.add_pattern(PatternElement::exact(3));

        assert_eq!(matcher.process_item(1).unwrap(), None);
        assert_eq!(matcher.process_item(2).unwrap(), None);
        assert_eq!(matcher.process_item(3).unwrap(), Some(3));
    }

    #[test]
    fn test_exact_match_with_settings() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        let mut settings = ElementSettings::default();
        settings.optional = false;
        settings.max_retries = 2;

        matcher.add_pattern(PatternElement::exact_with_settings(42, settings));

        assert_eq!(matcher.process_item(42).unwrap(), Some(42));
    }

    #[test]
    fn test_predicate_match() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::predicate(|x| *x > 0));
        matcher.add_pattern(PatternElement::predicate(|x| *x < 10));

        assert_eq!(matcher.process_item(5).unwrap(), None);
        assert_eq!(matcher.process_item(3).unwrap(), Some(3));
    }

    #[test]
    fn test_predicate_with_settings() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        let mut settings = ElementSettings::default();
        settings.timeout_ms = Some(1000);

        matcher.add_pattern(PatternElement::predicate_with_settings(
            |x| *x % 2 == 0,
            settings,
        ));

        assert_eq!(matcher.process_item(4).unwrap(), Some(4));
        assert_eq!(matcher.process_item(3).unwrap(), None);
    }

    #[test]
    fn test_range_match() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::range(1, 5));
        matcher.add_pattern(PatternElement::range(6, 10));

        assert_eq!(matcher.process_item(3).unwrap(), None);
        assert_eq!(matcher.process_item(8).unwrap(), Some(8));
    }

    #[test]
    fn test_range_with_settings() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        let mut settings = ElementSettings::default();
        settings.optional = true;

        matcher.add_pattern(PatternElement::range_with_settings(10, 20, settings));

        assert_eq!(matcher.process_item(15).unwrap(), Some(15));
        assert_eq!(matcher.process_item(25).unwrap(), None);
    }

    // === Extractor Tests ===

    #[test]
    fn test_extractor_extract() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        // Register an extractor that doubles large values
        matcher.register_extractor(1, |state| {
            if state.current_item > 10 {
                Ok(ExtractorAction::Extract(state.current_item * 2))
            } else {
                Ok(ExtractorAction::Continue)
            }
        });

        let mut settings = ElementSettings::default();
        settings.extractor_id = Some(1);
        matcher.add_pattern(PatternElement::exact_with_settings(15, settings));

        assert_eq!(matcher.process_item(15).unwrap(), Some(30));
    }

    #[test]
    fn test_extractor_continue() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        matcher.register_extractor(1, |_state| Ok(ExtractorAction::Continue));

        let mut settings = ElementSettings::default();
        settings.extractor_id = Some(1);
        matcher.add_pattern(PatternElement::exact_with_settings(5, settings));
        matcher.add_pattern(PatternElement::exact(10));

        assert_eq!(matcher.process_item(5).unwrap(), None);
        assert_eq!(matcher.process_item(10).unwrap(), Some(10));
    }

    #[test]
    fn test_extractor_restart() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        matcher.register_extractor(1, |_state| Ok(ExtractorAction::Restart));

        let mut settings = ElementSettings::default();
        settings.extractor_id = Some(1);
        matcher.add_pattern(PatternElement::exact_with_settings(5, settings));
        matcher.add_pattern(PatternElement::exact(10));

        assert_eq!(matcher.process_item(5).unwrap(), None);
        assert_eq!(matcher.current_position(), 0); // Should be reset
    }

    #[test]
    fn test_multiple_extractors() {
        // Test extractor 1: Double the value
        let mut matcher1 = Matcher::<i32, TestContext>::new(5);
        matcher1.register_extractor(1, |state| {
            Ok(ExtractorAction::Extract(state.current_item * 2))
        });

        let mut settings1 = ElementSettings::default();
        settings1.extractor_id = Some(1);
        matcher1.add_pattern(PatternElement::exact_with_settings(10, settings1));

        assert_eq!(matcher1.process_item(10).unwrap(), Some(20));

        // Test extractor 2: Triple the value
        let mut matcher2 = Matcher::<i32, TestContext>::new(5);
        matcher2.register_extractor(2, |state| {
            Ok(ExtractorAction::Extract(state.current_item * 3))
        });

        let mut settings2 = ElementSettings::default();
        settings2.extractor_id = Some(2);
        matcher2.add_pattern(PatternElement::exact_with_settings(5, settings2));

        assert_eq!(matcher2.process_item(5).unwrap(), Some(15));
    }

    // === Context Tests ===

    #[test]
    fn test_context_basic() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        let context = TestContext {
            name: "test".to_string(),
            value: 42,
            captured_values: vec![1, 2, 3],
            counters: HashMap::new(),
        };

        matcher.set_context(context.clone());
        assert_eq!(matcher.context(), Some(&context));
    }

    #[test]
    fn test_context_with_extractor() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        let context = TestContext::default();
        matcher.set_context(context);

        // Note: In this simplified design, extractors work with MatchState, not context
        // This is different from the old StatefulMatcher design
        matcher.register_extractor(1, |state| {
            if state.position == 0 {
                Ok(ExtractorAction::Extract(state.current_item + 100))
            } else {
                Ok(ExtractorAction::Continue)
            }
        });

        let mut settings = ElementSettings::default();
        settings.extractor_id = Some(1);
        matcher.add_pattern(PatternElement::exact_with_settings(42, settings));

        assert_eq!(matcher.process_item(42).unwrap(), Some(142));
    }

    // === State Management Tests ===

    #[test]
    fn test_reset() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));

        matcher.process_item(1).unwrap();
        assert_eq!(matcher.current_position(), 1);
        assert_eq!(matcher.total_processed(), 1);

        matcher.reset();
        assert_eq!(matcher.current_position(), 0);
        assert_eq!(matcher.total_processed(), 0);
    }

    #[test]
    fn test_state_inspection() {
        let mut matcher = Matcher::<i32, TestContext>::new(10);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));

        assert_eq!(matcher.window_size(), 10);
        assert_eq!(matcher.pattern_count(), 2);
        assert_eq!(matcher.current_position(), 0);
        assert_eq!(matcher.total_processed(), 0);
        assert!(!matcher.is_matching());

        matcher.process_item(1).unwrap();
        assert_eq!(matcher.current_position(), 1);
        assert_eq!(matcher.total_processed(), 1);
        assert!(matcher.is_matching());
    }

    #[test]
    fn test_window_size_management() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        assert_eq!(matcher.window_size(), 5);

        matcher.set_window_size(20);
        assert_eq!(matcher.window_size(), 20);
    }

    // === Multiple Item Processing Tests ===

    #[test]
    fn test_process_items() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));

        let items = vec![1, 2, 3, 1, 2, 4, 1, 2];
        let results = matcher.process_items(items).unwrap();

        // Should have found three complete patterns: [1,2] at positions 0-1, 3-4, and 6-7
        assert_eq!(results, vec![2, 2, 2]);
    }

    #[test]
    fn test_with_patterns_constructor() {
        let patterns = vec![
            PatternElement::exact(1),
            PatternElement::exact(2),
            PatternElement::exact(3),
        ];

        let mut matcher = Matcher::<i32, TestContext>::with_patterns(patterns, 10);

        assert_eq!(matcher.pattern_count(), 3);
        assert_eq!(matcher.window_size(), 10);

        assert_eq!(matcher.process_item(1).unwrap(), None);
        assert_eq!(matcher.process_item(2).unwrap(), None);
        assert_eq!(matcher.process_item(3).unwrap(), Some(3));
    }

    // === Error Handling Tests ===

    #[test]
    fn test_no_patterns_error() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        let result = matcher.process_item(42);
        assert!(matches!(result, Err(MatcherError::NoPatterns)));
    }

    #[test]
    fn test_extractor_error() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        matcher.register_extractor(1, |_state| {
            Err(ExtractorError::ProcessingFailed("Test error".to_string()))
        });

        let mut settings = ElementSettings::default();
        settings.extractor_id = Some(1);
        matcher.add_pattern(PatternElement::exact_with_settings(42, settings));

        let result = matcher.process_item(42);
        assert!(matches!(result, Err(MatcherError::ExtractorFailed(_))));
    }

    // === Complex Pattern Tests ===

    #[test]
    fn test_mixed_pattern_types() {
        let mut matcher = Matcher::<i32, TestContext>::new(10);

        // Pattern: exact(1), range(5-10), predicate(even)
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::range(5, 10));
        matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

        assert_eq!(matcher.process_item(1).unwrap(), None); // Match first
        assert_eq!(matcher.process_item(7).unwrap(), None); // Match second
        assert_eq!(matcher.process_item(8).unwrap(), Some(8)); // Match third, complete pattern
    }

    #[test]
    fn test_pattern_mismatch_reset() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));
        matcher.add_pattern(PatternElement::exact(3));

        assert_eq!(matcher.process_item(1).unwrap(), None); // Position 1
        assert_eq!(matcher.process_item(5).unwrap(), None); // Mismatch, reset to 0
        assert_eq!(matcher.current_position(), 0);

        assert_eq!(matcher.process_item(1).unwrap(), None); // Position 1 again
        assert_eq!(matcher.process_item(2).unwrap(), None); // Position 2
        assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Complete pattern
    }

    #[test]
    fn test_optional_elements() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);

        // First element is required
        matcher.add_pattern(PatternElement::exact(1));

        // Second element is optional
        let mut settings = ElementSettings::default();
        settings.optional = true;
        matcher.add_pattern(PatternElement::exact_with_settings(2, settings));

        // Third element is required
        matcher.add_pattern(PatternElement::exact(3));

        // Test with optional element present
        assert_eq!(matcher.process_item(1).unwrap(), None);
        assert_eq!(matcher.process_item(2).unwrap(), None);
        assert_eq!(matcher.process_item(3).unwrap(), Some(3));

        matcher.reset();

        // Test with optional element missing
        assert_eq!(matcher.process_item(1).unwrap(), None);
        assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Should skip optional 2
    }

    // === Edge Cases ===

    #[test]
    fn test_single_pattern_element() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(42));

        assert_eq!(matcher.process_item(42).unwrap(), Some(42));
    }

    #[test]
    fn test_empty_after_reset() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));

        matcher.process_item(1).unwrap();
        matcher.reset();

        assert_eq!(matcher.current_position(), 0);
        assert_eq!(matcher.total_processed(), 0);
        assert!(!matcher.is_matching());
    }

    #[test]
    fn test_default_constructor() {
        let matcher = Matcher::<i32, TestContext>::default();
        assert_eq!(matcher.window_size(), 10);
        assert_eq!(matcher.pattern_count(), 0);
    }

    // === Pattern Reference Tests ===

    #[test]
    fn test_patterns_reference() {
        let mut matcher = Matcher::<i32, TestContext>::new(5);
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::range(5, 10));

        let patterns = matcher.patterns();
        assert_eq!(patterns.len(), 2);
    }

    // === String Type Tests ===

    #[test]
    fn test_string_patterns() {
        let mut matcher = Matcher::<String, ()>::new(5);
        matcher.add_pattern(PatternElement::exact("hello".to_string()));
        matcher.add_pattern(PatternElement::predicate(|s: &String| s.len() > 3));

        assert_eq!(matcher.process_item("hello".to_string()).unwrap(), None);
        assert_eq!(
            matcher.process_item("world".to_string()).unwrap(),
            Some("world".to_string())
        );
    }

    // === Performance Test (Basic) ===

    #[test]
    fn test_large_sequence() {
        let mut matcher = Matcher::<usize, ()>::new(100);

        // Pattern to find sequence 1, 2, 3
        matcher.add_pattern(PatternElement::exact(1));
        matcher.add_pattern(PatternElement::exact(2));
        matcher.add_pattern(PatternElement::exact(3));

        let mut count = 0;
        for i in 0..1000 {
            if let Some(_) = matcher.process_item(i % 10).unwrap() {
                count += 1;
            }
        }

        // Should find some complete patterns in the sequence
        assert!(count > 0);
    }
}
