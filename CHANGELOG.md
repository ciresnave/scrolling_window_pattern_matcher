# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2025-01-XX

### Changed

- **BREAKING:** Complete API rewrite with extractor-driven architecture
- **BREAKING:** Field-based pattern syntax replaced with settings builders
- **BREAKING:** Callback system replaced with powerful extractor functions
- **BREAKING:** `match_window()` method replaced with `run()` method
- **BREAKING:** Return type changed from `HashMap<String, Vec<T>>` to `Result<(), MatcherError>`
- **BREAKING:** Pattern elements now use `ElementSettings` and `PatternSettings` for configuration

### Added

- Advanced extractor system with `ExtractorAction` enum
- Rich context information through `MatchState` struct
- Dynamic pattern manipulation (add/remove patterns at runtime)
- Comprehensive error handling with `MatcherError` and `ExtractorError`
- Priority-based pattern matching
- Jump, skip, and restart capabilities through extractors
- Pattern reference support for recursive patterns
- Repeat element for nested pattern repetition
- Panic handling in extractors
- Extensive documentation and examples

### Removed

- **BREAKING:** Field-based pattern configuration (e.g., `minimum_repeat`, `maximum_repeat`, `greedy`)
- **BREAKING:** Callback-based matching system
- **BREAKING:** Direct result return from matching operations
- **BREAKING:** `match_window()` and `match_iterator()` methods

### Migration Guide

#### Version 1.x (Old API)

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

let mut matcher = Matcher::new();
matcher.add_pattern("find_42", vec![
    PatternElement::Value {
        value: 42,
        minimum_repeat: 1,
        maximum_repeat: 1,
        greedy: false,
    }
]);

let data = vec![1, 42, 3, 42, 5];
let results = matcher.match_window(&data);
```

#### Version 2.x (New API)

```rust
use scrolling_window_pattern_matcher::{ElementSettings, Matcher, PatternElement};

let mut matcher = Matcher::new();
matcher.add_pattern(
    "find_42".to_string(),
    vec![PatternElement::Value {
        value: 42,
        settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
    }]
);

let data = vec![1, 42, 3, 42, 5];
let result = matcher.run(&data); // Returns Result<(), MatcherError>
```

For callbacks, use extractors:

```rust
// Version 2.x: Using extractors instead of callbacks
let extractor = Box::new(|state: &MatchState<i32>| {
    println!("Found match: {:?}", state.matched_items);
    Ok(ExtractorAction::Continue)
});

let pattern = vec![PatternElement::Value {
    value: 42,
    settings: Some(ElementSettings::new().extractor(extractor)),
}];
```

## [1.x.x] - Previous Versions

See Git history for changes in version 1.x releases.
