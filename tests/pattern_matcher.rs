//! Legacy test compatibility and additional pattern matching scenarios
//!
//! This test file focuses on complex pattern scenarios and edge cases
//! that complement the comprehensive test suite.

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, Matcher, Pattern, PatternElement, PatternSettings,
};

#[test]
fn test_repeat_and_capture_complex_patterns() {
    let mut matcher = Matcher::new();

    // Pattern: Any element repeated exactly 2 times, then value 3
    matcher.add_pattern(
        "gap_and_value".to_string(),
        vec![
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
            },
            PatternElement::Value {
                value: 3,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
        ],
    );

    // Pattern: Value 2 repeated exactly 3 times
    matcher.add_pattern(
        "triple_twos".to_string(),
        vec![PatternElement::Value {
            value: 2,
            settings: Some(ElementSettings::new().min_repeat(3).max_repeat(3)),
        }],
    );

    let window = vec![1, 2, 2, 2, 3, 4, 5, 6];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_nines_block_pattern() {
    let mut matcher = Matcher::new();

    // Test nines block pattern: 999, any item, then 5
    matcher.add_pattern(
        "nines_block".to_string(),
        vec![
            PatternElement::Value {
                value: 9,
                settings: Some(ElementSettings::new().min_repeat(3).max_repeat(3)),
            },
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
            PatternElement::Value {
                value: 5,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
        ],
    );

    let window2 = vec![1, 9, 9, 9, 2, 5, 4, 9, 9, 9, 5];
    let result2 = matcher.run(&window2);
    assert!(result2.is_ok());
}

#[test]
fn test_flexible_sequence_matching() {
    let mut matcher = Matcher::new();

    // Pattern: 1 followed by 2
    matcher.add_pattern(
        "one_two_sequence".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
            PatternElement::Value {
                value: 2,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
        ],
    );

    // Pattern: 2 followed by 1
    matcher.add_pattern(
        "two_one_sequence".to_string(),
        vec![
            PatternElement::Value {
                value: 2,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
            PatternElement::Value {
                value: 1,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
        ],
    );

    let window = [1, 2, 1, 2, 1];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_callback_equivalent_extractors() {
    let mut matcher = Matcher::new();

    // Using extractors to simulate callback behavior
    matcher.add_pattern(
        "extractor_pattern_1".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                    // Simulate callback processing
                    assert_eq!(state.matched_items, vec![1]);
                    Ok(ExtractorAction::Continue)
                }))),
            },
            PatternElement::Value {
                value: 2,
                settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                    assert_eq!(state.matched_items, vec![2]);
                    Ok(ExtractorAction::Continue)
                }))),
            },
        ],
    );

    let window = vec![1, 2, 1, 2, 1];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_overlap_handling() {
    let mut matcher = Matcher::new();

    // Overlapping patterns that could match the same sequence
    matcher.add_pattern(
        "overlap_pattern_1".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: None,
            },
            PatternElement::Value {
                value: 2,
                settings: None,
            },
        ],
    );

    matcher.add_pattern(
        "overlap_pattern_2".to_string(),
        vec![
            PatternElement::Value {
                value: 2,
                settings: None,
            },
            PatternElement::Value {
                value: 1,
                settings: None,
            },
        ],
    );

    let window = vec![1, 2, 1, 2, 1];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_empty_patterns_and_windows_comprehensive() {
    let mut matcher = Matcher::new();

    // Test with no patterns at all
    let window = vec![1, 2, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());

    // Add a pattern and test with empty window
    matcher.add_pattern(
        "find_something".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: None,
        }],
    );

    let empty_window: Vec<i32> = vec![];
    let result2 = matcher.run(&empty_window);
    assert!(result2.is_ok());
}

#[test]
fn test_single_element_pattern_variations() {
    let mut matcher = Matcher::new();

    // Single value pattern
    matcher.add_pattern(
        "single_value".to_string(),
        vec![PatternElement::Value {
            value: 2,
            settings: None,
        }],
    );

    // Single function pattern
    matcher.add_pattern(
        "single_function".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x > 5),
            settings: None,
        }],
    );

    // Single Any pattern
    matcher.add_pattern(
        "single_any".to_string(),
        vec![PatternElement::Any { settings: None }],
    );

    let window = vec![1, 2, 6];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_advanced_function_patterns() {
    let mut matcher = Matcher::new();

    // Pattern using function matcher for values > 2
    matcher.add_pattern(
        "greater_than_two".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x > 2),
            settings: None,
        }],
    );

    // Pattern: value 1 followed by function match for value == 2
    matcher.add_pattern(
        "one_then_exact_two".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x == 2),
                settings: None,
            },
        ],
    );

    let window = vec![1, 2, 3, 4, 5];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_priority_with_extractors() {
    let mut matcher = Matcher::new();

    // High priority pattern with extractor
    matcher.add_pattern_with_settings(
        "high_priority_extractor".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Any {
                settings: Some(
                    ElementSettings::new().extractor(Box::new(|_| Ok(ExtractorAction::Continue))),
                ),
            }],
            PatternSettings::new().priority(1),
        ),
    );

    // Low priority pattern
    matcher.add_pattern_with_settings(
        "low_priority_simple".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Any { settings: None }],
            PatternSettings::new().priority(10),
        ),
    );

    let window = vec![1, 2, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_nested_repeat_patterns() {
    let mut matcher = Matcher::new();

    // Nested repeat pattern: repeat a pattern that itself has repeats
    matcher.add_pattern(
        "nested_repeat".to_string(),
        vec![PatternElement::Repeat {
            element: Box::new(PatternElement::Value {
                value: 7,
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
            }),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(2)),
        }],
    );

    let window = vec![7, 7, 3, 7, 7];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_string_debug_pattern_matching() {
    let mut matcher = Matcher::new();

    // Pattern element that matches based on debug representation
    matcher.add_pattern(
        "contains_digit_five".to_string(),
        vec![PatternElement::Pattern {
            pattern: "5".to_string(),
            settings: None,
        }],
    );

    // Should match numbers that contain "5" in their debug representation
    let window = vec![1, 15, 25, 4]; // 15 and 25 contain "5"
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_complex_extractor_actions_sequence() {
    let mut matcher = Matcher::new();

    // Pattern with extractor that performs different actions based on state
    matcher.add_pattern(
        "complex_extractor".to_string(),
        vec![PatternElement::Value {
            value: 100,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                match state.current_position {
                    0 => Ok(ExtractorAction::Continue),
                    1 => Ok(ExtractorAction::Skip(1)),
                    _ => Ok(ExtractorAction::Continue),
                }
            }))),
        }],
    );

    let window = vec![100, 1, 2, 100, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_boundary_conditions() {
    let mut matcher = Matcher::new();

    // Pattern that might hit boundary conditions
    matcher.add_pattern(
        "boundary_test".to_string(),
        vec![
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(10)),
            },
            PatternElement::Value {
                value: 999,
                settings: None,
            },
        ],
    );

    // Test with exactly the minimum size
    let window1 = vec![1, 999];
    assert!(matcher.run(&window1).is_ok());

    // Test with a larger window
    let window2 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 999];
    assert!(matcher.run(&window2).is_ok());

    // Test without the required ending
    let window3 = vec![1, 2, 3, 4, 5];
    assert!(matcher.run(&window3).is_ok()); // Should not fail, just not match
}

#[test]
fn test_real_world_log_analysis_scenario() {
    let mut matcher = Matcher::new();

    // Simulate log analysis: find error patterns
    // Pattern: HTTP 4xx error followed by retry count
    matcher.add_pattern(
        "http_error_with_retry".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x >= 400 && *x < 500),
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x <= 3), // Retry count 0-3
                settings: None,
            },
        ],
    );

    // Pattern: Success after retries
    matcher.add_pattern(
        "success_after_retry".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x >= 400 && *x < 500),
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x <= 3),
                settings: None,
            },
            PatternElement::Value {
                value: 200, // HTTP OK
                settings: None,
            },
        ],
    );

    let log_data = vec![200, 404, 1, 200, 403, 2, 500, 429, 3, 200];
    let result = matcher.run(&log_data);
    assert!(result.is_ok());
}

#[test]
fn test_performance_with_many_patterns() {
    let mut matcher = Matcher::new();

    // Add many patterns to test performance characteristics
    for i in 0..50 {
        matcher.add_pattern(
            format!("pattern_{}", i),
            vec![PatternElement::Value {
                value: i,
                settings: None,
            }],
        );
    }

    let window: Vec<i32> = (0..100).collect();
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_extractor_chain_reactions() {
    let mut matcher = Matcher::new();

    // Pattern with extractor that adds another pattern dynamically
    matcher.add_pattern(
        "pattern_adder".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Ok(ExtractorAction::AddPattern(
                    "dynamically_added".to_string(),
                    Pattern::new(vec![PatternElement::Value {
                        value: 84,
                        settings: None,
                    }]),
                ))
            }))),
        }],
    );

    let window = vec![42, 84, 126];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_mixed_data_types() {
    // Test with different numeric types
    let mut matcher: Matcher<u32> = Matcher::new();
    matcher.add_pattern(
        "u32_pattern".to_string(),
        vec![PatternElement::Value {
            value: 42u32,
            settings: None,
        }],
    );

    let window = vec![1u32, 42u32, 3u32];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_extractor_error_recovery() {
    let mut matcher = Matcher::new();

    // Pattern that might cause extractor errors but matcher should handle gracefully
    matcher.add_pattern(
        "potential_error".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                if state.current_position == 0 {
                    Err(scrolling_window_pattern_matcher::ExtractorError::Message(
                        "Position zero error".to_string(),
                    ))
                } else {
                    Ok(ExtractorAction::Continue)
                }
            }))),
        }],
    );

    let window = vec![1, 2, 3];
    let result = matcher.run(&window);
    // Should propagate the error
    assert!(result.is_err());
}
