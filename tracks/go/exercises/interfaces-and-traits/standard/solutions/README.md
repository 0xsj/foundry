# Solutions: Pluggable Storage Backend

## Reference Solution Approach

The solution builds the storage system bottom-up: errors first, then interfaces, then implementations, then the decorator.

### Key decisions

**`NotFoundError` as a concrete type, not `fmt.Errorf`**

```go
type NotFoundError struct{ Key string }
func (e *NotFoundError) Error() string { return fmt.Sprintf("key %q not found", e.Key) }
```

Using `fmt.Errorf("key not found")` would work for callers who just check `err != nil`. But a concrete type enables `errors.As`:

```go
var nfe *NotFoundError
if errors.As(err, &nfe) {
    log.Printf("cache miss for key: %s", nfe.Key) // structured, not string parsing
}
```

**`MemoryStore.Get` returns a copy of the value**

```go
result := make([]byte, len(v))
copy(result, v)
return result, nil
```

If you return `m.data[key]` directly, the caller holds a reference to the internal slice. Any mutation by the caller corrupts the store's data. Copying is the safe default. (For high-throughput use cases, `io.Reader` semantics or immutable values are better — but for a learning exercise, explicit copy is clearest.)

**`TransactionalStore` embeds `Store`**

```go
type TransactionalStore interface {
    Store
    Begin() (Transaction, error)
}
```

`MemoryTxStore` satisfies `TransactionalStore`, but it can be passed anywhere a `Store` is expected. The caller doesn't need to know it's transactional unless they specifically call `Begin`. This is the "interface upgrade" pattern from the composition example.

**`MemoryTxStore` embeds `MemoryStore` by value**

```go
type MemoryTxStore struct {
    MemoryStore // embedded by value — promotes Get, Set, Delete, List
}
```

Embedding by value means `MemoryTxStore` gets all four `Store` methods promoted for free. Only `Begin()` needed to be added. If we embedded by pointer (`*MemoryStore`), we'd need to initialize it explicitly and handle the nil case.

**`MetricsStore` — the Decorator pattern**

`MetricsStore` has the same interface as `Store` but adds counting behavior around each call. The caller never needs to change — you can swap `NewMemoryStore()` for `NewMetricsStore(NewMemoryStore())` and everything works transparently. This is the Decorator pattern, which gets its own module later.

## Comparison of Variants

| Approach | Tradeoff |
|----------|---------|
| Reference: concrete `NotFoundError` | Callers can extract the key via `errors.As`; slightly more boilerplate |
| `fmt.Errorf("key %q not found", key)` | Simpler; callers can only check `err != nil` or string-match (fragile) |
| `errors.New("key not found")` | Simplest; loses the key completely |

| Approach | Tradeoff |
|----------|---------|
| Reference: `MemoryTxStore` embeds `MemoryStore` by value | Zero initialization; promotes all methods automatically |
| Embedding `*MemoryStore` by pointer | Allows sharing same store; requires explicit initialization; nil risk |
| Separate `txStore` with a `MemoryStore` field | More explicit; breaks promotion — must write delegation methods manually |

| Approach | Tradeoff |
|----------|---------|
| Reference: `MetricsStore.List` not counted | `ReadCount` reflects single-key lookups only; cleaner semantics |
| Count all reads including List | Simpler; `ReadCount` less meaningful — one List could be 1000 logical reads |
| Separate `ListCount` counter | Most precise; more API surface |

## Performance Notes

`MemoryStore` uses `sync.RWMutex` — concurrent reads don't block each other, only writes block. For a read-heavy workload, this matters.

`FileStore` reads and writes the entire file on every operation. For production use, you'd want to keep an in-memory cache and flush on writes (write-through cache), or use a proper embedded database like BoltDB or SQLite.

The `MetricsStore.Mutex` is separate from the backend's mutex. Counter increments and backend calls are not done under the same lock — this means the counts might be slightly ahead of the actual backend operations in a concurrent scenario. For exact ordering guarantees, both operations would need to be under the same lock (which would require intrusive changes to the backend). For monitoring purposes, off-by-one under concurrent load is acceptable.
