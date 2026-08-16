use super::*;

#[test]
fn test_load_precompiled_map() {
    let precompiled = Precompiled::from(nmt_nfkc()).unwrap();
    let results = precompiled.trie.common_prefix_search("\u{fb01}".as_bytes());
    assert_eq!(results, vec![2130]);
    // Check the null termination
    assert_eq!(&precompiled.normalized[2130..2133], "fi\0");

    let results = precompiled.trie.common_prefix_search(b" ");
    assert!(results.is_empty());

    let results = precompiled.trie.common_prefix_search("𝔾".as_bytes());
    assert_eq!(results, vec![1786]);
    assert_eq!(&precompiled.normalized[1786..1788], "G\0");

    assert_eq!(precompiled.transform("𝔾"), Some("G"));
    assert_eq!(precompiled.transform("𝕠"), Some("o"));
    assert_eq!(precompiled.transform("\u{200d}"), Some(" "));
}

#[test]
fn test_precompiled_failure_mode() {
    let precompiled = Precompiled::from(nmt_nfkc()).unwrap();
    let original = "เขาไม่ได้พูดสักคำ".to_string();
    let normalized = "เขาไม\u{e48}ได\u{e49}พ\u{e39}ดส\u{e31}กค\u{e4d}า".to_string();
    assert_eq!(precompiled.normalize_string(&original), normalized);
}

#[test]
fn test_precompiled_hindi() {
    let precompiled = Precompiled::from(nmt_nfkc()).unwrap();
    let original = "ड़ी दुख".to_string();
    let normalized = "ड\u{93c}ी द\u{941}ख".to_string();
    assert_eq!(precompiled.normalize_string(&original), normalized);
}

#[test]
fn test_precompiled_multi_char_replace_bug() {
    let precompiled = Precompiled::from(nmt_nfkc()).unwrap();
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

#[test]
fn test_serialization() {
    let precompiled = Precompiled::from(nmt_nfkc()).unwrap();

    let string = &serde_json::to_string(&precompiled).unwrap();
    let reconstructed: Precompiled = serde_json::from_str(string).unwrap();

    assert_eq!(reconstructed, precompiled);

    assert_eq!(string, include_str!("precompiled.json"));

    let string = std::fs::read_to_string("test.json").unwrap();
    let _reconstructed2: Precompiled = serde_json::from_str(&string).unwrap();
}

fn nmt_nfkc() -> &'static [u8] {
    include_bytes!("./nmt_nfkc.bin")
}

fn overlapping_prefix_charsmap() -> Vec<u8> {
    let mut trie = vec![0u32; 256];
    trie[0] = 0;
    trie[b'a' as usize] = b'a' as u32 | (1 << 8) | (1 << 10);
    trie[96] = 0;
    trie[2] = b'b' as u32 | (1 << 8) | (1 << 10);
    trie[3] = 2;
    trie[172] = 0xcc | (1 << 10);
    trie[45] = 0x80 | (1 << 8) | (1 << 10);
    trie[44] = 2;

    let mut charsmap = Vec::with_capacity(4 + trie.len() * 4 + 4);
    charsmap.extend_from_slice(&((trie.len() * 4) as u32).to_le_bytes());
    for unit in trie {
        charsmap.extend_from_slice(&unit.to_le_bytes());
    }
    charsmap.extend_from_slice(b"x\0y\0");
    charsmap
}

#[test]
fn test_normalize_string_uses_longest_prefix_match() {
    let charsmap = overlapping_prefix_charsmap();
    let precompiled = Precompiled::from(&charsmap).unwrap();

    assert_eq!(precompiled.transform_prefix("abc"), Some((2, "y")));
    assert_eq!(precompiled.transform_prefix("a\u{0300}z"), Some((3, "y")));
    assert_eq!(precompiled.normalize_string("ab"), "y");
    assert_eq!(precompiled.normalize_string("abc"), "yc");
    assert_eq!(precompiled.normalize_string("a\u{0300}"), "y");
    assert_eq!(precompiled.normalize_string("a\u{0300}z"), "yz");
}
