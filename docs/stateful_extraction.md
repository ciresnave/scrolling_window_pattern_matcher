# Unified Pattern Matching with Context Support

## Overview

This document describes the pattern matching capabilities with optional context support in the unified `Matcher<T, Context>` architecture. The matcher provides both stateless and stateful pattern matching in a single, simplified interface.

## Unified Architecture Benefits

The unified `Matcher<T, Context>` design eliminates the complexity of the previous dual matcher system while providing all the functionality needed for both simple and complex pattern matching scenarios.

### Key Features

- **Single Matcher Type**: `Matcher<T, Context>` handles both simple and context-aware matching
- **Optional Context**: Context parameter is optional - use `()` for stateless matching
- **Closure-Based Extractors**: Simple function registration without complex trait hierarchies
- **Item-by-Item Processing**: Streaming-friendly `process_item()` method
- **Memory Safe**: No borrowing conflicts or complex lifetime management

## Core Components

### 1. Unified Matcher

```rust
pub struct Matcher<T, Context> {
    window: VecDeque<T>,
    patterns: Vec<PatternElement<T, Context>>,
    position: usize,
    window_size: usize,
    context: Option<Context>,
    extractors: HashMap<usize, ExtractorFn<T, Context>>,
}
```

### 2. Pattern Elements

```rust
impl<T, Context> PatternElement<T, Context> {
    pub fn exact(value: T) -> Self
    pub fn predicate<F>(predicate: F) -> Self
    where F: Fn(&T) -> bool + Send + Sync + 'static
    pub fn range(min: T, max: T) -> Self
    where T: PartialOrd
}
```

### 3. Element Settings

```rust
pub struct ElementSettings<Context> {
    pub optional: bool,
    pub timeout_ms: Option<u64>,
    pub extractor_id: Option<usize>,
    _phantom: PhantomData<Context>,
}
```

### 4. Extractor System

```rust
type ExtractorFn<T, Context> = Box<dyn Fn(&MatchState<T, Context>) -> Result<ExtractorAction<T>, ExtractorError> + Send + Sync>;

pub enum ExtractorAction<T> {
    Continue,
    Extract(T),
    Restart,
}
```

## Usage Examples

### Simple Stateless Matching

```rust
use scrolling_window_pattern_matcher::{Matcher, PatternElement};

// Create matcher without context
let mut matcher = Matcher::<i32, ()>::new(10);

// Add simple patterns
matcher.add_pattern(PatternElement::exact(1));
matcher.add_pattern(PatternElement::exact(2));
matcher.add_pattern(PatternElement::exact(3));

// Process items
assert_eq!(matcher.process_item(1).unwrap(), None);
assert_eq!(matcher.process_item(2).unwrap(), None);
assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Pattern complete!
```

### Context-Aware Matching

```rust
#[derive(Default)]
struct ExtractionContext {
    captured_data: Vec<i32>,
    match_count: usize,
}

let mut matcher = Matcher::<i32, ExtractionContext>::new(10);
matcher.set_context(Some(ExtractionContext::default()));

// Register extractor that captures data
matcher.register_extractor(1, |state| {
    if let Some(context) = &state.context {
        // This would require mutable access in real implementation
        // For now, extractors work with immutable references
        println!("Would capture: {}", state.current_item);
    }
    Ok(ExtractorAction::Continue)
});

// Add pattern with extractor
let settings = ElementSettings {
    extractor_id: Some(1),
    ..Default::default()
};
matcher.add_pattern(PatternElement::exact_with_settings(42, settings));
```

### Advanced Pattern Matching

```rust
let mut matcher = Matcher::<i32, ()>::new(10);

// Register an extractor that doubles values
matcher.register_extractor(1, |state| {
    Ok(ExtractorAction::Extract(state.current_item * 2))
});

// Create pattern with custom settings
let mut settings = ElementSettings::default();
settings.optional = true;
settings.extractor_id = Some(1);

matcher.add_pattern(PatternElement::exact_with_settings(5, settings));

// Process item - will be doubled by extractor
assert_eq!(matcher.process_item(5).unwrap(), Some(10));
```

### Predicate-Based Matching

```rust
let mut matcher = Matcher::<i32, ()>::new(10);

// Match even numbers
matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

// Process items
assert_eq!(matcher.process_item(1).unwrap(), None); // Odd, no match
assert_eq!(matcher.process_item(2).unwrap(), Some(2)); // Even, matches!
```

### Range Matching

```rust
let mut matcher = Matcher::<i32, ()>::new(10);

// Match numbers between 10 and 20 (inclusive)
matcher.add_pattern(PatternElement::range(10, 20));

assert_eq!(matcher.process_item(5).unwrap(), None);  // Out of range
assert_eq!(matcher.process_item(15).unwrap(), Some(15)); // In range
```

## Benefits of Unified Design

### 1. Simplified API

- Single matcher type for all scenarios
- No need to choose between stateless/stateful variants
- Consistent method names and behavior

### 2. Flexible Context Support

- Optional context parameter - use `()` when not needed
- Any user-defined context type
- No lifetime or borrowing complications

### 3. Performance

- Single-pass processing
- Minimal overhead for context management
- Efficient memory usage with `VecDeque` window

### 4. Memory Safety

- No borrowing conflicts
- Clear ownership semantics
- Thread-safe extractors with `Send + Sync`

### 5. Extensibility

- Easy to add new pattern element types
- Simple extractor registration
- Configurable through `ElementSettings`

## Migration from Previous Versions

### From Dual Matcher System

```rust
// OLD: Complex dual system
let stateless = StatelessMatcher::new();
let mut stateful = StatefulMatcher::new();

// NEW: Unified approach
let mut matcher = Matcher::<i32, MyContext>::new(10);
```

### From Trait-Based Extractors

```rust
// OLD: Complex trait implementation
impl StatefulExtractor<i32, Context> for MyExtractor {
    fn extract(&mut self, state: &MatchState<i32>, context: &mut Context) -> Result<ExtractorAction<i32>, ExtractorError> {
        // Complex implementation
    }
}

// NEW: Simple closure
matcher.register_extractor(1, |state| {
    Ok(ExtractorAction::Extract(state.current_item * 2))
});
```

## Implementation Notes

### Current Limitations

1. **Immutable Context Access**: Extractors currently receive immutable references to context
2. **Single Pattern Sequence**: Each matcher handles one pattern sequence at a time
3. **No Pattern Removal**: Patterns cannot be removed after addition (by design for simplicity)

### Design Decisions

1. **Item-by-Item Processing**: Enables streaming and reduces memory usage
2. **Closure-Based Extractors**: Simpler than trait objects while maintaining flexibility
3. **Fixed Window Size**: Prevents unbounded memory growth
4. **Move Semantics**: Items are moved into the matcher for efficiency

## Future Enhancements

### Potential Improvements

1. **Mutable Context Access**: Enable extractors to modify context
2. **Multiple Pattern Support**: Handle multiple concurrent patterns
3. **Pattern Composition**: Combine patterns with logical operators
4. **Async Support**: Non-blocking pattern matching for high-throughput scenarios

### Performance Optimizations

1. **Zero-Copy Operations**: Minimize data copying where possible
2. **Vectorized Matching**: SIMD operations for bulk pattern matching
3. **Memory Pooling**: Reuse allocations for high-frequency operations

## Conclusion

The unified `Matcher<T, Context>` architecture successfully simplifies pattern matching while maintaining all the capabilities needed for real-world data processing. The design eliminates the complexity of the previous dual matcher system while providing better performance and easier usage.

This approach makes the library suitable for both simple pattern validation and complex data extraction scenarios, all through a single, consistent interface.
