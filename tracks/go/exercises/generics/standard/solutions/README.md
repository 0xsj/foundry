# Solution: Generic In-Memory Cache

## Approach

The solution implements `Cache[K, V]` using a `map[K]*entry[V]` for O(1) lookup and a `container/list.List` for O(1) LRU tracking. The key design decisions:

**1. Entry stores a `*list.Element`, not an index**

The LRU list is a doubly linked list. Each `entry` holds a pointer directly to its node in the list. This gives O(1) `MoveToFront` on access — no scanning, no searching.

**2. List elements store keys (K), not values**

When eviction runs, we need to remove the entry from both the LRU list and the map. Storing the key in the list element lets `evictLRU` do `delete(c.items, back.Value.(K))` directly. The type assertion `.(K)` is safe because we control every push.

**3. Lazy TTL expiration**

Expired entries are cleaned up on `Get` rather than via a background goroutine. This keeps the implementation simple. The tradeoff: stale entries stay in the map until someone tries to read them, and `Len` must iterate to count non-expired entries (O(n)). A production cache would use a min-heap or time wheel for O(log n) expiry, but that's out of scope here.

**4. `var zero V` for the zero value**

When returning "nothing" (miss, expiry), we need the zero value of V. Since V is unknown at compile time, `var zero V` is the only way to get it. You cannot write `return nil, false` or `return 0, false` because V might be a struct.

## Key Decisions

| Decision | Alternative | Why we chose this |
|----------|------------|-------------------|
| `container/list` for LRU | Slice with index tracking | O(1) move-to-front vs O(n) scan |
| Lazy TTL cleanup | Background goroutine | Simpler; no goroutine leak |
| `sync.Mutex` (not RWMutex) | `sync.RWMutex` | Get promotes to write on hit (MoveToFront), so an RWMutex would need upgrading on every hit — not a clean fit |
| `list.Element.Value.(K)` assertion | Separate key field | Keeps entry[V] simpler; assertion is safe because we control all pushes |

## Performance Notes

| Operation | Time | Notes |
|-----------|------|-------|
| `Get` (hit) | O(1) | Map lookup + list move |
| `Get` (miss/expired) | O(1) | Map lookup |
| `Set` (new) | O(1) | Map insert + list push + possible eviction (list.Back() is O(1)) |
| `Set` (update) | O(1) | Map update + list move |
| `Delete` | O(1) | Map delete + list remove |
| `Len` | O(n) | Iterates map to skip expired entries |
| `Flush` | O(1) | Replace map + reinit list |

## Variants

See `variants/` for alternative implementations.
