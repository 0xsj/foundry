// URL Shortener — Monolithic Starter
//
// This file is intentionally a big ball of mud. Everything is in one place:
// the data store, the hashing algorithm, the CLI parsing, and the analytics.
//
// Your job: break this into a library crate with four modules:
//   storage  — the Store struct
//   hasher   — the shorten() function
//   api      — Command enum and parse_args()
//   stats    — the Stats struct
//
// Then expose a clean public API from src/lib.rs using pub use re-exports.
//
// Run (as-is):  cargo run -- shorten https://example.com/long/path
// Run tests:    cargo test

use std::collections::HashMap;

// ============================================================
// STORAGE — in-memory URL store
// ============================================================

// TODO: Move to src/storage.rs
// Make the `entries` field private and expose operations via methods.

struct Store {
    entries: HashMap<String, String>,  // key -> original URL
    access_count: HashMap<String, u64>, // key -> number of resolves
}

impl Store {
    fn new() -> Store {
        Store {
            entries: HashMap::new(),
            access_count: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, url: String) {
        self.entries.insert(key.clone(), url);
        self.access_count.insert(key, 0);
    }

    fn resolve(&mut self, key: &str) -> Option<&str> {
        if self.entries.contains_key(key) {
            let count = self.access_count.entry(key.to_string()).or_insert(0);
            *count += 1;
        }
        self.entries.get(key).map(|s| s.as_str())
    }

    fn remove(&mut self, key: &str) -> bool {
        self.access_count.remove(key);
        self.entries.remove(key).is_some()
    }

    fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

// ============================================================
// HASHER — URL shortening algorithm
// ============================================================

// TODO: Move to src/hasher.rs

fn shorten(url: &str) -> String {
    // djb2-inspired hash, folded into a 6-character alphanumeric key
    let mut hash: u64 = 5381;
    for byte in url.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }

    let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let base = charset.len() as u64;

    let mut key = String::with_capacity(6);
    let mut n = hash;
    for _ in 0..6 {
        key.push(charset[(n % base) as usize]);
        n /= base;
    }
    key
}

// ============================================================
// API — command-line parsing
// ============================================================

// TODO: Move to src/api.rs

enum Command {
    Shorten { url: String },
    Resolve { key: String },
    Remove { key: String },
    Stats,
    Help,
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    match args.first().map(|s| s.as_str()) {
        Some("shorten") => {
            let url = args.get(1).ok_or("shorten requires a URL argument")?;
            Ok(Command::Shorten { url: url.clone() })
        }
        Some("resolve") => {
            let key = args.get(1).ok_or("resolve requires a key argument")?;
            Ok(Command::Resolve { key: key.clone() })
        }
        Some("remove") => {
            let key = args.get(1).ok_or("remove requires a key argument")?;
            Ok(Command::Remove { key: key.clone() })
        }
        Some("stats") => Ok(Command::Stats),
        Some("help") | None => Ok(Command::Help),
        Some(unknown) => Err(format!("unknown command: '{}'", unknown)),
    }
}

// ============================================================
// STATS — analytics tracking
// ============================================================

// TODO: Move to src/stats.rs

struct Stats {
    shortened_count: u64,
    resolved_count: u64,
    removed_count: u64,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            shortened_count: 0,
            resolved_count: 0,
            removed_count: 0,
        }
    }

    fn record_shorten(&mut self) {
        self.shortened_count += 1;
    }

    fn record_resolve(&mut self) {
        self.resolved_count += 1;
    }

    fn record_remove(&mut self) {
        self.removed_count += 1;
    }

    fn report(&self) -> String {
        format!(
            "Shortened: {}  Resolved: {}  Removed: {}",
            self.shortened_count, self.resolved_count, self.removed_count
        )
    }
}

// ============================================================
// MAIN — entry point
// ============================================================

// TODO: After the refactor, main() should only:
//   1. Collect std::env::args
//   2. Call parse_args()
//   3. Dispatch to the appropriate module
// No business logic here.

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    let mut store = Store::new();
    let mut stats = Stats::new();

    match parse_args(&raw_args) {
        Ok(Command::Shorten { url }) => {
            let key = shorten(&url);
            store.insert(key.clone(), url.clone());
            stats.record_shorten();
            println!("Shortened: {} -> {}", url, key);
        }
        Ok(Command::Resolve { key }) => {
            match store.resolve(&key) {
                Some(url) => {
                    stats.record_resolve();
                    println!("Resolved: {} -> {}", key, url);
                }
                None => println!("Error: key '{}' not found", key),
            }
        }
        Ok(Command::Remove { key }) => {
            if store.remove(&key) {
                stats.record_remove();
                println!("Removed: {}", key);
            } else {
                println!("Error: key '{}' not found", key);
            }
        }
        Ok(Command::Stats) => {
            println!("{}", stats.report());
            println!("Total entries in store: {}", store.total_entries());
        }
        Ok(Command::Help) => {
            println!("Usage:");
            println!("  shorten <url>  — shorten a URL");
            println!("  resolve <key>  — resolve a short key to its URL");
            println!("  remove <key>   — remove a key from the store");
            println!("  stats          — show usage statistics");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

// ============================================================
// TESTS — these test the monolithic code
// After your refactor, equivalent tests should live alongside each module
// and the integration tests should live in tests/
// ============================================================

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
        assert_eq!(key.len(), 6);
    }

    #[test]
    fn test_shorten_different_urls_different_keys() {
        let k1 = shorten("https://example.com/a");
        let k2 = shorten("https://example.com/b");
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_store_insert_and_resolve() {
        let mut store = Store::new();
        store.insert("abc123".to_string(), "https://example.com".to_string());

        assert_eq!(store.resolve("abc123"), Some("https://example.com"));
        assert_eq!(store.resolve("missing"), None);
    }

    #[test]
    fn test_store_remove() {
        let mut store = Store::new();
        store.insert("abc123".to_string(), "https://example.com".to_string());

        assert!(store.remove("abc123"));
        assert!(!store.remove("abc123")); // already removed
        assert_eq!(store.resolve("abc123"), None);
    }

    #[test]
    fn test_parse_args_shorten() {
        let args = vec!["shorten".to_string(), "https://example.com".to_string()];
        let cmd = parse_args(&args).unwrap();
        assert!(matches!(cmd, Command::Shorten { url } if url == "https://example.com"));
    }

    #[test]
    fn test_parse_args_missing_url() {
        let args = vec!["shorten".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn test_parse_args_unknown() {
        let args = vec!["upload".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn test_stats_report() {
        let mut stats = Stats::new();
        stats.record_shorten();
        stats.record_shorten();
        stats.record_resolve();

        assert_eq!(stats.report(), "Shortened: 2  Resolved: 1  Removed: 0");
    }
}
