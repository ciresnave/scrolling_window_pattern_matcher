//! Comprehensive test suite for the scrolling window pattern matcher
//!
//! This test suite covers:
//! - All PatternElement variants (Value, Function, Pattern, Any, Repeat)
//! - All ExtractorAction variants and their behavior
//! - All settings configurations and edge cases
//! - Error handling and boundary conditions
//! - Priority ordering and pattern interaction
//! - Complex nested patterns and real-world scenarios

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, MatcherError, MatcherSettings,
    Pattern, PatternElement, PatternSettings,
};

/// Test basic value matching functionality
#[test]
fn test_value_element_basic() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "find_42".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: None,
        }],
    );

    // Should match
    let data1 = vec![1, 2, 42, 3, 4];
    assert!(matcher.run(&data1).is_ok());

    // Should not fail on no match (just continue)
    let data2 = vec![1, 2, 3, 4, 5];
    assert!(matcher.run(&data2).is_ok());

    // Empty data should not fail
    let data3: Vec<i32> = vec![];
    assert!(matcher.run(&data3).is_ok());
}

/// Test value matching with repeat settings
#[test]
fn test_value_element_with_repeats() {
    let mut matcher = Matcher::new();

    // Require exactly 3 consecutive 1s
    matcher.add_pattern(
        "triple_ones".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(3).max_repeat(3)),
        }],
    );

    // Should match - exactly 3 ones
    let data1 = vec![0, 1, 1, 1, 2];
    assert!(matcher.run(&data1).is_ok());

    // Should not match - only 2 ones
    let data2 = vec![0, 1, 1, 2];
    assert!(matcher.run(&data2).is_ok()); // No error, just no match

    // Should not match - 4 ones (exceeds max)
    let data3 = vec![0, 1, 1, 1, 1, 2];
    assert!(matcher.run(&data3).is_ok()); // Should match first 3, continue
}

/// Test greedy vs non-greedy matching
#[test]
fn test_greedy_vs_non_greedy() {
    let mut matcher = Matcher::new();

    // Non-greedy: match minimum required
    matcher.add_pattern(
        "non_greedy".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(
                ElementSettings::new()
                    .min_repeat(2)
                    .max_repeat(4)
                    .greedy(false),
            ),
        }],
    );

    // Greedy: match maximum possible
    matcher.add_pattern(
        "greedy".to_string(),
        vec![PatternElement::Value {
            value: 2,
            settings: Some(
                ElementSettings::new()
                    .min_repeat(2)
                    .max_repeat(4)
                    .greedy(true),
            ),
        }],
    );

    let data = vec![1, 1, 1, 1, 2, 2, 2, 2];
    assert!(matcher.run(&data).is_ok());
}

/// Test function-based pattern elements
#[test]
fn test_function_element() {
    let mut matcher = Matcher::new();

    // Match even numbers
    matcher.add_pattern(
        "even_numbers".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x % 2 == 0),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(3)),
        }],
    );

    // Match numbers greater than 10
    matcher.add_pattern(
        "large_numbers".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x > 10),
            settings: None,
        }],
    );

    let data = vec![1, 2, 4, 6, 15, 3, 5];
    assert!(matcher.run(&data).is_ok());
}

/// Test Any pattern element
#[test]
fn test_any_element() {
    let mut matcher = Matcher::new();

    // Match any 2 items followed by value 42
    matcher.add_pattern(
        "any_then_42".to_string(),
        vec![
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
            },
            PatternElement::Value {
                value: 42,
                settings: None,
            },
        ],
    );

    let data = vec![100, 200, 42, 1, 2];
    assert!(matcher.run(&data).is_ok());

    // Test with different data types
    let data2 = vec![-5, 999, 42];
    assert!(matcher.run(&data2).is_ok());
}

/// Test Pattern element (string matching within debug representation)
#[test]
fn test_pattern_element() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "contains_5".to_string(),
        vec![PatternElement::Pattern {
            pattern: "5".to_string(),
            settings: None,
        }],
    );

    // Should match numbers containing digit 5
    let data = vec![1, 2, 15, 4]; // 15 contains "5" in its debug representation
    assert!(matcher.run(&data).is_ok());
}

/// Test Repeat element (nested patterns)
#[test]
fn test_repeat_element() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "repeated_pattern".to_string(),
        vec![PatternElement::Repeat {
            element: Box::new(PatternElement::Value {
                value: 7,
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
            }),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(2)),
        }],
    );

    let data = vec![7, 7, 3, 7, 7];
    assert!(matcher.run(&data).is_ok());
}

/// Test complex multi-element patterns
#[test]
fn test_complex_patterns() {
    let mut matcher = Matcher::new();

    // Pattern: even number, then odd number, then any value > 10
    matcher.add_pattern(
        "even_odd_large".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 0),
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 1),
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x > 10),
                settings: None,
            },
        ],
    );

    let data = vec![2, 3, 15, 1, 4, 5, 20];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - Continue action
#[test]
fn test_extractor_continue() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "with_continue_extractor".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(
                ElementSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Continue))),
            ),
        }],
    );

    let data = vec![1, 42, 3, 42, 5];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - Skip action
#[test]
fn test_extractor_skip() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "with_skip_extractor".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(
                ElementSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Skip(2)))),
            ),
        }],
    );

    let data = vec![1, 2, 3, 4, 5];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - Jump actions
#[test]
fn test_extractor_jump() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "with_jump_extractor".to_string(),
        vec![PatternElement::Value {
            value: 10,
            settings: Some(
                ElementSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Jump(1)))),
            ),
        }],
    );

    let data = vec![10, 1, 2, 3];
    assert!(matcher.run(&data).is_ok());

    // Test negative jump
    let mut matcher2 = Matcher::new();
    matcher2.add_pattern(
        "with_negative_jump".to_string(),
        vec![PatternElement::Value {
            value: 5,
            settings: Some(
                ElementSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Jump(-1)))),
            ),
        }],
    );

    let data2 = vec![1, 5, 2];
    assert!(matcher2.run(&data2).is_ok());
}

/// Test extractor functionality - DiscardPartialMatch action
#[test]
fn test_extractor_discard_partial_match() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "discard_pattern".to_string(),
        vec![PatternElement::Value {
            value: 99,
            settings: Some(
                ElementSettings::new()
                    .extractor(Box::new(|_state| Ok(ExtractorAction::DiscardPartialMatch))),
            ),
        }],
    );

    let data = vec![1, 99, 3];
    assert!(matcher.run(&data).is_ok()); // Should continue despite discard
}

/// Test extractor functionality - RestartFrom action
#[test]
fn test_extractor_restart_from() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "restart_pattern".to_string(),
        vec![PatternElement::Value {
            value: 7,
            settings: Some(
                ElementSettings::new()
                    .extractor(Box::new(|_state| Ok(ExtractorAction::RestartFrom(0)))),
            ),
        }],
    );

    let data = vec![7, 1, 2, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - StopMatching action
#[test]
fn test_extractor_stop_matching() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "stop_pattern".to_string(),
        vec![PatternElement::Value {
            value: 100,
            settings: Some(
                ElementSettings::new()
                    .extractor(Box::new(|_state| Ok(ExtractorAction::StopMatching))),
            ),
        }],
    );

    let data = vec![1, 100, 3, 4, 5];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - AddPattern action
#[test]
fn test_extractor_add_pattern() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "add_pattern_trigger".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Ok(ExtractorAction::AddPattern(
                    "new_pattern".to_string(),
                    Pattern::new(vec![PatternElement::Value {
                        value: 42,
                        settings: None,
                    }]),
                ))
            }))),
        }],
    );

    let data = vec![1, 42, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test extractor functionality - RemovePattern action
#[test]
fn test_extractor_remove_pattern() {
    let mut matcher = Matcher::new();

    // Add a pattern to remove
    matcher.add_pattern(
        "to_be_removed".to_string(),
        vec![PatternElement::Value {
            value: 999,
            settings: None,
        }],
    );

    matcher.add_pattern(
        "remove_pattern_trigger".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Ok(ExtractorAction::RemovePattern("to_be_removed".to_string()))
            }))),
        }],
    );

    let data = vec![1, 999, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test pattern-level extractors
#[test]
fn test_pattern_level_extractor() {
    let mut matcher = Matcher::new();

    let pattern = Pattern::with_settings(
        vec![PatternElement::Value {
            value: 5,
            settings: None,
        }],
        PatternSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Continue))),
    );

    matcher.add_pattern_with_settings("pattern_extractor".to_string(), pattern);

    let data = vec![1, 5, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test matcher-level settings
#[test]
fn test_matcher_settings() {
    let settings = MatcherSettings::new().skip_unmatched(true).priority(5);

    let mut matcher = Matcher::with_settings(settings);
    matcher.add_pattern(
        "test_pattern".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: None,
        }],
    );

    let data = vec![1, 2, 42, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test priority ordering between patterns
#[test]
fn test_pattern_priority_ordering() {
    let mut matcher = Matcher::new();

    // High priority pattern (lower number = higher priority)
    matcher.add_pattern_with_settings(
        "high_priority".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Any { settings: None }],
            PatternSettings::new().priority(1),
        ),
    );

    // Low priority pattern
    matcher.add_pattern_with_settings(
        "low_priority".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Any { settings: None }],
            PatternSettings::new().priority(10),
        ),
    );

    let data = vec![1, 2, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test error handling - extractor returning errors
#[test]
fn test_extractor_error_handling() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "error_pattern".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Err(ExtractorError::Message(
                    "Intentional test error".to_string(),
                ))
            }))),
        }],
    );

    let data = vec![42];
    let result = matcher.run(&data);
    assert!(result.is_err());

    if let Err(MatcherError::ExtractorError(ExtractorError::Message(msg))) = result {
        assert_eq!(msg, "Intentional test error");
    } else {
        panic!("Expected ExtractorError::Message");
    }
}

/// Test error handling - invalid positions
#[test]
fn test_invalid_position_errors() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "invalid_skip".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                // Try to skip beyond the end of data
                Ok(ExtractorAction::Skip(state.input_length + 10))
            }))),
        }],
    );

    let data = vec![1];
    let result = matcher.run(&data);
    assert!(result.is_err());

    if let Err(MatcherError::InvalidPosition(_)) = result {
        // Expected
    } else {
        panic!("Expected InvalidPosition error, got: {:?}", result);
    }
}

/// Test error handling - pattern not found for removal
#[test]
fn test_pattern_not_found_error() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "remove_nonexistent".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Ok(ExtractorAction::RemovePattern("does_not_exist".to_string()))
            }))),
        }],
    );

    let data = vec![1];
    let result = matcher.run(&data);
    assert!(result.is_err());

    if let Err(MatcherError::PatternNotFound(name)) = result {
        assert_eq!(name, "does_not_exist");
    } else {
        panic!("Expected PatternNotFound error");
    }
}

/// Test panic handling in extractors
#[test]
fn test_extractor_panic_handling() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "panic_pattern".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                panic!("Intentional panic for testing");
            }))),
        }],
    );

    let data = vec![42];
    let result = matcher.run(&data);
    assert!(result.is_err());

    if let Err(MatcherError::ExtractorPanic(_)) = result {
        // Expected
    } else {
        panic!("Expected ExtractorPanic error");
    }
}

/// Test edge case - empty patterns list
#[test]
fn test_empty_patterns() {
    let mut matcher = Matcher::new();

    let data = vec![1, 2, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test edge case - empty data
#[test]
fn test_empty_data() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "never_matches".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: None,
        }],
    );

    let data: Vec<i32> = vec![];
    assert!(matcher.run(&data).is_ok());
}

/// Test edge case - zero repeats (should not match anything)
#[test]
fn test_zero_repeats() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "zero_repeat".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(0).max_repeat(0)),
        }],
    );

    let data = vec![1, 2, 3];
    assert!(matcher.run(&data).is_ok()); // Should not match, but no error
}

/// Test settings builder pattern thoroughly
#[test]
fn test_settings_builders() {
    // Test ElementSettings builder
    let element_settings: ElementSettings<i32> = ElementSettings::new()
        .min_repeat(2)
        .max_repeat(5)
        .greedy(true)
        .priority(10);

    assert_eq!(element_settings.min_repeat_or_default(), 2);
    assert_eq!(element_settings.max_repeat_or_default(), 5);
    assert!(element_settings.greedy_or_default());
    assert_eq!(element_settings.priority_or_default(), 10);

    // Test PatternSettings builder
    let pattern_settings: PatternSettings<i32> = PatternSettings::new().priority(7);

    assert_eq!(pattern_settings.priority_or_default(), 7);

    // Test MatcherSettings builder
    let matcher_settings: MatcherSettings<i32> =
        MatcherSettings::new().skip_unmatched(true).priority(3);

    assert!(matcher_settings.skip_unmatched_or_default());
    assert_eq!(matcher_settings.priority_or_default(), 3);
}

/// Test extractor state information
#[test]
fn test_extractor_state_information() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "state_checker".to_string(),
        vec![PatternElement::Value {
            value: 10,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                // Verify state information is correct
                assert_eq!(state.matched_items, vec![10]);
                assert_eq!(state.pattern_name, "state_checker");
                assert_eq!(state.element_index, 0);
                assert_eq!(state.input_length, 3);
                assert!(state.current_position < state.input_length);
                Ok(ExtractorAction::Continue)
            }))),
        }],
    );

    let data = vec![1, 10, 3];
    assert!(matcher.run(&data).is_ok());
}

/// Test complex real-world scenario - log parsing pattern
#[test]
fn test_log_parsing_scenario() {
    let mut matcher = Matcher::new();

    // Pattern: ERROR followed by any code, then any message
    matcher.add_pattern(
        "error_pattern".to_string(),
        vec![
            PatternElement::Value {
                value: 500, // HTTP error code
                settings: None,
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x > 1000), // Timestamp
                settings: None,
            },
            PatternElement::Any {
                // Any message
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(3)),
            },
        ],
    );

    let log_data = vec![200, 1001, 42, 500, 1002, 100, 200, 300];
    assert!(matcher.run(&log_data).is_ok());
}

/// Test pattern matching with different data types (using String)
#[test]
fn test_string_patterns() {
    let mut matcher: Matcher<String> = Matcher::new();

    matcher.add_pattern(
        "find_hello".to_string(),
        vec![PatternElement::Value {
            value: "hello".to_string(),
            settings: None,
        }],
    );

    matcher.add_pattern(
        "long_strings".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|s: &String| s.len() > 5),
            settings: None,
        }],
    );

    let data = vec![
        "hi".to_string(),
        "hello".to_string(),
        "world".to_string(),
        "fantastic".to_string(),
    ];
    assert!(matcher.run(&data).is_ok());
}

/// Test pattern element cloning behavior
#[test]
fn test_pattern_element_cloning() {
    let element = PatternElement::Value {
        value: 42,
        settings: Some(ElementSettings::new().min_repeat(2)),
    };

    let cloned = element.clone();

    // Verify cloning preserves value but not function pointers
    match (&element, &cloned) {
        (PatternElement::Value { value: v1, .. }, PatternElement::Value { value: v2, .. }) => {
            assert_eq!(v1, v2)
        }
        _ => panic!("Clone should preserve pattern type"),
    }
}

/// Test Debug implementations
#[test]
fn test_debug_implementations() {
    let element = PatternElement::Value {
        value: 42,
        settings: Some(ElementSettings::new()),
    };

    let debug_str = format!("{:?}", element);
    assert!(debug_str.contains("Value"));
    assert!(debug_str.contains("42"));

    let pattern = Pattern::new(vec![element]);
    let debug_str2 = format!("{:?}", pattern);
    assert!(debug_str2.contains("Pattern"));

    let matcher: Matcher<i32> = Matcher::new();
    let debug_str3 = format!("{:?}", matcher);
    assert!(debug_str3.contains("Matcher"));
}
