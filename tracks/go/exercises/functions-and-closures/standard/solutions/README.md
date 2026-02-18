# Solution: Middleware Chain Builder

## Approach

The solution uses four key functional patterns working together:

1. **Named function types** (`Handler`, `Middleware`) make the code self-documenting — the types tell you what a function does before you read its body
2. **Function factories** (`Auth`, `RateLimit`) return closures that capture configuration and state
3. **Higher-order composition** (`Chain`) builds a nested call structure from a flat list
4. **Closure-as-state** (`RateLimit`) replaces what would be a struct with fields in OOP languages

## Key Decisions

### Chain uses reverse iteration

```go
for i := len(middlewares) - 1; i >= 0; i-- {
    final = middlewares[i](final)
}
```

Starting from the innermost and wrapping outward ensures `m1(m2(m3(handler)))`. If you iterated forward, you'd get `m3(m2(m1(handler)))` — the wrong order.

### Auth vs Logger signature

`Logger` has type `Middleware` directly — it doesn't need configuration, so it's already a `func(Handler) Handler`. `Auth` has type `func(string) Middleware` — one extra layer because it needs to capture a token. These two levels are both common in the wild; knowing when you need the extra wrapper is the skill.

### RateLimit uses a struct in the closure

Rather than a bare `map` and `time.Time`, we use a small `rateLimitState` struct. This keeps the closure tidy and makes the `sync.Mutex` obvious. The struct lives on the heap (captured by pointer via the closure).

### Recover uses an inner immediately-invoked function

```go
func() {
    defer func() {
        if rec := recover(); rec != nil { ... }
    }()
    resp = next(r)
}()
```

`recover()` only works when called directly from a deferred function. If you call it from a deferred function that calls another function, it returns `nil`. The inner IIFE creates the right scope for the `defer`+`recover` pair.

## Comparison of Approaches

| Aspect | This solution | Alternative: Struct-based |
|--------|---------------|--------------------------|
| State encapsulation | Closure captures variables | Struct fields |
| Thread safety | sync.Mutex in closure | sync.Mutex in struct |
| Testability | Each call = fresh state | Must reset struct fields |
| Discoverability | State is invisible | Fields are visible/inspectable |
| Composability | Trivially composable | Requires interface |

The closure approach wins for composability — any `func(Handler) Handler` is a Middleware. The struct approach wins if you need to inspect or serialize the state (e.g., expose current rate limit counts via a metrics endpoint).

## Performance Notes

- `Chain` itself allocates one closure per middleware per chain construction — but chain construction typically happens once at startup
- `RateLimit` allocates a new map on window reset — once per minute, not per request
- `Recover` uses an IIFE which has minimal overhead (no goroutine, no channel)
- The `sync.Mutex` in `RateLimit` is a potential bottleneck under very high concurrency — a sharded map or `sync.Map` would scale better

## Related Concepts

- [[patterns/strategy]] — `Handler` and `Middleware` are the strategy pattern applied to function types
- [[patterns/decorator]] — Middleware is textbook decorator: adds behavior without modifying the wrapped function
- [[fundamentals/closures]] — `Auth` and `RateLimit` demonstrate closure-as-state
- [[fundamentals/defer]] — `Recover` demonstrates the only correct way to use `recover()`
