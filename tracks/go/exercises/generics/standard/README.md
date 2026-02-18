# Exercise: Generic In-Memory Cache

## Scenario

You're building a shared caching library for a microservices platform. Several services need to cache expensive lookups — parsed configs, user session data, rate-limit counters, feature flags — but each service works with different key and value types. A stringly-typed `map[string]interface{}` cache exists already, but it causes runtime panics when values are cast to the wrong type, and the lack of type safety makes the code hard to reason about. Your job is to replace it with a generic cache that enforces key-value types at compile time, supports TTL-based expiration, and evicts the least recently used entries when the cache grows beyond its configured capacity.

## Brief

Implement `Cache[K, V]` — a generic, thread-safe, in-memory cache with TTL expiration and LRU eviction. The cache must be parameterized over key type `K` (must be `comparable` for use as map key) and value type `V` (unconstrained).

## Acceptance Criteria

- [ ] `New[K comparable, V any](opts Options) *Cache[K, V]` — constructor with a config struct
- [ ] `Set(key K, value V)` — stores a value; sets expiration from `Options.TTL` if non-zero
- [ ] `Get(key K) (V, bool)` — returns value and true if present and not expired; returns zero value and false otherwise
- [ ] `Delete(key K)` — removes a key
- [ ] `Len() int` — returns number of non-expired entries
- [ ] TTL: expired entries return false from `Get` and are not counted in `Len`
- [ ] LRU eviction: when `Options.MaxSize > 0` and the cache is full, `Set` evicts the least recently used entry before inserting the new one. "Recently used" means the most recent `Get` or `Set`.
- [ ] `Flush()` — removes all entries
- [ ] All methods are safe for concurrent use (use `sync.RWMutex` or `sync.Mutex`)

## Types Provided (do not change)

```go
// Options configures cache behavior.
type Options struct {
    // TTL is how long entries live before expiring. Zero means no expiration.
    TTL time.Duration
    // MaxSize is the maximum number of entries. Zero means unlimited.
    MaxSize int
}
```

## Constraints

- Standard library only (`sync`, `time`, `container/list`)
- `K` must satisfy `comparable` — enforce this at the type parameter level
- LRU eviction requires tracking access order — use `container/list` for O(1) eviction
- Do not use `interface{}` or `any` for values inside the implementation
- Thread safety is required — the tests run with `-race`

## Concepts Exercised

- Generic type declaration (`Cache[K, V]`)
- `comparable` constraint for map keys
- Generic constructor function with type inference
- Zero value of a type parameter (`var zero V`)
- Generic struct with methods
- `sync.Mutex` for concurrent access (covered in detail in the concurrency module)
- `container/list` for doubly linked list (LRU access order tracking)

## Hints

<details>
<summary>Hint 1: Cache structure</summary>

The cache needs to store both the value and metadata (expiry time, position in the LRU list):

```go
type entry[V any] struct {
    value   V
    expires time.Time     // zero time means no expiration
    element *list.Element // position in the LRU list
}

type Cache[K comparable, V any] struct {
    mu      sync.Mutex
    items   map[K]*entry[V]
    lru     *list.List        // front = most recent, back = least recent
    opts    Options
}
```

Each `list.Element` stores the key (so when we pop from the back, we know which map entry to delete).
</details>

<details>
<summary>Hint 2: LRU tracking</summary>

When an entry is accessed (Get or Set), move it to the front of the list:

```go
// On Get hit: move to front
l.lru.MoveToFront(entry.element)

// On Set new entry: push to front, store element in entry
elem := l.lru.PushFront(key)
entry.element = elem

// On eviction: take from back
back := l.lru.Back()
if back != nil {
    evictKey := back.Value.(K)  // the list element stores the key
    l.lru.Remove(back)
    delete(l.items, evictKey)
}
```
</details>

<details>
<summary>Hint 3: TTL expiry check</summary>

```go
func (c *Cache[K, V]) isExpired(e *entry[V]) bool {
    // zero time means no expiration
    return !e.expires.IsZero() && time.Now().After(e.expires)
}
```

In `Get`, check expiry before returning. If expired, delete the entry and return the zero value:

```go
if c.isExpired(e) {
    c.lru.Remove(e.element)
    delete(c.items, key)
    var zero V
    return zero, false
}
```
</details>

<details>
<summary>Hint 4: Zero value of a type parameter</summary>

You cannot write `return nil, false` or `return 0, false` because V could be anything. The idiomatic pattern is:

```go
var zero V
return zero, false
```

This returns whatever the zero value of V is: `0` for int, `""` for string, `nil` for pointers and slices, `false` for bool.
</details>
