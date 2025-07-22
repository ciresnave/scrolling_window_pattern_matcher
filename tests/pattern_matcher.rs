#[test]
fn test_repeat_and_gap_and_capture_name() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window = vec![1, 2, 2, 2, 3, 4, 5, 6];
    let patterns = vec![
        PatternBuilderErased::new()
            .name("gap_and_value")
            .any_elem()
            .min_repeat(2)
            .max_repeat(2)
            .value_elem(3)
            .capture_name("three")
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
    let _named = matcher.find_matches(&window, &patterns);
    // Add a triple_twos pattern for coverage
    let triple_twos = PatternBuilderErased::new()
        .name("triple_twos")
        .value_elem(2)
        .min_repeat(3)
        .max_repeat(3)
        .capture_name("twos")
        .build();
    let named2 = matcher.find_matches(&window, &[triple_twos]);
    assert!(named2.contains_key("triple_twos"));
    let twos_matches = &named2["triple_twos"];
    assert!(twos_matches.iter().any(|m| m["twos"] == vec![2,2,2]));

    // nines_block pattern
    let window2 = vec![1, 9, 9, 9, 2, 5, 4, 9, 9, 9, 5];
    let nines_block = PatternBuilderErased::new()
        .name("nines_block")
        .value_elem(9)
        .min_repeat(3)
        .max_repeat(3)
        .capture_name("nines")
        .any_elem()
        .min_repeat(1)
        .max_repeat(1)
        .value_elem(5)
        .capture_name("five")
        .build();
    let matcher2 = ScrollingWindowPatternMatcherRef::new(window2.len());
    let named3 = matcher2.find_matches(&window2, &[nines_block]);
    assert!(named3.contains_key("nines_block"));
    let nines_matches = &named3["nines_block"];
    assert!(nines_matches.iter().any(|m| m["nines"] == vec![9,9,9] && m["five"] == vec![5]));
}

#[test]
fn test_find_matches_flexible_array_and_refs() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window = [1, 2, 1, 2, 1];
    let patterns = vec![
        PatternBuilderErased::new()
            .value_elem(1)
            .value_elem(2)
            .build(),
        PatternBuilderErased::new()
            .value_elem(2)
            .value_elem(1)
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(window.len());
    let named = matcher.find_matches_flexible(&window, &patterns);
    assert!(named["pattern_0"].len() > 0);
    assert!(named["pattern_1"].len() > 0);
}


#[test]
fn test_callback_invocation() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    use std::rc::Rc;
    use std::cell::RefCell;
    let window = vec![1, 2, 1, 2, 1];
    let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
    let results1 = results.clone();
    let results2 = results.clone();
    let patterns = vec![
        PatternBuilderErased::new()
            .value_elem(1)
            .value_elem(2)
            .callback(Box::new(move |matched: &[i32]| results1.borrow_mut().push(matched.to_vec())))
            .overlap(false)
            .build(),
        PatternBuilderErased::new()
            .value_elem(2)
            .value_elem(1)
            .callback(Box::new(move |matched: &[i32]| results2.borrow_mut().push(matched.to_vec())))
            .overlap(true)
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    matcher.find_matches(&window, &patterns);
    let results = results.borrow();
    assert!(results.contains(&vec![1, 2]));
    assert!(results.contains(&vec![2, 1]));
}

#[test]
fn test_overlap_and_deduplication() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window = vec![1, 2, 1, 2, 1];
    let patterns = vec![
        PatternBuilderErased::new()
            .value_elem(1)
            .value_elem(2)
            .overlap(false)
            .deduplication(false)
            .build(),
        PatternBuilderErased::new()
            .value_elem(2)
            .value_elem(1)
            .overlap(true)
            .deduplication(true)
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    let named = matcher.find_matches(&window, &patterns);
    assert!(named["pattern_0"].len() > 0);
    assert!(named["pattern_1"].len() > 0);
}

#[test]
fn test_empty_patterns_and_windows() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window: Vec<i32> = vec![];
    let patterns = vec![PatternBuilderErased::new().value_elem(1).build()];
    let matcher = ScrollingWindowPatternMatcherRef::new(1);
    let named = matcher.find_matches(&window, &patterns);
    assert!(named.is_empty());

    let window = vec![1, 2, 3];
    let patterns: Vec<_> = vec![];
    let matcher = ScrollingWindowPatternMatcherRef::new(3);
    let named2 = matcher.find_matches(&window, &patterns);
    assert!(named2.is_empty());
}

#[test]
fn test_single_element_patterns() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window = vec![1, 2, 3];
    let patterns = vec![PatternBuilderErased::new().value_elem(2).build()];
    let matcher = ScrollingWindowPatternMatcherRef::new(3);
    let named = matcher.find_matches(&window, &patterns);
    assert!(named["pattern_0"].len() > 0);
}

#[test]
fn test_builder_pattern_and_edge_cases() {
    use scrolling_window_pattern_matcher::{PatternBuilderErased, ScrollingWindowPatternMatcherRef};
    let window = vec![1, 2, 3, 4, 5];
    let patterns = vec![
        PatternBuilderErased::new()
            .matcher_elem(|x: &i32| *x > 2)
            .build(),
        PatternBuilderErased::new()
            .value_elem(1)
            .matcher_elem(|x: &i32| *x == 2)
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    let named = matcher.find_matches(&window, &patterns);
    assert!(named["pattern_0"].len() > 0);
    assert!(named["pattern_1"].len() > 0);
}
