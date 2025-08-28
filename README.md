# ScrollingWindowPatternMatcher

A high-performance pattern matching library for Rust that processes streaming data with configurable window sizes and custom data extractors. This library allows you to create complex patterns that match against sequences of data, with powerful extractor functions that can modify matching behavior at runtime.

## 🚨 Major Version 2.0 Simplification

**This is a major simplification from previous versions with breaking API changes.** The dual architecture has been unified into a single `Matcher` that supports both simple pattern matching and advanced stateful operations with context.

## ✨ Features

- **Unified Architecture** - Single `Matcher` type that handles all pattern matching scenarios
- **Context Support** - Optional context for stateful operations and data capture
- **Function-based Patterns** - Custom matching logic with closures and predicates  
- **Stateful Extraction** - Access and modify context during pattern matching
- **Settings-based Configuration** - Clean configuration for element behavior
- **Rich Pattern Elements** - Exact matches, predicates, and range matching
- **Flexible Extraction** - Extract, continue, or restart pattern matching flow
- **Comprehensive Error Handling** - Detailed error types with proper error propagation
- **Memory Safe** - Zero-copy operations where possible with Rust's ownership guarantees

## 🚀 Quick Start

### Basic Pattern Matching

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

// Create a matcher with a window size
let mut matcher = Matcher::<i32, ()>::new(10);

// Add patterns to find sequence 1, 2, 3
matcher.add_pattern(PatternElement::exact(1));
matcher.add_pattern(PatternElement::exact(2));
matcher.add_pattern(PatternElement::exact(3));

// Process data items
assert_eq!(matcher.process_item(1).unwrap(), None);
assert_eq!(matcher.process_item(2).unwrap(), None);
assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Pattern complete!
```

### Function-based Pattern Matching

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

// Create a matcher with window size 10
let mut matcher = Matcher::<i32, ()>::new(10);

// Add patterns to find even numbers followed by odd numbers
matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 1));

// Process data
assert_eq!(matcher.process_item(2).unwrap(), None);  // Even number matches first pattern
assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Odd number completes the pattern
```

### Range-based Pattern Matching

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

// Create a matcher
let mut matcher = Matcher::<i32, ()>::new(10);

// Add patterns to find values in ranges
matcher.add_pattern(PatternElement::range(1, 5));    // 1 <= x <= 5
matcher.add_pattern(PatternElement::range(10, 15));  // 10 <= x <= 15

// Process data
assert_eq!(matcher.process_item(3).unwrap(), None);   // First range matches
assert_eq!(matcher.process_item(12).unwrap(), Some(12)); // Second range completes pattern
```

### Stateful Pattern Matching with Context

```rust
use scrolling_window_pattern_matcher::{
    Matcher, PatternElement, ElementSettings, ExtractorAction, ExtractorError
};

// Define a context to capture matched values
#[derive(Debug, Clone)]
struct MyContext {
    captured_values: Vec<i32>,
}

// Create a matcher with context
let mut matcher = Matcher::<i32, MyContext>::new(10);

// Set up context
let context = MyContext {
    captured_values: Vec::new(),
};
matcher.set_context(context);

// Register an extractor that captures values
matcher.register_extractor(1, |state| {
    if state.current_item > 10 {
        Ok(ExtractorAction::Extract(state.current_item * 2))
    } else {
        Ok(ExtractorAction::Continue)
    }
});

// Create a pattern with an extractor
let mut settings = ElementSettings::default();
settings.extractor_id = Some(1);
matcher.add_pattern(PatternElement::exact_with_settings(15, settings));

// Process an item - the extractor will double it
assert_eq!(matcher.process_item(15).unwrap(), Some(30));
```

## 🏗️ Pattern Elements

The library supports multiple types of pattern elements:

### Exact Match Elements

```rust
use scrolling_window_pattern_matcher::{PatternElement, ElementSettings};

// Simple exact match
let element = PatternElement::exact(42);

// Exact match with settings and extractor
let mut settings = ElementSettings::default();
settings.extractor_id = Some(1);
settings.optional = true;
let element = PatternElement::exact_with_settings(42, settings);
```

### Predicate Elements

```rust
use scrolling_window_pattern_matcher::PatternElement;

// Simple predicate
let element = PatternElement::predicate(|x: &i32| *x > 0);

// Predicate with settings
let mut settings = ElementSettings::default();
settings.max_retries = 3;
let element = PatternElement::predicate_with_settings(
    |x: &i32| x % 2 == 0, 
    settings
);
```

### Range Elements

```rust
use scrolling_window_pattern_matcher::PatternElement;

// Simple range (inclusive)
let element = PatternElement::range(1, 10);

// Range with settings
let mut settings = ElementSettings::default();
settings.timeout_ms = Some(1000);
let element = PatternElement::range_with_settings(1, 10, settings);
```

## ⚙️ Element Settings

Configure pattern element behavior with `ElementSettings`:

```rust
use scrolling_window_pattern_matcher::ElementSettings;

let mut settings = ElementSettings::default();
settings.max_retries = 3;           // Retry failed matches
settings.optional = true;           // Element is optional in pattern
settings.timeout_ms = Some(1000);   // Timeout for this element
settings.extractor_id = Some(1);    // Associated extractor ID

// Context can be added too
settings.context = Some(my_context);
```

## 🔍 Extractors

Extractors allow you to modify the matching flow and extract custom data:

### ExtractorAction Types

```rust
use scrolling_window_pattern_matcher::ExtractorAction;

// Continue normal pattern matching
ExtractorAction::Continue

// Extract data and complete the pattern
ExtractorAction::Extract(data)

// Restart pattern matching from the beginning
ExtractorAction::Restart
```

### Registering Extractors

```rust
// Register an extractor function with an ID
matcher.register_extractor(1, |state| {
    if state.current_item > 100 {
        Ok(ExtractorAction::Extract(state.current_item))
    } else {
        Ok(ExtractorAction::Continue)
    }
});

// The MatchState provides information about the current match
// - state.current_item: The item being processed
// - state.position: Position in the current pattern
// - state.total_processed: Total items processed so far
```

## 📊 Matcher API

### Core Methods

```rust
// Create a new matcher
let mut matcher = Matcher::<i32, MyContext>::new(window_size);

// Add pattern elements
matcher.add_pattern(PatternElement::exact(42));

// Process single items
let result = matcher.process_item(item)?;

// Process multiple items
let results = matcher.process_items(vec![1, 2, 3])?;

// Reset matcher state
matcher.reset();

// Register extractors
matcher.register_extractor(id, extractor_fn);

// Context management
matcher.set_context(context);
let context_ref = matcher.context();
```

### State Inspection

```rust
// Check current matching state
let position = matcher.current_position();
let total = matcher.total_processed();
let count = matcher.pattern_count();
let is_matching = matcher.is_matching();

// Window size management
let size = matcher.window_size();
matcher.set_window_size(new_size);
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run specific tests:

```bash
cargo test test_exact_match
```

## 📝 Examples

See the built-in tests for comprehensive examples:

- `tests::test_exact_match` - Basic exact value matching
- `tests::test_predicate_match` - Function-based pattern matching  
- `tests::test_range_match` - Range-based pattern matching
- `tests::test_extractor` - Custom data extraction
- `tests::test_context` - Context management
- `tests::test_reset` - State reset functionality

## 🚦 Error Handling

The library provides comprehensive error handling:

```rust
use scrolling_window_pattern_matcher::{MatcherError, ExtractorError};

match matcher.process_item(item) {
    Ok(Some(extracted)) => println!("Extracted: {:?}", extracted),
    Ok(None) => println!("No match yet"),
    Err(MatcherError::NoPatterns) => println!("No patterns configured"),
    Err(MatcherError::InvalidPattern(msg)) => println!("Invalid pattern: {}", msg),
    Err(MatcherError::ExtractorFailed(err)) => println!("Extractor failed: {}", err),
}
```

## 🔧 Advanced Usage

### Custom Context Types

```rust
#[derive(Debug, Clone)]
struct CustomContext {
    captured_data: Vec<String>,
    counters: HashMap<String, usize>,
    config: AppConfig,
}

let mut matcher = Matcher::<String, CustomContext>::new(50);
```

### Complex Extractors

```rust
matcher.register_extractor(1, |state| {
    match state.current_item {
        item if item > 100 => {
            // Extract large values
            Ok(ExtractorAction::Extract(item * 2))
        }
        item if item < 0 => {
            // Restart on negative values
            Ok(ExtractorAction::Restart)
        }
        _ => {
            // Continue normal processing
            Ok(ExtractorAction::Continue)
        }
    }
});
```

### Pattern Composition

```rust
// Build complex patterns step by step
matcher.add_pattern(PatternElement::exact(1));
matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
matcher.add_pattern(PatternElement::range(10, 20));

// Or with settings
let mut settings = ElementSettings::default();
settings.optional = true;
matcher.add_pattern(PatternElement::exact_with_settings(42, settings));
```

## 📈 Performance

The library is designed for high-performance streaming data processing:

- **Zero-copy operations** where possible
- **Minimal allocations** during pattern matching
- **Efficient state management** with small memory footprint
- **Configurable window sizes** to control memory usage
- **Async-friendly** design (no blocking operations)

## 📚 API Reference

### Types

- `Matcher<T, Context>` - Main pattern matcher with optional context
- `PatternElement<T, Context>` - Individual pattern elements  
- `ElementSettings<Context>` - Configuration for pattern elements
- `MatchState<T>` - Current state information for extractors
- `ExtractorAction<T>` - Actions that extractors can return
- `MatcherError` - Error types for matcher operations
- `ExtractorError` - Error types for extractor operations

### Key Traits

All generic types must implement:
- `T: Clone + PartialEq + fmt::Debug + PartialOrd` (for pattern matching)
- `Context: Clone + fmt::Debug` (for context operations)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Links

- [Documentation](https://docs.rs/scrolling_window_pattern_matcher)
- [Crates.io](https://crates.io/crates/scrolling_window_pattern_matcher)
- [Repository](https://github.com/user/scrolling_window_pattern_matcher)