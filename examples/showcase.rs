//! Comprehensive showcase of the unified pattern matcher API
//!
//! This example demonstrates all major features including:
//! - Pattern elements: exact, predicate, range
//! - Element settings: optional elements, extractors
//! - Context management and stateful processing
//! - Advanced extractors with different actions
//! - Real-world scenarios and use cases
//! - Error handling and edge cases

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, PatternElement,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ShowcaseContext {
    name: String,
    counters: HashMap<String, i32>,
    captured_data: Vec<i32>,
    metadata: String,
}

fn main() {
    println!("🔍 Scrolling Window Pattern Matcher - Comprehensive Showcase");
    println!("============================================================\n");

    // Basic pattern element demonstrations
    basic_pattern_elements();

    // Advanced pattern combinations
    advanced_combinations();

    // Extractor demonstrations
    extractor_demonstrations();

    // Optional elements showcase
    optional_elements_showcase();

    // Context and stateful processing
    context_processing();

    // Real-world scenarios
    real_world_scenarios();

    // Error handling examples
    error_handling_examples();

    println!("\n✅ All showcase examples completed successfully!");
}

fn basic_pattern_elements() {
    println!("📝 Basic Pattern Elements");
    println!("-------------------------");

    // Exact value matching
    println!("1. Exact Value Matching:");
    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);
    matcher.add_pattern(PatternElement::exact(42));

    for value in [10, 42, 30, 42] {
        if let Some(result) = matcher.process_item(value).unwrap() {
            println!("   ✓ Found exact match: {}", result);
        }
    }

    // Predicate matching
    println!("\n2. Predicate Matching (numbers > 50):");
    let mut matcher2 = Matcher::<i32, ShowcaseContext>::new(20);
    matcher2.add_pattern(PatternElement::predicate(|x| *x > 50));

    for value in [30, 60, 40, 75] {
        if let Some(result) = matcher2.process_item(value).unwrap() {
            println!("   ✓ Found value > 50: {}", result);
        }
    }

    // Range matching
    println!("\n3. Range Matching [10, 20]:");
    let mut matcher3 = Matcher::<i32, ShowcaseContext>::new(20);
    matcher3.add_pattern(PatternElement::range(10, 20));

    for value in [5, 15, 25, 18] {
        if let Some(result) = matcher3.process_item(value).unwrap() {
            println!("   ✓ Found value in range: {}", result);
        }
    }
    println!();
}

fn advanced_combinations() {
    println!("🔧 Advanced Pattern Combinations");
    println!("--------------------------------");

    // Complex sequence: even number, then range [10,30], then multiple of 5
    println!("1. Complex Sequence Pattern:");
    println!("   Pattern: even → range[10,30] → multiple of 5");

    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);
    matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
    matcher.add_pattern(PatternElement::range(10, 30));
    matcher.add_pattern(PatternElement::predicate(|x| *x % 5 == 0));

    let test_sequence = vec![2, 15, 10, 4, 20, 25, 8, 25, 15];
    println!("   Testing sequence: {:?}", test_sequence);

    for item in test_sequence {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Complete pattern match, final element: {}", result);
        }
    }

    // String pattern matching
    println!("\n2. String Pattern Matching:");
    let mut string_matcher = Matcher::<String, ShowcaseContext>::new(20);
    string_matcher.add_pattern(PatternElement::predicate(|s: &String| {
        s.starts_with("test")
    }));
    string_matcher.add_pattern(PatternElement::exact("middle".to_string()));
    string_matcher.add_pattern(PatternElement::predicate(|s: &String| s.len() > 5));

    let test_strings = vec![
        "test123".to_string(),
        "middle".to_string(),
        "lengthy_string".to_string(),
        "other".to_string(),
    ];

    println!("   Pattern: starts_with('test') → 'middle' → length > 5");
    for item in test_strings {
        if let Some(result) = string_matcher.process_item(item).unwrap() {
            println!("   ✓ String pattern complete: {}", result);
        }
    }
    println!();
}

fn extractor_demonstrations() {
    println!("⚙️ Extractor Demonstrations");
    println!("---------------------------");

    // Extract action: simple transformation
    println!("1. Transform Extractor (square values):");
    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);

    matcher.register_extractor(1, |state| {
        Ok(ExtractorAction::Extract(
            state.current_item * state.current_item,
        ))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::exact_with_settings(5, settings));

    if let Some(result) = matcher.process_item(5).unwrap() {
        println!("   ✓ Input: 5, Extracted: {} (5²)", result);
    }

    // Continue action: conditional processing
    println!("\n2. Conditional Extractor:");
    let mut matcher2 = Matcher::<i32, ShowcaseContext>::new(20);

    matcher2.register_extractor(2, |state| {
        if state.current_item > 10 {
            Ok(ExtractorAction::Extract(state.current_item * 2))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut settings2 = ElementSettings::default();
    settings2.extractor_id = Some(2);
    matcher2.add_pattern(PatternElement::exact_with_settings(15, settings2));

    if let Some(result) = matcher2.process_item(15).unwrap() {
        println!("   ✓ Input: 15 (>10), Extracted: {} (doubled)", result);
    }

    // Restart action: pattern reset
    println!("\n3. Restart Extractor:");
    let mut matcher3 = Matcher::<i32, ShowcaseContext>::new(20);

    matcher3.register_extractor(3, |state| {
        if state.current_item == 99 {
            Ok(ExtractorAction::Restart)
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut settings3 = ElementSettings::default();
    settings3.extractor_id = Some(3);
    matcher3.add_pattern(PatternElement::exact_with_settings(99, settings3));
    matcher3.add_pattern(PatternElement::exact(1));

    println!("   Pattern: 99 (restart) → 1");
    assert_eq!(matcher3.process_item(99).unwrap(), None); // Should restart
    assert_eq!(matcher3.current_position(), 0); // Should be reset
    println!("   ✓ Pattern restarted successfully");
    println!();
}

fn optional_elements_showcase() {
    println!("🔄 Optional Elements Showcase");
    println!("-----------------------------");

    println!("1. Single Optional Element:");
    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);

    matcher.add_pattern(PatternElement::exact(1));

    let mut settings = ElementSettings::default();
    settings.optional = true;
    matcher.add_pattern(PatternElement::exact_with_settings(2, settings));

    matcher.add_pattern(PatternElement::exact(3));

    println!("   Pattern: 1 → [2 optional] → 3");

    // With optional element
    println!("   Testing [1, 2, 3] (optional present):");
    for item in [1, 2, 3] {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Match: {}", result);
        }
    }

    matcher.reset();

    // Without optional element
    println!("   Testing [1, 3] (optional missing):");
    for item in [1, 3] {
        if let Some(result) = matcher.process_item(item).unwrap() {
            println!("   ✓ Match: {}", result);
        }
    }

    println!("\n2. Multiple Optional Elements:");
    let mut matcher2 = Matcher::<i32, ShowcaseContext>::new(20);

    matcher2.add_pattern(PatternElement::exact(10));

    let mut opt1 = ElementSettings::default();
    opt1.optional = true;
    matcher2.add_pattern(PatternElement::exact_with_settings(20, opt1));

    let mut opt2 = ElementSettings::default();
    opt2.optional = true;
    matcher2.add_pattern(PatternElement::exact_with_settings(30, opt2));

    matcher2.add_pattern(PatternElement::exact(40));

    println!("   Pattern: 10 → [20 optional] → [30 optional] → 40");

    // Test various combinations
    let test_cases = vec![
        vec![10, 20, 30, 40], // All present
        vec![10, 20, 40],     // Second optional missing
        vec![10, 30, 40],     // First optional missing
        vec![10, 40],         // Both optional missing
    ];

    for test_case in test_cases {
        println!("   Testing {:?}:", test_case);
        for item in test_case {
            if let Some(result) = matcher2.process_item(item).unwrap() {
                println!("   ✓ Match: {}", result);
            }
        }
        matcher2.reset();
    }
    println!();
}

fn context_processing() {
    println!("🗃️ Context and Stateful Processing");
    println!("----------------------------------");

    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);

    let context = ShowcaseContext {
        name: "context_demo".to_string(),
        counters: HashMap::new(),
        captured_data: vec![],
        metadata: "processing_demo".to_string(),
    };

    matcher.set_context(context);

    // Extractor that uses position information
    matcher.register_extractor(10, |state| {
        let multiplier = (state.position + 1) as i32;
        Ok(ExtractorAction::Extract(state.current_item * multiplier))
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(10);
    matcher.add_pattern(PatternElement::exact_with_settings(7, settings));

    println!("1. Position-aware Extractor:");
    println!("   Extractor multiplies value by (position + 1)");

    if let Some(result) = matcher.process_item(7).unwrap() {
        println!("   ✓ Input: 7, Position: 0, Result: {} (7 × 1)", result);
    }
    println!();
}

fn real_world_scenarios() {
    println!("🌍 Real-world Scenarios");
    println!("----------------------");

    // HTTP status code analysis
    println!("1. HTTP Status Code Analysis:");
    let mut http_matcher = Matcher::<i32, ShowcaseContext>::new(50);

    // Pattern: Client error (4xx) followed by server error (5xx)
    http_matcher.add_pattern(PatternElement::predicate(|&code| code >= 400 && code < 500));
    http_matcher.add_pattern(PatternElement::predicate(|&code| code >= 500 && code < 600));

    let status_codes = vec![200, 404, 500, 403, 502, 200, 401, 503];
    println!("   Status codes: {:?}", status_codes);
    println!("   Looking for pattern: 4xx → 5xx");

    let results = http_matcher.process_items(status_codes).unwrap();
    println!(
        "   ✓ Found {} error sequences: {:?}",
        results.len(),
        results
    );

    // Network port scanning detection
    println!("\n2. Network Port Scanning Detection:");
    let mut port_matcher = Matcher::<i32, ShowcaseContext>::new(50);

    port_matcher.register_extractor(20, |state| {
        println!(
            "   🚨 Potential port scan detected on port {}",
            state.current_item
        );
        Ok(ExtractorAction::Extract(state.current_item))
    });

    let mut port_settings = ElementSettings::default();
    port_settings.extractor_id = Some(20);

    // Detect high ports being accessed sequentially
    port_matcher.add_pattern(PatternElement::predicate_with_settings(
        |&port| port > 8000 && port < 9000,
        port_settings,
    ));

    let network_activity = vec![80, 443, 8080, 8081, 8082, 22, 8083];
    println!("   Network activity: {:?}", network_activity);

    for port in network_activity {
        port_matcher.process_item(port).unwrap();
    }

    // Financial transaction pattern
    println!("\n3. Financial Transaction Pattern:");
    let mut fin_matcher = Matcher::<i32, ShowcaseContext>::new(50);

    // Pattern: Large deposit followed by rapid small withdrawals
    fin_matcher.add_pattern(PatternElement::predicate(|&amount| amount > 10000));
    fin_matcher.add_pattern(PatternElement::predicate(|&amount| {
        amount > 0 && amount < 1000
    }));

    let transactions = vec![500, 15000, 200, 12000, 800, 300];
    println!("   Transactions: {:?}", transactions);
    println!("   Pattern: large deposit (>10k) → small withdrawal (<1k)");

    let suspicious = fin_matcher.process_items(transactions).unwrap();
    if !suspicious.is_empty() {
        println!("   🚨 Suspicious pattern detected: {:?}", suspicious);
    }
    println!();
}

fn error_handling_examples() {
    println!("❌ Error Handling Examples");
    println!("-------------------------");

    // Extractor error handling
    println!("1. Extractor Error Handling:");
    let mut matcher = Matcher::<i32, ShowcaseContext>::new(20);

    matcher.register_extractor(100, |state| {
        if state.current_item == 0 {
            Err(ExtractorError::ProcessingFailed(
                "Division by zero".to_string(),
            ))
        } else {
            Ok(ExtractorAction::Extract(100 / state.current_item))
        }
    });

    let mut error_settings = ElementSettings::default();
    error_settings.extractor_id = Some(100);
    matcher.add_pattern(PatternElement::exact_with_settings(0, error_settings));

    println!("   Testing division by zero extractor:");
    match matcher.process_item(0) {
        Ok(_) => println!("   ❌ Expected error but got success"),
        Err(e) => println!("   ✓ Correctly caught error: {}", e),
    }

    // No patterns error
    println!("\n2. No Patterns Error:");
    let mut empty_matcher = Matcher::<i32, ShowcaseContext>::new(20);

    match empty_matcher.process_item(42) {
        Ok(_) => println!("   ❌ Expected error but got success"),
        Err(e) => println!("   ✓ Correctly caught error: {}", e),
    }
    println!();
}
