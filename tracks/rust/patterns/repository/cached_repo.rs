// Repository Pattern: Caching Decorator
//
// Demonstrates: Generic CachedRepository<R> that wraps any repository
// implementing a trait, adding transparent read-through caching.
// Shows composition through generics -- the Rust equivalent of
// Go's interface wrapping or TypeScript's decorator pattern.
//
// Scenario: A session store repository with an in-memory cache layer.
// The cache intercepts reads and serves from memory when possible,
// falling through to the backing store on cache misses. Writes go
// through to the backing store and update the cache (write-through).
//
// Run: rustc cached_repo.rs && ./cached_repo
// Test: rustc --test cached_repo.rs && ./cached_repo

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: u64,
    pub metadata: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RepoError {
    NotFound { entity: String, id: String },
    Expired { entity: String, id: String },
    Internal { message: String },
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepoError::NotFound { entity, id } => write!(f, "{} not found: {}", entity, id),
            RepoError::Expired { entity, id } => write!(f, "{} expired: {}", entity, id),
            RepoError::Internal { message } => write!(f, "internal error: {}", message),
        }
    }
}

impl std::error::Error for RepoError {}

// ---------------------------------------------------------------------------
// Repository trait
// ---------------------------------------------------------------------------

pub trait SessionRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, RepoError>;
    fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepoError>;
    fn find_by_user(&self, user_id: &str) -> Result<Vec<Session>, RepoError>;
    fn save(&mut self, session: &Session) -> Result<(), RepoError>;
    fn delete(&mut self, id: &str) -> Result<bool, RepoError>;
    fn delete_expired(&mut self, current_time: u64) -> Result<usize, RepoError>;
}

// ---------------------------------------------------------------------------
// In-memory backing store (simulates a database)
// ---------------------------------------------------------------------------

pub struct InMemorySessionStore {
    sessions: HashMap<String, Session>,
    /// Track access count for demonstration
    access_count: u64,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            access_count: 0,
        }
    }

    pub fn access_count(&self) -> u64 {
        self.access_count
    }
}

impl SessionRepository for InMemorySessionStore {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, RepoError> {
        // In a real DB, this would be an I/O call. We track access to prove
        // the cache is working.
        // Note: we can't increment access_count here because &self is immutable.
        // This is a design choice -- see the CachedSessionRepo below for how
        // we handle this.
        Ok(self.sessions.get(id).cloned())
    }

    fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepoError> {
        Ok(self
            .sessions
            .values()
            .find(|s| s.token == token)
            .cloned())
    }

    fn find_by_user(&self, user_id: &str) -> Result<Vec<Session>, RepoError> {
        let sessions: Vec<Session> = self
            .sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect();
        Ok(sessions)
    }

    fn save(&mut self, session: &Session) -> Result<(), RepoError> {
        self.sessions.insert(session.id.clone(), session.clone());
        self.access_count += 1;
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        self.access_count += 1;
        Ok(self.sessions.remove(id).is_some())
    }

    fn delete_expired(&mut self, current_time: u64) -> Result<usize, RepoError> {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| s.expires_at > current_time);
        self.access_count += 1;
        Ok(before - self.sessions.len())
    }
}

// ---------------------------------------------------------------------------
// Caching decorator: CachedSessionRepo<R>
// ---------------------------------------------------------------------------

/// A caching decorator that wraps any `SessionRepository` implementation.
///
/// Design:
/// - Generic over `R: SessionRepository` -- works with any backend
/// - Read-through cache: checks cache first, falls through to inner repo
/// - Write-through: writes to inner repo first, then updates cache
/// - Cache invalidation on delete
/// - Only caches by-ID lookups (the most common case)
///
/// The cache is a simple HashMap. In production, you'd want:
/// - TTL-based expiration
/// - LRU eviction
/// - Size limits
pub struct CachedSessionRepo<R: SessionRepository> {
    inner: R,
    cache: HashMap<String, Session>,
    cache_hits: u64,
    cache_misses: u64,
}

impl<R: SessionRepository> CachedSessionRepo<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.cache_hits,
            misses: self.cache_misses,
            cached_entries: self.cache.len(),
            hit_rate: if self.cache_hits + self.cache_misses > 0 {
                self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Invalidate a specific cache entry.
    pub fn invalidate(&mut self, id: &str) {
        self.cache.remove(id);
    }

    /// Clear the entire cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get a reference to the inner repository (for inspection in tests).
    pub fn inner(&self) -> &R {
        &self.inner
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub cached_entries: usize,
    pub hit_rate: f64,
}

impl fmt::Display for CacheStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cache: {} hits, {} misses ({:.1}% hit rate), {} entries",
            self.hits,
            self.misses,
            self.hit_rate * 100.0,
            self.cached_entries
        )
    }
}

impl<R: SessionRepository> SessionRepository for CachedSessionRepo<R> {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, RepoError> {
        // Check cache first
        if let Some(session) = self.cache.get(id) {
            // Note: we can't increment cache_hits with &self.
            // In a production system, you'd use interior mutability (Cell/AtomicU64)
            // for cache counters. Here we accept the limitation for simplicity.
            return Ok(Some(session.clone()));
        }

        // Cache miss -- fall through to inner repo
        self.inner.find_by_id(id)
    }

    fn find_by_token(&self, token: &str) -> Result<Option<Session>, RepoError> {
        // Token lookups bypass cache (no token index in cache)
        self.inner.find_by_token(token)
    }

    fn find_by_user(&self, user_id: &str) -> Result<Vec<Session>, RepoError> {
        // User lookups bypass cache (no user index in cache)
        self.inner.find_by_user(user_id)
    }

    fn save(&mut self, session: &Session) -> Result<(), RepoError> {
        // Write-through: save to backing store first
        self.inner.save(session)?;
        // Then update cache
        self.cache.insert(session.id.clone(), session.clone());
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        // Invalidate cache
        self.cache.remove(id);
        // Delete from backing store
        self.inner.delete(id)
    }

    fn delete_expired(&mut self, current_time: u64) -> Result<usize, RepoError> {
        // Remove expired entries from cache
        self.cache.retain(|_, s| s.expires_at > current_time);
        // Delete from backing store
        self.inner.delete_expired(current_time)
    }
}

// A version with mutable stats tracking using a wrapper approach
// (demonstrates interior mutability alternative)

/// CachedSessionRepoMut tracks cache hits/misses properly by using
/// `&mut self` for read operations. This breaks the SessionRepository
/// trait contract (which uses `&self` for reads), so it's a separate
/// type that demonstrates the tradeoff.
pub struct CachedSessionRepoWithStats<R: SessionRepository> {
    inner: R,
    cache: HashMap<String, Session>,
    hits: u64,
    misses: u64,
}

impl<R: SessionRepository> CachedSessionRepoWithStats<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Find by ID with proper hit/miss tracking.
    /// Note: takes &mut self, so cannot implement SessionRepository trait.
    pub fn find_by_id_tracked(&mut self, id: &str) -> Result<Option<Session>, RepoError> {
        if let Some(session) = self.cache.get(id) {
            self.hits += 1;
            return Ok(Some(session.clone()));
        }

        self.misses += 1;
        let result = self.inner.find_by_id(id)?;

        // Populate cache on miss
        if let Some(ref session) = result {
            self.cache.insert(id.to_string(), session.clone());
        }

        Ok(result)
    }

    pub fn save(&mut self, session: &Session) -> Result<(), RepoError> {
        self.inner.save(session)?;
        self.cache.insert(session.id.clone(), session.clone());
        Ok(())
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            cached_entries: self.cache.len(),
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_session(id: &str, user_id: &str, token: &str, expires_at: u64) -> Session {
        Session {
            id: id.to_string(),
            user_id: user_id.to_string(),
            token: token.to_string(),
            expires_at,
            metadata: HashMap::new(),
        }
    }

    // --- Backing store tests ---

    #[test]
    fn test_backing_store_crud() {
        let mut store = InMemorySessionStore::new();
        let session = make_session("s1", "u1", "tok-abc", 9999);

        store.save(&session).unwrap();
        assert_eq!(store.find_by_id("s1").unwrap(), Some(session.clone()));
        assert!(store.delete("s1").unwrap());
        assert_eq!(store.find_by_id("s1").unwrap(), None);
    }

    #[test]
    fn test_backing_store_find_by_token() {
        let mut store = InMemorySessionStore::new();
        store
            .save(&make_session("s1", "u1", "tok-abc", 9999))
            .unwrap();

        let found = store.find_by_token("tok-abc").unwrap();
        assert_eq!(found.unwrap().id, "s1");

        assert!(store.find_by_token("tok-xyz").unwrap().is_none());
    }

    #[test]
    fn test_backing_store_find_by_user() {
        let mut store = InMemorySessionStore::new();
        store
            .save(&make_session("s1", "u1", "tok-1", 9999))
            .unwrap();
        store
            .save(&make_session("s2", "u1", "tok-2", 9999))
            .unwrap();
        store
            .save(&make_session("s3", "u2", "tok-3", 9999))
            .unwrap();

        let user1_sessions = store.find_by_user("u1").unwrap();
        assert_eq!(user1_sessions.len(), 2);
    }

    #[test]
    fn test_backing_store_delete_expired() {
        let mut store = InMemorySessionStore::new();
        store
            .save(&make_session("s1", "u1", "tok-1", 1000))
            .unwrap();
        store
            .save(&make_session("s2", "u1", "tok-2", 2000))
            .unwrap();
        store
            .save(&make_session("s3", "u2", "tok-3", 3000))
            .unwrap();

        let deleted = store.delete_expired(1500).unwrap();
        assert_eq!(deleted, 1); // s1 expired
        assert!(store.find_by_id("s1").unwrap().is_none());
        assert!(store.find_by_id("s2").unwrap().is_some());
        assert!(store.find_by_id("s3").unwrap().is_some());
    }

    // --- Cached repo tests ---

    #[test]
    fn test_cached_save_and_find() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);

        let session = make_session("s1", "u1", "tok-abc", 9999);
        cached.save(&session).unwrap();

        // Should be served from cache
        let found = cached.find_by_id("s1").unwrap();
        assert_eq!(found, Some(session));
    }

    #[test]
    fn test_cached_miss_falls_through() {
        let mut store = InMemorySessionStore::new();
        let session = make_session("s1", "u1", "tok-abc", 9999);
        store.save(&session).unwrap();

        // Create cache AFTER data exists in store -- cache is empty
        let cached = CachedSessionRepo::new(store);

        // Should fall through to inner store
        let found = cached.find_by_id("s1").unwrap();
        assert_eq!(found, Some(session));
    }

    #[test]
    fn test_cached_delete_invalidates() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);

        let session = make_session("s1", "u1", "tok-abc", 9999);
        cached.save(&session).unwrap();
        cached.delete("s1").unwrap();

        assert!(cached.find_by_id("s1").unwrap().is_none());
    }

    #[test]
    fn test_cached_delete_expired() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);

        cached
            .save(&make_session("s1", "u1", "tok-1", 1000))
            .unwrap();
        cached
            .save(&make_session("s2", "u1", "tok-2", 3000))
            .unwrap();

        let deleted = cached.delete_expired(2000).unwrap();
        assert_eq!(deleted, 1);

        // s1 should be gone from both cache and store
        assert!(cached.find_by_id("s1").unwrap().is_none());
        // s2 should still exist
        assert!(cached.find_by_id("s2").unwrap().is_some());
    }

    #[test]
    fn test_cached_invalidate() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);

        cached
            .save(&make_session("s1", "u1", "tok-abc", 9999))
            .unwrap();

        // Manually invalidate cache entry
        cached.invalidate("s1");

        // Should still find it (falls through to inner store)
        let found = cached.find_by_id("s1").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_cached_clear() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);

        cached
            .save(&make_session("s1", "u1", "tok-1", 9999))
            .unwrap();
        cached
            .save(&make_session("s2", "u2", "tok-2", 9999))
            .unwrap();

        cached.clear_cache();
        assert_eq!(cached.stats().cached_entries, 0);
    }

    // --- Stats tracking tests ---

    #[test]
    fn test_stats_tracking() {
        let mut store = InMemorySessionStore::new();
        store
            .save(&make_session("s1", "u1", "tok-abc", 9999))
            .unwrap();

        let mut cached = CachedSessionRepoWithStats::new(store);

        // First lookup: cache miss, populates cache
        cached.find_by_id_tracked("s1").unwrap();
        assert_eq!(cached.stats().misses, 1);
        assert_eq!(cached.stats().hits, 0);

        // Second lookup: cache hit
        cached.find_by_id_tracked("s1").unwrap();
        assert_eq!(cached.stats().misses, 1);
        assert_eq!(cached.stats().hits, 1);

        // Third lookup for missing key: cache miss
        cached.find_by_id_tracked("s999").unwrap();
        assert_eq!(cached.stats().misses, 2);
        assert_eq!(cached.stats().hits, 1);

        // Verify hit rate
        let stats = cached.stats();
        assert!((stats.hit_rate - 1.0 / 3.0).abs() < 0.01);
    }

    // --- Composition test: proves any SessionRepository works ---

    fn assert_repo_contract<R: SessionRepository>(repo: &mut R) {
        let session = make_session("test-1", "user-1", "tok-test", 9999);

        repo.save(&session).unwrap();
        assert!(repo.find_by_id("test-1").unwrap().is_some());
        assert!(repo.delete("test-1").unwrap());
        assert!(repo.find_by_id("test-1").unwrap().is_none());
    }

    #[test]
    fn test_contract_backing_store() {
        let mut store = InMemorySessionStore::new();
        assert_repo_contract(&mut store);
    }

    #[test]
    fn test_contract_cached() {
        let store = InMemorySessionStore::new();
        let mut cached = CachedSessionRepo::new(store);
        assert_repo_contract(&mut cached);
    }

    #[test]
    fn test_contract_double_cached() {
        // Cache on top of cache -- proves composition works at any depth
        let store = InMemorySessionStore::new();
        let cached_once = CachedSessionRepo::new(store);
        let mut cached_twice = CachedSessionRepo::new(cached_once);
        assert_repo_contract(&mut cached_twice);
    }
}

// ---------------------------------------------------------------------------
// Main: demo
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Caching Decorator Repository Demo ===\n");

    // Set up: backing store with some data
    let mut store = InMemorySessionStore::new();
    store
        .save(&Session {
            id: "sess-001".to_string(),
            user_id: "user-alice".to_string(),
            token: "tok-aaa-111".to_string(),
            expires_at: 99999,
            metadata: HashMap::from([
                ("ip".to_string(), "192.168.1.10".to_string()),
                ("user_agent".to_string(), "Mozilla/5.0".to_string()),
            ]),
        })
        .unwrap();

    store
        .save(&Session {
            id: "sess-002".to_string(),
            user_id: "user-bob".to_string(),
            token: "tok-bbb-222".to_string(),
            expires_at: 99999,
            metadata: HashMap::new(),
        })
        .unwrap();

    println!("Backing store writes: {}", store.access_count());

    // Wrap with cache
    let mut cached = CachedSessionRepoWithStats::new(store);

    // Simulate session lookups
    println!("\n--- Session lookups ---");

    // First lookup: cache miss
    let s = cached.find_by_id_tracked("sess-001").unwrap().unwrap();
    println!("  Found: {} (user: {})", s.id, s.user_id);
    println!("  {}", cached.stats());

    // Second lookup: cache hit
    let s = cached.find_by_id_tracked("sess-001").unwrap().unwrap();
    println!("  Found: {} (user: {})", s.id, s.user_id);
    println!("  {}", cached.stats());

    // Third lookup: different session, cache miss
    let s = cached.find_by_id_tracked("sess-002").unwrap().unwrap();
    println!("  Found: {} (user: {})", s.id, s.user_id);
    println!("  {}", cached.stats());

    // Fourth lookup: back to first, cache hit
    let s = cached.find_by_id_tracked("sess-001").unwrap().unwrap();
    println!("  Found: {} (user: {})", s.id, s.user_id);
    println!("  {}", cached.stats());

    // Save new session (write-through)
    println!("\n--- New session (write-through) ---");
    cached
        .save(&Session {
            id: "sess-003".to_string(),
            user_id: "user-charlie".to_string(),
            token: "tok-ccc-333".to_string(),
            expires_at: 99999,
            metadata: HashMap::new(),
        })
        .unwrap();

    // Immediately available in cache
    let s = cached.find_by_id_tracked("sess-003").unwrap().unwrap();
    println!("  Found new session: {} (cache hit!)", s.id);
    println!("  {}", cached.stats());

    // Miss for nonexistent
    println!("\n--- Missing session ---");
    let result = cached.find_by_id_tracked("sess-999").unwrap();
    println!("  sess-999: {:?}", result);
    println!("  {}", cached.stats());
}
