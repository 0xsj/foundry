# Variants: Generic Cache

## Variant A: Simple Cache (no LRU, no TTL)

The simplest possible generic cache — just a type-safe map with mutex:

```go
type SimpleCache[K comparable, V any] struct {
    mu    sync.RWMutex
    items map[K]V
}

func (c *SimpleCache[K, V]) Set(key K, val V) {
    c.mu.Lock()
    defer c.mu.Unlock()
    c.items[key] = val
}

func (c *SimpleCache[K, V]) Get(key K) (V, bool) {
    c.mu.RLock()
    defer c.mu.RUnlock()
    v, ok := c.items[key]
    return v, ok
}
```

Use when: You just need thread-safe key-value storage with type safety. No eviction, no TTL.

**Notice**: `RWMutex` is viable here because `Get` doesn't mutate state (no LRU update). This is better for read-heavy workloads.

## Variant B: TTL only (no LRU)

Drop the `container/list` entirely if you don't need capacity bounding:

```go
type entry[V any] struct {
    value   V
    expires time.Time
}

type TTLCache[K comparable, V any] struct {
    mu    sync.Mutex
    items map[K]entry[V]
    ttl   time.Duration
}
```

No `*list.Element` needed. `Get` just checks `time.Now().After(e.expires)`. Simpler, but unbounded growth.

Use when: You need expiry but don't care about memory bounds (e.g., a short-lived request cache).

## Variant C: Background eviction goroutine

Instead of lazy expiry in `Get`, run a background ticker that evicts all expired entries periodically:

```go
func (c *Cache[K, V]) startEvictionLoop(interval time.Duration) {
    go func() {
        ticker := time.NewTicker(interval)
        defer ticker.Stop()
        for range ticker.C {
            c.mu.Lock()
            for k, e := range c.items {
                if c.isExpired(e) {
                    c.lru.Remove(e.element)
                    delete(c.items, k)
                }
            }
            c.mu.Unlock()
        }
    }()
}
```

Tradeoffs:
- Pro: `Len()` becomes O(1) (all entries in the map are valid)
- Pro: Memory freed proactively, not lazily
- Con: Goroutine leak if cache is abandoned without calling `Close()`
- Con: Requires a `Close()` method and channel to signal shutdown

Use when: You have long-lived caches with high TTL churn and want predictable memory usage.

## Tradeoff Summary

| Variant | Memory | Get latency | Set latency | Complexity | Best for |
|---------|--------|-------------|-------------|-----------|----------|
| Simple | Unbounded | O(1) | O(1) | Low | Read-heavy, short-lived |
| TTL only | Unbounded | O(1) | O(1) | Low | Expiry without size limits |
| LRU + TTL (reference) | Bounded | O(1) | O(1) | Medium | General-purpose |
| Background eviction | Bounded | O(1) | O(1) | High | Long-lived, high churn |
