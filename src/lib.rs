//! # Scrolling Window Pattern Matcher
//!
//! A flexible pattern matching library with extractor-driven architecture for dynamic behavior modification.
//!
//! This library allows you to create complex patterns that match against sequences of data, with powerful
//! extractor functions that can modify matching behavior at runtime.
//!
//! ## Quick Start
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{ElementSettings, Matcher, PatternElement};
//!
//! // Create a matcher
//! let mut matcher = Matcher::new();
//!
//! // Add a pattern to find the value 42
//! matcher.add_pattern(
//!     "find_42".to_string(),
//!     vec![PatternElement::Value {
//!         value: 42,
//!         settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
//!     }]
//! );
//!
//! // Match against data
//! let data = vec![1, 42, 3, 42, 5];
//! let result = matcher.run(&data);
//! assert!(result.is_ok());
//! ```
//!
//! ## Key Features
//!
//! - **Extractor-driven architecture** - Dynamic modification of matching behavior through extractor functions
//! - **Settings-based configuration** - Clean builder pattern for pattern element configuration
//! - **Rich pattern elements** - Values, functions, pattern references, wildcards, and nested repeats
//! - **Advanced extractor actions** - Continue, Skip, Jump, pattern manipulation, and flow control
//! - **Comprehensive error handling** - Detailed error types with proper error propagation
//!
//! ## Breaking Changes from 1.x
//!
//! This is a complete rewrite from version 1.x with breaking API changes:
//! - Complete API rewrite with settings-based configuration
//! - Callback system replaced with extractor architecture
//! - Field-based pattern syntax replaced with settings builders
//! - Return type changed from `HashMap<String, Vec<T>>` to `Result<(), MatcherError>`

use std::collections::HashMap;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Actions that an extractor can take to modify matching behavior
#[derive(Debug, Clone)]
pub enum ExtractorAction<T> {
    Continue,
    Skip(usize),
    Jump(isize),
    DiscardPartialMatch,
    AddPattern(String, Pattern<T>),
    RemovePattern(String),
    StopMatching,
    RestartFrom(usize),
}

/// State information available to extractors during matching
#[derive(Debug)]
pub struct MatchState<T> {
    pub current_position: usize,
    pub matched_items: Vec<T>,
    pub pattern_name: String,
    pub element_index: usize,
    pub input_length: usize,
}

/// Error that can occur during extractor execution
#[derive(Debug, Clone)]
pub enum ExtractorError {
    Message(String),
    InvalidPosition(usize),
    PatternNotFound(String),
}

impl fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractorError::Message(msg) => write!(f, "Extractor error: {}", msg),
            ExtractorError::InvalidPosition(pos) => write!(f, "Invalid position: {}", pos),
            ExtractorError::PatternNotFound(name) => write!(f, "Pattern not found: {}", name),
        }
    }
}

impl std::error::Error for ExtractorError {}

/// Type alias for extractor functions
pub type Extractor<T> = Box<dyn Fn(&MatchState<T>) -> Result<ExtractorAction<T>, ExtractorError>>;

/// Settings that can be applied to any PatternElement
pub struct ElementSettings<T> {
    pub minimum_repeat: Option<usize>,
    pub maximum_repeat: Option<usize>,
    pub greedy: Option<bool>,
    pub priority: Option<u32>,
    pub extractor: Option<Extractor<T>>,
}

impl<T> fmt::Debug for ElementSettings<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementSettings")
            .field("minimum_repeat", &self.minimum_repeat)
            .field("maximum_repeat", &self.maximum_repeat)
            .field("greedy", &self.greedy)
            .field("priority", &self.priority)
            .field(
                "extractor",
                &self.extractor.as_ref().map(|_| "Extractor<T>"),
            )
            .finish()
    }
}

impl<T> Clone for ElementSettings<T> {
    fn clone(&self) -> Self {
        Self {
            minimum_repeat: self.minimum_repeat,
            maximum_repeat: self.maximum_repeat,
            greedy: self.greedy,
            priority: self.priority,
            extractor: None, // Cannot clone function pointers
        }
    }
}

impl<T> Default for ElementSettings<T> {
    fn default() -> Self {
        Self {
            minimum_repeat: Some(1),
            maximum_repeat: Some(1),
            greedy: Some(false),
            priority: Some(0),
            extractor: None,
        }
    }
}

impl<T> ElementSettings<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_repeat(mut self, count: usize) -> Self {
        self.minimum_repeat = Some(count);
        self
    }

    pub fn max_repeat(mut self, count: usize) -> Self {
        self.maximum_repeat = Some(count);
        self
    }

    pub fn greedy(mut self, greedy: bool) -> Self {
        self.greedy = Some(greedy);
        self
    }

    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn extractor(mut self, extractor: Extractor<T>) -> Self {
        self.extractor = Some(extractor);
        self
    }

    pub fn min_repeat_or_default(&self) -> usize {
        self.minimum_repeat.unwrap_or(1)
    }

    pub fn max_repeat_or_default(&self) -> usize {
        self.maximum_repeat.unwrap_or(1)
    }

    pub fn greedy_or_default(&self) -> bool {
        self.greedy.unwrap_or(false)
    }

    pub fn priority_or_default(&self) -> u32 {
        self.priority.unwrap_or(0)
    }
}

/// A pattern element that can match items of type T
pub enum PatternElement<T> {
    Function {
        function: Box<dyn Fn(&T) -> bool>,
        settings: Option<ElementSettings<T>>,
    },
    Value {
        value: T,
        settings: Option<ElementSettings<T>>,
    },
    Pattern {
        pattern: String,
        settings: Option<ElementSettings<T>>,
    },
    Any {
        settings: Option<ElementSettings<T>>,
    },
    Repeat {
        element: Box<PatternElement<T>>,
        settings: Option<ElementSettings<T>>,
    },
}

impl<T: Clone> Clone for PatternElement<T> {
    fn clone(&self) -> Self {
        match self {
            PatternElement::Function { settings, .. } => {
                PatternElement::Function {
                    function: Box::new(|_| false), // Cannot clone function, use dummy
                    settings: settings.clone(),
                }
            }
            PatternElement::Value { value, settings } => PatternElement::Value {
                value: value.clone(),
                settings: settings.clone(),
            },
            PatternElement::Pattern { pattern, settings } => PatternElement::Pattern {
                pattern: pattern.clone(),
                settings: settings.clone(),
            },
            PatternElement::Any { settings } => PatternElement::Any {
                settings: settings.clone(),
            },
            PatternElement::Repeat { element, settings } => PatternElement::Repeat {
                element: element.clone(),
                settings: settings.clone(),
            },
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for PatternElement<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternElement::Function { settings, .. } => f
                .debug_struct("Function")
                .field("function", &"Box<dyn Fn(&T) -> bool>")
                .field("settings", settings)
                .finish(),
            PatternElement::Value { value, settings } => f
                .debug_struct("Value")
                .field("value", value)
                .field("settings", settings)
                .finish(),
            PatternElement::Pattern { pattern, settings } => f
                .debug_struct("Pattern")
                .field("pattern", pattern)
                .field("settings", settings)
                .finish(),
            PatternElement::Any { settings } => {
                f.debug_struct("Any").field("settings", settings).finish()
            }
            PatternElement::Repeat { element, settings } => f
                .debug_struct("Repeat")
                .field("element", element)
                .field("settings", settings)
                .finish(),
        }
    }
}

impl<T> PatternElement<T> {
    pub fn settings_ref(&self) -> &Option<ElementSettings<T>> {
        match self {
            PatternElement::Function { settings, .. } => settings,
            PatternElement::Value { settings, .. } => settings,
            PatternElement::Pattern { settings, .. } => settings,
            PatternElement::Any { settings } => settings,
            PatternElement::Repeat { settings, .. } => settings,
        }
    }

    // Kept for backward compatibility, but will not include extractor
    pub fn settings(&self) -> ElementSettings<T> {
        match self {
            PatternElement::Function { settings, .. } => {
                settings.as_ref().cloned().unwrap_or_default()
            }
            PatternElement::Value { settings, .. } => {
                settings.as_ref().cloned().unwrap_or_default()
            }
            PatternElement::Pattern { settings, .. } => {
                settings.as_ref().cloned().unwrap_or_default()
            }
            PatternElement::Any { settings } => settings.as_ref().cloned().unwrap_or_default(),
            PatternElement::Repeat { settings, .. } => {
                settings.as_ref().cloned().unwrap_or_default()
            }
        }
    }
}

/// Settings for pattern-level behavior
pub struct PatternSettings<T> {
    pub priority: Option<u32>,
    pub extractor: Option<Extractor<T>>,
}

impl<T> fmt::Debug for PatternSettings<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatternSettings")
            .field("priority", &self.priority)
            .field(
                "extractor",
                &self.extractor.as_ref().map(|_| "Extractor<T>"),
            )
            .finish()
    }
}

impl<T> Clone for PatternSettings<T> {
    fn clone(&self) -> Self {
        Self {
            priority: self.priority,
            extractor: None, // Cannot clone function pointers
        }
    }
}

impl<T> Default for PatternSettings<T> {
    fn default() -> Self {
        Self {
            priority: Some(0),
            extractor: None,
        }
    }
}

impl<T> PatternSettings<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn extractor(mut self, extractor: Extractor<T>) -> Self {
        self.extractor = Some(extractor);
        self
    }

    pub fn priority_or_default(&self) -> u32 {
        self.priority.unwrap_or(0)
    }
}

/// A complete pattern consisting of multiple elements
pub struct Pattern<T> {
    pub elements: Vec<PatternElement<T>>,
    pub settings: Option<PatternSettings<T>>,
}

impl<T: fmt::Debug> fmt::Debug for Pattern<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pattern")
            .field("elements", &self.elements)
            .field("settings", &self.settings)
            .finish()
    }
}

impl<T> Clone for Pattern<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            settings: self.settings.clone(),
        }
    }
}

impl<T> Pattern<T> {
    pub fn new(elements: Vec<PatternElement<T>>) -> Self {
        Self {
            elements,
            settings: None,
        }
    }

    pub fn with_settings(elements: Vec<PatternElement<T>>, settings: PatternSettings<T>) -> Self {
        Self {
            elements,
            settings: Some(settings),
        }
    }

    pub fn settings_ref(&self) -> &Option<PatternSettings<T>> {
        &self.settings
    }

    // Kept for backward compatibility, but will not include extractor
    pub fn settings(&self) -> PatternSettings<T> {
        self.settings.as_ref().cloned().unwrap_or_default()
    }
}

/// Settings for matcher-level behavior
pub struct MatcherSettings<T> {
    pub skip_unmatched: Option<bool>,
    pub priority: Option<u32>,
    pub extractor: Option<Extractor<T>>,
}

impl<T> fmt::Debug for MatcherSettings<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MatcherSettings")
            .field("skip_unmatched", &self.skip_unmatched)
            .field("priority", &self.priority)
            .field(
                "extractor",
                &self.extractor.as_ref().map(|_| "Extractor<T>"),
            )
            .finish()
    }
}

impl<T> Default for MatcherSettings<T> {
    fn default() -> Self {
        Self {
            skip_unmatched: Some(false),
            priority: Some(0),
            extractor: None,
        }
    }
}

impl<T> MatcherSettings<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn skip_unmatched(mut self, skip: bool) -> Self {
        self.skip_unmatched = Some(skip);
        self
    }

    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn extractor(mut self, extractor: Extractor<T>) -> Self {
        self.extractor = Some(extractor);
        self
    }

    pub fn skip_unmatched_or_default(&self) -> bool {
        self.skip_unmatched.unwrap_or(false)
    }

    pub fn priority_or_default(&self) -> u32 {
        self.priority.unwrap_or(0)
    }
}

/// Errors that can occur during matching
#[derive(Debug, Clone)]
pub enum MatcherError {
    ExtractorPanic(String),
    InvalidAction(String),
    InvalidPosition(usize),
    PatternNotFound(String),
    ExtractorError(ExtractorError),
}

impl fmt::Display for MatcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatcherError::ExtractorPanic(msg) => write!(f, "Extractor panicked: {}", msg),
            MatcherError::InvalidAction(msg) => write!(f, "Invalid action: {}", msg),
            MatcherError::InvalidPosition(pos) => write!(f, "Invalid position: {}", pos),
            MatcherError::PatternNotFound(name) => write!(f, "Pattern not found: {}", name),
            MatcherError::ExtractorError(err) => write!(f, "Extractor error: {}", err),
        }
    }
}

impl std::error::Error for MatcherError {}

impl From<ExtractorError> for MatcherError {
    fn from(err: ExtractorError) -> Self {
        MatcherError::ExtractorError(err)
    }
}

/// Main pattern matcher
pub struct Matcher<T> {
    patterns: HashMap<String, Pattern<T>>,
    settings: MatcherSettings<T>,
}

impl<T: fmt::Debug> fmt::Debug for Matcher<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Matcher")
            .field("patterns", &self.patterns.keys().collect::<Vec<_>>())
            .field("settings", &self.settings)
            .finish()
    }
}

impl<T> Default for Matcher<T>
where
    T: Clone + PartialEq + fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Matcher<T>
where
    T: Clone + PartialEq + fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            settings: MatcherSettings::default(),
        }
    }

    pub fn with_settings(settings: MatcherSettings<T>) -> Self {
        Self {
            patterns: HashMap::new(),
            settings,
        }
    }

    pub fn add_pattern(&mut self, name: String, elements: Vec<PatternElement<T>>) {
        self.patterns.insert(name, Pattern::new(elements));
    }

    pub fn add_pattern_with_settings(&mut self, name: String, pattern: Pattern<T>) {
        self.patterns.insert(name, pattern);
    }

    pub fn remove_pattern(&mut self, name: &str) -> Option<Pattern<T>> {
        self.patterns.remove(name)
    }

    pub fn set_settings(&mut self, settings: MatcherSettings<T>) {
        self.settings = settings;
    }

    pub fn settings(&self) -> &MatcherSettings<T> {
        &self.settings
    }

    pub fn run(&mut self, data: &[T]) -> Result<(), MatcherError> {
        let mut position = 0;

        while position < data.len() {
            let mut matched = false;

            // Sort patterns by priority (lower number = higher priority)
            let mut pattern_list: Vec<_> = self.patterns.iter().collect();
            pattern_list.sort_by_key(|(_, pattern)| pattern.settings().priority_or_default());

            // Try each pattern
            for (pattern_name, pattern) in pattern_list {
                if let Some(match_length) =
                    self.try_match_pattern(data, position, pattern_name, pattern)?
                {
                    // Execute pattern-level extractor if present
                    let default_pattern_settings = PatternSettings::default();
                    let pattern_settings = pattern
                        .settings_ref()
                        .as_ref()
                        .unwrap_or(&default_pattern_settings);
                    if let Some(ref extractor) = pattern_settings.extractor {
                        let state = MatchState {
                            current_position: position,
                            matched_items: data[position..position + match_length].to_vec(),
                            pattern_name: pattern_name.clone(),
                            element_index: 0,
                            input_length: data.len(),
                        };

                        let action = self.execute_extractor(extractor, &state)?;
                        let is_continue = matches!(action, ExtractorAction::Continue);
                        let new_position =
                            self.handle_extractor_action(action, position, data.len())?;

                        // If the extractor action was Continue, we need to advance by the match length
                        // For other actions, the extractor action handler takes care of positioning
                        if is_continue {
                            position += match_length;
                        } else {
                            position = new_position;
                        }
                    } else {
                        // If match_length is 0, this means the pattern matched but consumed no input
                        // We need to advance at least 1 position to avoid infinite loops
                        position += if match_length == 0 { 1 } else { match_length };
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                position += 1;
            }
        }

        Ok(())
    }

    fn try_match_pattern(
        &self,
        data: &[T],
        position: usize,
        pattern_name: &str,
        pattern: &Pattern<T>,
    ) -> Result<Option<usize>, MatcherError> {
        let mut current_pos = position;

        for (element_index, element) in pattern.elements.iter().enumerate() {
            if let Some(match_length) =
                self.try_match_element(data, current_pos, element, pattern_name, element_index)?
            {
                current_pos += match_length;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(current_pos - position))
    }

    fn try_match_element(
        &self,
        data: &[T],
        position: usize,
        element: &PatternElement<T>,
        pattern_name: &str,
        element_index: usize,
    ) -> Result<Option<usize>, MatcherError> {
        if position >= data.len() {
            return Ok(None);
        }

        let settings = element.settings();
        let default_settings = ElementSettings::default();
        let actual_settings = element.settings_ref().as_ref().unwrap_or(&default_settings);

        let min_repeat = settings.min_repeat_or_default();
        let max_repeat = settings.max_repeat_or_default();

        // Special case: if max_repeat is 0, this is a negative assertion - the element must NOT match
        if max_repeat == 0 {
            if min_repeat == 0 {
                // Check if the element would match at the current position
                let item_matches = match element {
                    PatternElement::Function { function, .. } => function(&data[position]),
                    PatternElement::Value { value, .. } => &data[position] == value,
                    PatternElement::Pattern { pattern, .. } => {
                        format!("{:?}", data[position]).contains(pattern)
                    }
                    PatternElement::Any { .. } => true,
                    PatternElement::Repeat { .. } => {
                        // For repeat elements with max_repeat=0, we need to check if the inner element would match
                        // This is complex, so for now, let's treat it as not matching
                        false
                    }
                };

                if item_matches {
                    // The element matches but max_repeat=0, so this is a failed negative assertion
                    return Ok(None);
                } else {
                    // The element doesn't match and max_repeat=0, so the negative assertion succeeds
                    return Ok(Some(0));
                }
            } else {
                return Ok(None); // Can't satisfy min_repeat > 0 with max_repeat = 0
            }
        }

        let mut matched_count = 0;
        let mut current_pos = position;

        while matched_count < max_repeat && current_pos < data.len() {
            let item_matches = match element {
                PatternElement::Function { function, .. } => function(&data[current_pos]),
                PatternElement::Value { value, .. } => &data[current_pos] == value,
                PatternElement::Pattern { pattern, .. } => {
                    format!("{:?}", data[current_pos]).contains(pattern)
                }
                PatternElement::Any { .. } => true,
                PatternElement::Repeat { element: inner, .. } => {
                    return self.try_match_element(
                        data,
                        position,
                        inner,
                        pattern_name,
                        element_index,
                    );
                }
            };

            if item_matches {
                matched_count += 1;
                current_pos += 1;

                if let Some(ref extractor) = actual_settings.extractor {
                    let state = MatchState {
                        current_position: current_pos - 1,
                        matched_items: vec![data[current_pos - 1].clone()],
                        pattern_name: pattern_name.to_string(),
                        element_index,
                        input_length: data.len(),
                    };

                    let action = self.execute_extractor(extractor, &state)?;
                    match action {
                        ExtractorAction::Continue => {}
                        ExtractorAction::Skip(n) => {
                            let new_pos = current_pos + n;
                            if new_pos <= data.len() {
                                current_pos = new_pos;
                            } else {
                                return Err(MatcherError::InvalidPosition(new_pos));
                            }
                        }
                        ExtractorAction::DiscardPartialMatch => return Ok(None),
                        ExtractorAction::RemovePattern(pattern_name) => {
                            // Check if pattern exists without removing it (since we can't mutate here)
                            if !self.patterns.contains_key(&pattern_name) {
                                return Err(MatcherError::PatternNotFound(pattern_name));
                            }
                            // In a real implementation, we'd need to defer this action
                        }
                        ExtractorAction::AddPattern(_name, _pattern) => {
                            // Skip for now, this requires more complex handling
                        }
                        ExtractorAction::StopMatching => return Ok(None),
                        ExtractorAction::RestartFrom(pos) => {
                            if pos <= data.len() {
                                // This is complex to handle mid-element matching
                                // For now, just continue
                            } else {
                                return Err(MatcherError::InvalidPosition(pos));
                            }
                        }
                        ExtractorAction::Jump(offset) => {
                            let new_pos = if offset >= 0 {
                                current_pos + offset as usize
                            } else {
                                current_pos.saturating_sub((-offset) as usize)
                            };
                            if new_pos <= data.len() {
                                current_pos = new_pos;
                            } else {
                                return Err(MatcherError::InvalidPosition(new_pos));
                            }
                        }
                    }
                }

                if !settings.greedy_or_default() && matched_count >= min_repeat {
                    break;
                }
            } else {
                break;
            }
        }

        if matched_count >= min_repeat {
            Ok(Some(current_pos - position))
        } else {
            Ok(None)
        }
    }

    fn execute_extractor(
        &self,
        extractor: &Extractor<T>,
        state: &MatchState<T>,
    ) -> Result<ExtractorAction<T>, MatcherError> {
        match catch_unwind(AssertUnwindSafe(|| extractor(state))) {
            Ok(Ok(action)) => Ok(action),
            Ok(Err(err)) => Err(MatcherError::ExtractorError(err)),
            Err(_) => Err(MatcherError::ExtractorPanic(
                "Extractor function panicked".to_string(),
            )),
        }
    }

    fn handle_extractor_action(
        &mut self,
        action: ExtractorAction<T>,
        current_position: usize,
        data_length: usize,
    ) -> Result<usize, MatcherError> {
        match action {
            ExtractorAction::Continue => Ok(current_position),
            ExtractorAction::Skip(n) => {
                let new_pos = current_position + n;
                if new_pos <= data_length {
                    Ok(new_pos)
                } else {
                    Err(MatcherError::InvalidPosition(new_pos))
                }
            }
            ExtractorAction::Jump(offset) => {
                let new_pos = if offset >= 0 {
                    current_position + offset as usize
                } else {
                    current_position.saturating_sub((-offset) as usize)
                };
                if new_pos <= data_length {
                    Ok(new_pos)
                } else {
                    Err(MatcherError::InvalidPosition(new_pos))
                }
            }
            ExtractorAction::RestartFrom(pos) => {
                if pos <= data_length {
                    Ok(pos)
                } else {
                    Err(MatcherError::InvalidPosition(pos))
                }
            }
            ExtractorAction::AddPattern(name, pattern) => {
                self.patterns.insert(name, pattern);
                Ok(current_position)
            }
            ExtractorAction::RemovePattern(name) => {
                if self.patterns.remove(&name).is_some() {
                    Ok(current_position)
                } else {
                    Err(MatcherError::PatternNotFound(name))
                }
            }
            ExtractorAction::StopMatching => Ok(data_length),
            ExtractorAction::DiscardPartialMatch => Ok(current_position + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_value_matching() {
        let mut matcher = Matcher::new();
        matcher.add_pattern(
            "find_42".to_string(),
            vec![PatternElement::Value {
                value: 42,
                settings: None,
            }],
        );

        let data = vec![1, 2, 42, 3, 4];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_function_matching() {
        let mut matcher = Matcher::new();
        matcher.add_pattern(
            "even_numbers".to_string(),
            vec![PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 0),
                settings: None,
            }],
        );

        let data = vec![1, 2, 3, 4, 5];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_any_matching() {
        let mut matcher = Matcher::new();
        matcher.add_pattern(
            "any_item".to_string(),
            vec![PatternElement::Any { settings: None }],
        );

        let data = vec![1, 2, 3];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_repeat_pattern() {
        let mut matcher = Matcher::new();
        matcher.add_pattern(
            "repeated_ones".to_string(),
            vec![PatternElement::Value {
                value: 1,
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(3)),
            }],
        );

        let data = vec![1, 1, 1, 2, 2, 3];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_extractor_functionality() {
        let mut matcher = Matcher::new();

        matcher.add_pattern(
            "find_with_extractor".to_string(),
            vec![PatternElement::Value {
                value: 42,
                settings: Some(
                    ElementSettings::new()
                        .extractor(Box::new(|_state| Ok(ExtractorAction::Continue))),
                ),
            }],
        );

        let data = vec![1, 42, 3, 42, 5];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_complex_pattern() {
        let mut matcher = Matcher::new();
        matcher.add_pattern(
            "complex".to_string(),
            vec![
                PatternElement::Function {
                    function: Box::new(|x: &i32| *x % 2 == 0),
                    settings: None,
                },
                PatternElement::Value {
                    value: 5,
                    settings: None,
                },
                PatternElement::Any { settings: None },
            ],
        );

        let data = vec![2, 5, 8, 1, 3];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_settings_builder() {
        let settings: ElementSettings<i32> = ElementSettings::new()
            .min_repeat(2)
            .max_repeat(5)
            .greedy(true)
            .priority(10);

        assert_eq!(settings.min_repeat_or_default(), 2);
        assert_eq!(settings.max_repeat_or_default(), 5);
        assert!(settings.greedy_or_default());
        assert_eq!(settings.priority_or_default(), 10);
    }

    #[test]
    fn test_error_handling() {
        let mut matcher = Matcher::new();

        // Test extractor that returns an error when it actually gets executed
        matcher.add_pattern(
            "error_pattern".to_string(),
            vec![PatternElement::Value {
                value: 42,
                settings: Some(ElementSettings::new().extractor(Box::new(|_| {
                    Err(ExtractorError::Message("Test error".to_string()))
                }))),
            }],
        );

        // This will find the value 42 and execute the extractor, causing an error
        let data = vec![42];
        let result = matcher.run(&data);
        assert!(result.is_err());

        // Verify it's the correct error type
        match result {
            Err(MatcherError::ExtractorError(ExtractorError::Message(msg))) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected ExtractorError::Message"),
        }
    }

    #[test]
    fn test_extractor_actions() {
        let mut matcher = Matcher::new();

        matcher.add_pattern(
            "skip_test".to_string(),
            vec![PatternElement::Value {
                value: 1,
                settings: Some(
                    ElementSettings::new().extractor(Box::new(|_| Ok(ExtractorAction::Skip(2)))),
                ),
            }],
        );

        let data = vec![1, 2, 3, 4, 5];
        assert!(matcher.run(&data).is_ok());
    }

    #[test]
    fn test_priority_ordering() {
        let mut matcher = Matcher::new();

        matcher.add_pattern_with_settings(
            "low_priority".to_string(),
            Pattern::with_settings(
                vec![PatternElement::Any { settings: None }],
                PatternSettings::new().priority(10),
            ),
        );

        matcher.add_pattern_with_settings(
            "high_priority".to_string(),
            Pattern::with_settings(
                vec![PatternElement::Any { settings: None }],
                PatternSettings::new().priority(1),
            ),
        );

        let data = vec![1, 2, 3];
        assert!(matcher.run(&data).is_ok());
    }
}
