# Debugging Solution: Generics

## Bug 1: Missing `comparable` constraint for map key

**Location:** `Frequency[T any]`

**Symptom:** Compile error:
```
cannot use type T (type parameter) as a map key, T is not comparable
```

**Root cause:**

Go requires map keys to be `comparable` — types that support `==` and `!=`. This is enforced at the type parameter level. Constraining T to `any` allows any type, including slices and maps (which are not comparable) and structs with non-comparable fields.

When the compiler sees `make(map[T]int)` and `result[item]++`, it knows T must be used as a map key and enforces the `comparable` constraint — even if you don't write it in the type parameter.

**The bug:**
```go
func Frequency[T any](items []T) map[T]int {  // T any — too broad
```

**The fix:**
```go
func Frequency[T comparable](items []T) map[T]int {
```

**Why it matters:**

This is the most common constraint mistake. When you use a type parameter as a map key, you must constrain it to `comparable`. The compiler error is clear and immediate, which makes this the easiest bug to diagnose — just fix the constraint.

**Key insight:** `comparable` is a subset of `any`. Every `comparable` type satisfies `any`, but not every `any` type satisfies `comparable`. Changing `any` to `comparable` makes the constraint more restrictive, which is correct here.

---

## Bug 2: Missing `~` for named underlying types

**Location:** `StringLike` constraint, `WrapAll`

**Symptom:** Compile error when calling `WrapAll` with `[]Tag` or `[]EventName`:
```
Tag does not satisfy StringLike (Tag missing in ~string)
```

`WrapAll([]string{...}, ...)` works fine — only named types fail.

**Root cause:**

`interface{ string }` means "exactly the type `string`". It does NOT include named types like `Tag` whose underlying type is `string`. `Tag` and `string` are different types in Go's type system.

The `~` operator means "this type, or any type whose underlying type is this type." It's the bridge between the constraint and domain-specific named types.

**The bug:**
```go
type StringLike interface {
    string  // only accepts exact type string
}
```

**The fix:**
```go
type StringLike interface {
    ~string  // accepts string AND any named type with underlying type string
}
```

**Why it matters:**

In production code, you almost always work with named types (`UserID`, `EventName`, `Tag`, `Slug`) rather than raw primitives. Without `~`, your generic functions silently exclude all of them. The fix is always to ask: "should this work with named types too?" — and if yes, add `~`.

**Key insight:** Always use `~T` instead of `T` in union constraints unless you specifically want to exclude named types. The case where you want `T` without `~` (exact type only) is rare.

---

## Bug 3: Constraint too broad to call methods

**Location:** `SumScores[T any]`

**Symptom:** Function compiles but returns 0.0 instead of the correct sum. No compile error.

**Root cause:**

`T any` allows T to be anything. Since the compiler has no guarantee that T has a `Value() float64` method, `item.Value()` would be a compile error. The existing code works around this with a `fmt.Sscanf`/`fmt.Sprintf` trick — but `fmt.Sprintf("%v", item)` formats a `LatencyScore` as something like `{12.5}` (the struct representation), not `12.5`, so `fmt.Sscanf` fails to parse it and `val` stays 0.

This is a logic bug disguised as working code: it compiles, runs, and returns a number — just the wrong one.

**The bug:**
```go
func SumScores[T any](scores []T) float64 {
    total := 0.0
    for _, item := range scores {
        // fmt.Sprintf("%v", item) formats LatencyScore as "{12.5}", not "12.5"
        // Sscanf fails silently, val stays 0
        var val float64
        fmt.Sscanf(fmt.Sprintf("%v", item), "%f", &val)
        total += val
    }
    return total
}
```

**The fix:**
```go
func SumScores[T Score](scores []T) float64 {
    total := 0.0
    for _, item := range scores {
        total += item.Value()  // now valid: T is constrained to Score
    }
    return total
}
```

**Why it matters:**

This is the most insidious of the three bugs because it doesn't produce a compile error. It's an anti-pattern: using reflection-like tricks (`fmt.Sprintf`) to avoid proper constraint declaration. The correct approach is always to express *what you need from T* in the constraint. If you need `Value() float64`, add that to the constraint.

**Key insight:** When your generic function needs to call a method on T, that method must appear in the constraint. If it doesn't, you either have the wrong constraint, or the function shouldn't be generic at all (use a regular interface parameter instead).

---

## Related Pitfalls

- `comparable` vs `any` — always use `comparable` when T is a map key
- `~T` vs `T` — use `~T` when you want to include named types with underlying type T
- Constraint-method access — T's methods are only callable if the constraint declares them
