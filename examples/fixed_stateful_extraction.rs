//! Fixed Stateful Extraction using Unified API
//!
//! This example demonstrates stateful data extraction using the unified Matcher API
//! with context management and extractor functions for accumulating extracted data.

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, PatternElement,
};
use std::collections::HashMap;

/// Example context that accumulates extracted data
#[derive(Default, Debug, Clone)]
struct ExtractionContext {
    numbers: Vec<i32>,
    strings: Vec<String>,
    metadata: HashMap<String, String>,
    extraction_count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 🚀 UNIFIED: Stateful Extraction with Context Management ===\n");

    // Create our extraction context
    let context = ExtractionContext::default();

    // Demo 1: Number extraction from character sequences
    demo_number_extraction()?;

    // Demo 2: Word extraction with context
    demo_word_extraction_with_context()?;

    // Demo 3: Complex pattern with multiple extractors
    demo_complex_extraction()?;

    println!("\n=== 🎉 Unified API Success Summary ===");
    println!(
        "✅ Single Matcher Architecture: Simplified from dual StatelessMatcher/StatefulMatcher"
    );
    println!("✅ Context Support: Optional context parameter for stateful operations");
    println!("✅ Extractor Registry: Clean ID-based extractor registration");
    println!("✅ Type Safety: Full generic type safety with T and Context parameters");
    println!("✅ Performance: Efficient processing with minimal overhead");

    Ok(())
}

fn demo_number_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Number Extraction Demo:");

    let mut matcher = Matcher::<char, ExtractionContext>::new(50);

    // Set up context for number extraction
    let context = ExtractionContext::default();
    matcher.set_context(context);

    // Register number extraction extractor
    matcher.register_extractor(1, |state| {
        // Convert matched digit to number (simplified for single digits)
        let digit_char = state.current_item;
        if let Some(digit) = digit_char.to_digit(10) {
            println!(
                "   📊 Extracted number: {} at position {}",
                digit, state.position
            );
            Ok(ExtractorAction::Extract(digit_char))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut number_settings = ElementSettings::default();
    number_settings.extractor_id = Some(1);

    // Pattern to match digits
    matcher.add_pattern(PatternElement::predicate_with_settings(
        |c: &char| c.is_ascii_digit(),
        number_settings,
    ));

    let test_data = "a1b2c3d4e".chars().collect::<Vec<_>>();
    println!("   Input: {}", test_data.iter().collect::<String>());

    let results = matcher.process_items(test_data)?;
    println!("   Extracted {} digits: {:?}", results.len(), results);
    println!("   ✓ Number extraction completed\n");

    Ok(())
}

fn demo_word_extraction_with_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Word Extraction with Context:");

    let mut matcher = Matcher::<char, ExtractionContext>::new(50);

    // Set up context
    let mut context = ExtractionContext::default();
    context
        .metadata
        .insert("session".to_string(), "word_extraction".to_string());
    matcher.set_context(context);

    // Register word extraction extractor that counts letters
    matcher.register_extractor(2, |state| {
        let letter = state.current_item;
        if letter.is_ascii_alphabetic() {
            println!(
                "   📝 Found letter: '{}' at position {}",
                letter, state.position
            );
            // For word extraction, we'd accumulate characters in real context
            Ok(ExtractorAction::Extract(letter))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut word_settings = ElementSettings::default();
    word_settings.extractor_id = Some(2);

    matcher.add_pattern(PatternElement::predicate_with_settings(
        |c: &char| c.is_ascii_alphabetic(),
        word_settings,
    ));

    let test_data = "hello123world".chars().collect::<Vec<_>>();
    println!("   Input: {}", test_data.iter().collect::<String>());

    let results = matcher.process_items(test_data)?;
    println!("   Extracted {} letters: {:?}", results.len(), results);

    // Access context data (in real implementation, context would be mutable)
    if let Some(context) = matcher.context() {
        println!("   Context metadata: {:?}", context.metadata);
    }

    println!("   ✓ Word extraction with context completed\n");
    Ok(())
}

fn demo_complex_extraction() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Complex Pattern Extraction:");

    let mut matcher = Matcher::<i32, ExtractionContext>::new(100);

    // Set up complex context
    let mut context = ExtractionContext::default();
    context
        .metadata
        .insert("phase".to_string(), "complex_analysis".to_string());
    matcher.set_context(context);

    // Register multiple extractors for different value types

    // Small number extractor (1-10)
    matcher.register_extractor(10, |state| {
        let value = state.current_item;
        println!("   🔢 Small number detected: {}", value);
        Ok(ExtractorAction::Extract(value * 10)) // Amplify small numbers
    });

    // Large number extractor (>100)
    matcher.register_extractor(20, |state| {
        let value = state.current_item;
        println!("   📈 Large number detected: {}", value);
        Ok(ExtractorAction::Extract(value / 10)) // Reduce large numbers
    });

    // Pattern reset extractor
    matcher.register_extractor(30, |state| {
        println!("   ↩️ Reset trigger: value {}", state.current_item);
        Ok(ExtractorAction::Restart)
    });

    // Configure patterns with different extractors
    let mut small_settings = ElementSettings::default();
    small_settings.extractor_id = Some(10);
    matcher.add_pattern(PatternElement::predicate_with_settings(
        |&x| x >= 1 && x <= 10,
        small_settings,
    ));

    let mut large_settings = ElementSettings::default();
    large_settings.extractor_id = Some(20);
    matcher.add_pattern(PatternElement::predicate_with_settings(
        |&x| x > 100,
        large_settings,
    ));

    let mut reset_settings = ElementSettings::default();
    reset_settings.extractor_id = Some(30);
    matcher.add_pattern(PatternElement::exact_with_settings(999, reset_settings));

    let test_data = vec![5, 150, 3, 250, 999, 8, 200];
    println!("   Input: {:?}", test_data);
    println!("   Patterns: small(1-10)*10, large(>100)/10, reset(999)");

    let results = matcher.process_items(test_data)?;
    println!("   Final results: {:?}", results);

    println!("   ✓ Complex extraction completed\n");
    Ok(())
}
