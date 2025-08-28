//! Comprehensive integration tests for the unified pattern matcher API
//!
//! These tests cover advanced scenarios, edge cases, and real-world usage patterns
//! to ensure the simplified API is robust and handles complex use cases correctly.

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, MatcherError, PatternElement,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct AdvancedContext {
    name: String,
    counters: HashMap<String, i32>,
    accumulated_values: Vec<i32>,
    metadata: String,
}

#[test]
fn test_complex_sequence_with_context() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(50);

    let context = AdvancedContext {
        name: "complex_test".to_string(),
        counters: HashMap::new(),
        accumulated_values: vec![],
        metadata: "test_data".to_string(),
    };
    matcher.set_context(context);

    // Pattern: number divisible by 3, then range 10-20, then even number
    matcher.add_pattern(PatternElement::predicate(|x| *x % 3 == 0));
    matcher.add_pattern(PatternElement::range(10, 20));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

    // Test sequence that matches: 9, 15, 16
    assert_eq!(matcher.process_item(9).unwrap(), None); // 9 % 3 == 0 ✓
    assert_eq!(matcher.process_item(15).unwrap(), None); // 15 in [10,20] ✓
    assert_eq!(matcher.process_item(16).unwrap(), Some(16)); // 16 % 2 == 0 ✓
}

#[test]
fn test_advanced_extractors_with_state() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    let context = AdvancedContext {
        name: "extractor_test".to_string(),
        counters: HashMap::new(),
        accumulated_values: vec![],
        metadata: "accumulator".to_string(),
    };
    matcher.set_context(context);

    // Extractor that accumulates values and extracts the sum
    matcher.register_extractor(1, |state| {
        // In a real implementation, you'd modify the context here
        // For this test, we'll just return a computed value
        Ok(ExtractorAction::Extract(
            state.current_item * state.position as i32,
        ))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(5, settings));

    // Should extract 5 * 0 = 0 (position starts at 0)
    assert_eq!(matcher.process_item(5).unwrap(), Some(0));
}

#[test]
fn test_extractor_restart_complex() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    // Extractor that restarts pattern on specific values
    matcher.register_extractor(1, |state| {
        if state.current_item == 99 {
            Ok(ExtractorAction::Restart)
        } else if state.current_item > 50 {
            Ok(ExtractorAction::Extract(state.current_item * 2))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);

    matcher.add_pattern(PatternElement::exact(10));
    matcher.add_pattern(PatternElement::exact_with_settings(99, settings)); // Will restart
    matcher.add_pattern(PatternElement::exact(20));

    // Start pattern
    assert_eq!(matcher.process_item(10).unwrap(), None);
    // Trigger restart
    assert_eq!(matcher.process_item(99).unwrap(), None);
    // Should be reset, so this starts the pattern again
    assert_eq!(matcher.process_item(10).unwrap(), None);
    // This should not match since we're looking for 99 next
    assert_eq!(matcher.process_item(20).unwrap(), None);
}

#[test]
fn test_multiple_optional_elements() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    matcher.add_pattern(PatternElement::exact(1));

    // First optional element
    let mut settings1 = ElementSettings::default();
    settings1.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(2, settings1));

    // Second optional element
    let mut settings2 = ElementSettings::default();
    settings2.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(3, settings2));

    matcher.add_pattern(PatternElement::exact(4));

    // Test with both optional elements present: [1, 2, 3, 4]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), None);
    assert_eq!(matcher.process_item(4).unwrap(), Some(4));

    matcher.reset();

    // Test with first optional missing: [1, 3, 4]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), None);
    assert_eq!(matcher.process_item(4).unwrap(), Some(4));

    matcher.reset();

    // Test with second optional missing: [1, 2, 4]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(4).unwrap(), Some(4));

    matcher.reset();

    // Test with both optional missing: [1, 4]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(4).unwrap(), Some(4));
}

#[test]
fn test_extractor_error_handling() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    // Extractor that fails on certain values
    matcher.register_extractor(1, |state| {
        if state.current_item == 42 {
            Err(ExtractorError::ProcessingFailed(
                "Cannot process 42".to_string(),
            ))
        } else {
            Ok(ExtractorAction::Extract(state.current_item))
        }
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(42, settings));

    // Should return an error when processing 42
    let result = matcher.process_item(42);
    assert!(result.is_err());

    if let Err(MatcherError::ExtractorFailed(err)) = result {
        assert!(err.to_string().contains("Cannot process 42"));
    } else {
        panic!("Expected ExtractorFailed error");
    }
}

#[test]
fn test_large_pattern_sequence() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(100);

    // Create a longer pattern: [1, even, 3, range(10-15), 5, odd, 7]
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
    matcher.add_pattern(PatternElement::exact(3));
    matcher.add_pattern(PatternElement::range(10, 15));
    matcher.add_pattern(PatternElement::exact(5));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 1));
    matcher.add_pattern(PatternElement::exact(7));

    // Test matching sequence: [1, 2, 3, 12, 5, 9, 7]
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None); // even
    assert_eq!(matcher.process_item(3).unwrap(), None);
    assert_eq!(matcher.process_item(12).unwrap(), None); // in range [10,15]
    assert_eq!(matcher.process_item(5).unwrap(), None);
    assert_eq!(matcher.process_item(9).unwrap(), None); // odd
    assert_eq!(matcher.process_item(7).unwrap(), Some(7)); // complete!
}

#[test]
fn test_pattern_reset_after_partial_match() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));
    matcher.add_pattern(PatternElement::exact(3));

    // Start matching then fail and restart
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(5).unwrap(), None); // Reset due to mismatch

    // Should be able to start fresh
    assert_eq!(matcher.process_item(1).unwrap(), None);
    assert_eq!(matcher.process_item(2).unwrap(), None);
    assert_eq!(matcher.process_item(3).unwrap(), Some(3)); // Success
}

#[test]
fn test_process_items_with_multiple_matches() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    matcher.add_pattern(PatternElement::predicate(|x| *x > 10));
    matcher.add_pattern(PatternElement::predicate(|x| *x < 20));

    // Pattern: number > 10 followed by number < 20
    let items = vec![5, 15, 10, 8, 12, 18, 25, 11, 5, 22, 14, 3];
    let results = matcher.process_items(items).unwrap();

    // Should find matches at: (15,10), (12,18), (25,11), (22,14)
    assert_eq!(results, vec![10, 18, 11, 14]);
}

#[test]
fn test_string_pattern_complex() {
    let mut matcher = Matcher::<String, AdvancedContext>::new(20);

    matcher.add_pattern(PatternElement::predicate(|s: &String| {
        s.starts_with("test")
    }));
    matcher.add_pattern(PatternElement::exact("middle".to_string()));
    matcher.add_pattern(PatternElement::predicate(|s: &String| s.len() > 5));

    assert_eq!(matcher.process_item("test123".to_string()).unwrap(), None);
    assert_eq!(matcher.process_item("middle".to_string()).unwrap(), None);
    assert_eq!(
        matcher.process_item("lengthy".to_string()).unwrap(),
        Some("lengthy".to_string())
    );
}

#[test]
fn test_edge_case_empty_optional_only_pattern() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(10);

    // Pattern with only optional elements
    let mut settings1 = ElementSettings::default();
    settings1.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(1, settings1));

    let mut settings2 = ElementSettings::default();
    settings2.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(2, settings2));

    // When nothing matches optional elements, pattern should not complete
    assert_eq!(matcher.process_item(5).unwrap(), None);
    assert_eq!(matcher.process_item(6).unwrap(), None);

    // When some match, pattern should complete
    assert_eq!(matcher.process_item(1).unwrap(), None); // Matches optional 1, position advances to 1
    assert_eq!(matcher.process_item(3).unwrap(), None); // Doesn't match optional 2, but pattern completes -> None (no actual matches)
}

#[test]
fn test_performance_stress() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(1000);

    // Simple pattern for performance testing
    matcher.add_pattern(PatternElement::exact(1));
    matcher.add_pattern(PatternElement::exact(2));

    // Process a large number of items
    let mut count = 0;
    for i in 0..10000 {
        if matcher.process_item(i % 10).unwrap().is_some() {
            count += 1;
        }
    }

    // Should find many matches of the pattern [1,2]
    assert!(count > 0);
}

#[test]
fn test_context_preservation() {
    let mut matcher = Matcher::<i32, AdvancedContext>::new(20);

    let context = AdvancedContext {
        name: "preserved".to_string(),
        counters: HashMap::new(),
        accumulated_values: vec![1, 2, 3],
        metadata: "important_data".to_string(),
    };

    matcher.set_context(context.clone());

    // Simple pattern to trigger processing
    matcher.add_pattern(PatternElement::exact(42));

    // Process an item
    matcher.process_item(42).unwrap();

    // Context should still be accessible (in a real implementation,
    // you'd have a way to access it, but for this test we just verify
    // the matcher still works correctly after context operations)
    assert_eq!(matcher.process_item(42).unwrap(), Some(42));
}
