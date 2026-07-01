// spec, раздел "Ключи → Мнемоника и seed → Каноническая wordlist"

use std::sync::OnceLock;

use crate::{sha256_raw, Hash32};

const WORDLIST_RAW: &str = include_str!("../../../../Montana wordlist.txt");

pub const WORDLIST_SIZE: usize = 2048;

// Binding fingerprint из спеки v29.9.0 «Ключи → Мнемоника и seed → Каноническая
// wordlist» (строки 2663-2666): SHA-256 файла `Montana wordlist.txt` в canonical
// encoding = concat(word_i || 0x0A) для i ∈ [0, 2047] + trailing 0x0A.
pub const WORDLIST_FINGERPRINT: Hash32 = [
    0x2f, 0x5e, 0xed, 0x53, 0xa4, 0x72, 0x7b, 0x4b, 0xf8, 0x88, 0x0d, 0x8f, 0x3f, 0x19, 0x9e, 0xfc,
    0x90, 0xe5, 0x85, 0x03, 0x64, 0x6d, 0x9f, 0xf8, 0xef, 0xf3, 0xa2, 0xed, 0x3b, 0x24, 0xdb, 0xda,
];

pub fn wordlist() -> &'static [&'static str; WORDLIST_SIZE] {
    static CACHE: OnceLock<[&'static str; WORDLIST_SIZE]> = OnceLock::new();
    CACHE.get_or_init(init_wordlist)
}

fn init_wordlist() -> [&'static str; WORDLIST_SIZE] {
    let computed = sha256_raw(WORDLIST_RAW.as_bytes());
    // Несовпадение = corruption встроенного wordlist либо wrong file.
    // Protocol violation, не runtime error.
    assert_eq!(
        computed, WORDLIST_FINGERPRINT,
        "Montana wordlist fingerprint mismatch — встроенный wordlist повреждён или заменён"
    );

    let mut arr: [&'static str; WORDLIST_SIZE] = [""; WORDLIST_SIZE];
    let mut count = 0;
    for line in WORDLIST_RAW.lines() {
        assert!(
            count < WORDLIST_SIZE,
            "wordlist has more than {WORDLIST_SIZE} lines"
        );
        arr[count] = line;
        count += 1;
    }
    assert_eq!(
        count, WORDLIST_SIZE,
        "wordlist does not have exactly {WORDLIST_SIZE} lines"
    );

    for i in 1..WORDLIST_SIZE {
        assert!(
            arr[i - 1] < arr[i],
            "wordlist not lexicographically sorted at position {i}"
        );
    }

    arr
}

pub fn word_index(word: &str) -> Option<u16> {
    let words = wordlist();
    words.binary_search(&word).ok().map(|i| i as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_matches_spec() {
        let computed = sha256_raw(WORDLIST_RAW.as_bytes());
        assert_eq!(computed, WORDLIST_FINGERPRINT);
    }

    #[test]
    fn first_word_abandon() {
        let wl = wordlist();
        assert_eq!(wl[0], "abandon");
    }

    #[test]
    fn second_word_ability() {
        let wl = wordlist();
        assert_eq!(wl[1], "ability");
    }

    #[test]
    fn last_word_zoo() {
        let wl = wordlist();
        assert_eq!(wl[2047], "zoo");
    }

    #[test]
    fn exactly_2048_words() {
        let wl = wordlist();
        assert_eq!(wl.len(), 2048);
    }

    #[test]
    fn all_lowercase_ascii() {
        let wl = wordlist();
        for (i, w) in wl.iter().enumerate() {
            assert!(
                w.bytes().all(|b| b.is_ascii_lowercase()),
                "word {i} ({w}) has non-lowercase-ASCII bytes"
            );
        }
    }

    #[test]
    fn lexicographically_sorted() {
        let wl = wordlist();
        for i in 1..WORDLIST_SIZE {
            assert!(wl[i - 1] < wl[i]);
        }
    }

    #[test]
    fn word_index_abandon_is_zero() {
        assert_eq!(word_index("abandon"), Some(0));
    }

    #[test]
    fn word_index_zoo_is_2047() {
        assert_eq!(word_index("zoo"), Some(2047));
    }

    #[test]
    fn word_index_unknown_returns_none() {
        assert_eq!(word_index("notaword"), None);
        assert_eq!(word_index(""), None);
        assert_eq!(word_index("Abandon"), None); // case-sensitive
    }
}
