# Scrolling Window Pattern Matcher

This crate provides a generic pattern matcher that operates over a scrolling window (queue) of items.
Patterns can be defined as sequences of values, functions, or a mix of both. When a pattern matches,
an optional user-defined callback is invoked. The matcher supports optional deduplication of matches.

## Features

- Match patterns using values, functions, or both
- Optional deduplication of matches
- Support for overlapping matches
- Callback invocation on match
- No unnecessary trait bounds: PartialEq is only required for value-based patterns

## Usage

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
let window = vec![&1, &2, &3, &4];
let patterns = vec![
    PatternElem::Value(1),
    PatternElem::Matcher(Box::new(|x: &i32| *x == 2)),
];
let matcher = ScrollingWindowPatternMatcherRef {
    window: std::collections::VecDeque::new(),
    max_pattern_len: 0,
    next_index: 0,
};
let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
assert!(matches.contains(&(0, 0)));
assert!(matches.contains(&(1, 1)));
```

## Function-only Patterns

```rust
use scrolling_window_pattern_matcher::ScrollingWindowPatternMatcherRef;
let window = vec![&1, &2, &3, &4];
let patterns_fn: Vec<Box<dyn Fn(&i32) -> bool>> = vec![
    Box::new(|x| *x == 1),
    Box::new(|x| *x == 4),
];
let matcher = ScrollingWindowPatternMatcherRef {
    window: std::collections::VecDeque::new(),
    max_pattern_len: 0,
    next_index: 0,
};
let matches = matcher.find_matches_functions_only(&window, &patterns_fn[..], false, None::<fn(usize, usize)>);
assert!(matches.contains(&(0, 0)));
assert!(matches.contains(&(1, 3)));
```

## Deduplication and Overlapping Matches

Set `deduplicate` to `true` to avoid reporting the same match more than once.

## Callback Example

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
let window = vec![&1, &2, &3];
let patterns = vec![PatternElem::Value(2)];
let matcher = ScrollingWindowPatternMatcherRef {
    window: std::collections::VecDeque::new(),
    max_pattern_len: 0,
    next_index: 0,
};
let mut called = false;
let _ = matcher.find_matches(&window, &patterns, false, Some(|pid, idx| {
    assert_eq!(pid, 0);
    assert_eq!(idx, 1);
    called = true;
}));
assert!(called);
```

## Edge Cases

- Empty window or patterns: returns no matches
- Patterns can be all values, all functions, or mixed

## API

- `find_matches`: Use for value or mixed patterns (requires PartialEq for T)
- `find_matches_functions_only`: Use for function-only patterns (no trait bound required)

See the test module for more comprehensive examples and edge cases.
