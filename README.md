# ScrollingWindowPatternMatcher

A flexible, ergonomic pattern matcher for slices, arrays, and windows, supporting wildcards, custom logic, and builder patterns.

## Features

- Wildcard matching (`PatternElem::Any`)
- Flexible matcher signatures: pass window and patterns as Vec, slice, or array, and patterns as owned or referenced
- Ergonomic builder patterns
- Custom matcher logic
- Flexible callback and overlap configuration

## Choosing Between `find_matches` and `find_matches_flexible`

These two functions are the core of the crate. Understanding their differences will help you select the right one for your use case:

### `find_matches`

- **Signature:** `find_matches(&self, window: &[T], patterns: &[Pattern<T>])`
- **Accepts:** Slices (`&[T]`) for both window and patterns.
- **Performance:** Zero-copy; does not clone window elements. Efficient for large windows.
- **Trait Bounds:** Only requires `T: PartialEq + Clone` for matching logic.
- **Use When:**
  - You already have slices or references to arrays/Vectors.
  - You want maximum performance and minimal memory usage.
  - Your element type is not `Clone`.
- **Limitation:** Cannot accept owned containers directly (e.g., `Vec<T>` by value); must convert to a slice first.

### `find_matches_flexible`

- **Signature:** `find_matches_flexible(&self, window: W, patterns: P)` where `W: IntoIterator, W::Item: Borrow<T>`
- **Accepts:** Owned containers (`Vec<T>`, arrays), references to containers (`&Vec<T>`, `&[T]`), or slices.
- **Performance:** Clones all window elements into a new `Vec<T>`; may use more memory for large windows.
- **Trait Bounds:** Requires `T: Clone + PartialEq`.
- **Use When:**
  - You want ergonomic API and flexibility in passing owned or referenced data.
  - You don't mind cloning window elements.
  - You want to avoid manual conversion to slices.
- **Limitation:** Requires `T: Clone`; may be less efficient for large windows or non-cloneable types.

### Summary Table

| Function                | Accepts           | Performance   | Requires `T: Clone` | Use When                      |
|------------------------ |------------------ |-------------- |-------------------- |-------------------------------|
| `find_matches`          | Slices            | Zero-copy     | No (unless callback)| You have slices, want speed   |
| `find_matches_flexible` | Owned or borrowed | Clones window | Yes                | You want ergonomic flexibility|

**Tip:** If in doubt, use `find_matches` for performance, and `find_matches_flexible` for convenience.

## Choosing an Appropriate `window_len`

The `window_len` parameter in `ScrollingWindowPatternMatcherRef` determines the maximum number of elements (`T`) held in memory at one time for matching. It does **not** directly limit the length of patterns you can match, since partial matches are tracked independently.

### How to Choose `window_len`

- **For most use cases:** Set `window_len` to the length of your input data (e.g., `window.len()` for a slice or vector). This ensures all elements are available for matching and is the most ergonomic choice for batch processing.
- **For streaming or large datasets:** Use a smaller `window_len` to limit memory usage. The matcher will process data in chunks, but you must ensure your pattern logic can handle matches that span window boundaries (advanced usage).
- **For single-element processing:** You can set `window_len = 1` to process one element at a time, but this is rarely needed unless you have strict memory constraints or want to implement a custom streaming matcher.

### Trade-offs

- **Larger `window_len`:**
  - Pros: Simpler API, all data available for matching, best for batch or small datasets.
  - Cons: Higher memory usage for very large datasets.
- **Smaller `window_len`:**
  - Pros: Lower memory usage, suitable for streaming or real-time processing.
  - Cons: Requires careful handling of partial matches and patterns that span windows.

### Practical Advice

- For most users, set `window_len` to the size of your window or input data.
- If you need to process data in a streaming fashion, consider implementing logic to handle partial matches across window boundaries.
- Pattern length is **not** limited by `window_len`—the matcher tracks partial matches as needed.

**Summary:**
Set `window_len` to match your data size for convenience, or use a smaller value for streaming/low-memory scenarios. Pattern matching will work as long as your logic accounts for the chosen window size.

## Usage Example

```rust
use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
let window = vec![1, 2, 2, 2, 3, 4, 5, 6];
let patterns = vec![
    PatternBuilderErased::new()
        .name("triple_twos")
        .value_elem(2)
        .min_repeat(3)
        .capture_name("twos")
        .build(),
    PatternBuilderErased::new()
        .name("gap_and_value")
        .any_elem()
        .min_repeat(2) // gap of 2 elements
        .value_elem(3)
        .capture_name("three")
        .build(),
];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
let named = matcher.find_matches(&window, &patterns);
assert!(named.contains_key("triple_twos"));
let twos_matches = &named["triple_twos"];
assert!(twos_matches.iter().any(|m| m["twos"] == vec![2,2,2]));
assert!(named.contains_key("gap_and_value"));
let gap_matches = &named["gap_and_value"];
assert!(gap_matches.iter().any(|m| m["three"] == vec![3]));
```

## Documentation

See doc comments and tests for more examples.

## Planned Features

### Advanced Features

- Ergonomic builder API for repeat and capture settings (gaps are represented by PatternElem::Any with repeat settings)
- Named patterns and named captures
- Flexible callback/overlap configuration

```rust
use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
let window = vec![1, 2, 1, 2, 1];
let patterns = vec![
    PatternBuilderErased::new().value_elem(1).value_elem(2).build(),
    PatternBuilderErased::new().value_elem(2).value_elem(1).build(),
];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
let named = matcher.find_matches(&window, &patterns);
assert!(named["pattern_0"].len() > 0);
assert!(named["pattern_1"].len() > 0);
```

## Example: Callback pattern

This example demonstrates using a callback to process matches:

```rust
use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
let window = vec![1, 2, 1, 2, 1];
let patterns = vec![
    PatternBuilderErased::new().value_elem(1).value_elem(2).build(),
    PatternBuilderErased::new().value_elem(2).value_elem(1).build(),
];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
let named = matcher.find_matches(&window, &patterns);
assert!(named["pattern_0"].len() > 0);
assert!(named["pattern_1"].len() > 0);
```

## Example: Advanced callback with overlap settings

This example demonstrates using advanced callback and overlap settings:

```rust
use scrolling_window_pattern_matcher::{PatternBuilder, ScrollingWindowPatternMatcherRef};
let patterns = vec![
    PatternBuilderErased::new()
        .matcher_elem(|x: &i32| *x == 1)
        .matcher_elem(|x: &i32| *x == 2)
        .callback(|matched: &[i32]| println!("Matched: {:?}", matched))
        .overlap(false)
        .build(),
    PatternBuilderErased::new()
        .matcher_elem(|x: &i32| *x == 2)
        .matcher_elem(|x: &i32| *x == 3)
        .callback(|matched: &[i32]| println!("Matched: {:?}", matched))
        .overlap(true)
        .build(),
];
let window = vec![1, 2, 3, 4];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
matcher.find_matches(&window, &patterns);
```

- No unnecessary trait bounds: PartialEq is only required for value-based patterns
- Accepts Vec, slice, or array for both window and patterns (no manual conversion needed)

## Usage: Value/Mixed Patterns with Multiple Patterns

```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem};
let window = vec![&1, &2, &1, &2, &1];
let patterns = vec![
    PatternBuilderErased::new().value_elem(1).value_elem(2).build(),
    PatternBuilderErased::new().value_elem(2).value_elem(1).build(),
    PatternBuilderErased::new().value_elem(1).build(),
    PatternBuilderErased::new().value_elem(2).build(),
];
let matcher = ScrollingWindowPatternMatcherRef::new(5);
// You can pass Vec, slice, or array for window and patterns:
let matches = matcher.find_matches(&window, &patterns, false, None::<fn(usize, usize)>);
// Or:
let matches = matcher.find_matches(window.as_slice(), patterns.as_slice(), false, None::<fn(usize, usize)>);
// Or with arrays:
let arr_window = [&1, &2, &1, &2, &1];
let arr_patterns = [
    PatternBuilderErased::new().value_elem(1).value_elem(2).build(),
    PatternBuilderErased::new().value_elem(2).value_elem(1).build(),
```rust
use scrolling_window_pattern_matcher::{ScrollingWindowPatternMatcherRef, PatternElem, Pattern, PatternBuilder};
use std::rc::Rc;
use std::cell::RefCell;
let window = vec![1, 2, 1, 2, 1];
let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
let results1 = results.clone();
let results2 = results.clone();
let patterns = vec![
    PatternBuilderErased::new()
        .value_elem(1).value_elem(2)
        .callback(move |matched| results1.borrow_mut().push(matched.to_vec()))
        .overlap(false)
        .build(),
    PatternBuilderErased::new()
        .value_elem(2).value_elem(1)
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

## Overlap Settings

- `allow_overlap_with_others`: If false, this pattern will not match if it would overlap with any previous match.
- `allow_others_to_overlap`: If false, once this pattern matches, no future matches can overlap its matched region.

## Debug Logging

This crate uses the [log](https://docs.rs/log/) crate for debug logging. To enable debug output during development, add the following to your main function or test harness:

```rust
env_logger::init();
```

Then run your program or tests with:

``` Bash
RUST_LOG=debug cargo test
```

To disable logging (e.g., in production), do not initialize a logger, or set a higher log level:

``` Bash
RUST_LOG=info cargo run
```

Debug logs provide detailed information about matcher execution, pattern matching, overlap checks, and callback invocations.

## API

- `find_matches`: Use for value or mixed patterns (requires PartialEq for T), supports multiple patterns and multi-element patterns. Accepts any type convertible to a slice for window and patterns.
- `find_named_matches`: Returns named pattern/capture results as `HashMap<String, Vec<HashMap<String, Vec<T>>>>`.

See tests for more comprehensive examples and edge cases.

## Edge Cases

- Empty window or patterns: returns no matches
- Patterns can be all values, all functions, or mixed
- Multiple patterns and multi-element patterns supported
- Deduplication and overlap settings can be combined
- Patterns of length 1 and longer are supported
- Overlap exclusion can prevent some matches (see tests)
- Window and patterns can be Vec, slice, or array
- Gaps are represented by PatternElem::Any with repeat settings

## Example: Value patterns

```rust
let patterns = vec![
    PatternBuilderErased::new().value_elem(1).value_elem(2).build(),
    PatternBuilderErased::new().value_elem(2).value_elem(1).build(),
];
let window = vec![1, 2, 1, 2, 1];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
let named = matcher.find_matches(&window, &patterns);
assert!(named["pattern_0"].len() > 0);
assert!(named["pattern_1"].len() > 0);
```

## Planned Features (Not Yet Implemented)

All currently planned features have been implemented.

If you need additional features, please open an issue so we can discuss it!

## API Reference

All major types and builders are available at the crate root:

- `Pattern`, `PatternBuilder`
- `PatternElem` (struct-style variants; gaps are represented by PatternElem::Any with repeat settings)
- `ScrollingWindowPatternMatcherRef`
- `Callback`, `SliceCallback`

See the tests and examples above for usage patterns.

This example demonstrates using a callback to process matches:

```rust
use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
let patterns = vec![
    PatternBuilderErased::new()
        .value_elem(1).value_elem(2)
        .callback(Box::new(|matched: &[i32]| println!("Matched: {:?}", matched)))
        .build(),
];
let window = vec![1, 2, 1, 2, 1];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
matcher.find_matches(&window, &patterns);
```

## Example: Function patterns

This example shows how to use function-based patterns:

```rust
use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
let patterns = vec![
    PatternBuilderErased::new()
        .matcher_elem(|x: &i32| *x == 1)
        .build(),
    PatternBuilderErased::new()
        .matcher_elem(|x: &i32| *x == 4)
        .build(),
];
let window = vec![1, 2, 3, 4];
let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
let named = matcher.find_matches(&window, &patterns);
assert!(named["pattern_0"].len() > 0);
assert!(named["pattern_1"].len() > 0);
```
