use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, PatternElement,
};

#[test]
fn test_value_pattern_match() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "find_3".to_string(),
        vec![PatternElement::Value {
            value: 3,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let window = vec![1, 2, 3, 4, 5];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_wildcard_pattern_match() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "any_item".to_string(),
        vec![PatternElement::Any {
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let window = vec![1, 2, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_custom_matcher() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "even_numbers".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x % 2 == 0),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let window = vec![1, 2, 3, 4, 5];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_repeat_quantifier() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "repeated_ones".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
        }],
    );

    let window = vec![1, 1, 2, 2, 3, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_patterns() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "find_1".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    matcher.add_pattern(
        "find_2".to_string(),
        vec![PatternElement::Value {
            value: 2,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let window = vec![1, 2, 1, 2, 1];
    let result = matcher.run(&window);

    // Should successfully match without errors
    assert!(result.is_ok());
}

#[test]
fn test_empty_window() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "find_1".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let window: Vec<i32> = vec![];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_extractor_functionality() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "find_with_extractor".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(
                ElementSettings::new().extractor(Box::new(|_state| Ok(ExtractorAction::Continue))),
            ),
        }],
    );

    let window = vec![1, 42, 3, 42, 5];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_settings_builder() {
    let settings: ElementSettings<i32> = ElementSettings::new()
        .min_repeat(2)
        .max_repeat(5)
        .greedy(true)
        .priority(10);

    assert_eq!(settings.min_repeat_or_default(), 2);
    assert_eq!(settings.max_repeat_or_default(), 5);
    assert!(settings.greedy_or_default());
    assert_eq!(settings.priority_or_default(), 10);
}

#[test]
fn test_error_handling() {
    let mut matcher = Matcher::new();

    matcher.add_pattern(
        "error_pattern".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_| {
                Err(ExtractorError::Message("Test error".to_string()))
            }))),
        }],
    );

    let window = vec![42];
    let result = matcher.run(&window);
    assert!(result.is_err());
}

#[test]
fn test_pattern_element_debug() {
    let element = PatternElement::Value {
        value: 42,
        settings: Some(ElementSettings::new()),
    };

    let debug_str = format!("{:?}", element);
    assert!(debug_str.contains("Value"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_complex_pattern_sequence() {
    let mut matcher = Matcher::new();

    // Pattern: even number followed by value 5, then any item
    matcher.add_pattern(
        "complex_sequence".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 0),
                settings: None,
            },
            PatternElement::Value {
                value: 5,
                settings: None,
            },
            PatternElement::Any { settings: None },
        ],
    );

    let window = vec![2, 5, 8, 1, 3];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_pattern_not_matching() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "find_999".to_string(),
        vec![PatternElement::Value {
            value: 999,
            settings: None,
        }],
    );

    let window = vec![1, 2, 3, 4, 5];
    let result = matcher.run(&window);
    // Should succeed even when no patterns match
    assert!(result.is_ok());
}

#[test]
fn test_greedy_matching() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "greedy_ones".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(
                ElementSettings::new()
                    .min_repeat(1)
                    .max_repeat(4)
                    .greedy(true),
            ),
        }],
    );

    let window = vec![1, 1, 1, 1, 2];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}

#[test]
fn test_non_greedy_matching() {
    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "non_greedy_ones".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(
                ElementSettings::new()
                    .min_repeat(1)
                    .max_repeat(4)
                    .greedy(false),
            ),
        }],
    );

    let window = vec![1, 1, 1, 1, 2];
    let result = matcher.run(&window);
    assert!(result.is_ok());
}
