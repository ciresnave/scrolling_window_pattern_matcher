//! Comprehensive demonstration of the Scrolling Window Pattern Matcher
//!
//! This example showcases all major features of the library including:
//! - All PatternElement types (Value, Function, Pattern, Any, Repeat)
//! - Settings configuration with builder patterns
//! - Extractor functions for dynamic behavior modification
//! - Priority ordering and pattern interaction
//! - Error handling and edge cases
//! - Real-world use case scenarios

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, MatcherSettings, Pattern,
    PatternElement, PatternSettings,
};

fn main() {
    println!("🔍 Scrolling Window Pattern Matcher - Comprehensive Showcase");
    println!("============================================================\n");

    // Basic value matching
    basic_value_matching();

    // Function-based matching
    function_based_matching();

    // Advanced pattern combinations
    advanced_pattern_combinations();

    // Extractor demonstrations
    extractor_demonstrations();

    // Priority and settings showcase
    priority_and_settings_showcase();

    // Real-world scenarios
    real_world_scenarios();

    // Error handling examples
    error_handling_examples();

    println!("\n✅ All examples completed successfully!");
}

fn basic_value_matching() {
    println!("📝 Basic Value Matching");
    println!("-----------------------");

    let mut matcher = Matcher::new();

    // Simple value matching
    matcher.add_pattern(
        "find_42".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: None,
        }],
    );

    // Value with repeat settings
    matcher.add_pattern(
        "triple_sevens".to_string(),
        vec![PatternElement::Value {
            value: 7,
            settings: Some(ElementSettings::new().min_repeat(3).max_repeat(3)),
        }],
    );

    let data = vec![1, 7, 7, 7, 42, 8, 9, 42];
    match matcher.run(&data) {
        Ok(()) => println!("✓ Successfully matched patterns in: {:?}", data),
        Err(e) => println!("✗ Error: {}", e),
    }
    println!();
}

fn function_based_matching() {
    println!("🔧 Function-Based Matching");
    println!("--------------------------");

    let mut matcher = Matcher::new();

    // Match even numbers
    matcher.add_pattern(
        "even_numbers".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x % 2 == 0),
            settings: Some(
                ElementSettings::new()
                    .min_repeat(1)
                    .max_repeat(3)
                    .greedy(true),
            ),
        }],
    );

    // Match prime numbers (simple check for small numbers)
    matcher.add_pattern(
        "small_primes".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| {
                let n = *x;
                if n < 2 {
                    return false;
                }
                if n == 2 {
                    return true;
                }
                if n % 2 == 0 {
                    return false;
                }
                for i in 3..=(n as f64).sqrt() as i32 {
                    if n % i == 0 {
                        return false;
                    }
                }
                true
            }),
            settings: None,
        }],
    );

    // Match numbers in range
    matcher.add_pattern(
        "range_10_to_20".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x >= 10 && *x <= 20),
            settings: Some(ElementSettings::new().min_repeat(2).max_repeat(5)),
        }],
    );

    let data = vec![2, 4, 6, 3, 5, 7, 11, 13, 15, 18, 20, 25];
    match matcher.run(&data) {
        Ok(()) => println!("✓ Function patterns matched in: {:?}", data),
        Err(e) => println!("✗ Error: {}", e),
    }
    println!();
}

fn advanced_pattern_combinations() {
    println!("🎯 Advanced Pattern Combinations");
    println!("--------------------------------");

    let mut matcher = Matcher::new();

    // Complex sequence: even number, then odd number, then any number > 10
    matcher.add_pattern(
        "even_odd_large_sequence".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 0),
                settings: Some(ElementSettings::new().priority(1)),
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x % 2 == 1),
                settings: Some(ElementSettings::new().priority(1)),
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x > 10),
                settings: Some(ElementSettings::new().priority(1)),
            },
        ],
    );

    // Pattern with Any elements
    matcher.add_pattern(
        "any_sandwich".to_string(),
        vec![
            PatternElement::Value {
                value: 1,
                settings: None,
            },
            PatternElement::Any {
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(4)),
            },
            PatternElement::Value {
                value: 9,
                settings: None,
            },
        ],
    );

    // Nested Repeat pattern
    matcher.add_pattern(
        "nested_pattern".to_string(),
        vec![PatternElement::Repeat {
            element: Box::new(PatternElement::Value {
                value: 5,
                settings: Some(ElementSettings::new().min_repeat(2).max_repeat(2)),
            }),
            settings: Some(ElementSettings::new().min_repeat(1).max_repeat(2)),
        }],
    );

    // Pattern using debug string matching
    matcher.add_pattern(
        "contains_three".to_string(),
        vec![PatternElement::Pattern {
            pattern: "3".to_string(),
            settings: None,
        }],
    );

    let data = vec![2, 3, 15, 1, 7, 8, 9, 5, 5, 13, 23, 33];
    match matcher.run(&data) {
        Ok(()) => println!("✓ Advanced patterns matched in: {:?}", data),
        Err(e) => println!("✗ Error: {}", e),
    }
    println!();
}

fn extractor_demonstrations() {
    println!("⚡ Extractor Function Demonstrations");
    println!("-----------------------------------");

    // Example 1: Continue extractor
    demonstrate_continue_extractor();

    // Example 2: Skip extractor
    demonstrate_skip_extractor();

    // Example 3: Dynamic pattern addition
    demonstrate_dynamic_pattern_addition();

    // Example 4: Jump and restart
    demonstrate_jump_and_restart();

    println!();
}

fn demonstrate_continue_extractor() {
    println!("  📊 Continue Extractor (logging matches):");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "logged_matches".to_string(),
        vec![PatternElement::Value {
            value: 100,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                println!(
                    "    🎯 Match found at position {}: {:?}",
                    state.current_position, state.matched_items
                );
                Ok(ExtractorAction::Continue)
            }))),
        }],
    );

    let data = vec![1, 100, 2, 100, 3];
    match matcher.run(&data) {
        Ok(()) => println!("    ✓ Logged all matches"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn demonstrate_skip_extractor() {
    println!("  ⏭️ Skip Extractor (skipping ahead):");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "skip_pattern".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                println!(
                    "    🦘 Skipping 2 positions from {}",
                    state.current_position
                );
                Ok(ExtractorAction::Skip(2))
            }))),
        }],
    );

    let data = vec![1, 2, 3, 4, 5];
    match matcher.run(&data) {
        Ok(()) => println!("    ✓ Skip operation completed"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn demonstrate_dynamic_pattern_addition() {
    println!("  🔄 Dynamic Pattern Addition:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "pattern_creator".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                println!("    ➕ Adding new pattern for value 84");
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

    let data = vec![42, 84, 126];
    match matcher.run(&data) {
        Ok(()) => println!("    ✓ Dynamic pattern addition successful"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn demonstrate_jump_and_restart() {
    println!("  🎯 Jump and Restart Operations:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "jump_pattern".to_string(),
        vec![PatternElement::Value {
            value: 10,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                if state.current_position < 3 {
                    println!("    ↩️ Restarting from position 0");
                    Ok(ExtractorAction::RestartFrom(0))
                } else {
                    println!("    ➡️ Jumping forward by 1");
                    Ok(ExtractorAction::Jump(1))
                }
            }))),
        }],
    );

    let data = vec![10, 1, 2, 10, 4];
    match matcher.run(&data) {
        Ok(()) => println!("    ✓ Jump operations completed"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn priority_and_settings_showcase() {
    println!("🏆 Priority and Settings Showcase");
    println!("---------------------------------");

    let mut matcher =
        Matcher::with_settings(MatcherSettings::new().skip_unmatched(true).priority(1));

    // High priority pattern
    matcher.add_pattern_with_settings(
        "high_priority".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Any {
                settings: Some(ElementSettings::new().priority(1)),
            }],
            PatternSettings::new().priority(1),
        ),
    );

    // Medium priority pattern with extractor
    matcher.add_pattern_with_settings(
        "medium_priority".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Value {
                value: 50,
                settings: Some(
                    ElementSettings::new()
                        .priority(5)
                        .extractor(Box::new(|state| {
                            println!(
                                "  🎖️ Medium priority pattern matched: {:?}",
                                state.matched_items
                            );
                            Ok(ExtractorAction::Continue)
                        })),
                ),
            }],
            PatternSettings::new().priority(5),
        ),
    );

    // Low priority pattern
    matcher.add_pattern_with_settings(
        "low_priority".to_string(),
        Pattern::with_settings(
            vec![PatternElement::Function {
                function: Box::new(|x: &i32| *x > 100),
                settings: Some(ElementSettings::new().priority(10)),
            }],
            PatternSettings::new().priority(10),
        ),
    );

    let data = vec![1, 50, 150, 2, 50];
    match matcher.run(&data) {
        Ok(()) => println!("✓ Priority-ordered matching completed: {:?}", data),
        Err(e) => println!("✗ Error: {}", e),
    }
    println!();
}

fn real_world_scenarios() {
    println!("🌍 Real-World Scenarios");
    println!("-----------------------");

    // Scenario 1: Log Analysis
    log_analysis_scenario();

    // Scenario 2: Network Traffic Analysis
    network_traffic_analysis();

    // Scenario 3: Financial Transaction Monitoring
    financial_transaction_monitoring();

    println!();
}

fn log_analysis_scenario() {
    println!("  📋 Log Analysis - HTTP Error Detection:");

    let mut matcher = Matcher::new();

    // Pattern: HTTP client error (4xx) followed by retry attempt
    matcher.add_pattern(
        "client_error_retry".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x >= 400 && *x < 500),
                settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                    println!(
                        "    ⚠️ HTTP Client Error detected: {}",
                        state.matched_items[0]
                    );
                    Ok(ExtractorAction::Continue)
                }))),
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x <= 3), // Retry count
                settings: None,
            },
        ],
    );

    // Pattern: Critical server error (5xx)
    matcher.add_pattern(
        "server_error".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x >= 500 && *x < 600),
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                println!(
                    "    🚨 CRITICAL: Server Error detected: {}",
                    state.matched_items[0]
                );
                Ok(ExtractorAction::Continue)
            }))),
        }],
    );

    // Simulated log data: [status_code, retry_count, status_code, ...]
    let log_data = vec![200, 404, 1, 200, 500, 503, 2, 200];
    match matcher.run(&log_data) {
        Ok(()) => println!("    ✓ Log analysis completed"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn network_traffic_analysis() {
    println!("  🌐 Network Traffic Analysis - Suspicious Patterns:");

    let mut matcher = Matcher::new();

    // Pattern: Port scanning detection (rapid consecutive port access)
    matcher.add_pattern(
        "port_scan".to_string(),
        vec![PatternElement::Function {
            function: Box::new(|x: &i32| *x > 1000 && *x < 65536), // Port range
            settings: Some(
                ElementSettings::new()
                    .min_repeat(5)
                    .max_repeat(10)
                    .extractor(Box::new(|state| {
                        println!(
                            "    🔍 Potential port scan detected: {} consecutive port accesses",
                            state.matched_items.len()
                        );
                        Ok(ExtractorAction::Continue)
                    })),
            ),
        }],
    );

    // Simulated network data (port numbers)
    let network_data = vec![80, 443, 1001, 1002, 1003, 1004, 1005, 1006, 22, 25];
    match matcher.run(&network_data) {
        Ok(()) => println!("    ✓ Network analysis completed"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn financial_transaction_monitoring() {
    println!("  💰 Financial Transaction Monitoring:");

    let mut matcher = Matcher::new();

    // Pattern: Large transaction followed by multiple small transactions (suspicious)
    matcher.add_pattern(
        "structuring_pattern".to_string(),
        vec![
            PatternElement::Function {
                function: Box::new(|x: &i32| *x > 10000), // Large transaction
                settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                    println!("    💸 Large transaction detected: ${}", state.matched_items[0]);
                    Ok(ExtractorAction::Continue)
                }))),
            },
            PatternElement::Function {
                function: Box::new(|x: &i32| *x > 0 && *x < 1000), // Small transactions
                settings: Some(ElementSettings::new()
                    .min_repeat(3)
                    .max_repeat(10)
                    .extractor(Box::new(|state| {
                        println!("    🚩 Potential structuring: {} small transactions after large one",
                                state.matched_items.len());
                        Ok(ExtractorAction::Continue)
                    }))
                ),
            },
        ],
    );

    // Simulated transaction amounts
    let transactions = vec![500, 15000, 900, 800, 700, 600, 2000];
    match matcher.run(&transactions) {
        Ok(()) => println!("    ✓ Transaction monitoring completed"),
        Err(e) => println!("    ✗ Error: {}", e),
    }
}

fn error_handling_examples() {
    println!("🚨 Error Handling Examples");
    println!("--------------------------");

    // Example 1: Extractor error
    demonstrate_extractor_error();

    // Example 2: Invalid position error
    demonstrate_invalid_position_error();

    // Example 3: Pattern not found error
    demonstrate_pattern_not_found_error();

    // Example 4: Panic handling
    demonstrate_panic_handling();

    println!();
}

fn demonstrate_extractor_error() {
    println!("  ❌ Extractor Error Example:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "error_prone".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Err(ExtractorError::Message(
                    "Intentional error for demonstration".to_string(),
                ))
            }))),
        }],
    );

    let data = vec![42];
    match matcher.run(&data) {
        Ok(()) => println!("    ❓ Unexpected success"),
        Err(e) => println!("    ✓ Expected error caught: {}", e),
    }
}

fn demonstrate_invalid_position_error() {
    println!("  📍 Invalid Position Error Example:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "invalid_skip".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|state| {
                // Try to skip beyond data length
                Ok(ExtractorAction::Skip(state.input_length + 10))
            }))),
        }],
    );

    let data = vec![1];
    match matcher.run(&data) {
        Ok(()) => println!("    ❓ Unexpected success"),
        Err(e) => println!("    ✓ Expected error caught: {}", e),
    }
}

fn demonstrate_pattern_not_found_error() {
    println!("  🔍 Pattern Not Found Error Example:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "pattern_remover".to_string(),
        vec![PatternElement::Value {
            value: 1,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                Ok(ExtractorAction::RemovePattern(
                    "nonexistent_pattern".to_string(),
                ))
            }))),
        }],
    );

    let data = vec![1];
    match matcher.run(&data) {
        Ok(()) => println!("    ❓ Unexpected success"),
        Err(e) => println!("    ✓ Expected error caught: {}", e),
    }
}

fn demonstrate_panic_handling() {
    println!("  💥 Panic Handling Example:");

    let mut matcher = Matcher::new();
    matcher.add_pattern(
        "panic_prone".to_string(),
        vec![PatternElement::Value {
            value: 42,
            settings: Some(ElementSettings::new().extractor(Box::new(|_state| {
                panic!("Intentional panic for demonstration");
            }))),
        }],
    );

    let data = vec![42];
    match matcher.run(&data) {
        Ok(()) => println!("    ❓ Unexpected success"),
        Err(e) => println!("    ✓ Panic caught and handled: {}", e),
    }
}
