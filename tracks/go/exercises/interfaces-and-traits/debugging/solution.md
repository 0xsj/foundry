# Debugging Solution

## Bug 1: Nil Interface Trap

**Location:** `newBackend` function

**The Symptom:** `TestNewBackendReturnsNilWhenNotConfigured` fails because `backend != nil` — even though the function intended to return "nothing."

**The Bug:**

```go
func newBackend(useDisk bool, diskPath string) Storer {
    var mem *MemoryBackend  // nil *MemoryBackend

    if useDisk {
        return NewFileBackend(diskPath)
    }
    return mem  // BUG: returns interface{type: *MemoryBackend, value: nil}
}
```

When `mem` (a nil `*MemoryBackend`) is returned as a `Storer` interface, Go wraps it into an interface value with:
- **dynamic type:** `*MemoryBackend`
- **dynamic value:** `nil`

That interface value is **not nil** — it has a type. The caller's `if backend == nil` check passes, but any method call on it dereferences the nil pointer and panics.

**The Fix:**

```go
func newBackend(useDisk bool, diskPath string) Storer {
    if useDisk {
        return NewFileBackend(diskPath)
    }
    return nil  // untyped nil — interface{type: nil, value: nil}
}
```

Return `nil` directly (the untyped nil), not a typed nil via a variable.

**Why it happens:** A `Storer` interface value is nil only if *both* its dynamic type and dynamic value are nil. Returning a typed variable — even a nil pointer — sets the dynamic type, making the interface non-nil. This is Go's most famous footgun, especially in functions that return interface types.

**Prevention:**
- When a function returns an interface, never use an intermediate typed variable for the "no result" case. Use `return nil`.
- Use the nil interface trap detector pattern in tests: assign the concrete nil to an interface and check `== nil`.

**Related pitfall:** [[go-nil-interface-trap]]

---

## Bug 2: Value Receiver on Mutating Methods

**Location:** `CacheManager.Set` and `CacheManager.Get`

**The Symptom:** `TestCacheManagerSetCount` fails with `SetCount = 0, want 3`. `Set` is called three times but the counter never changes.

**The Bug:**

```go
// BUG: value receiver — m is a copy of CacheManager
func (m CacheManager) Set(key string, value []byte) error {
    m.stats.SetCount++  // increments the copy's counter
    return m.backend.Set(key, value)
}   // copy is discarded — original m.stats.SetCount is unchanged
```

`CacheManager` is passed by value. `m.stats.SetCount++` modifies the copy's counter, not the original's. The actual Set to `m.backend` works (maps are reference types — the backend's map is shared), but the counter increment is lost.

This explains why `TestCacheManagerSetActuallyStores` passes — the data is stored correctly. Only the counter tracking fails.

**The Fix:**

```go
// Fix: pointer receiver — m is the original
func (m *CacheManager) Set(key string, value []byte) error {
    m.stats.SetCount++
    return m.backend.Set(key, value)
}

func (m *CacheManager) Get(key string) ([]byte, error) {
    m.stats.GetCount++
    return m.backend.Get(key)
}
```

**Why it happens:** Go is pass-by-value. A value receiver gets a copy of the struct. Any modification to the copy is discarded when the method returns. This is correct behavior for read-only methods but wrong for methods that update fields.

The insidious part: when the struct contains reference types (like a map or pointer), mutations *to the reference* propagate (the backend's map gets updated). But mutations *to the struct itself* (like updating a counter) do not.

**Prevention:**
- If any method needs to modify the struct, use pointer receivers on *all* methods for that type.
- If you see a method that both modifies state and calls other methods, check receiver consistency.
- Run `go vet` — it warns about inconsistent receiver types.

---

## Bug 3: Type Assertion Without Comma-Ok

**Location:** `CacheManager.InspectBackend`

**The Symptom:** `TestCacheManagerInspectUnknownBackend` panics with:
```
panic: interface conversion: cache.Storer is *cache.wrappedBackend, not *cache.FileBackend
```

**The Bug:**

```go
case *FileBackend:
    fb := m.backend.(*FileBackend)  // BUG: single-return — panics if type doesn't match
    return fmt.Sprintf("file backend at %s", fb.Path)
```

Inside the `*FileBackend` case of a type switch, using `m.backend.(*FileBackend)` *should* be safe — the switch guarantees the type. But this is a fragile pattern. In concurrent code, `m.backend` could be swapped between the type switch evaluation and the assertion. More importantly, it trains the habit of using unsafe assertions, which then get copy-pasted into contexts where they do panic.

In the test, `wrappedBackend` hits the `default` case in the switch — but the unsafe assertion code is a code smell that could panic in a slightly different arrangement.

**The Fix:**

```go
case *FileBackend:
    if fb, ok := m.backend.(*FileBackend); ok {
        return fmt.Sprintf("file backend at %s", fb.Path)
    }
    return "file backend (type assertion failed)"
```

Or, more idiomatically inside a type switch, capture the binding in the switch:

```go
switch st := m.backend.(type) {
case *MemoryBackend:
    _ = st  // st is *MemoryBackend here
    return "memory backend (no config)"
case *FileBackend:
    return fmt.Sprintf("file backend at %s", st.Path)  // st is *FileBackend here
default:
    return fmt.Sprintf("unknown backend: %T", st)
}
```

This is the cleanest form — bind the concrete type in the switch guard, access fields directly.

**Why it happens:** The single-return type assertion `x.(T)` panics at runtime if the dynamic type of `x` is not `T`. It's only safe when you have an absolute compile-time guarantee of the type — which you almost never do when working with interfaces.

**Prevention:**
- Always use the two-return form `v, ok := x.(T)` outside of type switches.
- Inside type switches, use `switch v := x.(type)` to bind the concrete type — then no assertion is needed.
- The only legitimate use of single-return assertions is in tests or when you've just assigned the value yourself.

---

## Summary

| Bug | Root Cause | Fix |
|-----|-----------|-----|
| `newBackend` returns non-nil interface | Returned typed nil (`var mem *MemoryBackend`) as interface | Return `nil` directly |
| `Set`/`Get` counters never update | Value receiver on mutating methods — modifies copy | Change to pointer receivers |
| `InspectBackend` unsafe assertion | Single-return type assertion outside a type switch binding | Use comma-ok form or bind type in switch guard |
