// Package cache provides a generic in-memory cache with TTL expiration and LRU eviction.
// Run tests: go test -v -race ./...
package cache

import (
	"container/list"
	"sync"
	"time"
)

// Options configures cache behavior.
type Options struct {
	// TTL is how long entries live before expiring. Zero means no expiration.
	TTL time.Duration
	// MaxSize is the maximum number of entries before LRU eviction kicks in.
	// Zero means unlimited.
	MaxSize int
}

// entry holds a cached value and its metadata.
// V is the value type — the same type parameter as the containing Cache.
type entry[V any] struct {
	value   V
	expires time.Time   // zero time means no expiration
	element *list.Element // position in the LRU list
}

// Cache is a generic, concurrent, in-memory cache with TTL expiration and LRU eviction.
//
// K must be comparable — it is used as a map key.
// V can be any type.
type Cache[K comparable, V any] struct {
	mu    sync.Mutex
	items map[K]*entry[V]
	lru   *list.List // front = most recently used, back = least recently used
	opts  Options
}

// New creates and returns a new Cache with the given options.
func New[K comparable, V any](opts Options) *Cache[K, V] {
	// TODO: Initialize and return a Cache.
	// Hints:
	//   - items should be a make(map[K]*entry[V])
	//   - lru should be list.New()
	panic("not implemented")
}

// Set stores key-value pair in the cache.
//
// If a TTL is configured, the entry expires after opts.TTL from now.
// If MaxSize is configured and the cache is full, the least recently used
// entry is evicted before inserting the new one.
// If the key already exists, its value and expiry are updated and it is
// moved to the front of the LRU list.
func (c *Cache[K, V]) Set(key K, value V) {
	// TODO: Implement
	//
	// Steps:
	// 1. Lock the mutex
	// 2. If key exists, update value, reset expiry, move to LRU front
	// 3. If key is new:
	//    a. Evict LRU entry if cache is full (MaxSize > 0 && len == MaxSize)
	//    b. Push key to front of LRU list
	//    c. Create entry with value, expiry, and list element
	//    d. Store in items map
	panic("not implemented")
}

// Get retrieves a value from the cache.
//
// Returns (value, true) if the key exists and has not expired.
// Returns (zero, false) if the key does not exist or has expired.
// A successful Get moves the entry to the front of the LRU list.
func (c *Cache[K, V]) Get(key K) (V, bool) {
	// TODO: Implement
	//
	// Steps:
	// 1. Lock the mutex (use RLock if you implement RWMutex, or regular Lock)
	// 2. Look up key in items
	// 3. If not found, return zero value and false
	// 4. If found but expired, delete from map, remove from LRU, return zero and false
	// 5. If found and valid, move to LRU front, return value and true
	//
	// Zero value pattern:
	//   var zero V
	//   return zero, false
	panic("not implemented")
}

// Delete removes a key from the cache. No-op if the key does not exist.
func (c *Cache[K, V]) Delete(key K) {
	// TODO: Implement
	panic("not implemented")
}

// Len returns the number of non-expired entries in the cache.
func (c *Cache[K, V]) Len() int {
	// TODO: Implement
	// Note: iterate items and count only non-expired entries.
	// Do not count entries where isExpired returns true.
	panic("not implemented")
}

// Flush removes all entries from the cache.
func (c *Cache[K, V]) Flush() {
	// TODO: Implement
	panic("not implemented")
}

// isExpired reports whether entry e has passed its expiration time.
// An entry with a zero expires time never expires.
func (c *Cache[K, V]) isExpired(e *entry[V]) bool {
	// TODO: Implement
	// Hint: use time.Now().After(e.expires) but guard against zero time.
	panic("not implemented")
}
