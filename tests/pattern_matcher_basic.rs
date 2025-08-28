//! Basic integration tests for the unified pattern matcher API
//!
//! These tests verify that the simplified API works correctly for basic
//! pattern matching scenarios.

use scrolling_window_pattern_matcher::{ElementSettings, ExtractorAction, Matcher, PatternElement};

#[derive(Debug, Clone)]
struct TestContext {
    name: String,
    captured_values: Vec<i32>,
}

#[test]
fn test_exact_value_match() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::exact(42));

    // Single exact match
    assert_eq!(matcher.process_item(42).unwrap(), Some(42));
    assert_eq!(matcher.process_item(43).unwrap(), None);
}

#[test]
fn test_predicate_match() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

    // Should match even numbers
    assert_eq!(matcher.process_item(2).unwrap(), Some(2));
    assert_eq!(matcher.process_item(4).unwrap(), Some(4));
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), None);
}

#[test]
fn test_range_match() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::range(10, 20));

    // Should match values in range [10, 20]
    assert_eq!(matcher.process_item(10).unwrap(), Some(10));
    assert_eq!(matcher.process_item(15).unwrap(), Some(15));
    assert_eq!(matcher.process_item(20).unwrap(), Some(20));
    assert_eq!(matcher.process_item(9).unwrap(), None);
    assert_eq!(matcher.process_item(21).unwrap(), None);
}

#[test]
fn test_sequence_pattern() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));
    matcher.add_pattern(PatternElement::exact(3));

    // Should match sequence [1, 2, 3]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), Some(3));

    // Reset and try again
    matcher.reset();
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), Some(3));
}

#[test]
fn test_pattern_mismatch_reset() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));

    // Start matching then fail
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), None); // Should reset

    // Try again
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), Some(2)); // Should succeed
}

#[test]
fn test_optional_elements() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    matcher.add_pattern(PatternElement::exact(1));

    let mut settings = ElementSettings::default();
    settings.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(2, settings));

    matcher.add_pattern(PatternElement::exact(3));

    // Test with optional element present
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), Some(3));

    matcher.reset();

    // Test with optional element missing
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), Some(3));
}

#[test]
fn test_extractors() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    // Register an extractor that doubles the value
    matcher.register_extractor(1, |state| {
        Ok(ExtractorAction::Extract(state.current_item * 2))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(10, settings));

    // Should extract doubled value
    assert_eq!(matcher.process_item(10).unwrap(), Some(20));
}

#[test]
fn test_context_usage() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    let context = TestContext {
        name: "test_context".to_string(),
        captured_values: vec![],
    };
    matcher.set_context(context);

    // Register an extractor that uses context
    matcher.register_extractor(1, |state| {
        Ok(ExtractorAction::Extract(state.current_item + 100))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(5, settings));

    assert_eq!(matcher.process_item(5).unwrap(), Some(105));
}

#[test]
fn test_process_multiple_items() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));

    let items = vec![1, 2, 3, 1, 2, 4, 5];
    let results = matcher.process_items(items).unwrap();

    // Should find pattern [1,2] twice
    assert_eq!(results, vec![2, 2]);
}

#[test]
fn test_string_patterns() {
    let mut matcher = Matcher::<String, TestContext>::new(10);
    matcher.add_pattern(PatternElement::exact("hello".to_string()));
    matcher.add_pattern(PatternElement::exact("world".to_string()));

    assert_eq!(matcher.process_item("hello".to_string()).unwrap(), None);
    assert_eq!(
        matcher.process_item("world".to_string()).unwrap(),
        Some("world".to_string())
    );
}

#[test]
fn test_complex_mixed_patterns() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    // Pattern: exact(1), even number, range(10-20)
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
    matcher.add_pattern(PatternElement::range(10, 20));

    assert_eq!(matcher.process_item(1).unwrap(), None); // Match first
    assert_eq!(matcher.process_item(8).unwrap(), None); // Match second
    assert_eq!(matcher.process_item(15).unwrap(), Some(15)); // Complete pattern
}

#[test]
fn test_window_size_constructor() {
    let patterns = vec![PatternElement::exact(1), PatternElement::exact(2)];

    let matcher = Matcher::<i32, TestContext>::with_patterns(patterns, 20);
    assert_eq!(matcher.window_size(), 20);
    assert_eq!(matcher.pattern_count(), 2);
}

#[test]
fn test_no_patterns_error() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    // Should error when trying to process without any patterns
    let result = matcher.process_item(1);
    assert!(result.is_err());
}

#[test]
fn test_extractor_restart_action() {
    let mut matcher = Matcher::<i32, TestContext>::new(10);

    // Register an extractor that restarts on value 5
    matcher.register_extractor(1, |state| {
        if state.current_item == 5 {
            Ok(ExtractorAction::Restart)
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(5, settings));

    // Should restart and return None
    assert_eq!(matcher.process_item(5).unwrap(), None);
    assert_eq!(matcher.current_position(), 0); // Should be reset
}
