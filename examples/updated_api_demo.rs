//! # Unified API Demo
//!
//! This example demonstrates the current unified API functionality showcasing
//! all features of the simplified, single-matcher architecture.

use scrolling_window_pattern_matcher::{
    ElementSettings, ExtractorAction, ExtractorError, Matcher, PatternElement,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified API Demo ===\n");

    // Demo 1: Basic Pattern Matching
    demo_basic_patterns()?;

    // Demo 2: Advanced Pattern Combinations
    demo_advanced_patterns()?;

    // Demo 3: Extractor Processing
    demo_extractor_processing()?;

    // Demo 4: Real-world Data Processing
    demo_data_processing()?;

    println!("=== All demos completed successfully! ===");
    Ok(())
}

fn demo_basic_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Basic Pattern Matching:");

    // Exact value matching
    let mut matcher = Matcher::<i32, ()>::new(10);
    matcher.add_pattern(PatternElement::exact(42));

    let data = vec![1, 42, 3, 42, 5];
    println!("   Data: {:?}", data);
    println!("   Looking for value 42...");

    let results = matcher.process_items(data)?;
    println!("   Found {} matches: {:?}", results.len(), results);

    // Predicate matching
    let mut pred_matcher = Matcher::<i32, ()>::new(10);
    pred_matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));

    let data2 = vec![1, 2, 3, 4, 5, 6];
    println!("\n   Data: {:?}", data2);
    println!("   Looking for even numbers...");

    let results2 = pred_matcher.process_items(data2)?;
    println!("   Found {} even numbers: {:?}", results2.len(), results2);

    // Range matching
    let mut range_matcher = Matcher::<i32, ()>::new(10);
    range_matcher.add_pattern(PatternElement::range(10, 20));

    let data3 = vec![5, 15, 25, 18, 30];
    println!("\n   Data: {:?}", data3);
    println!("   Looking for values in range [10, 20]...");

    let results3 = range_matcher.process_items(data3)?;
    println!(
        "   Found {} values in range: {:?}",
        results3.len(),
        results3
    );

    println!("   ✓ Basic pattern matching completed\n");
    Ok(())
}

fn demo_advanced_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Advanced Pattern Combinations:");

    // Sequence pattern: even → odd → greater than 10
    let mut seq_matcher = Matcher::<i32, ()>::new(20);
    seq_matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 0));
    seq_matcher.add_pattern(PatternElement::predicate(|x| *x % 2 == 1));
    seq_matcher.add_pattern(PatternElement::predicate(|x| *x > 10));

    let data = vec![2, 3, 15, 4, 7, 11, 8, 9, 12];
    println!("   Data: {:?}", data);
    println!("   Pattern: even → odd → greater than 10");

    let results = seq_matcher.process_items(data)?;
    println!(
        "   Found {} complete sequences, final elements: {:?}",
        results.len(),
        results
    );

    // Optional elements pattern
    let mut opt_matcher = Matcher::<i32, ()>::new(20);
    opt_matcher.add_pattern(PatternElement::exact(1));

    let mut opt_settings = ElementSettings::default();
    opt_settings.optional = true;
    opt_matcher.add_pattern(PatternElement::exact_with_settings(2, opt_settings));

    opt_matcher.add_pattern(PatternElement::exact(3));

    println!("\n   Pattern: 1 → [2 optional] → 3");

    // Test with optional element
    let data_with_opt = vec![1, 2, 3];
    let results_with = opt_matcher.process_items(data_with_opt)?;
    println!("   With optional [1,2,3]: {} matches", results_with.len());

    opt_matcher.reset();

    // Test without optional element
    let data_without_opt = vec![1, 3];
    let results_without = opt_matcher.process_items(data_without_opt)?;
    println!(
        "   Without optional [1,3]: {} matches",
        results_without.len()
    );

    println!("   ✓ Advanced patterns completed\n");
    Ok(())
}

fn demo_extractor_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Extractor Processing:");

    let mut matcher = Matcher::<i32, ()>::new(20);

    // Register data transformation extractor
    matcher.register_extractor(1, |state| {
        let value = state.current_item;
        println!(
            "   📊 Processing value: {} at position {}",
            value, state.position
        );

        // Square values greater than 5
        if value > 5 {
            Ok(ExtractorAction::Extract(value * value))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut settings = ElementSettings::default();
    settings.extractor_id = Some(1);
    matcher.add_pattern(PatternElement::predicate_with_settings(
        |x| *x > 0,
        settings,
    ));

    let data = vec![3, 7, 2, 8, 1];
    println!("   Data: {:?}", data);
    println!("   Squaring values > 5...");

    let results = matcher.process_items(data)?;
    println!("   Results: {:?}", results);

    // Restart extractor example
    let mut restart_matcher = Matcher::<i32, ()>::new(20);

    restart_matcher.register_extractor(10, |state| {
        if state.current_item == 99 {
            println!("   ↩️ Reset trigger: restarting pattern");
            Ok(ExtractorAction::Restart)
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut restart_settings = ElementSettings::default();
    restart_settings.extractor_id = Some(10);
    restart_matcher.add_pattern(PatternElement::exact_with_settings(99, restart_settings));
    restart_matcher.add_pattern(PatternElement::exact(5));

    println!("\n   Pattern: 99 (restart) → 5");
    let restart_data = vec![99, 5];

    for item in restart_data {
        if let Some(result) = restart_matcher.process_item(item)? {
            println!("   ✓ Final match: {}", result);
        }
    }

    println!("   ✓ Extractor processing completed\n");
    Ok(())
}

fn demo_data_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Real-world Data Processing:");

    // Simulating sensor data anomaly detection
    let mut sensor_matcher = Matcher::<i32, ()>::new(50);

    // Pattern: detect anomalies (values outside normal range)
    sensor_matcher.register_extractor(100, |state| {
        let value = state.current_item;
        if value < 10 || value > 90 {
            println!(
                "   🚨 ANOMALY detected: {} at position {}",
                value, state.position
            );
            Ok(ExtractorAction::Extract(value))
        } else {
            Ok(ExtractorAction::Continue)
        }
    });

    let mut anomaly_settings = ElementSettings::default();
    anomaly_settings.extractor_id = Some(100);
    sensor_matcher.add_pattern(PatternElement::predicate_with_settings(
        |&x| x < 10 || x > 90,
        anomaly_settings,
    ));

    // Simulated sensor readings
    let sensor_data = vec![45, 52, 48, 95, 51, 47, 5, 49, 53, 91, 48, 46, 51, 2, 50];

    println!("   Sensor Data: {:?}", sensor_data);
    println!("   Normal range: [10, 90]");
    println!("   Detecting anomalies...");

    let anomalies = sensor_matcher.process_items(sensor_data)?;
    println!("   Detected {} anomalies: {:?}", anomalies.len(), anomalies);

    // Financial transaction monitoring
    println!("\n   Financial Transaction Analysis:");
    let mut finance_matcher = Matcher::<i32, ()>::new(50);

    // Detect suspicious pattern: large deposit followed by small withdrawals
    finance_matcher.add_pattern(PatternElement::predicate(|&amount| amount > 10000));
    finance_matcher.add_pattern(PatternElement::predicate(|&amount| {
        amount > 0 && amount < 1000
    }));

    let transactions = vec![500, 15000, 800, 12000, 200, 18000, 300, 400];
    println!("   Transactions: {:?}", transactions);
    println!("   Pattern: large deposit (>10k) → small withdrawal (<1k)");

    let suspicious = finance_matcher.process_items(transactions)?;
    if !suspicious.is_empty() {
        println!("   🚩 Suspicious patterns detected: {:?}", suspicious);
    } else {
        println!("   ✅ No suspicious patterns found");
    }

    println!("   ✓ Real-world processing completed\n");
    Ok(())
}
