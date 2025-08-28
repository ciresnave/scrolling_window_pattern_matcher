# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.1] - 2025-08-28

### Fixed
- Updated all documentation to reflect unified architecture
- Cleaned up development and backup files for release
- Enhanced library-level documentation with comprehensive examples
- Improved CHANGELOG, BORROWING_SOLUTION, and stateful extraction documentation

### Technical Details
- All 55 tests continue to pass (29 unit + 26 integration tests)
- All 4 examples updated and working correctly
- Project structure cleaned for production release
- Documentation consistency verified across all files

## [3.0.0] - 2025-01-26

### Breaking Changes

- **MAJOR ARCHITECTURE CHANGE**: Unified dual matcher design into single `Matcher<T, Context>` type
- Removed `StatelessMatcher` and `StatefulMatcher` - replaced with unified `Matcher`
- Removed `StatefulExtractor` trait - replaced with closure-based extractor system
- Removed `run()` method - replaced with `process_item()` for item-by-item processing
- Removed field-based configuration - all settings now use `ElementSettings` struct

### Added

- **Unified Architecture**: Single `Matcher<T, Context>` for both simple and complex matching scenarios
- **Enhanced Pattern Elements**:
  - `PatternElement::exact()` for exact value matching
  - `PatternElement::predicate()` for custom predicate functions
  - `PatternElement::range()` for inclusive range matching
- **ElementSettings System**: Comprehensive configuration including optional elements, timeouts, and extractors
- **Closure-based Extractors**: Simple function registration with `register_extractor()`
- **Context Support**: Optional context parameter for stateful operations and data capture
- **Improved Error Handling**: Comprehensive error types with proper propagation
- **Memory Safety**: Zero-copy operations where possible with Rust's ownership guarantees

### Enhanced

- **Performance**: Optimized single-pass processing with reduced overhead
- **API Simplicity**: Unified interface reduces cognitive load and potential misuse
- **Test Coverage**: Expanded from 6 to 55 comprehensive tests covering all functionality
- **Documentation**: Complete rewrite with unified API examples and usage patterns

### Removed

- Complex dual matcher architecture that added unnecessary complexity
- Field-based pattern configuration that was error-prone
- Separate stateful extraction trait system
- Iterator-based `run()` method in favor of item-by-item processing

### Migration Guide

#### Version 2.x (Old API)

```rust
use scrolling_window_pattern_matcher::{StatelessMatcher, StatefulMatcher};

// Old dual matcher approach
let mut stateless = StatelessMatcher::new();
let mut stateful = StatefulMatcher::new();
```

#### Version 3.x (New Unified API)

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

// New unified approach
let mut matcher = Matcher::<i32, ()>::new(10);
matcher.add_pattern(PatternElement::exact(42));

// Process items one by one
if let Some(result) = matcher.process_item(42).unwrap() {
    println!("Pattern matched: {}", result);
}
```

For extractors, use the new closure-based system:

```rust
// Register an extractor
matcher.register_extractor(1, |state| {
    println!("Matched: {}", state.current_item);
    Ok(ExtractorAction::Continue)
});

// Use with pattern element
let settings = ElementSettings {
    extractor_id: Some(1),
    ..Default::default()
};
matcher.add_pattern(PatternElement::exact_with_settings(42, settings));
```

## [2.x.x] - Previous Versions

See Git history for changes in version 2.x releases with extractor-driven architecture.

## [1.x.x] - Initial Versions

See Git history for changes in version 1.x releases with callback-based system.
