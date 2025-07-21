# Builder Pattern Usage

This crate supports ergonomic builder patterns for both `PatternElem` and `Pattern`:

```rust
let elem = PatternElemBuilder::new().value(42).build();
let pat = PatternBuilder::new()
    .pattern(vec![PatternElem::Value(42)])
    .deduplication(true)
    .overlap(false)
    .build();

- Mixed patterns: You can combine value and function elements in a single pattern.
- Empty patterns or windows: Returns no matches.
- Overlap and deduplication: Fine-grained control per pattern.
- Callback invocation: Use closures to collect or process matches.

- Builder pattern: Use for complex pattern construction and configuration.

# Contributing

We welcome contributions! Here’s how to get started:

## Project Structure
- `src/lib.rs`: Main matcher API and types
- `src/lib_tests.rs`: Comprehensive unit tests
- `README.md`: User and contributor documentation

## Running and Writing Tests
- Run all tests: `cargo test`
- Add new tests in `src/lib_tests.rs` using the existing patterns as examples
- Doc tests: Add examples to doc comments for public methods

## Coding Style
- Follow Rust’s standard formatting (`cargo fmt`)
- Use clear, descriptive names and thorough doc comments
- Prefer builder patterns for complex configuration

## Submitting Issues and Pull Requests
- Open issues for bugs, feature requests, or documentation improvements
- Fork the repo, create a feature branch, and submit a pull request
- Include tests and documentation for new features

## Adding Features or Improving Docs
- Add new matcher types or pattern features in `src/lib.rs`
- Expand doc comments and README with examples and edge cases
- Add tests for all new functionality

## Benchmarks and Debug Logging
- Use the [log](https://docs.rs/log/) crate for debug output
- Enable logging: `env_logger::init();` in your main/test harness
- Run with debug logs: `RUST_LOG=debug cargo test`

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem, Pattern, PatternBuilder};
let window = vec![1, 2, 1, 2, 1];
let patterns = vec![
    Pattern::new(vec![PatternElem::Value(1), PatternElem::Value(2)]),
    Pattern::new(vec![PatternElem::Value(2), PatternElem::Value(1)]),
    Pattern::new(vec![PatternElem::Value(1)]),
    Pattern::new(vec![PatternElem::Value(2)]),
];
let matcher = ScrollingWindowPatternMatcherRef::new(5);
let matches = matcher.find_matches(&window, &patterns);
assert!(matches.contains(&(0, 0))); // [1,2] at 0
assert!(matches.contains(&(1, 1))); // [2,1] at 1
assert!(matches.contains(&(2, 2))); // [1] at 2
assert!(matches.contains(&(3, 3))); // [2] at 3
```

- No unnecessary trait bounds: PartialEq is only required for value-based patterns
- Accepts Vec, slice, or array for both window and patterns (no manual conversion needed)

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
// You can pass Vec, slice, or array for window and patterns:
let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
// Or:
let matches = matcher.find_matches(window.as_slice(), patterns.as_slice(), false, None::<fn(usize, usize)>);
// Or with arrays:
let arr_window = [&1, &2, &1, &2, &1];
let arr_patterns = [
    vec![PatternElem::Value(1), PatternElem::Value(2)],
    vec![PatternElem::Value(2), PatternElem::Value(1)],
```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem, Pattern, PatternBuilder};
use std::rc::Rc;
use std::cell::RefCell;
let window = vec![1, 2, 1, 2, 1];
let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
let results1 = results.clone();
let results2 = results.clone();
let patterns = vec![
    PatternBuilder::new()
        .pattern(vec![PatternElem::Value(1), PatternElem::Value(2)])
        .callback(move |matched| results1.borrow_mut().push(matched.to_vec()))
        .overlap(false)
        .build(),
    PatternBuilder::new()
        .pattern(vec![PatternElem::Value(2), PatternElem::Value(1)])
        .callback(move |matched| results2.borrow_mut().push(matched.to_vec()))
        .overlap(true)
        .build(),
];
let matcher = ScrollingWindowPatternMatcherRef::new(5);
matcher.find_matches(&window, &patterns);
let results = results.borrow();
assert!(results.contains(&vec![1, 2]));
assert!(results.contains(&vec![2, 1]));
```

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

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowFunctionPatternMatcherRef, PatternWithCallbackFn};
use std::rc::Rc;
use std::cell::RefCell;
let window = vec![&1, &2, &3, &4, &5];
let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
let results1 = results.clone();
let results2 = results.clone();
let patterns = vec![
    PatternWithCallbackFn {
        pattern: vec![Box::new(|x: &i32| *x == 1), Box::new(|x: &i32| *x == 2)],
        callback: Box::new(move |matched| results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
        allow_overlap_with_others: false,
        allow_others_to_overlap: true,
    },
    PatternWithCallbackFn {
        pattern: vec![Box::new(|x: &i32| *x == 2), Box::new(|x: &i32| *x == 3)],
        callback: Box::new(move |matched| results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
        allow_overlap_with_others: true,
        allow_others_to_overlap: false,
    },
];
let matcher = ScrollingWindowFunctionPatternMatcherRef::new(4);
- `find_matches`: Use for value, mixed, or callback patterns (requires PartialEq for T), supports multiple patterns and multi-element patterns. Accepts any type convertible to a slice for window and patterns. Automatically invokes callbacks and respects overlap/deduplication settings.


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

- `find_matches`: Use for value or mixed patterns (requires PartialEq for T), supports multiple patterns and multi-element patterns. Accepts any type convertible to a slice for window and patterns.
- `find_matches_with_callbacks`: Use for value/mixed patterns with per-pattern callbacks and overlap settings

See tests for more comprehensive examples and edge cases.

## Edge Cases

- Empty window or patterns: returns no matches
- Patterns can be all values, all functions, or mixed
- Multiple patterns and multi-element patterns supported
- Deduplication and overlap settings can be combined
- Patterns of length 1 and longer are supported
- Overlap exclusion can prevent some matches (see tests)

- Window and patterns can be Vec, slice, or array

//! ## Example: Value patterns
//! let patterns = vec![
//!     Pattern::new(vec![&1, &2]),
//!     Pattern::new(vec![&2, &1]),
//! ];
//! let matches = matcher.find_matches(&window, &patterns);
//! assert!(matches.contains(&(0, 0))); // [1,2] at 0
//! assert!(matches.contains(&(1, 1))); // [2,1] at 1
//! assert!(matches.contains(&(2, 0))); // [1] at 0
//! assert!(matches.contains(&(3, 1))); // [2] at 1
//!
//! ## Example: Callback pattern
//! let patterns = vec![
//!     Pattern::with_callback(vec![&1, &2], |x: &&i32| **x == 1 ||**x == 2),
//! ];
//! let matches = matcher.find_matches(&window, &patterns);
//! assert!(matches.contains(&(0, 0))); // [1,2] at 0
//! assert!(matches.contains(&(1, 1))); // [2,1] at 1
//! assert!(matches.contains(&(2, 0))); // [1] at 0
//! assert!(matches.contains(&(3, 1))); // [2] at 1
//!

//! let patterns_fn: Vec<Vec<Box<dyn Fn(&i32) -> bool>>> = vec![
//!     vec![Box::new(|x| *x == 1)],
//!     vec![Box::new(|x|*x == 4)],
 //! ];
 //! let matches = matcher.find_matches(&window, &patterns_fn, false, None::<fn(usize, usize)>);
 //! assert!(matches.contains(&(0, 0)));
 //! assert!(matches.contains(&(1, 3)));
 //!
 with callback
//! let patterns = vec![

//!         pattern: vec![Box::new(|x: &i32| *x == 1), Box::new(|x: &i32|*x == 2)],
 //!         callback: Box::new(move |matched| results1.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
 //!         allow_overlap_with_others: false,
 //!         allow_others_to_overlap: true,
 //!     },

 //!         pattern: vec![Box::new(|x: &i32| *x == 2), Box::new(|x: &i32|*x == 3)],
 //!         callback: Box::new(move |matched| results2.borrow_mut().push(matched.iter().map(|x| **x).collect::<Vec<_>>())),
 //!         allow_overlap_with_others: true,
 //!         allow_others_to_overlap: false,
 //!     },
 //! ];
 //! let matches = matcher.find_matches_with_callbacks(&window, &patterns);
 //! assert!(results.contains(&vec![1, 2]));
 //! assert!(results.contains(&vec![2, 3]));
