// Generic Cache with Expiration — Proposed Code for Review
//
// A generic, expiration-aware cache for webhook endpoint resolution.
// Caches resolved endpoints to avoid re-resolving on every delivery attempt.
//
// Run: rustc proposed.rs && ./proposed

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

// ----- Expiry policy trait -----

// ISSUE: The Expiry trait takes a generic parameter E for the entry type.
// But `is_expired` only needs the timestamp — it never inspects the entry.
// Should E be a generic parameter on the trait, or is there a simpler design?
pub trait Expiry<E> {
    fn is_expired(&self, inserted_at: Instant) -> bool;
    fn default_ttl(&self) -> Duration;
}

// TTL-based expiry: entries expire after a fixed duration.
pub struct TtlExpiry {
    ttl: Duration,
}

impl TtlExpiry {
    pub fn new(ttl: Duration) -> Self {
        TtlExpiry { ttl }
    }
}

// Because Expiry is generic over E, we must impl it for every entry type we use.
// This impl uses a blanket impl with E: Sized, but notice E is never used in the body.
impl<E: Sized> Expiry<E> for TtlExpiry {
    fn is_expired(&self, inserted_at: Instant) -> bool {
        inserted_at.elapsed() > self.ttl
    }

    fn default_ttl(&self) -> Duration {
        self.ttl
    }
}

// ----- Cache entry -----

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
}

// ----- The cache struct -----

// ISSUE: This struct has 4 type parameters: K, V, E (expiry policy), and M (a "marker"
// for the cache strategy). That's a lot of generics for a simple cache.
//
// ISSUE: The trait bounds are on the struct definition itself (K: Hash + Eq + Clone + Debug,
// V: Clone + Debug). These bounds are required for SOME methods, not all. Having them on the
// struct means even code that just wants to hold a Cache<K, V, E, M> without calling methods
// on it must satisfy all bounds. Should these bounds move somewhere else?
//
// ISSUE: PhantomData<M> — the M type parameter is supposed to represent a "cache strategy"
// (LRU, FIFO, etc.), but it's never used in any method. It's pure phantom complexity.
// Does it serve a real purpose here?
pub struct Cache<K, V, E, M>
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: Clone + fmt::Debug,
    E: Expiry<V>,
{
    entries: HashMap<K, CacheEntry<V>>,
    expiry_policy: E,
    max_capacity: usize,
    hits: u64,
    misses: u64,
    _strategy: PhantomData<M>,
}

// Strategy marker types — never instantiated, only used as phantom type parameters.
pub struct LruMarker;
pub struct FifoMarker;

impl<K, V, E, M> Cache<K, V, E, M>
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: Clone + fmt::Debug,
    E: Expiry<V>,
{
    pub fn new(expiry_policy: E, max_capacity: usize) -> Self {
        Cache {
            entries: HashMap::new(),
            expiry_policy,
            max_capacity,
            hits: 0,
            misses: 0,
            _strategy: PhantomData,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check if entry exists and is not expired
        let is_expired = self
            .entries
            .get(key)
            .map(|entry| self.expiry_policy.is_expired(entry.inserted_at))
            .unwrap_or(false);

        if is_expired {
            self.entries.remove(key);
            self.misses += 1;
            return None;
        }

        match self.entries.get(key) {
            Some(entry) => {
                self.hits += 1;
                Some(&entry.value)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // Evict expired entries if at capacity
        if self.entries.len() >= self.max_capacity {
            self.evict_expired();
        }

        // If still at capacity after eviction, remove oldest entry
        if self.entries.len() >= self.max_capacity {
            // ISSUE: This eviction logic is identical regardless of M (LruMarker vs FifoMarker).
            // The marker type promises different strategies but the code ignores it.
            if let Some(oldest_key) = self.find_oldest_key() {
                self.entries.remove(&oldest_key);
            }
        }

        self.entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }

    fn evict_expired(&mut self) {
        let expired_keys: Vec<K> = self
            .entries
            .iter()
            .filter(|(_, entry)| self.expiry_policy.is_expired(entry.inserted_at))
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            self.entries.remove(&key);
        }
    }

    fn find_oldest_key(&self) -> Option<K> {
        self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.inserted_at)
            .map(|(k, _)| k.clone())
    }
}

// ----- Stats reporting -----

// ISSUE: This function is generic over K, V, E, M — but it only reads hits, misses,
// and len(). It doesn't touch K, V, E, or M at all. Every unique combination of
// <K, V, E, M> that calls this function generates a separate monomorphized copy.
// For a function that just formats some u64s and a usize, that's wasteful.
// Is there a way to avoid the monomorphization bloat?
pub fn cache_stats<K, V, E, M>(cache: &Cache<K, V, E, M>) -> String
where
    K: std::hash::Hash + Eq + Clone + fmt::Debug,
    V: Clone + fmt::Debug,
    E: Expiry<V>,
{
    format!(
        "Cache stats: {} entries, {:.1}% hit rate ({} hits, {} misses)",
        cache.len(),
        cache.hit_rate() * 100.0,
        cache.hits,
        cache.misses,
    )
}

// ----- Convenience type aliases -----

/// A webhook endpoint cache with TTL expiry and "LRU" strategy.
pub type WebhookCache = Cache<String, ResolvedEndpoint, TtlExpiry, LruMarker>;

/// A config value cache with TTL expiry and "FIFO" strategy.
pub type ConfigCache = Cache<String, String, TtlExpiry, FifoMarker>;

// ----- Domain types -----

#[derive(Debug, Clone)]
pub struct ResolvedEndpoint {
    pub url: String,
    pub ip: String,
    pub resolved_at: Instant,
}

impl ResolvedEndpoint {
    pub fn new(url: &str, ip: &str) -> Self {
        ResolvedEndpoint {
            url: url.to_string(),
            ip: ip.to_string(),
            resolved_at: Instant::now(),
        }
    }
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cache() -> WebhookCache {
        Cache::new(TtlExpiry::new(Duration::from_secs(60)), 100)
    }

    #[test]
    fn test_insert_and_get() {
        let mut cache = make_cache();
        let endpoint = ResolvedEndpoint::new("https://api.example.com/webhook", "93.184.216.34");

        cache.insert("example".to_string(), endpoint);
        assert_eq!(cache.len(), 1);

        let result = cache.get(&"example".to_string());
        assert!(result.is_some());
        assert_eq!(result.unwrap().url, "https://api.example.com/webhook");
    }

    #[test]
    fn test_miss() {
        let mut cache = make_cache();
        assert!(cache.get(&"nonexistent".to_string()).is_none());
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate() {
        let mut cache = make_cache();
        let endpoint = ResolvedEndpoint::new("https://hooks.example.com", "10.0.0.1");

        cache.insert("hooks".to_string(), endpoint);
        cache.get(&"hooks".to_string());    // hit
        cache.get(&"hooks".to_string());    // hit
        cache.get(&"missing".to_string());  // miss

        // 2 hits, 1 miss = 66.7%
        assert!((cache.hit_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut cache: WebhookCache = Cache::new(TtlExpiry::new(Duration::from_secs(60)), 2);

        cache.insert("a".to_string(), ResolvedEndpoint::new("https://a.com", "1.1.1.1"));
        std::thread::sleep(Duration::from_millis(10)); // ensure different timestamps
        cache.insert("b".to_string(), ResolvedEndpoint::new("https://b.com", "2.2.2.2"));
        std::thread::sleep(Duration::from_millis(10));

        // Cache is full (capacity=2). Inserting c should evict the oldest (a).
        cache.insert("c".to_string(), ResolvedEndpoint::new("https://c.com", "3.3.3.3"));

        assert_eq!(cache.len(), 2);
        assert!(cache.get(&"a".to_string()).is_none()); // evicted
        assert!(cache.get(&"c".to_string()).is_some());
    }

    #[test]
    fn test_remove() {
        let mut cache = make_cache();
        cache.insert("temp".to_string(), ResolvedEndpoint::new("https://temp.com", "4.4.4.4"));
        assert_eq!(cache.len(), 1);

        let removed = cache.remove(&"temp".to_string());
        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = make_cache();
        cache.insert("x".to_string(), ResolvedEndpoint::new("https://x.com", "5.5.5.5"));
        cache.get(&"x".to_string());

        let stats = cache_stats(&cache);
        assert!(stats.contains("1 entries"));
        assert!(stats.contains("hit rate"));
    }

    #[test]
    fn test_config_cache() {
        let mut cache: ConfigCache = Cache::new(TtlExpiry::new(Duration::from_secs(300)), 50);
        cache.insert("db_host".to_string(), "localhost:5432".to_string());
        cache.insert("redis_url".to_string(), "redis://127.0.0.1".to_string());

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"db_host".to_string()).unwrap(), "localhost:5432");
    }
}

fn main() {
    let mut cache: WebhookCache = Cache::new(TtlExpiry::new(Duration::from_secs(30)), 1000);

    let endpoints = vec![
        ("stripe", "https://api.stripe.com/webhook", "104.16.242.34"),
        ("github", "https://api.github.com/hooks", "140.82.121.6"),
        ("slack", "https://hooks.slack.com/services", "34.226.14.0"),
    ];

    for (name, url, ip) in &endpoints {
        cache.insert(name.to_string(), ResolvedEndpoint::new(url, ip));
    }

    println!("Cached {} endpoints", cache.len());

    // Simulate lookups
    for name in &["stripe", "github", "missing", "slack", "unknown"] {
        match cache.get(&name.to_string()) {
            Some(ep) => println!("  {} -> {} ({})", name, ep.url, ep.ip),
            None => println!("  {} -> MISS", name),
        }
    }

    println!("\n{}", cache_stats(&cache));
}
