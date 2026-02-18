# Standard Exercise: API Gateway Middleware Stack

## Scenario

Your team is building an API gateway that sits in front of multiple backend services. Every request passes through a stack of middleware before reaching the appropriate backend handler. The middleware stack needs to be composable -- each piece handles exactly one concern, and the operations team can configure which middleware is active per route.

You've been asked to implement the core middleware components and a composition mechanism. The gateway will be used by both internal services and external API consumers, so it must handle authentication, rate limiting, logging, request tracing, panic recovery, and response compression.

## Brief

Implement a set of composable HTTP middleware functions and a `Chain` helper that composes them. Each middleware follows the `func(http.Handler) http.Handler` convention.

## Acceptance Criteria

### Core Infrastructure
- [ ] `Middleware` type alias: `type Middleware func(http.Handler) http.Handler`
- [ ] `Chain(handler http.Handler, middlewares ...Middleware) http.Handler` composes middlewares so the first in the list executes first on the request path
- [ ] Context key types are unexported (no string keys)

### Middleware: Request ID (`WithRequestID`)
- [ ] Reads `X-Request-ID` from incoming request header; if absent, generates a unique ID
- [ ] Sets the request ID on the response header `X-Request-ID`
- [ ] Stores the request ID in the request context (retrievable via `RequestIDFromCtx`)
- [ ] Generated IDs are unique across concurrent requests

### Middleware: Logging (`WithLogging`)
- [ ] Accepts an `io.Writer` for log output (testable -- no global logger)
- [ ] Logs: method, path, status code, duration, request ID
- [ ] Captures status code via a `ResponseWriter` wrapper
- [ ] Logs after the response is written (not before)

### Middleware: Authentication (`WithAuth`)
- [ ] Accepts a `func(string) (string, error)` validator (token in, client ID out, or error)
- [ ] Reads `Authorization: Bearer <token>` header
- [ ] Returns 401 if header is missing, 403 if token is invalid
- [ ] On success, stores client ID in context (retrievable via `ClientIDFromCtx`)
- [ ] Does not read or consume the request body

### Middleware: Rate Limiting (`WithRateLimit`)
- [ ] Accepts `requestsPerWindow int` and `window time.Duration`
- [ ] Identifies clients by context client ID (from auth middleware), falls back to `RemoteAddr`
- [ ] Returns 429 with `Retry-After` header when limit is exceeded
- [ ] Thread-safe for concurrent requests

### Middleware: Panic Recovery (`WithRecovery`)
- [ ] Catches panics from downstream handlers
- [ ] Writes a 500 response with JSON body `{"error": "internal server error"}`
- [ ] Logs the panic value and stack trace to the provided `io.Writer`
- [ ] Does NOT re-panic -- the server stays up

### Middleware: Response Compression (`WithCompression`)
- [ ] Checks `Accept-Encoding` header for `gzip` support
- [ ] If supported, wraps ResponseWriter with gzip.Writer
- [ ] Sets `Content-Encoding: gzip` and removes `Content-Length` (since it changes)
- [ ] Falls through without compression if client doesn't accept gzip
- [ ] Properly closes the gzip writer after the handler completes

## Constraints

- Use only the standard library (`net/http`, `compress/gzip`, `sync`, `context`, etc.)
- No third-party packages
- All middleware must be safe for concurrent use
- The `Chain` helper must apply middleware so that the list order matches request execution order

## Hints

<details>
<summary>Hint 1: Chain helper direction</summary>

If you want `Chain(h, A, B, C)` to execute A first, you need to apply them in reverse:
```go
for i := len(middlewares) - 1; i >= 0; i-- {
    handler = middlewares[i](handler)
}
```
This makes A the outermost wrapper, so it runs first.
</details>

<details>
<summary>Hint 2: Status code capture</summary>

Create a `responseRecorder` struct that embeds `http.ResponseWriter` and overrides `WriteHeader`. Remember that `Write()` implicitly calls `WriteHeader(200)` if it hasn't been called yet.
</details>

<details>
<summary>Hint 3: Context key types</summary>

Use unexported struct types for context keys to prevent collisions:
```go
type contextKey struct{ name string }
var requestIDKey = &contextKey{"request-id"}
```
Using pointer values means each key is unique even if the string matches.
</details>

<details>
<summary>Hint 4: Gzip ResponseWriter</summary>

You need to wrap the ResponseWriter with a struct that writes to both the gzip.Writer and delegates Header/WriteHeader to the original. Don't forget to call `gzipWriter.Close()` when the handler is done -- gzip buffers data and needs a final flush.
</details>

<details>
<summary>Hint 5: Rate limiter concurrency</summary>

Use a `sync.Mutex` to protect the client map. A `sync.Map` is also valid but harder to do windowed cleanup with. For the window, reset counters when `time.Since(lastReset) > window`.
</details>
