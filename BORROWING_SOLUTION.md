# Borrowing Solution and Memory Management

## Overview

This document describes the memory management and borrowing strategies used in the unified `Matcher<T, Context>` architecture. With the simplification to a single matcher type, many of the complex borrowing issues that existed in the previous dual matcher system have been eliminated.

## Unified Architecture Benefits

### Simplified Ownership Model

The unified `Matcher<T, Context>` design provides several advantages:

- **Single Owner**: Only one matcher instance owns all pattern and state data
- **Clear Lifetimes**: No complex borrowing between stateless and stateful components
- **Reduced Complexity**: Eliminates the need for complex trait object management

### Memory Safety Guarantees

The current implementation ensures memory safety through:

```rust
pub struct Matcher<T, Context> {
    window: VecDeque<T>,
    patterns: Vec<PatternElement<T, Context>>,
    position: usize,
    window_size: usize,
    context: Option<Context>,
    extractors: HashMap<usize, Box<dyn Fn(&MatchState<T, Context>) -> Result<ExtractorAction<T>, ExtractorError> + Send + Sync>>,
}
```

## Key Design Decisions

### 1. Owned Data in Window

The matcher owns its sliding window data:

```rust
window: VecDeque<T>
```

This eliminates borrowing issues since the matcher has full ownership of the data it processes.

### 2. Closure-Based Extractors

Instead of trait objects with complex lifetimes, extractors are simple closures:

```rust
type ExtractorFn<T, Context> = Box<dyn Fn(&MatchState<T, Context>) -> Result<ExtractorAction<T>, ExtractorError> + Send + Sync>;
```

This provides:

- Thread safety with `Send + Sync`
- Clear lifetime management
- No borrowing conflicts between extractors

### 3. Item-by-Item Processing

The `process_item()` method takes ownership of individual items:

```rust
pub fn process_item(&mut self, item: T) -> Result<Option<T>, MatcherError>
```

This design:

- Avoids iterator borrowing complexities
- Provides clear ownership transfer
- Enables streaming data processing

## Migration from Previous Architecture

### Old Dual Matcher Issues

The previous dual matcher system had these borrowing challenges:

```rust
// OLD: Complex borrowing between components
let stateless = StatelessMatcher::new();
let mut stateful = StatefulMatcher::new(&stateless); // Borrowing issues here
```

### New Unified Solution

The unified architecture eliminates these issues:

```rust
// NEW: Single owner, no borrowing conflicts
let mut matcher = Matcher::<i32, ()>::new(10);
matcher.add_pattern(PatternElement::exact(42));
```

## Best Practices

### 1. Context Lifetime Management

When using context, ensure proper lifetime bounds:

```rust
let mut matcher = Matcher::<i32, MyContext>::new(10);
let context = MyContext::new();
matcher.set_context(Some(context));
```

### 2. Extractor Ownership

Extractors capture what they need by value:

```rust
let multiplier = 2;
matcher.register_extractor(1, move |state| {
    Ok(ExtractorAction::Extract(state.current_item * multiplier))
});
```

### 3. Pattern Element Ownership

Pattern elements own their comparison data:

```rust
matcher.add_pattern(PatternElement::exact(42)); // 42 is moved into the pattern
```

## Performance Considerations

### Zero-Copy Where Possible

The design minimizes copying through:

- Reference passing in match state: `&MatchState<T, Context>`
- Efficient window management with `VecDeque<T>`
- Move semantics for item processing

### Memory Efficiency

- Fixed-size window prevents unbounded growth
- Extractors stored as function pointers, not heavy trait objects
- Context is optional and only allocated when needed

## Thread Safety

The matcher is thread-safe when:

- `T: Send + Sync`
- `Context: Send + Sync`
- All extractors implement `Send + Sync` (enforced by type system)

## Conclusion

The unified architecture successfully eliminates the borrowing complexities that existed in the previous dual matcher system while maintaining performance and flexibility. The single ownership model provides clear semantics and memory safety guarantees.
