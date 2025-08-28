//! Simple demo showcasing the unified pattern matcher API
//!
//! This example demonstrates basic usage patterns with the simplified API.

use scrolling_window_pattern_matcher::{ElementSettings, ExtractorAction, Matcher, PatternElement};

#[derive(Debug, Clone)]
struct DemoContext {
    name: String,
    processed_count: usize,
}

fn main() {
    println!("=== ScrollingWindowPatternMatcher - Unified API Demo ===\n");

    // Example 1: Simple exact value matching
    println!("1. Simple Exact Value Matching");
    simple_exact_matching();

    // Example 2: Predicate-based matching
    println!("2. Predicate-based Matching");
    predicate_matching();

    // Example 3: Range matching
    println!("3. Range Matching");
    range_matching();

    // Example 4: Sequence patterns
    println!("4. Sequence Patterns");
    sequence_patterns();

    // Example 5: Optional elements
    println!("5. Optional Elements");
    optional_elements();

    // Example 6: Extractors
    println!("6. Data Extractors");
    data_extractors();

    // Example 7: Processing multiple items
    println!("7. Processing Item Streams");
    stream_processing();

    println!("=== Demo Complete ===");
}

fn simple_exact_matching() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);
    matcher.add_pattern(PatternElement::exact(42));

    let test_items = vec![1, 42, 3, 42, 5];
    println!("   Testing items: {:?}", test_items);

    for item in test_items {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Found match: {}", result);
        }
    }
    println!();
}

fn predicate_matching() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

    let test_items = vec![1, 2, 3, 4, 5, 6];
    println!("   Testing items: {:?}", test_items);
    println!("   Looking for even numbers:");

    for item in test_items {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Found even number: {}", result);
        }
    }
    println!();
}

fn range_matching() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);
    matcher.add_pattern(PatternElement::range(10, 20));

    let test_items = vec![5, 15, 25, 12, 8];
    println!("   Testing items: {:?}", test_items);
    println!("   Looking for numbers in range [10, 20]:");

    for item in test_items {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Found number in range: {}", result);
        }
    }
    println!();
}

fn sequence_patterns() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);

    // Pattern: 1 followed by 2 followed by 3
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));
    matcher.add_pattern(PatternElement::exact(3));

    let test_items = vec![1, 2, 3, 4, 1, 2, 3, 5];
    println!("   Testing items: {:?}", test_items);
    println!("   Looking for sequence [1, 2, 3]:");

    for item in test_items {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Found complete sequence, final element: {}", result);
        }
    }
    println!();
}

fn optional_elements() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);

    // Pattern: 1, optionally 2, then 3
    matcher.add_pattern(PatternElement::exact(1));

    let mut settings = ElementSettings::default();
    settings.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(2, settings));

    matcher.add_pattern(PatternElement::exact(3));

    println!("   Pattern: 1, (optional 2), 3");

    // Test with optional element present
    let test1 = vec![1, 2, 3];
    println!("   Testing {:?} (optional present):", test1);
    for item in test1 {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Match found: {}", result);
        }
    }

    matcher.reset();

    // Test with optional element missing
    let test2 = vec![1, 3];
    println!("   Testing {:?} (optional missing):", test2);
    for item in test2 {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Match found: {}", result);
        }
    }
    println!();
}

fn data_extractors() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);

    // Register an extractor that doubles the value
    matcher.register_extractor(1, |state| {
        Ok(ExtractorAction::Extract(state.current_item * 2))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(10, settings));

    println!("   Testing extractor that doubles the value:");
    if let Some(result) = matcher.process_item(10).unwrap() {
        println!("   ✓ Input: 10, Extracted: {}", result);
    }
    println!();
}

fn stream_processing() {
    let mut matcher = Matcher::<i32, DemoContext>::new(10);

    // Pattern: any even number followed by any odd number
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 1));

    let stream = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    println!("   Processing stream: {:?}", stream);
    println!("   Looking for pattern: even followed by odd");

    let results = matcher.process_items(stream).unwrap();
    println!("   ✓ Found {} matches: {:?}", results.len(), results);
    println!();
}
