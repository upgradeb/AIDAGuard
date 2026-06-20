// T-PRX-01~05: Stream — safe length detection
use aidaguard_proxy::stream::find_safe_len;
use aidaguard_proxy::stream::IncrementalRestorer;
use aidaguard_core::replacer::PlaceholderMap;

fn make_map() -> PlaceholderMap {
    let mut map = PlaceholderMap::new();
    map.insert("13899991234", "PHONE_CN");
    map
}

#[test] fn test_find_safe_len_no_placeholder() {
    let map = make_map();
    assert_eq!(find_safe_len("normal text", &map), "normal text".len());
}
#[test] fn test_find_safe_len_partial_prefix() {
    let map = make_map();
    let text = "my phone [[PHONE_CN@";
    let safe = find_safe_len(text, &map);
    let trailing = &text[safe..];
    assert!(trailing.starts_with("[[PHONE_CN@"));
}
#[test] fn test_find_safe_len_complete_placeholder() {
    let map = make_map();
    let placeholder = map.placeholders().next().unwrap().clone();
    let text = format!("my phone {}", placeholder);
    assert_eq!(find_safe_len(&text, &map), text.len());
}
#[test] fn test_find_safe_len_complete_then_incomplete() {
    let map = make_map();
    let placeholder = map.placeholders().next().unwrap().clone();
    let text = format!("my phone {} another [", placeholder);
    let safe = find_safe_len(&text, &map);
    assert_eq!(&text[safe..], "[");
}
#[test] fn test_find_safe_len_single_bracket() {
    let map = make_map();
    let text = "text [";
    let safe = find_safe_len(text, &map);
    assert_eq!(&text[safe..], "[");
}

// T-PRX-06~15: IncrementalRestorer
#[test] fn test_incremental_restorer_new_empty() {
    let map = PlaceholderMap::new();
    let mut restorer = IncrementalRestorer::new(map);
    assert_eq!(restorer.process_chunk(""), "");
}

#[test] fn test_incremental_restorer_process_plain_text() {
    let map = PlaceholderMap::new();
    let mut restorer = IncrementalRestorer::new(map);
    assert_eq!(restorer.process_chunk("hello world"), "hello world");
}

#[test] fn test_incremental_restorer_process_complete_placeholder() {
    let mut map = PlaceholderMap::new();
    let placeholder = map.insert("secret", "PHONE");
    let mut restorer = IncrementalRestorer::new(map);
    let text = format!("my number is {}", placeholder);
    let result = restorer.process_chunk(&text);
    assert!(result.contains("secret"));
    assert!(!result.contains("[[PHONE@"));
}

#[test] fn test_incremental_restorer_process_incomplete_placeholder_split() {
    let mut map = PlaceholderMap::new();
    let placeholder = map.insert("secret", "PHONE");
    let mut restorer = IncrementalRestorer::new(map);
    // Split the placeholder across two chunks
    let mid = placeholder.len() / 2;
    let chunk1 = format!("prefix {}", &placeholder[..mid]);
    let chunk2 = &placeholder[mid..];
    let result1 = restorer.process_chunk(&chunk1);
    // First chunk: text before placeholder start is returned
    assert!(result1.starts_with("prefix "));
    let result2 = restorer.process_chunk(chunk2);
    // Second chunk: the placeholder is restored
    assert!(result2.contains("secret"));
}

#[test] fn test_incremental_restorer_process_multiple_placeholders() {
    let mut map = PlaceholderMap::new();
    let ph1 = map.insert("alice", "NAME");
    let ph2 = map.insert("bob", "NAME");
    let mut restorer = IncrementalRestorer::new(map);
    let text = format!("{} and {}", ph1, ph2);
    let result = restorer.process_chunk(&text);
    assert!(result.contains("alice"));
    assert!(result.contains("bob"));
}

#[test] fn test_incremental_restorer_finish_remaining_buffer() {
    let mut map = PlaceholderMap::new();
    let placeholder = map.insert("secret", "PHONE");
    let mut restorer = IncrementalRestorer::new(map);
    // Send only the start of the placeholder so it gets buffered
    let chunk = format!("prefix {}", &placeholder[..placeholder.len() / 2]);
    restorer.process_chunk(&chunk);
    let remaining = restorer.finish();
    // finish() returns the remaining buffered content
    // Since the placeholder is incomplete, it cannot be restored;
    // the raw buffered text (containing the partial placeholder) is returned
    assert!(!remaining.is_empty());
    assert!(remaining.contains("[["));
}

#[test] fn test_incremental_restorer_finish_empty_buffer() {
    let map = PlaceholderMap::new();
    let mut restorer = IncrementalRestorer::new(map);
    restorer.process_chunk("hello");
    // Buffer should be cleared after processing plain text
    assert_eq!(restorer.finish(), "");
}

#[test] fn test_incremental_restorer_restore_complete() {
    let mut map = PlaceholderMap::new();
    let placeholder = map.insert("secret", "PHONE");
    let restorer = IncrementalRestorer::new(map);
    let text = format!("my number is {}", placeholder);
    let result = restorer.restore_complete(&text);
    assert_eq!(result, "my number is secret");
}

#[test] fn test_incremental_restorer_unknown_placeholder_kept() {
    let map = PlaceholderMap::new();
    let mut restorer = IncrementalRestorer::new(map);
    let text = "data [[UNKNOWN@xyz]] end";
    let result = restorer.process_chunk(text);
    assert!(result.contains("[[UNKNOWN@xyz]]"));
}

#[test] fn test_incremental_restorer_brackets_not_placeholder() {
    let map = PlaceholderMap::new();
    let mut restorer = IncrementalRestorer::new(map);
    // Lone [[ without matching ]] should be buffered
    let result = restorer.process_chunk("text [[ more");
    assert!(result.starts_with("text "));
    // The [[ should remain in the buffer, finish flushes it
    let remaining = restorer.finish();
    assert!(remaining.contains("[["));
}
