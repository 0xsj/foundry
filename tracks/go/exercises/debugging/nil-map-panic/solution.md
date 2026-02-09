# Solution — Nil Map Panic

## Root Cause

In Go, a **nil map can be read from but not written to**. The `NewCache()` constructor creates a `Cache` struct with an uninitialized `data` field, which defaults to `nil`.

```go
func NewCache() *Cache {
    return &Cache{}  // Cache.data is nil
}
```

When `Set()` tries to assign to `c.data[key]`, it panics because you cannot write to a nil map.

**Why does `Get()` work?** Reading from a nil map returns the zero value and `false` — it doesn't panic. This asymmetry is a common Go gotcha.

## The Bug

```go
// This panics if c.data is nil
c.data[key] = value
```

## The Fix

Initialize the map in the constructor using `make()`:

```go
func NewCache() *Cache {
    return &Cache{
        data: make(map[string]string),
    }
}
```

Alternatively, use a composite literal:

```go
func NewCache() *Cache {
    return &Cache{
        data: map[string]string{},  // empty but initialized
    }
}
```

## Why This Happens

**Mental Model Gap:** Coming from languages where maps/objects are automatically initialized (JavaScript, Python), the Go zero value of `nil` for maps is not immediately usable for writes.

- **nil slice**: Can be appended to (`append` handles nil)
- **nil map**: Cannot be written to (panics)

This inconsistency catches many Go beginners.

## How to Avoid

1. **Always use `make()` for maps** — never rely on zero value
2. **Constructor pattern**: Initialize maps in `NewXxx()` constructors
3. **Linting**: `go vet` catches *some* nil map writes (not all)

## Related Pitfalls

- [[go-nil-map-panic]] in the vault
- Nil slices are usable, nil maps are not
- Reading from nil map doesn't panic, writing does

## Debugging Techniques Used

1. **Read the panic message**: "assignment to entry in nil map" directly tells you the problem
2. **Check initialization**: Look at where the struct is created
3. **Zero values**: Remember that Go initializes all fields to their zero value
4. **REPL experimentation** (optional): Try `var m map[string]string; m["key"] = "val"` in a Go playground to reproduce

## Frequency

⭐⭐⭐⭐⭐ **Very common** for Go beginners, especially those coming from dynamic languages.
