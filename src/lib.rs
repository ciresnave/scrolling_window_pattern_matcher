//! # Scrolling Window Pattern Matcher
//!
//! This crate provides a generic pattern matcher that operates over a scrolling window (queue) of items.
//! Patterns can be defined as sequences of values, functions, or a mix of both. When a pattern matches,
//! an optional user-defined callback is invoked. The matcher supports optional deduplication of matches and per-pattern overlap settings.
//!
//! ## Features
//! - Match patterns using values, functions, or both
//! - Optional deduplication of matches
//! - Support for overlapping matches (per-pattern control)
//! - Callback invocation on match (per-pattern)
//! - No unnecessary trait bounds: PartialEq is only required for value-based patterns
//!
//! ## Usage: Value/Mixed Patterns with Multiple Patterns
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
//! let window = vec![&1, &2, &1, &2, &1];
//! let patterns = vec![
//!     vec![PatternElem::Value(1), PatternElem::Value(2)],
//!     vec![PatternElem::Value(2), PatternElem::Value(1)],
//!     vec![PatternElem::Value(1)],
//!     vec![PatternElem::Value(2)],
//! ];
//! let matcher = ScrollingWindowPatternMatcherRef::new(5);
//! let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
//! assert!(matches.contains(&(0, 0))); // [1,2] at 0
//! assert!(matches.contains(&(1, 1))); // [2,1] at 1
//! assert!(matches.contains(&(2, 0))); // [1] at 0
//! assert!(matches.contains(&(3, 1))); // [2] at 1
//! ```
//!
//! ## Usage: Patterns with Callbacks and Overlap Settings
//!
//! ```rust
//! use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem, PatternWithCallback};
//! use std::rc::Rc;
//! use std::cell::RefCell;
//! let window = vec![&1, &2, &1, &2, &1];
//! let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
//! let results1 = results.clone();
//! let results2 = results.clone();
//! let patterns = vec![
//!     PatternWithCallback {
//!         pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
//!         callback: Box::new(move |matched| results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
//!         allow_overlap_with_others: false,
//!         allow_others_to_overlap: true,
//!     },
//!     PatternWithCallback {
//!         pattern: vec![PatternElem::Value(2), PatternElem::Value(1)],
//!         callback: Box::new(move |matched| results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
//!         allow_overlap_with_others: true,
//!         allow_others_to_overlap: true,
//!     },
//! ];
//! let matcher = ScrollingWindowPatternMatcherRef::new(5);
//! matcher.find_matches_with_callbacks(&window, &patterns);
//! let results = results.borrow();
//! assert!(results.contains(&vec![1, 2]));
//! assert!(results.contains(&vec![2, 1]));
//! ```
//!
//! ## Edge Cases
//!
//! - Empty window or patterns: returns no matches
//! - Patterns can be all values, all functions, or mixed
//! - Multiple patterns and multi-element patterns supported
//! - Deduplication and overlap settings can be combined
//!
//! ## API
//!
//! - `find_matches`: Use for value or mixed patterns (requires PartialEq for T), supports multiple patterns and multi-element patterns
//! - `find_matches_with_callbacks`: Use for value/mixed patterns with per-pattern callbacks and overlap settings
//! - `find_matches_functions_only`: Use for function-only patterns (no trait bound required)
//! - `find_matches_with_callbacks` (function matcher): Use for function-only patterns with per-pattern callbacks and overlap settings
//!
//! See tests for more comprehensive examples and edge cases.

use std::collections::{VecDeque, HashMap};
use log::debug;

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

    /// Find matches for multiple patterns (each a sequence of PatternElem<T>).
    pub fn find_matches<'b, F>(
        &self,
        window: &'b [&'a T],
        patterns: &[Vec<PatternElem<T>>],
        deduplicate: bool,
        mut on_match: Option<F>,
    ) -> Vec<(PatternId, usize)>
    where
        F: FnMut(PatternId, usize),
    {
        let mut matches = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (pat_id, pattern) in patterns.iter().enumerate() {
            let pat_len = pattern.len();
            if pat_len == 0 || pat_len > window.len() {
                continue;
            }
            for i in 0..=window.len() - pat_len {
                let window_slice = &window[i..i + pat_len];
                let is_match = pattern.iter().zip(window_slice).all(|(pelem, item)| match pelem {
                    PatternElem::Value(val) => val == *item,
                    PatternElem::Matcher(f) => f(item),
                });
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
    pub fn find_matches_with_callbacks(
        &self,
        window: &[&'a T],
        patterns: &[PatternWithCallback<'a, T>],
    ) {
        debug!("Starting find_matches_with_callbacks: window_len={}, patterns_len={}", window.len(), patterns.len());
        let mut matched_regions: Vec<(usize, usize, bool, usize)> = Vec::new(); // (start, len, allow_others_to_overlap, pat_id)
        for (pat_id, pat) in patterns.iter().enumerate() {
            debug!("Checking pattern {}: allow_overlap_with_others={}, allow_others_to_overlap={}, pattern_len={}", pat_id, pat.allow_overlap_with_others, pat.allow_others_to_overlap, pat.pattern.len());
            let pat_len = pat.pattern.len();
            if pat_len == 0 || pat_len > window.len() {
                debug!("Pattern {} skipped: invalid length", pat_id);
                continue;
            }
            let mut i = 0;
            while i <= window.len() - pat_len {
                debug!("Pattern {} at window position {}", pat_id, i);
                // Check overlap with previous matches (across all patterns, including self)
                let mut overlaps = false;
                for &(m_start, m_len, m_allow_others, m_pat_id) in &matched_regions {
                    if regions_overlap(i, pat_len, m_start, m_len) {
                        debug!("Pattern {} at position {} overlaps with previous match (pattern {}, region {}-{})", pat_id, i, m_pat_id, m_start, m_start + m_len);
                        // If either this pattern or the previous matched pattern disallows overlap, skip
                        if !pat.allow_overlap_with_others || !m_allow_others {
                            overlaps = true;
                            break;
                        }
                    }
                }
                if overlaps {
                    debug!("Pattern {} at position {} skipped due to overlap exclusion", pat_id, i);
                    i += 1;
                    continue;
                }
                // Check pattern match
                let window_slice = &window[i..i + pat_len];
                let is_match = pat.pattern.iter().zip(window_slice).all(|(pelem, item)| match pelem {
                    PatternElem::Value(val) => val == *item,
                    PatternElem::Matcher(f) => f(item),
                });
                debug!("Pattern {} at position {} match result: {}", pat_id, i, is_match);
                if is_match {
                    debug!("Pattern {} matched at position {}. Invoking callback.", pat_id, i);
                    (pat.callback)(window_slice);
                    matched_regions.push((i, pat_len, pat.allow_others_to_overlap, pat_id));
                    if !pat.allow_others_to_overlap {
                        i += pat_len;
                        continue;
                    }
                }
                i += 1;
            }
        }
        debug!("Completed find_matches_with_callbacks");
    }
}

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

    /// Find matches using patterns with callbacks and overlap settings (function-only)
    pub fn find_matches_with_callbacks(
        &mut self,
        window: &[&'a T],
        patterns: &[PatternWithCallbackFn<'a, T>],
    ) {
        let mut matched_regions: Vec<(usize, usize, bool)> = Vec::new(); // (start, len, allow_others_to_overlap)
        for (_pat_id, pat) in patterns.iter().enumerate() {
            let pat_len = pat.pattern.len();
            if pat_len == 0 || pat_len > window.len() {
                continue;
            }
            for i in 0..=window.len() - pat_len {
                // Check overlap with previous matches
                let mut overlaps = false;
                for &(m_start, m_len, m_allow_others) in &matched_regions {
                    if regions_overlap(i, pat_len, m_start, m_len) {
                        if !pat.allow_overlap_with_others || !m_allow_others {
                            overlaps = true;
                            break;
                        }
                    }
                }
                if overlaps {
                    continue;
                }
                // Check pattern match
                let window_slice = &window[i..i + pat_len];
                let is_match = pat.pattern.iter().zip(window_slice).all(|(f, item)| f(item));
                if is_match {
                    (pat.callback)(window_slice);
                    matched_regions.push((i, pat_len, pat.allow_others_to_overlap));
                    if !pat.allow_others_to_overlap {
                        for _ in &pat.pattern {
                            self.window.pop_front();
                        }
                    }
                }
            }
        }
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

    /// Find matches for the given patterns, returning a map of index to pattern IDs and references to items.
    pub fn find_matches(&self, patterns: &[Vec<PatternElem<T>>]) -> HashMap<u64, (Vec<PatternId>, &T)>
    where
        T: PartialEq,
    {
        let mut matches: HashMap<u64, (Vec<PatternId>, &T)> = HashMap::new();
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
                    let t = &window_slice[0].1;
                    matches.entry(idx).or_insert_with(|| (vec![], t)).0.push(pat_id);
                }
            }
        }
        matches
    }
}

pub type PatternId = usize;

/// Pattern with callback and overlap settings for value/mixed patterns
pub struct PatternWithCallback<'a, T> {
    pub pattern: Vec<PatternElem<T>>,
    pub callback: Box<dyn Fn(&[&'a T]) + 'a>,
    pub allow_overlap_with_others: bool,
    pub allow_others_to_overlap: bool,
}

/// Pattern with callback and overlap settings for function-only patterns
pub struct PatternWithCallbackFn<'a, T> {
    pub pattern: Vec<Box<dyn Fn(&T) -> bool>>,
    pub callback: Box<dyn Fn(&[&'a T]) + 'a>,
    pub allow_overlap_with_others: bool,
    pub allow_others_to_overlap: bool,
}

fn regions_overlap(a_start: usize, a_len: usize, b_start: usize, b_len: usize) -> bool {
    let a_end = a_start + a_len;
    let b_end = b_start + b_len;
    // Only true overlaps (shared elements) are excluded
    a_start < b_end && b_start < a_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_value_and_function_patterns() {
        init_logger();
        let window = vec![&1, &2, &3, &4];
        let patterns = vec![vec![PatternElem::Value(1)], vec![PatternElem::Matcher(Box::new(|x: &i32| *x == 2))]];
        let matcher = ScrollingWindowPatternMatcherRef::new(4);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert!(matches.contains(&(0, 0)));
        assert!(matches.contains(&(1, 1)));
    }

    #[test]
    fn test_function_only_patterns() {
        init_logger();
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
        init_logger();
        let matcher = ScrollingWindowPatternMatcherRef {
            window: VecDeque::new(),
            max_pattern_len: 0,
            next_index: 0,
        };
        let window = vec![&1, &2, &1, &2, &1];
        let patterns = vec![
            vec![PatternElem::Value(1)],
            vec![PatternElem::Matcher(Box::new(|x: &i32| *x == 2))],
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
        init_logger();
        let matcher = ScrollingWindowPatternMatcherRef {
            window: VecDeque::new(),
            max_pattern_len: 0,
            next_index: 0,
        };
        let window = vec![&1, &1, &1];
        let patterns = vec![vec![PatternElem::Value(1)]];
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert_eq!(matches, vec![(0, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn test_pattern_with_callback() {
        init_logger();
        let window = vec![&1, &2, &3, &4, &5];
        let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
        let results1 = results.clone();
        let results2 = results.clone();
        let patterns = vec![
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
                callback: Box::new(move |matched| {
                    results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: false,
                allow_others_to_overlap: true,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(3), PatternElem::Value(4)],
                callback: Box::new(move |matched| {
                    results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: true,
                allow_others_to_overlap: false,
            },
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(5);
        matcher.find_matches_with_callbacks(&window, &patterns);
        let results = results.borrow();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&vec![1, 2]));
        assert!(results.contains(&vec![3, 4]));
    }

    #[test]
    fn test_function_pattern_with_callback() {
        init_logger();
        let window = vec![&1, &2, &3, &4, &5];
        let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
        let results1 = results.clone();
        let results2 = results.clone();
        let patterns = vec![
            PatternWithCallbackFn {
                pattern: vec![
                    Box::new(|x: &i32| *x == 1) as Box<dyn Fn(&i32) -> bool>,
                    Box::new(|x: &i32| *x == 2) as Box<dyn Fn(&i32) -> bool>,
                ],
                callback: Box::new(move |matched| results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
                allow_overlap_with_others: false,
                allow_others_to_overlap: true,
            },
            PatternWithCallbackFn {
                pattern: vec![
                    Box::new(|x: &i32| *x == 2) as Box<dyn Fn(&i32) -> bool>,
                    Box::new(|x: &i32| *x == 3) as Box<dyn Fn(&i32) -> bool>,
                ],
                callback: Box::new(move |matched| results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
                allow_overlap_with_others: true,
                allow_others_to_overlap: false,
            },
        ];
        let mut matcher = ScrollingWindowFunctionPatternMatcherRef::new(4);
        matcher.find_matches_with_callbacks(&window, &patterns);
        let results = results.borrow();
        assert!(results.contains(&vec![1, 2]));
        assert!(results.contains(&vec![2, 3]));
    }

    #[test]
    fn test_empty_window() {
        init_logger();
        let window: Vec<&i32> = vec![];
        let patterns = vec![
            vec![PatternElem::Value(1), PatternElem::Value(2)],
            vec![PatternElem::Matcher(Box::new(|x: &i32| *x == 3))],
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(2);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_empty_patterns() {
        init_logger();
        let window = vec![&1, &2, &3, &4];
        let patterns: Vec<Vec<PatternElem<i32>>> = vec![];
        let matcher = ScrollingWindowPatternMatcherRef::new(4);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_longer_than_window() {
        init_logger();
        let window = vec![&1, &2];
        let patterns = vec![
            vec![PatternElem::Value(1), PatternElem::Value(2), PatternElem::Value(3)],
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(2);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_deduplication_with_callbacks() {
        init_logger();
        let window = vec![&1, &2, &1, &2, &1];
        let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
        let results1 = results.clone();
        let results2 = results.clone();
        let patterns = vec![
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
                callback: Box::new(move |matched| {
                    results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: false,
                allow_others_to_overlap: true,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(2), PatternElem::Value(1)],
                callback: Box::new(move |matched| {
                    results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: true,
                allow_others_to_overlap: true,
            },
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(5);
        matcher.find_matches_with_callbacks(&window, &patterns);
        let results = results.borrow();
        assert_eq!(results.len(), 4); // Only 4 matches due to overlap exclusion
        assert!(results.contains(&vec![1, 2]));
        assert!(results.contains(&vec![2, 1]));
        // The other two matches are [1,2] and [2,1] at later positions
    }

    #[test]
    fn test_overlap_settings() {
        init_logger();
        let window = vec![&1, &2, &1, &2, &1];
        let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
        let results1 = results.clone();
        let results2 = results.clone();
        let results3 = results.clone();
        let patterns = vec![
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
                callback: Box::new(move |matched| {
                    results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: false,
                allow_others_to_overlap: false,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(2), PatternElem::Value(1)],
                callback: Box::new(move |matched| {
                    results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: false,
                allow_others_to_overlap: false,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
                callback: Box::new(move |matched| {
                    results3.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>());
                }),
                allow_overlap_with_others: true,
                allow_others_to_overlap: true,
            },
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(5);
        matcher.find_matches_with_callbacks(&window, &patterns);
        let results = results.borrow();
        assert_eq!(results.len(), 2); // Only 2 matches due to overlap exclusion
        assert!(results.contains(&vec![1, 2]));
        // [2, 1] is not present due to overlap exclusion
    }

    #[test]
    fn test_multi_pattern_combinations() {
        init_logger();
        let window = vec![&1, &2, &1, &2, &1, &2];
        let patterns = vec![
            vec![PatternElem::Value(1), PatternElem::Value(2)],
            vec![PatternElem::Value(2), PatternElem::Value(1)],
            vec![PatternElem::Value(1)],
            vec![PatternElem::Value(2)],
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(6);
        let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
        let p0 = matches.iter().filter(|(pid, _)| *pid == 0).count();
        let p1 = matches.iter().filter(|(pid, _)| *pid == 1).count();
        let p2 = matches.iter().filter(|(pid, _)| *pid == 2).count();
        let p3 = matches.iter().filter(|(pid, _)| *pid == 3).count();
        assert_eq!(p0, 3);
        assert_eq!(p1, 2);
        assert_eq!(p2, 3);
        assert_eq!(p3, 3);
        let matches = matcher.find_matches(&window, &patterns, true, None::<fn(usize, usize)>);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 0).count(), 3);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 1).count(), 2);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 2).count(), 3);
        assert_eq!(matches.iter().filter(|(pid, _)| *pid == 3).count(), 3);
    }

    #[test]
    fn test_multi_pattern_with_callbacks_and_overlap() {
        init_logger();
        let window = vec![&1, &2, &1, &2, &1, &2];
        let results: Rc<RefCell<Vec<(usize, Vec<i32>)>>> = Rc::new(RefCell::new(vec![]));
        let results1 = results.clone();
        let results2 = results.clone();
        let results3 = results.clone();
        let results4 = results.clone();
        let patterns = vec![
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
                callback: Box::new(move |matched| results1.borrow_mut().push((0, matched.iter().map(|x| **x).collect()))),
                allow_overlap_with_others: false,
                allow_others_to_overlap: true,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(2), PatternElem::Value(1)],
                callback: Box::new(move |matched| results2.borrow_mut().push((1, matched.iter().map(|x| **x).collect()))),
                allow_overlap_with_others: true,
                allow_others_to_overlap: false,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(1)],
                callback: Box::new(move |matched| results3.borrow_mut().push((2, matched.iter().map(|x| **x).collect()))),
                allow_overlap_with_others: true,
                allow_others_to_overlap: true,
            },
            PatternWithCallback {
                pattern: vec![PatternElem::Value(2)],
                callback: Box::new(move |matched| results4.borrow_mut().push((3, matched.iter().map(|x| **x).collect()))),
                allow_overlap_with_others: true,
                allow_others_to_overlap: true,
            },
        ];
        let matcher = ScrollingWindowPatternMatcherRef::new(6);
        matcher.find_matches_with_callbacks(&window, &patterns);
        let results = results.borrow();
        let p0 = results.iter().filter(|(pid, _)| *pid == 0).count();
        let p1 = results.iter().filter(|(pid, _)| *pid == 1).count();
        let p2 = results.iter().filter(|(pid, _)| *pid == 2).count();
        let p3 = results.iter().filter(|(pid, _)| *pid == 3).count();
        assert!(p0 >= 1);
        assert!(p1 >= 1);
        assert!(p2 >= 1);
        assert!(p3 >= 1);
    }
}
