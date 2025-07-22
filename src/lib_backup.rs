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
    /// Matches a specific value
    Value(T),
    /// Matches using a custom predicate
    Matcher(Box<dyn Fn(&T) -> bool + 'static>),
    /// Matches any value (wildcard)
    Any,
}

impl<T: fmt::Debug> fmt::Debug for PatternElem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternElem::Value(v) => write!(f, "Value({:?})", v),
            PatternElem::Matcher(_) => write!(f, "Matcher(..)"),
            PatternElem::Any => write!(f, "Any"),
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
}

// Allow Pattern<T> to be used as AsRef<Pattern<T>>
impl<T> AsRef<Pattern<T>> for Pattern<T> {
    fn as_ref(&self) -> &Pattern<T> {
        self
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
        }
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

/// Builder for Pattern<T>
///
/// Use this builder to construct complex patterns with custom callbacks, overlap, and deduplication settings.
pub struct PatternBuilder<T> {
    pattern: Vec<PatternElem<T>>,
    callback: Option<SliceCallback<T>>,
    overlap: bool,
    deduplication: bool,
}

impl<T> PatternBuilder<T> {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            pattern: Vec::new(),
            callback: None,
            overlap: true,
            deduplication: false,
        }
    }
    /// Set the pattern elements
    pub fn pattern(mut self, elems: Vec<PatternElem<T>>) -> Self {
        self.pattern = elems;
        self
    }
    /// Set the callback
    pub fn callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(&[T]) + 'static,
    {
        self.callback = Some(Box::new(cb));
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
    /// Build the pattern
    pub fn build(self) -> Pattern<T> {
        Pattern {
            pattern: self.pattern,
            callback: self.callback,
            overlap: self.overlap,
            deduplication: self.deduplication,
        }
    }
}

/// Type alias for a callback on a slice
///
/// The callback receives a slice of matched window elements.
pub type SliceCallback<T> = Box<dyn Fn(&[T]) + 'static>;

/// The main matcher struct
///
/// Use this struct to perform pattern matching on windows of data. Supports flexible and ergonomic APIs.
#[derive(Debug, Clone)]
pub struct ScrollingWindowPatternMatcherRef {
    pub window_len: usize,
}

impl ScrollingWindowPatternMatcherRef {
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
    /// use scrolling_window_pattern_matcher::{Pattern, PatternElem, ScrollingWindowPatternMatcherRef};
    /// let window = vec![1, 2, 1, 2, 1];
    /// let patterns = vec![
    ///     Pattern::new(vec![PatternElem::Value(1), PatternElem::Value(2)]),
    ///     Pattern::new(vec![PatternElem::Value(2), PatternElem::Value(1)]),
    /// ];
    /// let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
    /// let matches = matcher.find_matches_flexible(window, &patterns);
    /// assert!(matches.contains(&(0, 0)));
    /// assert!(matches.contains(&(1, 1)));
    /// ```
    ///
    /// You can also pass patterns as references:
    ///
    /// ```rust
    /// use scrolling_window_pattern_matcher::{Pattern, PatternElem, ScrollingWindowPatternMatcherRef};
    /// let window = [1, 2, 1, 2, 1];
    /// let p1 = Pattern::new(vec![PatternElem::Value(1), PatternElem::Value(2)]);
    /// let p2 = Pattern::new(vec![PatternElem::Value(2), PatternElem::Value(1)]);
    /// let patterns = [&p1, &p2];
    /// let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
    /// let matches = matcher.find_matches_flexible(window, &patterns);
    /// assert!(matches.contains(&(0, 0)));
    /// assert!(matches.contains(&(1, 1)));
    /// ```
    pub fn find_matches_flexible<T, W, P>(&self, window: W, patterns: P) -> Vec<(usize, usize)>
    where
        W: IntoIterator,
        W::Item: Borrow<T>,
        T: Clone + PartialEq,
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
        self.find_matches(&window_vec, &patterns_vec)
    }

    /// Find matches for a slice and a slice of patterns
    ///
    /// Returns a vector of (window index, pattern index) for all matches.
    ///
    /// Accepts slices for both window and patterns. No cloning is performed; matching is zero-copy and efficient.
    ///
    /// Trait bounds:
    /// - `T: PartialEq + Clone`: Window elements must be comparable and cloneable.
    ///
    /// Use this method for maximum performance and minimal memory usage.
    pub fn find_matches<T>(&self, window: &[T], patterns: &[Pattern<T>]) -> Vec<(usize, usize)>
    where
        T: PartialEq + Clone,
    {
        let mut matches = Vec::new();
        for (p_idx, pat) in patterns.iter().enumerate() {
            let pat_len = pat.pattern.len();
            if pat_len == 0 || window.len() < pat_len {
                continue;
            }
            let mut matched_ranges: Vec<(usize, usize)> = Vec::new();
            let mut w_idx = 0;
            while w_idx <= window.len() - pat_len {
                let candidate = &window[w_idx..w_idx + pat_len];
                let mut matched = true;
                for (offset, elem) in pat.pattern.iter().enumerate() {
                    match elem {
                        PatternElem::Value(v) => {
                            if &candidate[offset] != v {
                                matched = false;
                                break;
                            }
                        }
                        PatternElem::Matcher(f) => {
                            if !f(&candidate[offset]) {
                                matched = false;
                                break;
                            }
                        }
                        PatternElem::Any => {
                            // Always matches
                        }
                    }
                }
                if matched {
                    // Overlap checks per-pattern
                    let mut overlaps = false;
                    if !pat.overlap {
                        for &(start, end) in &matched_ranges {
                            let range_start = w_idx;
                            let range_end = w_idx + pat_len - 1;
                            if !(range_end < start || range_start > end) {
                                overlaps = true;
                                break;
                            }
                        }
                    }
                    if pat.overlap || !overlaps {
                        // Invoke callback if present
                        if let Some(ref cb) = pat.callback {
                            cb(candidate);
                        }
                        matches.push((w_idx, p_idx));
                        if pat.deduplication {
                            matched_ranges.push((w_idx, w_idx + pat_len - 1));
                        }
                    }
                }
                w_idx += 1;
            }
        }
        matches
    }
}

// Re-export major types and builders for crate consumers

// Manual Clone for PatternElem, skipping Matcher
/// Manual implementation of Clone for PatternElem.
///
/// - `Value(v)`: Clones the value.
/// - `Any`: Returns Any.
/// - `Matcher`: Panics (cannot clone closures).
impl<T: Clone> Clone for PatternElem<T> {
    fn clone(&self) -> Self {
        match self {
            PatternElem::Value(v) => PatternElem::Value(v.clone()),
            PatternElem::Any => PatternElem::Any,
            PatternElem::Matcher(_) => panic!("Cannot clone PatternElem::Matcher"),
        }
    }
}

// Manual Clone for Pattern, skipping callback
/// Manual implementation of Clone for Pattern.
///
/// - Clones pattern elements, overlap, and deduplication settings.
/// - Callback is not cloned (set to None).
impl<T: Clone> Clone for Pattern<T> {
    fn clone(&self) -> Self {
        Pattern {
            pattern: self.pattern.clone(),
            callback: None, // Cannot clone callback
            overlap: self.overlap,
            deduplication: self.deduplication,
        }
    }
}
