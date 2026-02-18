# Standard Exercise: Middleware Pipeline

## Scenario

You are building the middleware layer for an internal API gateway. The gateway sits
between external clients and a fleet of backend microservices. Every request passes
through a configurable stack of middleware before reaching the backend handler. Each
middleware handles one cross-cutting concern: logging, authentication, rate limiting,
timeout enforcement, and metrics collection.

The middleware must be composable -- any subset can be enabled, and order matters
(auth should run before rate limiting, timing should wrap everything).

## Brief

Implement a `Handler` trait and five middleware types that each wrap an inner `Handler`,
adding one concern. Build a working pipeline that processes requests through the
full stack using generic (static dispatch) decoration.

## Acceptance Criteria

1. **`Request` struct** with fields: `method: String`, `path: String`,
   `headers: HashMap<String, String>`, `body: String`
   - Constructor: `Request::new(method: &str, path: &str) -> Request`
   - Builder: `Request::with_header(self, key: &str, value: &str) -> Request`

2. **`Response` struct** with fields: `status: u16`, `body: String`,
   `headers: HashMap<String, String>`
   - Constructors: `Response::ok(body: &str)`, `Response::error(status: u16, msg: &str)`

3. **`Handler` trait** with method:
   `fn handle(&self, req: &Request) -> Response`

4. **`EchoHandler`** -- core handler that returns the request path and method in the body.

5. **`LoggingMiddleware<H: Handler>`** -- prints `"[log] METHOD /path"` before calling inner,
   prints `"[log] STATUS"` after.

6. **`AuthMiddleware<H: Handler>`** -- checks for `"Authorization"` header. If missing,
   returns 401. If value is not in the allowed tokens list, returns 403. Skips auth
   for `"/health"` path.

7. **`RateLimitMiddleware<H: Handler>`** -- tracks request count per path using a
   `HashMap<String, u32>` inside a `RefCell`. If count exceeds `max_requests`, returns 429.
   Has a `fn reset(&self)` method to clear counts.

8. **`TimeoutMiddleware<H: Handler>`** -- records the start time, calls inner handler,
   checks if elapsed time exceeds the configured duration. If so, returns 504 instead
   of the inner handler's response. (Cooperative timeout -- checks after completion.)

9. **`MetricsMiddleware<H: Handler>`** -- tracks total request count and error count
   (status >= 400) using `Cell<u64>`. Has `fn request_count(&self)` and
   `fn error_count(&self)` accessors.

10. **Pipeline composition** -- build and use a stack:
    `Metrics<Logging<Timeout<Auth<RateLimit<EchoHandler>>>>>`

## Constraints

- No external crates -- stdlib only
- Use generic decoration (static dispatch), not `Box<dyn Handler>`
- Interior mutability via `Cell`/`RefCell` for middleware that tracks state
- All provided tests must pass

## Hints

<details>
<summary>Hint 1: Interior mutability for rate limiter</summary>

Since `handle` takes `&self`, you cannot mutate a `HashMap` directly. Wrap it in
`RefCell<HashMap<String, u32>>`:

```rust
use std::cell::RefCell;

struct RateLimitMiddleware<H: Handler> {
    inner: H,
    max_requests: u32,
    counts: RefCell<HashMap<String, u32>>,
}
```

Access with `self.counts.borrow()` and `self.counts.borrow_mut()`.

</details>

<details>
<summary>Hint 2: Cooperative timeout</summary>

You cannot interrupt a synchronous function from another thread without `unsafe`.
Instead, measure time before and after the inner handler call:

```rust
let start = std::time::Instant::now();
let response = self.inner.handle(req);
if start.elapsed() > self.timeout {
    return Response::error(504, "gateway timeout");
}
response
```

</details>

<details>
<summary>Hint 3: Composing the stack</summary>

Build inside-out. The innermost handler is created first, then wrapped layer by layer:

```rust
let handler = MetricsMiddleware::new(
    LoggingMiddleware::new(
        TimeoutMiddleware::new(
            AuthMiddleware::new(
                RateLimitMiddleware::new(EchoHandler, 5),
                vec!["Bearer tok123"],
            ),
            Duration::from_millis(100),
        ),
    ),
);
```

The type is resolved at compile time -- zero dynamic dispatch overhead.

</details>

<details>
<summary>Hint 4: Skipping auth for certain paths</summary>

In `AuthMiddleware::handle`, check the path before doing auth:

```rust
if req.path == "/health" {
    return self.inner.handle(req);
}
```

</details>
