use kaf_cli::parser::ChapterDetector;

#[test]
fn custom_pattern_cache_shared_across_detectors() {
    let pattern = r"^\d+\.\s+.*";
    let lines = vec!["", "1. Introduction", "This is content.", ""];

    // 第一个检测器首次编译并缓存正则
    let detector1 = ChapterDetector::new();
    let result1 = detector1.detect_chapter("1. Introduction", 1, &lines, Some(pattern));
    assert!(result1.is_some(), "第一个检测器应匹配自定义模式");
    assert!(result1.unwrap().is_match);

    // 第二个检测器应复用同一缓存，无需重新编译即可匹配
    let detector2 = ChapterDetector::new();
    let result2 = detector2.detect_chapter("2. Next Chapter", 1, &lines, Some(pattern));
    assert!(result2.is_some(), "第二个检测器应复用缓存匹配成功");
    assert!(result2.unwrap().is_match);

    // 同一模式多次调用仍成功
    let result3 = detector1.detect_chapter("3. Another", 1, &lines, Some(pattern));
    assert!(result3.is_some());
}
