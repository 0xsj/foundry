// Package cache provides a generic in-memory cache with TTL expiration and LRU eviction.
package cache

import (
	"container/list"
	"sync"
	"time"
)

// Options configures cache behavior.
type Options struct {
	TTL     time.Duration
	MaxSize int
}

// entry holds a cached value plus the metadata needed for TTL and LRU tracking.
//
// The entry is generic over V — it stores exactly the type the cache was
// instantiated with. No boxing, no interface{}, no type assertions at runtime.
type entry[V any] struct {
	value   V
	expires time.Time    // zero means no expiration
	element *list.Element // points to this entry's node in the LRU list
}

// Cache is a generic, concurrent in-memory cache.
//
// K must be comparable — it is used as a map key (compiler enforces this).
// V is unconstrained — any value type works.
//
// The zero value is not usable — always use New().
type Cache[K comparable, V any] struct {
	mu    sync.Mutex
	items map[K]*entry[V]
	lru   *list.List // front = MRU, back = LRU; each element stores K (the key)
	opts  Options
}

// New creates and returns an initialized Cache.
//
// Type inference: New[string, int](opts) can usually be written as
// New[string, int](opts) because both K and V must be specified — they
// don't appear in the Options argument and cannot be inferred.
func New[K comparable, V any](opts Options) *Cache[K, V] {
	return &Cache[K, V]{
		items: make(map[K]*entry[V]),
		lru:   list.New(),
		opts:  opts,
	}
}

// Set stores a key-value pair.
//
// Key design decisions:
// - If the key already exists, we update in-place and move to front.
//   This avoids growing the list unnecessarily.
// - If the cache is at capacity, we evict before inserting — so MaxSize is
//   a hard cap, not a soft hint.
// - The list element stores the key (K), not the entry pointer. This lets
//   us look up and delete from the map in O(1) during eviction.
func (c *Cache[K, V]) Set(key K, value V) {
	c.mu.Lock()
	defer c.mu.Unlock()

	// Update existing entry
	if e, ok := c.items[key]; ok {
		e.value = value
		e.expires = c.expireTime()
		c.lru.MoveToFront(e.element)
		return
	}

	// Evict LRU entry if at capacity
	if c.opts.MaxSize > 0 && len(c.items) >= c.opts.MaxSize {
		c.evictLRU()
	}

	// Insert new entry
	elem := c.lru.PushFront(key) // list element stores K so eviction can find the map key
	c.items[key] = &entry[V]{
		value:   value,
		expires: c.expireTime(),
		element: elem,
	}
}

// Get retrieves a value from the cache.
//
// The zero value of V (`var zero V`) is the idiomatic way to return "nothing"
// when V is unknown. You can't write 0 or "" or nil — V could be any type.
func (c *Cache[K, V]) Get(key K) (V, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()

	e, ok := c.items[key]
	if !ok {
		var zero V
		return zero, false
	}

	// Treat expired entries as absent; clean them up lazily
	if c.isExpired(e) {
		c.lru.Remove(e.element)
		delete(c.items, key)
		var zero V
		return zero, false
	}

	// Hit: move to front so this entry is now the MRU
	c.lru.MoveToFront(e.element)
	return e.value, true
}

// Delete removes a key from the cache. Safe to call on missing keys.
func (c *Cache[K, V]) Delete(key K) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if e, ok := c.items[key]; ok {
		c.lru.Remove(e.element)
		delete(c.items, key)
	}
}

// Len returns the count of non-expired entries.
//
// Note: this is O(n) when TTL is set, because we must check each entry.
// For a production cache you'd track expiry separately for O(1) Len.
// Here we prioritize correctness over micro-optimization.
func (c *Cache[K, V]) Len() int {
	c.mu.Lock()
	defer c.mu.Unlock()

	count := 0
	for _, e := range c.items {
		if !c.isExpired(e) {
			count++
		}
	}
	return count
}

// Flush removes all entries from the cache and resets the LRU list.
func (c *Cache[K, V]) Flush() {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.items = make(map[K]*entry[V])
	c.lru.Init()
}

// ============================================================================
// Internal helpers (unexported)
// ============================================================================

// expireTime returns the expiration time for a new entry based on the configured TTL.
// Zero time means no expiration.
func (c *Cache[K, V]) expireTime() time.Time {
	if c.opts.TTL == 0 {
		return time.Time{} // zero time = never expires
	}
	return time.Now().Add(c.opts.TTL)
}

// isExpired reports whether entry e has passed its expiration time.
func (c *Cache[K, V]) isExpired(e *entry[V]) bool {
	return !e.expires.IsZero() && time.Now().After(e.expires)
}

// evictLRU removes the least recently used entry from the cache.
// Caller must hold c.mu.
func (c *Cache[K, V]) evictLRU() {
	back := c.lru.Back()
	if back == nil {
		return
	}
	// The list element's Value is the key K we stored when inserting
	evictKey := back.Value.(K)
	c.lru.Remove(back)
	delete(c.items, evictKey)
}
