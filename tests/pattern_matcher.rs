use scrolling_window_pattern_matcher::*;

#[test]
fn test_value_only_patterns() {
    let window = vec![1, 2, 1, 2, 1];
    let patterns = vec![
        Pattern::new(vec![PatternElem::Value(1), PatternElem::Value(2)]),
        Pattern::new(vec![PatternElem::Value(2), PatternElem::Value(1)]),
        Pattern::new(vec![PatternElem::Value(1)]),
        Pattern::new(vec![PatternElem::Value(2)]),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.contains(&(0, 0))); // [1,2] at 0
    assert!(matches.contains(&(1, 1))); // [2,1] at 1
    assert!(matches.contains(&(2, 2))); // [1] at 2
    assert!(matches.contains(&(3, 3))); // [2] at 3
}

#[test]
fn test_mixed_value_function_patterns() {
    let window = vec![1, 2, 3, 4];
    let patterns = vec![
        Pattern::new(vec![PatternElem::Matcher(Box::new(|x: &i32| *x == 1))]),
        Pattern::new(vec![PatternElem::Matcher(Box::new(|x: &i32| *x == 4))]),
        Pattern::new(vec![PatternElem::Value(2), PatternElem::Matcher(Box::new(|x: &i32| *x == 3))]),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(4);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.contains(&(0, 0)));
    assert!(matches.contains(&(3, 1)));
    assert!(matches.contains(&(1, 2)));
}

#[test]
fn test_callback_invocation() {
    use std::rc::Rc;
    use std::cell::RefCell;
    let window = vec![1, 2, 1, 2, 1];
    let results: Rc<RefCell<Vec<Vec<i32>>>> = Rc::new(RefCell::new(vec![]));
    let results1 = results.clone();
    let results2 = results.clone();
    let patterns = vec![
        PatternBuilder::new()
            .pattern(vec![PatternElem::Value(1), PatternElem::Value(2)])
            .callback(Box::new(move |matched: &[i32]| results1.borrow_mut().push(matched.to_vec())))
            .overlap(false)
            .build(),
        PatternBuilder::new()
            .pattern(vec![PatternElem::Value(2), PatternElem::Value(1)])
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
    let window = vec![1, 2, 1, 2, 1];
    let patterns = vec![
        PatternBuilder::new()
            .pattern(vec![PatternElem::Value(1), PatternElem::Value(2)])
            .overlap(false)
            .deduplication(false)
            .build(),
        PatternBuilder::new()
            .pattern(vec![PatternElem::Value(2), PatternElem::Value(1)])
            .overlap(true)
            .deduplication(true)
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    let matches = matcher.find_matches(&window, &patterns);
    // Overlap exclusion prevents some matches
    assert!(matches.contains(&(0, 0)));
    assert!(matches.contains(&(1, 1)));
}

#[test]
fn test_empty_patterns_and_windows() {
    let window: Vec<i32> = vec![];
    let patterns = vec![Pattern::new(vec![PatternElem::Value(1)])];
    let matcher = ScrollingWindowPatternMatcherRef::new(1);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.is_empty());

    let window = vec![1, 2, 3];
    let patterns: Vec<Pattern<i32>> = vec![];
    let matcher = ScrollingWindowPatternMatcherRef::new(3);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.is_empty());
}

#[test]
fn test_single_element_patterns() {
    let window = vec![1, 2, 3];
    let patterns = vec![Pattern::new(vec![PatternElem::Value(2)])];
    let matcher = ScrollingWindowPatternMatcherRef::new(3);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.contains(&(1, 0)));
}

#[test]
fn test_builder_pattern_and_edge_cases() {
    let window = vec![1, 2, 3, 4, 5];
    let patterns = vec![
        PatternBuilder::new()
            .pattern(vec![PatternElem::Matcher(Box::new(|x: &i32| *x > 2))])
            .build(),
        PatternBuilder::new()
            .pattern(vec![PatternElem::Value(1), PatternElem::Matcher(Box::new(|x: &i32| *x == 2))])
            .build(),
    ];
    let matcher = ScrollingWindowPatternMatcherRef::new(5);
    let matches = matcher.find_matches(&window, &patterns);
    assert!(matches.contains(&(2, 0)));
    assert!(matches.contains(&(0, 1)));
}
