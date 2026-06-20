// T-PRX-01~05: Stream — safe length detection
use aidaguard_proxy::stream::find_safe_len;
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
