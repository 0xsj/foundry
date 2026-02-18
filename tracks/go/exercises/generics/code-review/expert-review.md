# Expert Review: Generic Utility Functions

## Critical Issues

### 1. `LookupOrDefault`: Missing `comparable` constraint on K

**Location:** `LookupOrDefault[K any, V any]`

`K` is used as a map key (`m[key]`). Go requires map keys to be `comparable`. The `K any` constraint will cause a compile error:

```
cannot use type K (type parameter) as a map key, K is not comparable
```

The function as written doesn't compile.

**Fix:**
```go
func LookupOrDefault[K comparable, V any](m map[K]V, key K, def V) V {
```

**Concept:** Any type parameter used as a map key, in a map literal, or compared with `==` must be constrained to `comparable`. This is the single most common generics compile error. See [[go-generics-comparable-constraint]] in the vault.

---

## Major Concerns

### 2. `Transform`: Three type parameters when two would do — and a simpler signature would do better

**Location:** `Transform[A, B, C any]`

The three-parameter version `Transform[A, B, C any](in []A, fn func(A) B, postProcess func(B) C)` is a composition of two separate `Map` operations:

```go
Transform(names, strings.ToUpper, func(s string) string { return "[" + s + "]" })
// is equivalent to:
Map(Map(names, strings.ToUpper), func(s string) string { return "[" + s + "]" })
```

Looking at actual usage in `exampleUsage()`, the caller always wants this:

```go
Transform(names, strings.ToUpper, func(s string) string { return "[" + s + "]" })
```

The real question: does this three-argument form exist in 15 codebases? Or was it invented for a single call site? If callers routinely chain two transforms, expose a two-parameter `Map` and let them compose:

```go
// Simpler: two type parameters, one transform
func Map[T, U any](in []T, fn func(T) U) []U {
    result := make([]U, len(in))
    for i, v := range in {
        result[i] = fn(v)
    }
    return result
}

// Usage is explicit and readable:
result := Map(Map(names, strings.ToUpper), addBrackets)
```

The three-type-parameter version adds cognitive overhead for every reader who encounters it. They must mentally track A, B, and C. `Map` + `Map` is self-documenting.

**When three type parameters are justified:** When the relationship between all three types is genuinely necessary — for example, a function that zips two slices `[]A` and `[]B` into `[]C`. Here, C is purely derivable from B via a function, so it adds no information.

**Fix:** Replace with a simpler two-parameter `Map`:
```go
func Map[T, U any](in []T, fn func(T) U) []U { ... }
```

---

### 3. `FormatDuration`: Generic where a regular `time.Duration` function is cleaner

**Location:** `FormatDuration[T Numeric]`

The function's purpose is to format durations as human-readable strings. The Go standard library has `time.Duration` for exactly this purpose — `time.Duration` is `int64` nanoseconds and has methods like `String()`, `Seconds()`, `Minutes()`, `Milliseconds()`.

The generic version:
1. Accepts `int`, `float32`, `uint8`, etc. — most of which make no sense for a duration formatter
2. Assumes the input is in milliseconds — but `time.Duration` is in nanoseconds. The `exampleUsage` demonstrates this confusion: `FormatDuration(time.Second * 2 / 1e6)` is incorrect — `time.Second * 2` is `2000000000` nanoseconds, dividing by `1e6` gives `2000` (the millisecond value), but this is non-obvious and fragile.
3. Forces callers to remember "this is in milliseconds" — a plain `time.Duration` self-documents its unit

**The idiomatic Go version:**
```go
// FormatDuration formats a time.Duration as a human-readable string.
// No generics needed — time.Duration is the right type for durations.
func FormatDuration(d time.Duration) string {
    switch {
    case d < time.Second:
        return fmt.Sprintf("%dms", d.Milliseconds())
    case d < time.Minute:
        return fmt.Sprintf("%.1fs", d.Seconds())
    default:
        return fmt.Sprintf("%.1fm", d.Minutes())
    }
}
```

This is clearer, uses the standard library correctly, and doesn't require callers to know about the milliseconds assumption.

**Rule of thumb:** Don't use generics to make a function work with "any numeric type" when there's a more specific type that exactly represents your domain. `time.Duration` is the duration type. Use it.

---

## Minor Suggestions

### 4. `ValidateAll`: Good design, but consider naming convention

`ValidateAll` is well-designed — generic over T, collects all errors, variadic validators. The only suggestion: the name `Validator[T any]` might conflict with a common interface name in the codebase. Consider `ValidatorFn[T any]` to signal it's a function type, not an interface:

```go
type ValidatorFn[T any] func(T) error
```

This follows Go conventions for function types (`HandlerFunc`, `ReducerFunc`, etc.).

Also minor: when all validators pass, `ValidateAll` returns `nil` (not `[]error{}`). This is correct Go idiom — callers check `if len(errs) > 0`. Just worth documenting explicitly.

### 5. `Transform`/`Map` naming

If the three-type-parameter version is replaced with `Map`, consider also providing `Filter` and `Reduce` for a consistent collection utilities API. These three functions are the natural trio and are already shown as motivation in the lesson. A partial API (only `Map`) can be confusing.

---

## Positive Feedback

- `ValidateAll` is a clean, practical generic design — it solves a real problem (collecting multiple validation errors) with minimal complexity. The variadic `...Validator[T]` makes it easy to compose validators without any framework.
- Using named types (`Validator[T]`) instead of inline function types (`func(T) error`) makes the signatures more readable and shows understanding of type aliases.
- The `Numeric` constraint is correctly defined with `~` — it would work with named types like `type Latency float64`.

---

## Summary

| # | Severity | Function | Issue |
|---|----------|----------|-------|
| 1 | Critical | `LookupOrDefault` | `K any` used as map key — compile error; needs `comparable` |
| 2 | Major | `Transform` | Over-genericized: 3 type params for what should be `Map[T, U]` |
| 3 | Major | `FormatDuration` | Generics used to avoid `time.Duration` — makes the unit implicit and error-prone |
| 4 | Minor | `ValidateAll` | Rename `Validator` to `ValidatorFn` to follow Go function type conventions |
| 5 | Minor | Overall | Consider adding `Filter` and `Reduce` alongside `Map` for a complete API |
