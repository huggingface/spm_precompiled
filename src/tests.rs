use super::*;

#[test]
fn test_serialization() {
    let precompiled = Precompiled::from(&nmt_nfkc()).unwrap();

    let string = &serde_json::to_string(&precompiled).unwrap();
    let reconstructed: Precompiled = serde_json::from_str(string).unwrap();

    assert_eq!(reconstructed, precompiled)
}

#[test]
fn test_load_precompiled_map() {
    let precompiled = Precompiled::from(&nmt_nfkc()).unwrap();
    let results = precompiled
        .trie
        .common_prefix_search(&"\u{fb01}".as_bytes());
    assert_eq!(results, vec![2130]);
    // Check the null termination
    assert_eq!(&precompiled.normalized[2130..2133], "fi\0");

    let results = precompiled.trie.common_prefix_search(b" ");
    assert!(results.is_empty());

    let results = precompiled.trie.common_prefix_search(&"𝔾".as_bytes());
    assert_eq!(results, vec![1786]);
    assert_eq!(&precompiled.normalized[1786..1788], "G\0");

    assert_eq!(precompiled.transform(&"𝔾"), Some("G"));
    assert_eq!(precompiled.transform(&"𝕠"), Some("o"));
    assert_eq!(precompiled.transform(&"\u{200d}"), Some(" "));
}

#[test]
fn test_precompiled_failure_mode() {
    let precompiled = Precompiled::from(&nmt_nfkc()).unwrap();
    let original = "เขาไม่ได้พูดสักคำ".to_string();
    let normalized = "เขาไม\u{e48}ได\u{e49}พ\u{e39}ดส\u{e31}กค\u{e4d}า".to_string();
    assert_eq!(precompiled.normalize_string(&original), normalized);
}

#[test]
fn test_precompiled_hindi() {
    let precompiled = Precompiled::from(&nmt_nfkc()).unwrap();
    let original = "ड़ी दुख".to_string();
    let normalized = "ड\u{93c}ी द\u{941}ख".to_string();
    assert_eq!(precompiled.normalize_string(&original), normalized);
}

#[test]
fn test_precompiled_multi_char_replace_bug() {
    let precompiled = Precompiled::from(&nmt_nfkc()).unwrap();
    // آپ
    let original_bytes = vec![0xd8, 0xa7, 0xd9, 0x93];
    let results = precompiled.trie.common_prefix_search(&original_bytes);
    assert_eq!(results, vec![4050]);
    let original = String::from_utf8(original_bytes).unwrap();
    // This grapheme is actually 2 chars.
    let normalized = "آ".to_string();

    assert_eq!(&precompiled.normalized[4050..4053], "آ\0");
    assert_eq!(precompiled.normalize_string(&original), normalized);
}
