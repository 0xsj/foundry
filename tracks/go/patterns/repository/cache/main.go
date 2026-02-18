// Repository Pattern: Caching Decorator
//
// Demonstrates how the repository interface enables transparent caching.
// A CachingConfigStore wraps any ConfigStore implementation with an
// in-memory cache. The caller (ConfigService) doesn't know caching exists --
// it just calls the same interface methods.
//
// Key ideas:
//   - Decorator pattern applied to a repository interface
//   - Cache wraps any ConfigStore implementation transparently
//   - Write-through cache: writes update both cache and backing store
//   - TTL-based expiration
//   - The business logic (ConfigService) is unchanged whether caching is enabled or not
//
// Run: go run ./cache/

package main

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"
)

// =============================================================================
// Domain Types
// =============================================================================

// ConfigEntry represents a configuration key-value pair.
type ConfigEntry struct {
	Key         string
	Value       string
	Environment string // "production", "staging", "development"
	UpdatedAt   time.Time
	UpdatedBy   string
}

var ErrNotFound = errors.New("not found")

// =============================================================================
// Repository Interface
// =============================================================================

// ConfigStore defines operations for configuration management.
type ConfigStore interface {
	Get(ctx context.Context, env, key string) (*ConfigEntry, error)
	Set(ctx context.Context, entry *ConfigEntry) error
	List(ctx context.Context, env string) ([]*ConfigEntry, error)
	Delete(ctx context.Context, env, key string) error
}

// =============================================================================
// Base Implementation: In-Memory Store (simulates a slow database)
// =============================================================================

// SlowConfigStore simulates a database-backed config store with artificial latency.
// In production, this would be a PostgresConfigStore or DynamoDBConfigStore.
type SlowConfigStore struct {
	mu      sync.RWMutex
	entries map[string]*ConfigEntry // key: "env:key"
	latency time.Duration
	reads   int // track how many reads hit the "database"
}

var _ ConfigStore = (*SlowConfigStore)(nil)

func NewSlowConfigStore(latency time.Duration) *SlowConfigStore {
	return &SlowConfigStore{
		entries: make(map[string]*ConfigEntry),
		latency: latency,
	}
}

func (s *SlowConfigStore) Reads() int {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.reads
}

func (s *SlowConfigStore) compoundKey(env, key string) string {
	return env + ":" + key
}

func (s *SlowConfigStore) Get(_ context.Context, env, key string) (*ConfigEntry, error) {
	time.Sleep(s.latency) // simulate database latency

	s.mu.Lock()
	s.reads++
	s.mu.Unlock()

	s.mu.RLock()
	defer s.mu.RUnlock()

	e, ok := s.entries[s.compoundKey(env, key)]
	if !ok {
		return nil, ErrNotFound
	}
	copy := *e
	return &copy, nil
}

func (s *SlowConfigStore) Set(_ context.Context, entry *ConfigEntry) error {
	time.Sleep(s.latency) // simulate database latency

	s.mu.Lock()
	defer s.mu.Unlock()

	entry.UpdatedAt = time.Now()
	stored := *entry
	s.entries[s.compoundKey(entry.Environment, entry.Key)] = &stored
	return nil
}

func (s *SlowConfigStore) List(_ context.Context, env string) ([]*ConfigEntry, error) {
	time.Sleep(s.latency) // simulate database latency

	s.mu.Lock()
	s.reads++
	s.mu.Unlock()

	s.mu.RLock()
	defer s.mu.RUnlock()

	var result []*ConfigEntry
	for _, e := range s.entries {
		if e.Environment == env {
			copy := *e
			result = append(result, &copy)
		}
	}
	return result, nil
}

func (s *SlowConfigStore) Delete(_ context.Context, env, key string) error {
	time.Sleep(s.latency) // simulate database latency

	s.mu.Lock()
	defer s.mu.Unlock()

	ck := s.compoundKey(env, key)
	if _, ok := s.entries[ck]; !ok {
		return ErrNotFound
	}
	delete(s.entries, ck)
	return nil
}

// =============================================================================
// Caching Decorator
// =============================================================================

type cacheItem struct {
	entry     *ConfigEntry
	expiresAt time.Time
}

// CachingConfigStore wraps any ConfigStore with an in-memory cache.
// It satisfies the same ConfigStore interface, so the caller doesn't
// know caching is happening.
//
// Strategy: write-through cache
//   - Get: check cache first, fall through to delegate on miss
//   - Set: write to delegate first, update cache on success
//   - Delete: delete from delegate first, evict from cache on success
//   - List: always delegates (not cached -- too complex to keep consistent)
type CachingConfigStore struct {
	delegate ConfigStore
	mu       sync.RWMutex
	cache    map[string]*cacheItem // key: "env:key"
	ttl      time.Duration
	hits     int
	misses   int
}

var _ ConfigStore = (*CachingConfigStore)(nil)

func NewCachingConfigStore(delegate ConfigStore, ttl time.Duration) *CachingConfigStore {
	return &CachingConfigStore{
		delegate: delegate,
		cache:    make(map[string]*cacheItem),
		ttl:      ttl,
	}
}

func (c *CachingConfigStore) Stats() (hits, misses int) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.hits, c.misses
}

func (c *CachingConfigStore) compoundKey(env, key string) string {
	return env + ":" + key
}

func (c *CachingConfigStore) Get(ctx context.Context, env, key string) (*ConfigEntry, error) {
	ck := c.compoundKey(env, key)

	// Check cache first
	c.mu.RLock()
	item, ok := c.cache[ck]
	if ok && time.Now().Before(item.expiresAt) {
		c.mu.RUnlock()

		c.mu.Lock()
		c.hits++
		c.mu.Unlock()

		// Return a copy from cache
		copy := *item.entry
		return &copy, nil
	}
	c.mu.RUnlock()

	// Cache miss -- delegate to backing store
	c.mu.Lock()
	c.misses++
	c.mu.Unlock()

	entry, err := c.delegate.Get(ctx, env, key)
	if err != nil {
		return nil, err
	}

	// Populate cache
	c.mu.Lock()
	cached := *entry
	c.cache[ck] = &cacheItem{
		entry:     &cached,
		expiresAt: time.Now().Add(c.ttl),
	}
	c.mu.Unlock()

	return entry, nil
}

func (c *CachingConfigStore) Set(ctx context.Context, entry *ConfigEntry) error {
	// Write-through: write to backing store first
	if err := c.delegate.Set(ctx, entry); err != nil {
		return err
	}

	// Update cache on success
	ck := c.compoundKey(entry.Environment, entry.Key)
	c.mu.Lock()
	cached := *entry
	c.cache[ck] = &cacheItem{
		entry:     &cached,
		expiresAt: time.Now().Add(c.ttl),
	}
	c.mu.Unlock()

	return nil
}

func (c *CachingConfigStore) List(ctx context.Context, env string) ([]*ConfigEntry, error) {
	// List is not cached -- too complex to keep consistent when
	// individual entries are added/removed. Always delegate.
	return c.delegate.List(ctx, env)
}

func (c *CachingConfigStore) Delete(ctx context.Context, env, key string) error {
	// Delete from backing store first
	if err := c.delegate.Delete(ctx, env, key); err != nil {
		return err
	}

	// Evict from cache on success
	ck := c.compoundKey(env, key)
	c.mu.Lock()
	delete(c.cache, ck)
	c.mu.Unlock()

	return nil
}

// =============================================================================
// Business Logic: ConfigService
// =============================================================================

// ConfigService manages application configuration.
// It depends on ConfigStore -- it doesn't know or care about caching.
type ConfigService struct {
	store ConfigStore
}

func NewConfigService(store ConfigStore) *ConfigService {
	return &ConfigService{store: store}
}

func (s *ConfigService) GetConfig(ctx context.Context, env, key string) (string, error) {
	entry, err := s.store.Get(ctx, env, key)
	if err != nil {
		return "", err
	}
	return entry.Value, nil
}

func (s *ConfigService) SetConfig(ctx context.Context, env, key, value, updatedBy string) error {
	return s.store.Set(ctx, &ConfigEntry{
		Key:         key,
		Value:       value,
		Environment: env,
		UpdatedBy:   updatedBy,
	})
}

// =============================================================================
// Main
// =============================================================================

func main() {
	ctx := context.Background()

	fmt.Println("=== Repository Pattern: Caching Decorator ===")
	fmt.Println(strings.Repeat("-", 55))

	// Create the slow backing store (simulates 50ms database latency)
	backing := NewSlowConfigStore(50 * time.Millisecond)

	// Wrap it with a cache (30-second TTL)
	cached := NewCachingConfigStore(backing, 30*time.Second)

	// The service gets the cached store -- it doesn't know about caching
	svc := NewConfigService(cached)

	// Seed some config values
	fmt.Println("\n1. Setting config values (writes go through to backing store):")
	svc.SetConfig(ctx, "production", "db_host", "prod-db.internal:5432", "deploy-bot")
	svc.SetConfig(ctx, "production", "max_connections", "100", "deploy-bot")
	svc.SetConfig(ctx, "production", "log_level", "warn", "ops-team")
	svc.SetConfig(ctx, "staging", "db_host", "staging-db.internal:5432", "deploy-bot")
	svc.SetConfig(ctx, "staging", "log_level", "debug", "dev-team")
	fmt.Println("   5 config entries written")

	// First read: cache miss (hits backing store)
	fmt.Println("\n2. First read (cache miss -- hits 'database'):")
	start := time.Now()
	val, _ := svc.GetConfig(ctx, "production", "db_host")
	elapsed := time.Since(start)
	fmt.Printf("   db_host = %s (took %v)\n", val, elapsed.Round(time.Millisecond))

	// Second read: cache hit (fast)
	fmt.Println("\n3. Second read (cache hit -- no database call):")
	start = time.Now()
	val, _ = svc.GetConfig(ctx, "production", "db_host")
	elapsed = time.Since(start)
	fmt.Printf("   db_host = %s (took %v)\n", val, elapsed.Round(time.Microsecond))

	// Multiple reads to show cache effectiveness
	fmt.Println("\n4. 10 rapid reads of the same key:")
	start = time.Now()
	for i := 0; i < 10; i++ {
		svc.GetConfig(ctx, "production", "db_host")
	}
	elapsed = time.Since(start)
	fmt.Printf("   10 reads took %v total\n", elapsed.Round(time.Microsecond))

	// Read different key (cache miss, then hit)
	fmt.Println("\n5. Read a different key (cache miss on first, hit on second):")
	start = time.Now()
	val, _ = svc.GetConfig(ctx, "production", "log_level")
	fmt.Printf("   log_level = %s (took %v — cache miss)\n", val, time.Since(start).Round(time.Millisecond))

	start = time.Now()
	val, _ = svc.GetConfig(ctx, "production", "log_level")
	fmt.Printf("   log_level = %s (took %v — cache hit)\n", val, time.Since(start).Round(time.Microsecond))

	// Show cache stats
	hits, misses := cached.Stats()
	dbReads := backing.Reads()
	fmt.Printf("\n6. Cache stats:\n")
	fmt.Printf("   Cache hits:   %d\n", hits)
	fmt.Printf("   Cache misses: %d\n", misses)
	fmt.Printf("   DB reads:     %d\n", dbReads)
	fmt.Printf("   Hit rate:     %.0f%%\n", float64(hits)/float64(hits+misses)*100)

	// Write-through: update a cached key
	fmt.Println("\n7. Updating a cached key (write-through):")
	svc.SetConfig(ctx, "production", "log_level", "info", "ops-team")
	val, _ = svc.GetConfig(ctx, "production", "log_level")
	fmt.Printf("   log_level = %s (updated value served from cache)\n", val)

	// Demonstrate without cache for comparison
	fmt.Println("\n8. Comparison: Same operations WITHOUT cache:")
	nocacheSvc := NewConfigService(backing)
	start = time.Now()
	for i := 0; i < 10; i++ {
		nocacheSvc.GetConfig(ctx, "production", "db_host")
	}
	elapsed = time.Since(start)
	fmt.Printf("   10 reads took %v (every read hits the 'database')\n", elapsed.Round(time.Millisecond))

	fmt.Println(strings.Repeat("-", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- CachingConfigStore wraps any ConfigStore transparently")
	fmt.Println("- ConfigService doesn't know caching exists -- same interface")
	fmt.Println("- Write-through: Set() updates both backing store and cache")
	fmt.Println("- Cache is one wrapper; you could add logging, metrics, etc.")
	fmt.Println("- Composition: backing -> cache -> logging -> metrics -> service")
	fmt.Println("- Each layer satisfies the same ConfigStore interface")
}
