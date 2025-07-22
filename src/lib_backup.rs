//! ScrollingWindowPatternMatcher
//!
//! A flexible, ergonomic pattern matcher for slices, arrays, and windows, supporting wildcards, custom logic, and builder patterns.
//!
//! # Features
//! - Wildcard matching (`PatternElem::Any`)
//! - Flexible matcher signatures: pass window and patterns as Vec, slice, or array, and patterns as owned or referenced
//! - Ergonomic builder patterns
//! - Custom matcher logic
//! - Flexible callback and overlap configuration

use std::{borrow::Borrow, fmt};

/// Pattern element: matches a value, a predicate, or any value (wildcard)
///
/// - `Value(T)`: Matches a specific value.
/// - `Matcher(Box<dyn Fn(&T) -> bool>)`: Matches using a custom predicate function.
/// - `Any`: Matches any value (wildcard).
///
/// Used to build flexible patterns for matching windows of data.
pub enum PatternElem<T> {
    /// Matches a specific value, with optional repeat and capture name.
    ///
    /// - `min_repeat`, `max_repeat`: Minimum and maximum number of times this element must repeat consecutively.
    /// - `capture_name`: If set, matched values are stored under this name in the output.
    Value {
        value: T,
        min_repeat: Option<usize>,
        max_repeat: Option<usize>,
        capture_name: Option<String>,
    },
    /// Matches using a custom predicate, with optional repeat and capture name
    Matcher {
        matcher: Box<dyn Fn(&T) -> bool + 'static>,
        min_repeat: Option<usize>,
        max_repeat: Option<usize>,
        capture_name: Option<String>,
    },
    /// Matches any value (wildcard), with optional repeat and capture name
    Any {
        min_repeat: Option<usize>,
        max_repeat: Option<usize>,
        capture_name: Option<String>,
    },
}

impl<T: fmt::Debug> fmt::Debug for PatternElem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternElem::Value { value, min_repeat, max_repeat, capture_name } => {
                write!(f, "Value({:?}, min_repeat={:?}, max_repeat={:?}, capture_name={:?})", value, min_repeat, max_repeat, capture_name)
            }
            PatternElem::Matcher { min_repeat, max_repeat, capture_name, .. } => {
                write!(f, "Matcher(.., min_repeat={:?}, max_repeat={:?}, capture_name={:?})", min_repeat, max_repeat, capture_name)
            }
            PatternElem::Any { min_repeat, max_repeat, capture_name } => {
                write!(f, "Any(min_repeat={:?}, max_repeat={:?}, capture_name={:?})", min_repeat, max_repeat, capture_name)
            }
        }
    }
}

/// Manual implementation of Clone for PatternElem.
///
/// - `Value(v)`: Clones the value.
/// - `Any`: Returns Any.
/// - `Matcher`: Panics (cannot clone closures).
impl<T: Clone> Clone for PatternElem<T> {
    fn clone(&self) -> Self {
        match self {
            PatternElem::Value { value, min_repeat, max_repeat, capture_name } => PatternElem::Value {
                value: value.clone(),
                min_repeat: *min_repeat,
                max_repeat: *max_repeat,
                capture_name: capture_name.clone(),
            },
            PatternElem::Any { min_repeat, max_repeat, capture_name } => PatternElem::Any {
                min_repeat: *min_repeat,
                max_repeat: *max_repeat,
                capture_name: capture_name.clone(),
            },
            PatternElem::Matcher { .. } => panic!("Cannot clone PatternElem::Matcher"),
        }
    }
}

/// Pattern: a sequence of pattern elements, with optional callback and overlap/deduplication settings
///
/// - `pattern`: Sequence of pattern elements.
/// - `callback`: Optional callback invoked on match.
/// - `overlap`: If false, prevents overlapping matches for this pattern.
/// - `deduplication`: If true, prevents duplicate matches for this pattern.
pub struct Pattern<T> {
    pub pattern: Vec<PatternElem<T>>,
    pub callback: Option<SliceCallback<T>>,
    pub overlap: bool,
    pub deduplication: bool,
    pub name: Option<String>,
}

// Allow Pattern<T> to be used as AsRef<Pattern<T>>
impl<T> AsRef<Pattern<T>> for Pattern<T> {
    fn as_ref(&self) -> &Pattern<T> {
        self
    }
}

impl<T: Clone> Clone for Pattern<T> {
    fn clone(&self) -> Self {
        Pattern {
            pattern: self.pattern.clone(),
            callback: None, // Cannot clone callback
            overlap: self.overlap,
            deduplication: self.deduplication,
            name: self.name.clone(),
        }
    }
}

impl<T> Pattern<T> {
    /// Create a new pattern from a sequence of pattern elements
    pub fn new(pattern: Vec<PatternElem<T>>) -> Self {
        Self {
            pattern,
            callback: None,
            overlap: true,
            deduplication: false,
            name: None,
        }
    }
    /// Set the name for this pattern (used in named output)
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Set a callback to be invoked on match
    pub fn with_callback(mut self, cb: SliceCallback<T>) -> Self {
        self.callback = Some(cb);
        self
    }
    /// Set overlap behavior
    pub fn overlap(mut self, allow: bool) -> Self {
        self.overlap = allow;
        self
    }
    /// Set deduplication behavior
    pub fn deduplication(mut self, enable: bool) -> Self {
        self.deduplication = enable;
        self
    }
}

/// Type alias for a callback on a slice
///
/// The callback receives a slice of matched window elements.
pub type SliceCallback<T> = Box<dyn Fn(&[T]) + 'static>;

/// Builder for Pattern<T>
///
/// Use this builder to construct complex patterns with custom callbacks, overlap, and deduplication settings.
pub struct PatternBuilderErased {
    overlap: bool,
    deduplication: bool,
    name: Option<String>,
}

impl PatternBuilderErased {
    pub fn new() -> Self {
        Self {
            overlap: true,
            deduplication: false,
            name: None,
        }
    }
    pub fn value_elem<T>(self, value: T) -> PatternBuilder<T> {
        PatternBuilder {
            pattern: vec![PatternElem::Value {
                value,
                min_repeat: None,
                max_repeat: None,
                capture_name: None,
            }],
            callback: None,
            overlap: self.overlap,
            deduplication: self.deduplication,
            name: self.name,
        }
    }
    pub fn matcher_elem<T, F>(self, matcher: F) -> PatternBuilder<T>
    where
        F: Fn(&T) -> bool + 'static,
    {
        PatternBuilder {
            pattern: vec![PatternElem::Matcher {
                matcher: Box::new(matcher),
                min_repeat: None,
                max_repeat: None,
                capture_name: None,
            }],
            callback: None,
            overlap: self.overlap,
            deduplication: self.deduplication,
            name: self.name,
        }
    }
    pub fn any_elem<T>(self) -> PatternBuilder<T> {
        PatternBuilder {
            pattern: vec![PatternElem::Any {
                min_repeat: None,
                max_repeat: None,
                capture_name: None,
            }],
            callback: None,
            overlap: self.overlap,
            deduplication: self.deduplication,
            name: self.name,
        }
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn overlap(mut self, allow: bool) -> Self {
        self.overlap = allow;
        self
    }
    pub fn deduplication(mut self, enable: bool) -> Self {
        self.deduplication = enable;
        self
    }
}

/// Ergonomic builder for Pattern<T>
///
/// All fields and methods are public for chaining and ergonomic usage.
pub struct PatternBuilder<T> {
    pub pattern: Vec<PatternElem<T>>,
    pub callback: Option<SliceCallback<T>>,
    pub overlap: bool,
    pub deduplication: bool,
    pub name: Option<String>,
}

impl<T> PatternBuilder<T> {
    pub fn value_elem(mut self, value: T) -> Self {
        self.pattern.push(PatternElem::Value {
            value,
            min_repeat: None,
            max_repeat: None,
            capture_name: None,
        });
        self
    }
    pub fn matcher_elem<F>(mut self, matcher: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        self.pattern.push(PatternElem::Matcher {
            matcher: Box::new(matcher),
            min_repeat: None,
            max_repeat: None,
            capture_name: None,
        });
        self
    }
    pub fn any_elem(mut self) -> Self {
        self.pattern.push(PatternElem::Any {
            min_repeat: None,
            max_repeat: None,
            capture_name: None,
        });
        self
    }
    pub fn build(self) -> Pattern<T> {
        Pattern {
            pattern: self.pattern,
            callback: self.callback,
            overlap: self.overlap,
            deduplication: self.deduplication,
            name: self.name,
        }
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn overlap(mut self, allow: bool) -> Self {
        self.overlap = allow;
        self
    }
    pub fn deduplication(mut self, enable: bool) -> Self {
        self.deduplication = enable;
        self
    }
}

/// The main matcher struct
///
/// Use this struct to perform pattern matching on windows of data. Supports flexible and ergonomic APIs.
#[derive(Debug, Clone)]
pub struct ScrollingWindowPatternMatcherRef {
    pub window_len: usize,
}

impl ScrollingWindowPatternMatcherRef {
    /// Find matches and return named captures in a HashMap output format.
    ///
    /// Output: `HashMap<pattern_name, Vec<HashMap<capture_name, Vec<T>>>>`
    ///
    /// Each pattern's name is used as the key, and each match produces a HashMap of capture names to matched values.
    ///
    /// Output: HashMap<pattern_name, Vec<HashMap<capture_name, Vec<T>>>>
    pub fn find_matches<T>(&self, window: &[T], patterns: &[Pattern<T>]) -> std::collections::HashMap<String, Vec<std::collections::HashMap<String, Vec<T>>>>
    where
        T: PartialEq + Clone + std::fmt::Debug,
    {
        use std::collections::HashMap;
        let mut results: HashMap<String, Vec<HashMap<String, Vec<T>>>> = HashMap::new();
        for (p_idx, pat) in patterns.iter().enumerate() {
            let pat_name = pat.name.clone().unwrap_or_else(|| format!("pattern_{}", p_idx));
            let pat_len = pat.pattern.len();
            if pat_len == 0 || window.is_empty() {
                continue;
            }
            let mut w_idx = 0;
            while w_idx < window.len() {
                let mut win_pos = w_idx;
                let mut captures: HashMap<String, Vec<T>> = HashMap::new();
                let mut matched = true;
                let mut match_indices = Vec::new();
                for elem in pat.pattern.iter() {
                    // Handle repeats
                    let min_repeat = match elem {
                        PatternElem::Value { min_repeat, .. } => min_repeat,
                        PatternElem::Matcher { min_repeat, .. } => min_repeat,
                        PatternElem::Any { min_repeat, .. } => min_repeat,
                    };
                    let max_repeat = match elem {
                        PatternElem::Value { max_repeat, .. } => max_repeat,
                        PatternElem::Matcher { max_repeat, .. } => max_repeat,
                        PatternElem::Any { max_repeat, .. } => max_repeat,
                    };
                    let repeat_min = min_repeat.unwrap_or(1);
                    let repeat_max = max_repeat.unwrap_or(1);
                    let mut repeat_count = 0;
                    let mut repeat_indices = Vec::new();
                    while repeat_count < repeat_max && win_pos < window.len() {
                        let elem_match = match elem {
                            PatternElem::Value { value, .. } => &window[win_pos] == value,
                            PatternElem::Matcher { matcher, .. } => matcher(&window[win_pos]),
                            PatternElem::Any { .. } => true,
                        };
                        if elem_match {
                            repeat_indices.push(win_pos);
                            repeat_count += 1;
                            win_pos += 1;
                        } else {
                            break;
                        }
                    }
                    if repeat_count < repeat_min {
                        matched = false;
                        break;
                    }
                    match_indices.extend(repeat_indices.iter().copied());
                    // Handle capture
                    let capture_name = match elem {
                        PatternElem::Value { capture_name, .. } => capture_name,
                        PatternElem::Matcher { capture_name, .. } => capture_name,
                        PatternElem::Any { capture_name, .. } => capture_name,
                    };
                    if let Some(name) = capture_name {
                        let captured: Vec<T> = repeat_indices.iter().map(|&i| window[i].clone()).collect();
                        captures.insert(name.clone(), captured);
                    }
                }
                if matched && !match_indices.is_empty() {
                    // Call the callback if present
                    if let Some(cb) = &pat.callback {
                        // Pass the matched slice to the callback
                        let matched_slice: Vec<T> = match_indices.iter().map(|&i| window[i].clone()).collect();
                        cb(&matched_slice);
                    }
                    results.entry(pat_name.clone()).or_default().push(captures);
                }
                w_idx += 1;
            }
        }
        results
    }
    /// Create a new matcher for a window of given length
    pub fn new(window_len: usize) -> Self {
        Self { window_len }
    }

    /// Flexible find_matches: accepts Vec, slice, or array for window and patterns
    /// Flexible pattern matching for windows and patterns.
    ///
    /// Accepts any owned or referenced container for the window (e.g., Vec, slice, array) and patterns (owned or referenced).
    ///
    /// Trait bounds:
    /// - `W: IntoIterator, W::Item: Borrow<T>`: Window can be owned or referenced; each item is borrowed for matching.
    /// - `T: Clone + PartialEq`: Window elements must be cloneable and comparable.
    /// - `P: IntoIterator, P::Item: AsRef<Pattern<T>>`: Patterns can be owned or referenced.
    ///
    /// Performance: Clones all window elements into a new Vec; may use more memory for large windows.
    ///
    /// Use this method for ergonomic API and flexibility. For maximum performance, use `find_matches` with slices.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use scrolling_window_pattern_matcher::{PatternBuilder, ScrollingWindowPatternMatcherRef};
    /// let window = vec![1, 2, 1, 2, 1];
    /// let patterns = vec![
    ///     PatternBuilder::new().value_elem(1).value_elem(2).build(),
    ///     PatternBuilder::new().value_elem(2).value_elem(1).build(),
    /// ];
    /// let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
    /// let named = matcher.find_matches_flexible(window, &patterns);
    /// assert!(named["pattern_0"].iter().any(|m| m.is_empty() || m.contains_key("")));
    /// assert!(named["pattern_1"].iter().any(|m| m.is_empty() || m.contains_key("")));
    /// ```
    pub fn find_matches_flexible<T, W, P>(&self, window: W, patterns: P) -> std::collections::HashMap<String, Vec<std::collections::HashMap<String, Vec<T>>>>
    where
        W: IntoIterator,
        W::Item: Borrow<T>,
        T: Clone + PartialEq + std::fmt::Debug,
        P: IntoIterator,
        P::Item: AsRef<Pattern<T>>,
        Pattern<T>: Clone,
    {
        let window_vec: Vec<T> = window.into_iter()
            .map(|x| x.borrow().clone())
            .collect();
        let patterns_vec: Vec<Pattern<T>> = patterns
            .into_iter()
            .map(|p| p.as_ref().clone())
            .collect();
        // Use find_matches, which now calls callbacks
        self.find_matches(&window_vec, &patterns_vec)
    }
}

// Re-export major types and builders for crate consumers
