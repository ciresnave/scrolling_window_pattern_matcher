# Scrolling Window Pattern Matcher

This crate provides a generic pattern matcher that operates over a scrolling window (queue) of items.
Patterns can be defined as sequences of values, functions, or a mix of both. When a pattern matches,
an optional user-defined callback is invoked. The matcher supports optional deduplication of matches and per-pattern overlap settings.

## Features

- Match patterns using values, functions, or both
- Optional deduplication of matches
- Support for overlapping matches (per-pattern control)
- Callback invocation on match (per-pattern)
- No unnecessary trait bounds: PartialEq is only required for value-based patterns

## Usage: Value/Mixed Patterns with Multiple Patterns

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
let window = vec![&1, &2, &1, &2, &1];
let patterns = vec![
    vec![PatternElem::Value(1), PatternElem::Value(2)],
    vec![PatternElem::Value(2), PatternElem::Value(1)],
    vec![PatternElem::Value(1)],
    vec![PatternElem::Value(2)],
];
let matcher = ScrollingWindowPatternMatcherRef::new(5);
let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
assert!(matches.contains(&(0, 0))); // [1,2] at 0
assert!(matches.contains(&(1, 1))); // [2,1] at 1
assert!(matches.contains(&(2, 0))); // [1] at 0
assert!(matches.contains(&(3, 1))); // [2] at 1
```

## Usage: Patterns with Callbacks and Overlap Settings

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem, PatternWithCallback};
use std::rc::Rc;
use std::cell::RefCell;
let window = vec![&1, &2, &1, &2, &1];
let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
let results1 = results.clone();
let results2 = results.clone();
let patterns = vec![
    PatternWithCallback {
        pattern: vec![PatternElem::Value(1), PatternElem::Value(2)],
        callback: Box::new(move |matched| results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
        allow_overlap_with_others: false,
        allow_others_to_overlap: true,
    },
    PatternWithCallback {
        pattern: vec![PatternElem::Value(2), PatternElem::Value(1)],
        callback: Box::new(move |matched| results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
        allow_overlap_with_others: true,
        allow_others_to_overlap: true,
    },
];
let matcher = ScrollingWindowPatternMatcherRef::new(5);
matcher.find_matches_with_callbacks(&window, &patterns);
let results = results.borrow();
assert!(results.contains(&vec![1, 2]));
assert!(results.contains(&vec![2, 1]));
```

## Overlap Settings

- `allow_overlap_with_others`: If false, this pattern will not match if it would overlap with any previous match.
- `allow_others_to_overlap`: If false, once this pattern matches, no future matches can overlap its matched region.

## Debug Logging

This crate uses the [log](https://docs.rs/log/) crate for debug logging. To enable debug output during development, add the following to your main function or test harness:

```rust
env_logger::init();
```

Then run your program or tests with:

```
RUST_LOG=debug cargo test
```

To disable logging (e.g., in production), do not initialize a logger, or set a higher log level:

```
RUST_LOG=info cargo run
```

Debug logs provide detailed information about matcher execution, pattern matching, overlap checks, and callback invocations.

## API

- `ScrollingWindowPatternMatcherRef::find_matches`: Value/mixed patterns with multiple patterns and multi-element support.
- `ScrollingWindowPatternMatcherRef::find_matches_with_callbacks`: Value/mixed patterns with per-pattern callbacks and overlap settings.
- `ScrollingWindowFunctionPatternMatcherRef::find_matches`: Function-only patterns with multiple patterns.
- `ScrollingWindowFunctionPatternMatcherRef::find_matches_with_callbacks`: Function-only patterns with per-pattern callbacks and overlap settings.

See the test module for more comprehensive examples and edge cases.
