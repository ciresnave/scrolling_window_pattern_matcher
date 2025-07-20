//! # Scrolling Window Pattern Matcher
//!
//! This crate provides a generic pattern matcher that operates over a scrolling window (queue) of items.
//! Patterns can be defined as sequences of values, functions, or a mix of both. When a pattern matches,
//! an optional user-defined callback is invoked. The matcher supports optional deduplication of matches.
//!
//! ## Features
//! - Match patterns using values, functions, or both
//! - Optional deduplication of matches
//! - Support for overlapping matches
//! - Callback invocation on match
//! - No unnecessary trait bounds: PartialEq is only required for value-based patterns
//!
//! ## Usage
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
//! let window = vec![&1, &2, &3, &4];
//! let patterns = vec![
//!     PatternElem::Value(1),
//!     PatternElem::Matcher(Box::new(|x: &i32| *x == 2)),
//! ];
//! let matcher = ScrollingWindowPatternMatcherRef::new(4);
//! let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
//! assert!(matches.contains(&(0, 0)));
//! assert!(matches.contains(&(1, 1)));
//! ```
//!
//! ## Function-only Patterns
//!
//! ```rust
//! use scrolling_window_pattern_matcher::ScrollingWindowFunctionPatternMatcherRef;
//! let window = vec![&1, &2, &3, &4];
//! let patterns_fn: Vec<Box<dyn Fn(&i32) -> bool>> = vec![
//!     Box::new(|x| *x == 1),
//!     Box::new(|x| *x == 4),
//! ];
//! let matcher = ScrollingWindowFunctionPatternMatcherRef::new(4);
//! let matches = matcher.find_matches(&window, &patterns_fn[..], false, None::<fn(usize, usize)>);
//! assert!(matches.contains(&(0, 0)));
//! assert!(matches.contains(&(1, 3)));
//! ```
//!
//! ## Deduplication and Overlapping Matches
//!
//! Set `deduplicate` to `true` to avoid reporting the same match more than once.
//!
//! ## Callback Example
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
//! let window = vec![&1, &2, &3];
//! let patterns = vec![PatternElem::Value(2)];
//! let matcher = ScrollingWindowPatternMatcherRef::new(3);
//! let mut called = false;
//! let _ = matcher.find_matches(&window, &patterns, false, Some(|pid, idx| {
//!     assert_eq!(pid, 0);
//!     assert_eq!(idx, 1);
//!     called = true;
//! }));
//! assert!(called);
//! ```
//!
//! ## Edge Cases
//!
//! - Empty window or patterns: returns no matches
//! - Patterns can be all values, all functions, or mixed
//!
//! ## API
//!
//! - `find_matches`: Use for value or mixed patterns (requires PartialEq for T)
//! - `find_matches_functions_only`: Use for function-only patterns (no trait bound required)
//!
//! See tests for more examples.

use std::collections::{VecDeque, HashMap};

/// A single element in a pattern: either an exact value or a matcher function.
pub enum PatternElem<T> {
    Value(T),
    Matcher(Box<dyn Fn(&T) -> bool>),
}

impl<T: Clone> Clone for PatternElem<T> {
    fn clone(&self) -> Self {
        match self {
            PatternElem::Value(val) => PatternElem::Value(val.clone()),
            PatternElem::Matcher(_) => panic!("Cannot clone matcher function. Use Arc if you need to clone matchers."),
        }
    }
}

impl<T> PatternElem<T> {
    pub fn matches(&self, item: &T) -> bool {
        match self {
            PatternElem::Value(_) => panic!("Direct value matching requires T: PartialEq. Use matches_value instead."),
            PatternElem::Matcher(f) => f(item),
        }
    }

    pub fn matches_value(val: &T, item: &T) -> bool
    where
        T: PartialEq,
    {
        val == item
    }
}

/// A generic pattern matcher that scrolls through a queue of items of type `T`.
pub struct ScrollingWindowPatternMatcherRef<'a, T> {
    window: VecDeque<(u64, &'a T)>,
    max_pattern_len: usize,
    next_index: u64,
}

impl<'a, T: PartialEq> ScrollingWindowPatternMatcherRef<'a, T> {
    /// Create a new matcher with a maximum pattern length (window size).
    pub fn new(max_pattern_len: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_pattern_len),
            max_pattern_len,
            next_index: 0,
        }
    }

    /// Push a new item into the window, maintaining the max size.
    pub fn push(&mut self, item: &'a T) {
        if self.window.len() == self.max_pattern_len {
            self.window.pop_front();
        }
        self.window.push_back((self.next_index, item));
        self.next_index += 1;
    }

    /// Find matches using value or function patterns (requires PartialEq for value comparison)
    pub fn find_matches<'b, F>(
        &self,
        window: &'b [&'a T],
        patterns: &[PatternElem<T>],
        deduplicate: bool,
        mut on_match: Option<F>,
    ) -> Vec<(PatternId, usize)>
    where
        F: FnMut(PatternId, usize),
    {
        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (pat_id, pattern) in patterns.iter().enumerate() {
            let pat_len = 1; // Each PatternElem is a single element
            if pat_len == 0 || pat_len > window.len() {
                continue;
            }
            for i in 0..=window.len() - pat_len {
                let item = window[i];
                let is_match = match pattern {
                    PatternElem::Value(val) => val == item,
                    PatternElem::Matcher(f) => f(item),
                };
                if is_match {
                    if !deduplicate || seen.insert((pat_id, i)) {
                        if let Some(ref mut cb) = on_match {
                            cb(pat_id, i);
                        }
                        matches.push((pat_id, i));
                    }
                }
            }
        }
        matches
    }
}

/// A function-only pattern matcher that scrolls through a queue of items of type `T`.
pub struct ScrollingWindowFunctionPatternMatcherRef<'a, T> {
    window: VecDeque<(u64, &'a T)>,
    max_pattern_len: usize,
    next_index: u64,
}

impl<'a, T> ScrollingWindowFunctionPatternMatcherRef<'a, T> {
    /// Create a new matcher for function-only patterns (no PartialEq bound required).
    pub fn new(max_pattern_len: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_pattern_len),
            max_pattern_len,
            next_index: 0,
        }
    }

    /// Push a new item into the window, maintaining the max size.
    pub fn push(&mut self, item: &'a T) {
        if self.window.len() == self.max_pattern_len {
            self.window.pop_front();
        }
        self.window.push_back((self.next_index, item));
        self.next_index += 1;
    }

    /// Find matches using only function patterns (no PartialEq bound required)
    pub fn find_matches<'b, F>(
        &self,
        window: &'b [&'a T],
        patterns: &[Box<dyn Fn(&T) -> bool>],
        deduplicate: bool,
        mut on_match: Option<F>,
    ) -> Vec<(usize, usize)>
    where
        F: FnMut(usize, usize),
    {
        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (pat_id, pattern) in patterns.iter().enumerate() {
            let pat_len = 1;
            if pat_len == 0 || pat_len > window.len() {
                continue;
            }
            for i in 0..=window.len() - pat_len {
                let item = window[i];
                if pattern(item) {
                    if !deduplicate || seen.insert((pat_id, i)) {
                        if let Some(ref mut cb) = on_match {
                            cb(pat_id, i);
                        }
                        matches.push((pat_id, i));
                    }
                }
            }
        }
        matches
    }
}

pub struct ScrollingWindowPatternMatcherOwned<T> {
    window: VecDeque<(u64, T)>,
    max_pattern_len: usize,
    next_index: u64,
}

impl<T: Clone> ScrollingWindowPatternMatcherOwned<T> {
    /// Create a new matcher with a maximum pattern length (window size).
    pub fn new(max_pattern_len: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_pattern_len),
            max_pattern_len,
            next_index: 0,
        }
    }

    /// Push a new item into the window, maintaining the max size.
    pub fn push(&mut self, item: T) {
        if self.window.len() == self.max_pattern_len {
            self.window.pop_front();
        }
        self.window.push_back((self.next_index, item));
        self.next_index += 1;
    }

    /// Find matches for the given patterns, returning a map of index to pattern IDs and owned items.
    pub fn find_matches(&self, patterns: &[Vec<PatternElem<T>>]) -> HashMap<u64, (Vec<PatternId>, T)>
    where
        T: PartialEq,
    {
        let mut matches: HashMap<u64, (Vec<PatternId>, T)> = HashMap::new();
        let min_pat_len = patterns.iter().map(|p| p.len()).min().unwrap_or(0);
        if self.window.len() < min_pat_len {
            return matches;
        }
        for (pat_id, pattern) in patterns.iter().enumerate() {
            let pat_len = pattern.len();
            if pat_len == 0 || pat_len > self.window.len() {
                continue;
            }
            for i in 0..=self.window.len() - pat_len {
                let window_slice = &self.window.as_slices().0[i..i+pat_len];
                if window_slice.iter().zip(pattern.iter()).all(|((_, item), pelem)| pelem.matches(item)) {
                    let idx = window_slice[0].0;
                    let t = window_slice[0].1.clone();
                    matches.entry(idx).or_insert_with(|| (vec![], t)).0.push(pat_id);
                }
            }
        }
        matches
    }
}

pub type PatternId = usize;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn test_value_and_function_patterns() {
        let window = vec![&1, &2, &3, &4];
        let patterns = vec![
            PatternElem::Value(1),
            PatternElem::Matcher(Box::new(|x: &i32| *x == 2)),
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(4);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert!(matches.contains(&(0, 0)));
        assert!(matches.contains(&(1, 1)));
    }

    #[test]
    fn test_function_only_patterns() {
        let window = vec![&1, &2, &3, &4];
        let patterns_fn: Vec<Box<dyn Fn(&i32) -> bool>> = vec![
            Box::new(|x| *x == 1),
            Box::new(|x| *x == 4),
        ];
        let matcher = ScrollingWindowFunctionPatternMatcherRef::new(4);
        let matches = matcher.find_matches(&window, &patterns_fn[..], false, None::<fn(usize, usize)>);
        assert!(matches.contains(&(0, 0)));
        assert!(matches.contains(&(1, 3)));
    }

    #[test]
    fn test_mixed_patterns_and_deduplication() {
        let matcher = ScrollingWindowPatternMatcherRef {
            window: VecDeque::new(),
            max_pattern_len: 0,
            next_index: 0,
        };
        let window = vec![&1, &2, &1, &2, &1];
        let patterns = vec![
            PatternElem::Value(1),
            PatternElem::Matcher(Box::new(|x: &i32| *x == 2)),
        ];
        // No deduplication
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 0).count(), 3);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 1).count(), 2);
        // With deduplication
        let matches = matcher.find_matches(&window, &patterns, true, None::<fn(usize, usize)>);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 0).count(), 3);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 1).count(), 2);
    }

    #[test]
    fn test_overlapping_matches() {
        let matcher = ScrollingWindowPatternMatcherRef {
            window: VecDeque::new(),
            max_pattern_len: 0,
            next_index: 0,
        };
        let window = vec![&1, &1, &1];
        let patterns = vec![PatternElem::Value(1)];
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert_eq!(matches, vec![(0, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn test_function_only_edge_cases() {
        let matcher = ScrollingWindowFunctionPatternMatcherRef {
            window: VecDeque::new(),
            max_pattern_len: 0,
            next_index: 0,
        };
        let window: Vec<&i32> = vec![];
        let patterns_fn: Vec<Box<dyn Fn(&i32) -> bool>> = vec![Box::new(|_| true)];
        let matches = matcher.find_matches(&window, &patterns_fn[..], false, None::<fn(usize, usize)>);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_callback_invocation() {
        let window = vec![&1, &2, &3];
        let patterns = vec![PatternElem::Value(2)];
        let matcher = ScrollingWindowPatternMatcherRef::new(3);
        let mut called = false;
        let _ = matcher.find_matches(&window, &patterns, false, Some(|pid, idx| {
            assert_eq!(pid, 0);
            assert_eq!(idx, 1);
            called = true;
        }));
        assert!(called);
    }
}
