# Expert Review: Resource Tracker PR

## Critical Issues

### 1. `sync.Pool` misuse — dirty objects returned to callers

**Location:** `Allocate()` and `Release()` — `pool.Get()` / `pool.Put()`

`sync.Pool` is designed for objects that are fully reset before reuse. The workflow here breaks that contract in two ways:

**Problem A:** `Release` puts `meta` back in the pool without zeroing it:
```go
t.pool.Put(meta) // meta still has ID, Kind, Tags, ExpiresAt from the released allocation
```

**Problem B:** `Allocate` calls `pool.Get()` but doesn't reset the object before use:
```go
meta := t.pool.Get().(*ResourceMeta)
// meta may contain stale fields from a previous allocation
meta.ID = id      // sets ID — OK
meta.Kind = kind  // sets Kind — OK
meta.AllocatedAt = time.Now() // sets AllocatedAt — OK
// meta.ExpiresAt is NOT reset — may contain the previous resource's expiry!
// meta.Tags is NOT reset — may contain the previous resource's tags!
```

If resource `"server-1"` had `ExpiresAt` set, was released, and then `"server-2"` was allocated (getting the recycled object), `"server-2"` would inherit `"server-1"`'s expiry. Silent data corruption.

**Fix — reset on Get, or reset on Put:**

```go
// Preferred: reset on Get (defensive — pool contents may have come from anywhere)
meta := t.pool.Get().(*ResourceMeta)
*meta = ResourceMeta{}  // zero out all fields before use

// Or: reset on Put (equivalent, but assumes all Put callers reset)
func (t *ResourceTracker) Release(id string) error {
    // ...
    *meta = ResourceMeta{}  // zero before returning to pool
    t.pool.Put(meta)
    // ...
}
```

**Concept:** `sync.Pool` is only safe for objects that are completely overwritten between uses, or explicitly reset. If any field might survive a put-get cycle, you have a data contamination bug. The pool in `BuildReport` (for `*bytes.Buffer`) correctly calls `buf.Reset()` before use — that's the pattern to follow.

---

## Major Concerns

### 2. `*sync.Mutex` embedded by pointer — mutex must be embedded by value

**Location:** `ResourceTracker` struct and `NewResourceTracker`

```go
type ResourceTracker struct {
    *sync.Mutex  // pointer to Mutex
    // ...
}

return &ResourceTracker{
    Mutex: &sync.Mutex{},
    // ...
}
```

`sync.Mutex` must not be copied after first use. Embedding a `*sync.Mutex` instead of `sync.Mutex` creates a separate allocation that can be (accidentally or deliberately) copied and shared by multiple `ResourceTracker` instances. Two trackers sharing the same mutex would interfere with each other's locking — a race condition.

The Go `sync` package documentation says explicitly: **"A Mutex must not be copied after first use."** Embedding by pointer also means the zero value of `ResourceTracker` has a nil mutex, which will panic on `Lock()`.

```go
// Fix: embed by value
type ResourceTracker struct {
    mu        sync.Mutex  // unexported, by value — cannot be copied
    resources map[string]*ResourceMeta
    // ...
}

func (t *ResourceTracker) Allocate(...) error {
    t.mu.Lock()
    defer t.mu.Unlock()
    // ...
}
```

Embedding `sync.Mutex` by value (exported, to use `t.Lock()`/`t.Unlock()` directly) is a valid pattern for small types. Using an unexported field `mu sync.Mutex` is cleaner — it doesn't pollute the exported API with `Lock` and `Unlock` methods that callers shouldn't call.

---

### 3. `*[]string` for `Tags` field — pointer to slice is unnecessary and confusing

**Location:** `ResourceMeta.Tags` field, `Allocate`, `GetTags`

```go
Tags *[]string  // pointer to slice
```

A slice is already a reference type — it's a 3-word header (pointer, length, capacity) that points to an underlying array. Taking a pointer to a slice adds an extra level of indirection with no benefit. It also makes the API awkward: callers get `*[]string` back from `GetTags`, and must dereference before ranging.

There's also a subtle bug: `meta.Tags = &tags` in `Allocate` takes the address of the `tags` parameter. In Go, function parameters are values — `tags` is a copy of the slice header, but it shares the underlying array with the caller's slice. The caller can still mutate the elements through their original slice, and those mutations will be visible through `meta.Tags`. This creates unexpected aliasing.

```go
// Fix: store []string directly, copy to avoid aliasing
type ResourceMeta struct {
    // ...
    Tags []string  // not *[]string
}

// In Allocate:
tagsCopy := make([]string, len(tags))
copy(tagsCopy, tags)
meta.Tags = tagsCopy

// GetTags returns []string directly
func (t *ResourceTracker) GetTags(id string) ([]string, bool) {
    // ...
    return meta.Tags, true
}
```

---

### 4. `*TrackerStats` field — pointer to small struct is unnecessary

**Location:** `ResourceTracker.stats *TrackerStats`

`TrackerStats` has three `int` fields — 24 bytes total. Storing it as `*TrackerStats` adds a heap allocation for a struct that's already inside a heap-allocated `ResourceTracker`. The pointer indirection makes reads and writes slower (one extra memory access to follow the pointer), not faster.

```go
// Current: two allocations (ResourceTracker + TrackerStats)
stats: &TrackerStats{}

// Fix: embed by value — one allocation, no pointer dereference
type ResourceTracker struct {
    // ...
    stats TrackerStats  // by value — 24 bytes inside ResourceTracker
}

// No change needed to Stats() — it already returns a copy:
func (t *ResourceTracker) Stats() TrackerStats {
    t.mu.Lock()
    defer t.mu.Unlock()
    return t.stats  // returns a copy of the embedded struct
}
```

The rule: use `*T` for fields when `T` is large (copying is expensive), when nil is a meaningful state, or when sharing one instance across multiple structs. For small, always-present, single-owner structs, embed by value.

---

## Minor Suggestions

### 5. `ExpiresAt *time.Time` — taking address of parameter is safe here

In `SetExpiry`, `meta.ExpiresAt = &expiry` takes the address of the `expiry` parameter (a local copy). This is fine — Go will allocate `expiry` on the heap since its address escapes. The value is correctly stored. (Contrast with `Tags = &tags` in Issue 3, which is a problem because the slice header points to shared underlying data.)

This is worth a brief comment to clarify intent:
```go
// expiry is a local copy (time.Time is a value type) — taking its address is safe.
// Go's escape analysis will heap-allocate it since the address escapes this function.
meta.ExpiresAt = &expiry
```

### 6. `PurgeExpired` doesn't return expired resources to pool correctly

After fixing Issue 1 (the pool reset bug), `PurgeExpired` should also reset objects before putting them in the pool:

```go
func (t *ResourceTracker) PurgeExpired() int {
    // ...
    for id, meta := range t.resources {
        if meta.ExpiresAt != nil && time.Now().After(*meta.ExpiresAt) {
            delete(t.resources, id)
            *meta = ResourceMeta{}  // reset before returning to pool
            t.pool.Put(meta)
            count++
        }
    }
    // ...
}
```

### 7. `ResourceKind` has no `String()` method

Error messages and logs will print integers (`kind=0`) instead of names (`kind=server`). For any `iota` enum that appears in log output or error strings, implement `fmt.Stringer`.

---

## Positive Feedback

- `ExpiresAt *time.Time` as an optional expiry field is the correct pattern — nil cleanly means "no expiry"
- `IsExpired` correctly checks for nil before dereferencing `ExpiresAt`
- `Stats()` correctly returns a value copy rather than a pointer to live stats
- `BuildReport` correctly uses `sync.Pool` — reset before use, return after use — this is the reference implementation for the pool pattern
- Mutex protection around all map access is correct
- `PurgeExpired` correctly iterates and deletes from a map in a single pass (safe in Go — deleting from a map during range is defined behavior)

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `sync.Pool` — dirty objects returned to callers | Pool objects must be reset before reuse |
| 2 | Major | `*sync.Mutex` embedded by pointer — mutex aliasing risk | Mutex must be embedded by value |
| 3 | Major | `*[]string` — unnecessary pointer to slice | Slices are already reference types |
| 4 | Major | `*TrackerStats` — unnecessary pointer to small struct | Prefer embedding small structs by value |
| 5 | Minor | `&expiry` in SetExpiry — correct but deserves a comment | Escape analysis + address-of-parameter |
| 6 | Minor | `PurgeExpired` doesn't reset before pool.Put | Consistent pool reset discipline |
| 7 | Minor | `ResourceKind` missing `String()` | fmt.Stringer on iota enums |
