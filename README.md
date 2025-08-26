# ScrollingWindowPatternMatcher

A flexible pattern matching library for Rust with an advanced extractor system for dynamic behavior modification. This library allows you to create complex patterns that match against sequences of data, with powerful extractor functions that can modify matching behavior at runtime.

## 🚨 Major Version 2.0 Rewrite

**This is a complete rewrite from version 1.x with breaking API changes.** The previous field-based pattern syntax has been replaced with a settings-based approach, and the callback system has been replaced with a more powerful extractor architecture.

### Migration from 1.x

The API has fundamentally changed. Please see the examples and documentation below for the new approach.

## ✨ Features

- **Extractor-driven architecture** - Dynamic modification of matching behavior through extractor functions
- **Settings-based configuration** - Clean builder pattern for pattern element configuration
- **Rich pattern elements** - Values, functions, pattern references, wildcards, and nested repeats
- **Advanced extractor actions** - Continue, Skip, Jump, pattern manipulation, and flow control
- **Comprehensive error handling** - Detailed error types with proper error propagation
- **Zero-copy when possible** - Efficient matching with minimal allocations
- **Priority-based matching** - Control pattern matching order with priority settings

## 🚀 Quick Start

```rust
use scrolling_window_pattern_matcher::{ElementSettings, Matcher, PatternElement};

// Create a matcher
let mut matcher = Matcher::new();

// Add a pattern to find the value 42
matcher.add_pattern(
    "find_42".to_string(),
    vec![PatternElement::Value {
        value: 42,
        settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
    }]
);

// Match against data
let data = vec![1, 42, 3, 42, 5];
let result = matcher.run(&data);
assert!(result.is_ok());
```

## 🏗️ Pattern Elements

### Value

Matches a specific value:

```rust
PatternElement::Value {
    value: 42,
    settings: Some(
        ElementSettings::new()
            .min_repeat(1)     // Must match at least this many times
            .max_repeat(3)     // Match at most this many times
            .greedy(true)      // Match as many as possible (true) or as few as possible (false)
            .priority(10)      // Lower numbers = higher priority
    ),
}
```

### Function

Matches using custom logic:

```rust
PatternElement::Function {
    function: Box::new(|x: &i32| x % 2 == 0),  // Custom predicate function
    settings: Some(
        ElementSettings::new()
            .min_repeat(1)
            .max_repeat(5)
            .greedy(true)
    ),
}
```

### Any

Wildcard matching:

```rust
PatternElement::Any {
    settings: Some(
        ElementSettings::new()
            .min_repeat(1)     // Match any 1-3 consecutive items
            .max_repeat(3)
            .greedy(false)
    ),
}
```

### Pattern Reference

References another named pattern:

```rust
PatternElement::Pattern {
    pattern: "other_pattern_name".to_string(),
    settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
}
```

### Repeat

Repeats a nested pattern element:

```rust
PatternElement::Repeat {
    element: Box::new(PatternElement::Value {
        value: 5,
        settings: None,
    }),
    settings: Some(
        ElementSettings::new()
            .min_repeat(2)
            .max_repeat(4)
            .greedy(true)
    ),
}
```

## ⚡ Extractor System

The extractor system is the core feature that allows dynamic modification of matching behavior:

### Basic Extractor

```rust
use scrolling_window_pattern_matcher::{ExtractorAction, MatchState};

let extractor = Box::new(|state: &MatchState<i32>| {
    println!("Matched at position {}: {:?}",
             state.current_position,
             state.matched_items);
    Ok(ExtractorAction::Continue)
});

let pattern = vec![PatternElement::Value {
    value: 42,
    settings: Some(ElementSettings::new().extractor(extractor)),
}];
```

### Extractor Actions

#### Continue

Continue normal matching:

```rust
Ok(ExtractorAction::Continue)
```

#### Skip

Skip ahead by N positions:

```rust
Ok(ExtractorAction::Skip(3))  // Skip 3 positions ahead
```

#### Jump

Jump relative to current position:

```rust
Ok(ExtractorAction::Jump(-2))  // Jump back 2 positions
Ok(ExtractorAction::Jump(5))   // Jump forward 5 positions
```

#### Pattern Manipulation

Add or remove patterns dynamically:

```rust
// Add a new pattern
Ok(ExtractorAction::AddPattern(
    "new_pattern".to_string(),
    Pattern::new(vec![/* pattern elements */])
))

// Remove a pattern
Ok(ExtractorAction::RemovePattern("old_pattern".to_string()))
```

#### Flow Control

```rust
Ok(ExtractorAction::StopMatching)           // Stop all matching
Ok(ExtractorAction::DiscardPartialMatch)    // Discard current match
Ok(ExtractorAction::RestartFrom(0))         // Restart from specific position
```

### Match State Information

Extractors receive rich context information:

```rust
let extractor = Box::new(|state: &MatchState<i32>| {
    println!("Current position: {}", state.current_position);
    println!("Matched items: {:?}", state.matched_items);
    println!("Pattern name: {}", state.pattern_name);
    println!("Element index: {}", state.element_index);
    println!("Input length: {}", state.input_length);
    Ok(ExtractorAction::Continue)
});
```

## 🔧 Complex Examples

### Multi-element Pattern

```rust
// Pattern: value 1, then any item, then value 3
matcher.add_pattern(
    "one_any_three".to_string(),
    vec![
        PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        },
        PatternElement::Any {
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        },
        PatternElement::Value {
            value: 3,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        },
    ]
);
```

### Advanced Extractor Usage

```rust
use scrolling_window_pattern_matcher::{ExtractorAction, ExtractorError, MatchState};

// Extractor that logs matches and skips ahead
let logging_extractor = Box::new(|state: &MatchState<i32>| -> Result<ExtractorAction<i32>, ExtractorError> {
    println!("🎯 Match found at position {}: {:?}",
             state.current_position,
             state.matched_items);

    // Skip ahead by 2 positions after each match
    Ok(ExtractorAction::Skip(2))
});

// Extractor that adds new patterns based on matched values
let dynamic_extractor = Box::new(|state: &MatchState<i32>| -> Result<ExtractorAction<i32>, ExtractorError> {
    if let Some(&first_value) = state.matched_items.first() {
        if first_value > 50 {
            let new_pattern = Pattern::new(vec![
                PatternElement::Value {
                    value: first_value + 10,
                    settings: Some(ElementSettings::new()),
                }
            ]);
            return Ok(ExtractorAction::AddPattern(
                format!("dynamic_{}", first_value),
                new_pattern
            ));
        }
    }
    Ok(ExtractorAction::Continue)
});
```

### Priority-based Matching

```rust
// Higher priority pattern (lower number = higher priority)
matcher.add_pattern(
    "high_priority".to_string(),
    vec![PatternElement::Value {
        value: 100,
        settings: Some(
            ElementSettings::new()
                .priority(1)  // High priority
                .min_repeat(1)
                .max_repeat(1)
        ),
    }]
);

// Lower priority pattern
matcher.add_pattern(
    "low_priority".to_string(),
    vec![PatternElement::Any {
        settings: Some(
            ElementSettings::new()
                .priority(10)  // Lower priority
                .min_repeat(1)
                .max_repeat(1)
        ),
    }]
);
```

## 🌍 Real-World Examples

### Log Analysis

```rust
// Detect HTTP error codes
let error_detector = Box::new(|state: &MatchState<i32>| -> Result<ExtractorAction<i32>, ExtractorError> {
    if let Some(&code) = state.matched_items.first() {
        match code {
            400..=499 => println!("⚠️  Client Error detected: {}", code),
            500..=599 => println!("🚨 Server Error detected: {}", code),
            _ => {}
        }
    }
    Ok(ExtractorAction::Continue)
});

matcher.add_pattern(
    "http_errors".to_string(),
    vec![PatternElement::Function {
        function: Box::new(|&code| (400..=599).contains(&code)),
        settings: Some(ElementSettings::new().extractor(error_detector)),
    }]
);
```

### Network Traffic Analysis

```rust
// Detect potential port scanning (rapid consecutive port accesses)
let scan_detector = Box::new(|state: &MatchState<i32>| -> Result<ExtractorAction<i32>, ExtractorError> {
    if state.matched_items.len() >= 3 {
        println!("🔍 Potential port scan detected: {} consecutive port accesses",
                 state.matched_items.len());
    }
    Ok(ExtractorAction::Continue)
});

matcher.add_pattern(
    "port_scan".to_string(),
    vec![PatternElement::Function {
        function: Box::new(|&port| (1..=65535).contains(&port)),
        settings: Some(
            ElementSettings::new()
                .min_repeat(3)
                .max_repeat(10)
                .greedy(true)
                .extractor(scan_detector)
        ),
    }]
);
```

## 🚨 Error Handling

The library provides comprehensive error handling:

```rust
use scrolling_window_pattern_matcher::{MatcherError, ExtractorError};

match matcher.run(&data) {
    Ok(()) => println!("Matching completed successfully"),
    Err(MatcherError::ExtractorError(ExtractorError::InvalidPosition(pos))) => {
        println!("Extractor tried to access invalid position: {}", pos);
    }
    Err(MatcherError::ExtractorError(ExtractorError::PatternNotFound(name))) => {
        println!("Extractor referenced non-existent pattern: {}", name);
    }
    Err(MatcherError::ExtractorError(ExtractorError::Message(msg))) => {
        println!("Extractor error: {}", msg);
    }
    Err(e) => println!("Other error: {}", e),
}
```

## 📚 API Reference

### Core Types

- `Matcher<T>` - Main matcher struct
- `PatternElement<T>` - Individual pattern elements (Value, Function, Any, Pattern, Repeat)
- `ElementSettings<T>` - Configuration for pattern elements
- `PatternSettings<T>` - Configuration for entire patterns
- `ExtractorAction<T>` - Actions that extractors can return
- `MatchState<T>` - Context information provided to extractors

### Matcher Methods

- `new()` - Create a new matcher
- `with_settings(settings)` - Create matcher with custom settings
- `add_pattern(name, elements)` - Add a pattern
- `add_pattern_with_settings(name, pattern)` - Add pattern with custom settings
- `remove_pattern(name)` - Remove a pattern
- `run(data)` - Execute matching on data slice

### Settings Builders

- `ElementSettings::new()` - Create element settings builder
- `PatternSettings::new()` - Create pattern settings builder

Builder methods: `.min_repeat()`, `.max_repeat()`, `.greedy()`, `.priority()`, `.extractor()`

## 🎯 Design Philosophy

This library prioritizes **flexibility and extensibility** through:

- **Extractor-driven architecture** - Extractors provide unlimited customization capabilities
- **Settings-based configuration** - Clean, discoverable API through builder patterns
- **Type safety** - Rust's type system prevents common pattern matching errors
- **Error transparency** - Comprehensive error types with detailed context
- **Performance awareness** - Efficient algorithms with minimal allocations where possible

## 📝 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🔄 Breaking Changes from 1.x

- Complete API rewrite with settings-based configuration
- Callback system replaced with extractor architecture
- `match_window()` replaced with `run()` method
- Field-based pattern syntax replaced with settings builders
- Return type changed from `HashMap<String, Vec<T>>` to `Result<(), MatcherError>`
- Pattern matching results now handled through extractors rather than return values
