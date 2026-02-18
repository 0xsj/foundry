# Exercise: Pluggable Storage Backend

## Scenario

Your platform stores configuration, session tokens, and feature flags in a key-value store. Initially everything used an in-memory store, but you now need to support file-backed storage for persistence across restarts. A transactional layer is required for configuration writes — changes should be atomic or not applied at all. The system must be testable without touching disk.

## Brief

Implement a storage backend system with the following types:

1. **`Store` interface** — the base interface: `Get`, `Set`, `Delete`, `List`
2. **`MemoryStore`** — in-memory implementation of `Store`
3. **`FileStore`** — file-backed implementation of `Store` (reads/writes a JSON file)
4. **`Transaction`** interface — `Commit() error` and `Rollback() error`
5. **`TransactionalStore` interface** — composes `Store` + `Begin() (Transaction, error)`
6. **`MemoryTxStore`** — extends `MemoryStore` to support transactions (buffered writes, committed atomically)
7. **`MetricsStore`** — wraps any `Store` and counts reads/writes (Decorator pattern preview)

### Store Interface

```go
type Store interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
    List(prefix string) ([]string, error)
}
```

### Transaction Interface

```go
type Transaction interface {
    Set(key string, value []byte) error
    Delete(key string) error
    Commit() error
    Rollback() error
}
```

### TransactionalStore Interface

```go
type TransactionalStore interface {
    Store
    Begin() (Transaction, error)
}
```

### MetricsStore

```go
type MetricsStore struct {
    // wraps a Store (any implementation)
    // tracks ReadCount, WriteCount, DeleteCount
}

func NewMetricsStore(backend Store) *MetricsStore
func (m *MetricsStore) Metrics() StoreMetrics
// Also implements Store by delegating to backend
```

## Acceptance Criteria

- [ ] `Store` interface defined with `Get`, `Set`, `Delete`, `List`
- [ ] `MemoryStore` satisfies `Store` with thread-safe operations (`sync.RWMutex`)
- [ ] `FileStore` satisfies `Store`; reads/writes a JSON file; returns `NotFoundError` on missing keys
- [ ] `Transaction` interface defined with `Set`, `Delete`, `Commit`, `Rollback`
- [ ] `TransactionalStore` composes `Store` and `Begin()`
- [ ] `MemoryTxStore` satisfies `TransactionalStore`; uncommitted writes are invisible to `Get`/`List`
- [ ] `Commit()` applies buffered changes atomically; `Rollback()` discards them
- [ ] `MetricsStore` wraps any `Store` and counts `ReadCount`, `WriteCount`, `DeleteCount`
- [ ] `MetricsStore` satisfies `Store` (delegation)
- [ ] `NotFoundError` is a custom error type with the missing key; `errors.As` works on it
- [ ] All interface guards (`var _ Store = ...`) present for all implementations
- [ ] All tests in `starter/main_test.go` pass

## Constraints

- Standard library only — no third-party packages
- `MemoryStore` must be safe for concurrent use (use `sync.RWMutex`)
- `FileStore` uses `encoding/json` for serialization; file format is `map[string]string`
- `FileStore.Get` returns `*NotFoundError` (not a generic error) for missing keys
- `MemoryTxStore.Begin()` returns a new transaction — concurrent transactions are not required
- `MetricsStore` must work with any `Store` — including `MemoryStore`, `FileStore`, and another `MetricsStore`
- Do not use global state — all state lives in structs

## Concepts Exercised

- Interface definition and implicit satisfaction
- Interface composition (`TransactionalStore` embeds `Store`)
- Interface guards (`var _ Store = (*MemoryStore)(nil)`)
- Accept interfaces, return structs (constructors return concrete types)
- Value vs pointer receivers and interface satisfaction
- Custom error type (`NotFoundError`) satisfying the `error` interface
- `errors.As` for typed error extraction
- Decorator pattern preview: `MetricsStore` wraps `Store`

## Hints

<details>
<summary>Hint 1: Interface guard syntax</summary>

```go
var _ Store = (*MemoryStore)(nil)
```

This line doesn't allocate anything — it's a compile-time check that `*MemoryStore` satisfies `Store`. If a method is missing, you get a compile error at this line with the exact method name.
</details>

<details>
<summary>Hint 2: MemoryTxStore transaction — buffering writes</summary>

```go
type memoryTx struct {
    parent  *MemoryTxStore
    pending map[string][]byte   // keys to write on commit
    deleted map[string]struct{} // keys to delete on commit
}
```

When `Commit()` is called, apply `pending` and `deleted` to the parent store's data map (under lock). `Rollback()` just discards the `memoryTx`.
</details>

<details>
<summary>Hint 3: MetricsStore — wrapping any Store</summary>

```go
type MetricsStore struct {
    backend Store
    metrics StoreMetrics
    mu      sync.Mutex
}

func (m *MetricsStore) Get(key string) ([]byte, error) {
    m.mu.Lock()
    m.metrics.ReadCount++
    m.mu.Unlock()
    return m.backend.Get(key)
}
```

`MetricsStore` satisfies `Store` by implementing all four methods, each of which increments a counter and delegates to `backend`.
</details>

<details>
<summary>Hint 4: FileStore — using encoding/json</summary>

```go
type FileStore struct {
    path string
    mu   sync.RWMutex
}

// load reads the file and unmarshals to map[string]string
// Returns an empty map if the file doesn't exist yet.

// save marshals the map and writes it atomically (write to temp, rename).
```

Use `os.ReadFile` / `os.WriteFile`. For bonus correctness, write to a temp file and rename — but `os.WriteFile` is acceptable for this exercise.
</details>
