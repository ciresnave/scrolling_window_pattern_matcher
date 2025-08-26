use scrolling_window_pattern_matcher::{ElementSettings, Matcher, PatternElement};

fn main() {
    println!("=== ScrollingWindowPatternMatcher - Simplified API Demo ===\n");

    // Create a new matcher
    let mut matcher = Matcher::new();

    // Example 1: Simple value matching
    println!("1. Simple Value Matching");
    matcher.add_pattern(
        "find_42".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let data1 = vec![1, 42, 3, 42, 5];
    let results1 = matcher.run(&data1);
    println!("   Data: {:?}", data1);
    println!("   Results: {:?}\n", results1);

    // Example 2: Function-based matching
    println!("2. Function-based Matching (even numbers)");
    let mut matcher2 = Matcher::new();
    matcher2.add_pattern(
        "even_numbers".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| x % 2 == 0),
            settings: Some(
                ElementSettings::new()
                    .min_repeat(1)
                    .max_repeat(3)
                    .greedy(true),
            ),
        }],
    );

    let data2 = vec![1, 2, 4, 6, 7];
    let results2 = matcher2.run(&data2);
    println!("   Data: {:?}", data2);
    println!("   Results: {:?}\n", results2);

    // Example 3: Complex pattern with multiple elements
    println!("3. Complex Pattern (value 1, any item, value 3)");
    let mut matcher3 = Matcher::new();
    matcher3.add_pattern(
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
        ],
    );

    let data3 = vec![1, 2, 3, 4, 1, 5, 3];
    let results3 = matcher3.run(&data3);
    println!("   Data: {:?}", data3);
    println!("   Results: {:?}\n", results3);

    // Example 4: Quantifier usage (repeating patterns)
    println!("4. Quantifier Usage (2-4 consecutive 2s)");
    let mut matcher4 = Matcher::new();
    matcher4.add_pattern(
        "repeat_twos".to_string(),
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

    let data4 = vec![1, 2, 2, 2, 3];
    let results4 = matcher4.run(&data4);
    println!("   Data: {:?}", data4);
    println!("   Results: {:?}\n", results4);

    // Example 5: Basic usage (callbacks removed as not available in current API)
    println!("5. Basic Pattern Matching");
    let mut matcher5 = Matcher::new();
    matcher5.add_pattern(
        "find_99".to_string(),
        vec![PatternElement::Value {
            value: 99,
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let data5 = vec![1, 99, 3];
    let results5 = matcher5.run(&data5);
    println!("   Data: {:?}", data5);
    println!("   Results: {:?}\n", results5);

    // Example 6: Multiple patterns
    println!("6. Multiple Patterns");
    let mut matcher6 = Matcher::new();

    // Add first pattern
    matcher6.add_pattern(
        "find_large".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x > 10),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    // Add second pattern
    matcher6.add_pattern(
        "find_small".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x < 5),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
        }],
    );

    let data6 = vec![5, 15, 2, 20];
    let results6 = matcher6.run(&data6);
    println!("   Data: {:?}", data6);
    println!("   Results: {:?}\n", results6);

    // Example 7: Any element matching
    println!("7. Any Element Matching");
    let mut matcher7 = Matcher::new();
    matcher7.add_pattern(
        "any_sequence".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(3)),
            },
            PatternElement::Value {
                value: 5,
                settings: Some(ElementSettings::new().min_repeat(1).max_repeat(1)),
            },
        ],
    );

    let data7 = vec![1, 2, 3, 4, 5];
    let results7 = matcher7.run(&data7);
    println!("   Data: {:?}", data7);
    println!("   Results: {:?}\n", results7);

    println!("=== Demo Complete ===");
}
