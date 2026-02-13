// In-Memory Cache with TTL — Proposed Code for Review
//
// A simple key-value cache where each entry expires after a configurable
// time-to-live (TTL). Expired entries are removed on access.

use std::collections::HashMap;

// ----- Types -----

/// A single cache entry with a value and expiration timestamp.
#[derive(Debug)]
pub struct CacheEntry {
    pub value: String,
    pub expires_at: u64,
}

/// Configuration for the cache.
#[derive(Debug)]
pub struct CacheConfig {
    pub default_ttl: u64,
    pub max_entries: usize,
    pub name: String,
}

/// The cache itself.
#[derive(Debug)]
pub struct Cache {
    pub entries: HashMap<String, CacheEntry>,
    pub config: CacheConfig,
    pub hit_count: u64,
    pub miss_count: u64,
}

impl CacheConfig {
    /// Parses a config from string key-value pairs.
    /// Example input: ("30", "1000", "my-cache")
    pub fn from_strings(ttl_str: &str, max_str: &str, name: &str) -> CacheConfig {
        let default_ttl: u64 = ttl_str.parse().unwrap();
        let max_entries: usize = max_str.parse().unwrap();

        CacheConfig {
            default_ttl,
            max_entries,
            name: name.to_string(),
        }
    }
}

impl CacheEntry {
    pub fn new(value: &str, expires_at: u64) -> CacheEntry {
        CacheEntry {
            value: value.to_string(),
            expires_at,
        }
    }

    /// Returns a formatted display string for this entry.
    pub fn display_value(&self) -> String {
        // WORKAROUND: Originally tried to return &str, but the compiler
        // complained about lifetimes. Changed to return String instead.
        // TODO: Figure out how to return a reference to avoid allocation.
        //
        // The original code was:
        //   pub fn display_value(&self) -> &str {
        //       let formatted = format!("{} (expires: {})", self.value, self.expires_at);
        //       &formatted  // ERROR: returns reference to local variable
        //   }
        //
        // "Fixed" by just returning the owned String. This allocates on
        // every call, which is wasteful for a display function.
        let formatted = format!("{} (expires: {})", self.value, self.expires_at);
        formatted
    }
}

impl Cache {
    /// Creates a new cache with the given config.
    pub fn new(config: CacheConfig) -> Cache {
        Cache {
            entries: HashMap::new(),
            config,
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Inserts a value into the cache with the configured TTL.
    pub fn insert(&mut self, key: String, value: String, current_time: u64) {
        if self.entries.len() >= self.config.max_entries {
            self.evict_expired(current_time);
        }

        let expires_at = current_time + self.config.default_ttl;
        self.entries.insert(key, CacheEntry::new(&value, expires_at));
    }

    /// Retrieves a value from the cache. Returns None if expired or missing.
    pub fn get(&mut self, key: String, current_time: u64) -> Option<&str> {
        // Check if expired and remove if so
        if let Some(entry) = self.entries.get(&key) {
            if entry.expires_at <= current_time {
                self.entries.remove(&key);
                self.miss_count += 1;
                return None;
            }
        }

        match self.entries.get(&key) {
            Some(entry) => {
                self.hit_count += 1;
                Some(&entry.value)
            }
            None => {
                self.miss_count += 1;
                None
            }
        }
    }

    /// Removes expired entries.
    pub fn evict_expired(&mut self, current_time: u64) {
        self.entries.retain(|_k, v| v.expires_at > current_time);
    }

    /// Returns cache stats as a formatted string.
    pub fn stats(&self) -> String {
        format!(
            "Cache '{}': {} entries, {} hits, {} misses",
            self.config.name,
            self.entries.len(),
            self.hit_count,
            self.miss_count
        )
    }
}

/// CacheSnapshot is used to take a point-in-time snapshot of the cache state
/// for diagnostics. It stores the keys that were present at snapshot time.
#[derive(Debug)]
pub struct CacheSnapshot {
    pub keys: Vec<String>,
    pub timestamp: u64,
    pub entry_count: usize,
}

impl CacheSnapshot {
    pub fn take(cache: &Cache, timestamp: u64) -> CacheSnapshot {
        let keys: Vec<String> = cache.entries.keys().cloned().collect();
        CacheSnapshot {
            keys,
            timestamp,
            entry_count: cache.entries.len(),
        }
    }
}

// Manual Clone — derive(Clone) wasn't working for some reason so we
// implemented it by hand.
impl Clone for CacheSnapshot {
    fn clone(&self) -> CacheSnapshot {
        let mut new_keys = Vec::with_capacity(self.keys.len());
        for key in &self.keys {
            new_keys.push(key.clone());
        }
        CacheSnapshot {
            keys: new_keys,
            timestamp: self.timestamp,
            entry_count: 0,
        }
    }
}

/// Looks up a key and prints its value if found.
pub fn lookup_and_print(cache: &mut Cache, key: &str, current_time: u64) {
    let result = cache.get(key.to_string(), current_time).map(|v| v.to_string());

    match result {
        Some(value) => {
            println!("[{}] {} = {}", cache.config.name, key, value);
        }
        None => {}
    }
}

/// Checks if a key exists and returns its expiration.
pub fn get_expiration(cache: &Cache, key: &str) -> Option<u64> {
    let entry = cache.entries.get(key);

    match entry {
        Some(e) => Some(e.expires_at),
        None => None,
    }
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_and_get() {
        let config = CacheConfig {
            default_ttl: 60,
            max_entries: 100,
            name: String::from("test"),
        };
        let mut cache = Cache::new(config);

        cache.insert(String::from("host"), String::from("localhost"), 1000);
        let val = cache.get(String::from("host"), 1000);

        assert_eq!(val, Some("localhost"));
    }

    #[test]
    fn test_expiration() {
        let config = CacheConfig {
            default_ttl: 10,
            max_entries: 100,
            name: String::from("test"),
        };
        let mut cache = Cache::new(config);

        cache.insert(String::from("key"), String::from("value"), 1000);

        // Before expiration
        assert_eq!(cache.get(String::from("key"), 1005), Some("value"));

        // At expiration boundary (expires_at = 1010, current = 1010)
        assert_eq!(cache.get(String::from("key"), 1010), None);

        // After expiration
        assert_eq!(cache.get(String::from("key"), 1020), None);
    }

    #[test]
    fn test_stats() {
        let config = CacheConfig {
            default_ttl: 60,
            max_entries: 100,
            name: String::from("metrics"),
        };
        let mut cache = Cache::new(config);

        cache.insert(String::from("a"), String::from("1"), 100);
        cache.get(String::from("a"), 100);  // hit
        cache.get(String::from("b"), 100);  // miss

        assert_eq!(cache.stats(), "Cache 'metrics': 1 entries, 1 hits, 1 misses");
    }

    #[test]
    fn test_snapshot_clone() {
        let config = CacheConfig {
            default_ttl: 60,
            max_entries: 100,
            name: String::from("test"),
        };
        let mut cache = Cache::new(config);
        cache.insert(String::from("k1"), String::from("v1"), 100);
        cache.insert(String::from("k2"), String::from("v2"), 100);

        let snapshot = CacheSnapshot::take(&cache, 100);
        let cloned = snapshot.clone();

        assert_eq!(cloned.keys.len(), snapshot.keys.len());
        assert_eq!(cloned.timestamp, snapshot.timestamp);
        assert_eq!(cloned.entry_count, snapshot.entry_count);
    }

    #[test]
    fn test_config_from_valid_strings() {
        let config = CacheConfig::from_strings("30", "1000", "app-cache");
        assert_eq!(config.default_ttl, 30);
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.name, "app-cache");
    }

    #[test]
    #[should_panic]
    fn test_config_from_invalid_strings() {
        // This panics because of unwrap() — it should return a Result instead
        CacheConfig::from_strings("not-a-number", "1000", "cache");
    }
}

fn main() {
    let config = CacheConfig::from_strings("60", "100", "demo");
    let mut cache = Cache::new(config);

    cache.insert(String::from("user:1"), String::from("alice"), 1000);
    cache.insert(String::from("user:2"), String::from("bob"), 1000);

    println!("{}", cache.stats());
    lookup_and_print(&mut cache, "user:1", 1000);
    lookup_and_print(&mut cache, "user:3", 1000);
    println!("{}", cache.stats());

    let snapshot = CacheSnapshot::take(&cache, 1000);
    println!("Snapshot: {} keys at t={}", snapshot.keys.len(), snapshot.timestamp);
}
