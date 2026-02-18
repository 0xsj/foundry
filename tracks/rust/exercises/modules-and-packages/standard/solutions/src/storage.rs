// url-shortener/src/storage.rs
//
// The storage module owns the in-memory URL store.
//
// Key decisions:
// - `entries` is private — external code cannot reach into the HashMap.
//   All operations go through the public methods. This preserves our freedom
//   to change the storage backend later (swap HashMap for a DB) without
//   breaking callers.
// - `resolve` returns `Option<&str>` — borrowed from the store, zero allocation.
// - `len` is a simple accessor rather than exposing the HashMap itself.

use std::collections::HashMap;

/// An in-memory key-value store mapping short keys to original URLs.
pub struct Store {
    // Private — encapsulated. External code uses `insert`, `resolve`, `remove`.
    entries: HashMap<String, String>,
}

impl Store {
    /// Create a new empty store.
    pub fn new() -> Store {
        Store {
            entries: HashMap::new(),
        }
    }

    /// Insert a key-URL pair. Overwrites any existing entry for this key.
    pub fn insert(&mut self, key: String, url: String) {
        self.entries.insert(key, url);
    }

    /// Resolve a key to its original URL. Returns `None` if the key doesn't exist.
    ///
    /// Returns a borrowed `&str` — no allocation. The returned reference is
    /// tied to the lifetime of `&self`, so the store must outlive the reference.
    pub fn resolve(&self, key: &str) -> Option<&str> {
        // HashMap::get with &str works on String keys via the Borrow trait.
        self.entries.get(key).map(|s| s.as_str())
    }

    /// Remove a key from the store. Returns true if the key existed.
    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    /// Returns the number of entries in the store.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// Default implementation delegates to new().
impl Default for Store {
    fn default() -> Store {
        Store::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_resolve() {
        let mut store = Store::new();
        store.insert("abc123".to_string(), "https://example.com".to_string());

        assert_eq!(store.resolve("abc123"), Some("https://example.com"));
        assert_eq!(store.resolve("missing"), None);
    }

    #[test]
    fn test_remove_existing() {
        let mut store = Store::new();
        store.insert("k1".to_string(), "https://a.com".to_string());

        assert!(store.remove("k1"));
        assert_eq!(store.resolve("k1"), None);
    }

    #[test]
    fn test_remove_missing_returns_false() {
        let mut store = Store::new();
        assert!(!store.remove("does-not-exist"));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut store = Store::new();
        assert!(store.is_empty());

        store.insert("k1".to_string(), "https://a.com".to_string());
        store.insert("k2".to_string(), "https://b.com".to_string());

        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_insert_overwrites() {
        let mut store = Store::new();
        store.insert("k1".to_string(), "https://original.com".to_string());
        store.insert("k1".to_string(), "https://updated.com".to_string());

        assert_eq!(store.resolve("k1"), Some("https://updated.com"));
        assert_eq!(store.len(), 1); // still just one entry
    }
}
