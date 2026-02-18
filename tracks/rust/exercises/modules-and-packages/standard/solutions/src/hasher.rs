// url-shortener/src/hasher.rs
//
// The hasher module owns the URL shortening algorithm.
//
// This is a pure function — no state, no side effects. It's in its own module
// because:
//   1. The algorithm is a distinct concern from storage or CLI parsing
//   2. We might want to swap it out (different hash, collision detection)
//      without touching storage.rs
//   3. It's independently testable

/// Shorten a URL to a 6-character alphanumeric key.
///
/// Uses a djb2-inspired hash function. The same URL always produces the
/// same key (deterministic). Different URLs produce different keys with
/// high probability (no collision detection in this implementation).
pub fn shorten(url: &str) -> String {
    let hash = djb2_hash(url);
    encode_base36(hash, 6)
}

/// djb2-inspired hash. Simple, fast, good distribution for short strings.
/// Private — this is an implementation detail of `shorten`.
fn djb2_hash(input: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// Encode a u64 as a base-36 string of exactly `length` characters.
/// Uses lowercase letters and digits: a-z, 0-9.
/// Private — implementation detail.
fn encode_base36(mut n: u64, length: usize) -> String {
    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let base = charset.len() as u64;

    let mut key = String::with_capacity(length);
    for _ in 0..length {
        key.push(charset[(n % base) as usize]);
        n /= base;
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shorten_is_deterministic() {
        let url = "https://example.com/very/long/path?query=value";
        assert_eq!(shorten(url), shorten(url));
    }

    #[test]
    fn test_shorten_produces_six_chars() {
        let key = shorten("https://example.com");
        assert_eq!(key.len(), 6, "key should be exactly 6 characters, got '{}'", key);
    }

    #[test]
    fn test_shorten_only_alphanumeric() {
        let key = shorten("https://example.com/path");
        assert!(
            key.chars().all(|c| c.is_ascii_alphanumeric()),
            "key should only contain alphanumeric characters, got '{}'",
            key
        );
    }

    #[test]
    fn test_shorten_different_urls_different_keys() {
        let k1 = shorten("https://example.com/a");
        let k2 = shorten("https://example.com/b");
        assert_ne!(k1, k2, "different URLs should produce different keys");
    }

    #[test]
    fn test_shorten_empty_string() {
        // Should not panic — empty input produces a valid (though useless) key
        let key = shorten("");
        assert_eq!(key.len(), 6);
    }
}
