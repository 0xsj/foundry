---
title: "Go: Nil Map Panic"
languages: [go]
category: fundamentals
severity: high
related: [[variables-and-types]], [[maps-and-sets]]
---

# Go: Nil Map Panic

## The Mistake

```go
var m map[string]int
m["key"] = 42  // PANIC: assignment to entry in nil map
```

**What goes wrong:** Runtime panic when trying to write to a nil map.

---

## Why It Happens

**Mental Model Gap:** Coming from languages where maps/objects are automatically initialized (JavaScript, Python, Ruby), the Go zero value of `nil` for maps is surprising.

**Key behavior:**
- A nil map **can be read from** (returns zero value, no panic)
- A nil map **cannot be written to** (panics)

This asymmetry with slices (nil slices can be appended to) makes it extra confusing.

```go
// This works fine
var m map[string]int
val := m["key"]        // returns 0, no panic

// This panics
m["key"] = 42          // PANIC
```

---

## The Fix

Always initialize maps with `make()` or a composite literal:

```go
// Option 1: make
m := make(map[string]int)
m["key"] = 42  // OK

// Option 2: composite literal
m := map[string]int{}
m["key"] = 42  // OK

// Option 3: with initial values
m := map[string]int{
    "key": 42,
}
```

---

## How to Avoid

1. **Always use `make()` for maps** — never rely on zero value for writes
2. **Constructor pattern**: Initialize maps in constructor functions
   ```go
   type Cache struct {
       data map[string]string
   }

   func NewCache() *Cache {
       return &Cache{
           data: make(map[string]string),
       }
   }
   ```
3. **Lint rules**: `go vet` catches *some* nil map writes (not all cases)
4. **Code review checklist**: Watch for `var m map[...]` without initialization

---

## Comparison to Slices

| Type | Zero Value | Read | Write | Append |
|---|---|---|---|---|
| `[]T` | `nil` | Returns zero | Panics | Works (returns new slice) |
| `map[K]V` | `nil` | Returns zero | **Panics** | N/A |

**Why the difference?** `append()` is designed to handle nil slices and return a new slice. Maps have no equivalent function — you write directly to the map.

---

## Related Concepts

- [[variables-and-types#zero-values]]
- [[maps-and-sets]]
- [[constructors-and-initialization]]

---

## Frequency

⭐⭐⭐⭐⭐ (Very common for Go beginners)

---

## Real-World Impact

**Production incidents:** This bug often appears when structs with map fields are created without proper initialization. The code compiles fine, passes simple tests, but crashes when first used in production.

**Example:**
```go
type Handler struct {
    cache map[string]Response  // oops, never initialized
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
    h.cache[r.URL.Path] = response  // PANIC in production
}
```

**Lesson:** Always initialize maps in constructors or use `sync.Map` for concurrent access (which handles nil safely).
