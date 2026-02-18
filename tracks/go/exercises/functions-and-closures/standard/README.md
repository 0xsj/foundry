# Exercise: Middleware Chain Builder

## Scenario

You're building a lightweight HTTP-like request pipeline for an internal API gateway. The gateway routes requests through a series of middleware layers before reaching the actual handler: authentication checks, request logging, rate limiting, and payload validation all need to run in sequence. Each middleware can either pass the request to the next layer or short-circuit with an error response — exactly like `net/http` middleware, but simplified so you can build it from scratch.

## Brief

Implement a composable middleware system where:

1. A `Handler` is a function that processes a `Request` and returns a `Response`
2. A `Middleware` is a function that wraps a `Handler`, adding behavior before and/or after the inner handler
3. A `Chain` composes multiple middlewares together in order (first registered runs outermost)
4. Concrete middlewares: `Logger`, `Auth`, `RateLimit`, and `Recover`

## Acceptance Criteria

- [ ] `Handler` type defined as `func(Request) Response`
- [ ] `Middleware` type defined as `func(Handler) Handler`
- [ ] `Chain(middlewares ...Middleware) Middleware` — composes middlewares so the first argument wraps outermost (runs first on the way in)
- [ ] `Logger(next Handler) Handler` — logs method, path, status, and duration for every request
- [ ] `Auth(token string) Middleware` — returns 401 if the `Authorization` header doesn't match `token`; uses a closure to capture `token`
- [ ] `RateLimit(maxPerMin int) Middleware` — tracks call counts per client IP; returns 429 after `maxPerMin` calls within the same minute window; uses a closure to capture state
- [ ] `Recover(next Handler) Handler` — catches any panic inside `next`, returns a 500 response instead of crashing
- [ ] Middlewares execute in registration order (first registered = outermost = runs first)
- [ ] Short-circuiting works: if `Auth` returns 401, `RateLimit` and the actual handler do not execute

## Types Provided (do not change)

```go
type Request struct {
    Method  string
    Path    string
    Headers map[string]string
    Body    string
    // ClientIP is used by RateLimit to track per-client call counts
    ClientIP string
}

type Response struct {
    Status  int
    Body    string
    Headers map[string]string
}
```

## Constraints

- No external packages — standard library only
- `RateLimit` state must live in a closure, not a global variable
- `Auth` must capture its token via closure, not a package-level variable
- The chain must work with any number of middlewares (including zero)
- All middleware functions must follow the `func(Handler) Handler` signature

## Concepts Exercised

- Function types (`Handler`, `Middleware` as named types)
- Higher-order functions (functions that accept and return functions)
- Closures (capturing state in `Auth`, `RateLimit`)
- Function composition (`Chain` building a nested call structure)
- `defer` and `recover` (`Recover` middleware)
- Variadic functions (`Chain` takes `...Middleware`)

## Hints

<details>
<summary>Hint 1: How Chain composes middlewares</summary>

Think of middleware like nested function calls. If you have `m1`, `m2`, `m3`, the chain should produce:

```
m1(m2(m3(handler)))
```

When a request comes in, `m1` runs first, calls `m2`, which calls `m3`, which calls `handler`.

To build this, iterate the middlewares in reverse order:

```go
func Chain(middlewares ...Middleware) Middleware {
    return func(final Handler) Handler {
        // wrap from the inside out
        for i := len(middlewares) - 1; i >= 0; i-- {
            final = middlewares[i](final)
        }
        return final
    }
}
```
</details>

<details>
<summary>Hint 2: Auth middleware structure</summary>

`Auth` takes a token and returns a `Middleware`. The token is captured by the closure:

```go
func Auth(token string) Middleware {
    return func(next Handler) Handler {
        return func(r Request) Response {
            // check r.Headers["Authorization"] against token
            // if mismatch, return Response{Status: 401}
            // otherwise, return next(r)
        }
    }
}
```
</details>

<details>
<summary>Hint 3: RateLimit state</summary>

The rate limiter needs to count calls per IP per minute window. A `map[string]int` captured in the closure is the right shape. To track "this minute", you can use `time.Now().Truncate(time.Minute)` as a key — or track a window start time and reset when a new minute begins.

```go
func RateLimit(maxPerMin int) Middleware {
    counts := make(map[string]int)  // captured by closure
    // ...
}
```
</details>

<details>
<summary>Hint 4: Recover middleware</summary>

`recover()` only works when called directly from a deferred function. The pattern:

```go
func Recover(next Handler) Handler {
    return func(r Request) Response {
        var resp Response
        func() {
            defer func() {
                if rec := recover(); rec != nil {
                    resp = Response{Status: 500, Body: "internal error"}
                }
            }()
            resp = next(r)
        }()
        return resp
    }
}
```
</details>
