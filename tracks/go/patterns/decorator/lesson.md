# Decorator / Middleware Pattern -- Go

## The Problem Decorator Solves

You have working code -- a function, an interface implementation, an HTTP handler -- and you need to add behavior to it. Logging, authentication, timing, retries, rate limiting, caching. The naive approach is to edit the original function, adding `if` checks and cross-cutting logic until the core business logic is buried under layers of infrastructure concerns.

The Decorator pattern wraps existing behavior with additional behavior, without modifying the original. The wrapped thing doesn't know it's been wrapped. The wrapper satisfies the same interface as the thing it wraps. This means decorators compose: you can stack logging on top of auth on top of rate limiting, and each layer is independent, testable, and removable.

If you've used Express.js or Koa in TypeScript, you already know this pattern intimately:

```typescript
// TypeScript/Express: middleware is decoration
app.use(cors());           // decorator 1: add CORS headers
app.use(authenticate());   // decorator 2: verify JWT
app.use(rateLimit());      // decorator 3: enforce rate limits
app.get('/users', handler) // the actual handler being decorated
```

In Go, the decorator pattern appears in three primary forms:

1. **HTTP middleware** -- `func(http.Handler) http.Handler` -- the most common form
2. **Function decorators** -- wrapping `func` types with additional behavior
3. **Interface decorators** -- wrapping an interface implementation with a struct that adds behavior

All three follow the same principle: take something in, return the same type out, add behavior in between. The Go standard library is built on this pattern -- `io.Reader` wrappers, `net/http` middleware, `context` decorators. Once you see it, you'll see it everywhere.

### Your notes
<!-- User adds insights here during learning -->


---

## HTTP Middleware: Go's Most Common Decorator

### The `http.Handler` Interface

Everything in Go's HTTP world revolves around one interface:

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}
```

A middleware is a function that takes a `Handler` and returns a new `Handler`. The returned handler does something before and/or after calling the original:

```go
func withLogging(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        start := time.Now()
        log.Printf("started %s %s", r.Method, r.URL.Path)

        next.ServeHTTP(w, r)  // call the original handler

        log.Printf("completed %s %s in %v", r.Method, r.URL.Path, time.Since(start))
    })
}
```

The signature `func(http.Handler) http.Handler` is the canonical Go middleware signature. Every major Go web framework (chi, echo, gin, gorilla) uses this or a close variant.

### The `http.HandlerFunc` Adapter

Notice the `http.HandlerFunc` cast in the example above. This is a critical piece of Go's decorator machinery:

```go
// In net/http:
type HandlerFunc func(ResponseWriter, *Request)

func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

`HandlerFunc` is a function type that implements the `Handler` interface. This means any function with the right signature can be used as a `Handler` by casting it to `HandlerFunc`. This is the adapter pattern enabling the decorator pattern -- it bridges the gap between functions and interfaces.

This is roughly equivalent to TypeScript's approach where a class can satisfy an interface, but in Go the function itself satisfies the interface through a type that adds the required method:

```typescript
// TypeScript equivalent concept: a function that satisfies an interface
interface Handler {
    serveHTTP(w: ResponseWriter, r: Request): void;
}

// In Go, HandlerFunc does this adaptation automatically
// No need for a class wrapper -- the function IS the handler
```

### Middleware Composition and Ordering

Middleware wraps from outside in, but executes inside out. This is the most common source of confusion:

```go
// Wrapping order: logging wraps auth wraps handler
final := withLogging(withAuth(handler))

// Execution order:
// 1. withLogging: "started GET /users"
// 2. withAuth: check token → valid
// 3. handler: process request, write response
// 4. withAuth: (after handler, if any post-processing)
// 5. withLogging: "completed GET /users in 45ms"
```

The outermost wrapper runs first on the way in and last on the way out. Think of it like layers of an onion -- request goes in through each layer, response comes back out through each layer in reverse order.

A `Chain` helper makes this readable:

```go
func Chain(handler http.Handler, middlewares ...func(http.Handler) http.Handler) http.Handler {
    // Apply in reverse so the first middleware in the list is the outermost
    for i := len(middlewares) - 1; i >= 0; i-- {
        handler = middlewares[i](handler)
    }
    return handler
}

// Usage: reads top-to-bottom in execution order
final := Chain(handler,
    withRequestID,   // 1st: assign request ID
    withLogging,     // 2nd: log request
    withAuth,        // 3rd: check authentication
    withRateLimit,   // 4th: enforce rate limits
)
```

### The ResponseWriter Wrapping Problem

A common need in middleware is capturing the status code or response body. But `http.ResponseWriter` is an interface, and the default implementation doesn't expose what was written. The solution is a decorator on `ResponseWriter` itself:

```go
type statusRecorder struct {
    http.ResponseWriter
    statusCode int
}

func (sr *statusRecorder) WriteHeader(code int) {
    sr.statusCode = code
    sr.ResponseWriter.WriteHeader(code)
}

func withLogging(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        recorder := &statusRecorder{ResponseWriter: w, statusCode: 200}
        next.ServeHTTP(recorder, r)
        log.Printf("%s %s → %d", r.Method, r.URL.Path, recorder.statusCode)
    })
}
```

**Critical gotcha:** When you wrap `ResponseWriter`, you lose access to optional interfaces that the original writer implements -- `http.Flusher`, `http.Hijacker`, `http.Pusher`. If downstream code needs to flush for SSE or hijack for WebSockets, your wrapper breaks it. We'll see this in the code review exercise. The fix is to implement those interfaces on your wrapper too, delegating to the underlying writer.

### Your notes
<!-- User adds insights here during learning -->


---

## Function Decorators: Wrapping Plain Functions

Not everything in Go is an HTTP handler. The decorator pattern works on any function type. Define the function signature as a named type, then write wrappers that take and return that type.

### Retry Decorator

```go
type Operation func(ctx context.Context) error

func WithRetry(op Operation, maxAttempts int, backoff time.Duration) Operation {
    return func(ctx context.Context) error {
        var lastErr error
        for attempt := 1; attempt <= maxAttempts; attempt++ {
            lastErr = op(ctx)
            if lastErr == nil {
                return nil
            }
            if attempt < maxAttempts {
                select {
                case <-time.After(backoff * time.Duration(attempt)):
                case <-ctx.Done():
                    return ctx.Err()
                }
            }
        }
        return fmt.Errorf("after %d attempts: %w", maxAttempts, lastErr)
    }
}
```

### Timing Decorator

```go
func WithTiming(name string, op Operation) Operation {
    return func(ctx context.Context) error {
        start := time.Now()
        err := op(ctx)
        log.Printf("[%s] completed in %v (err: %v)", name, time.Since(start), err)
        return err
    }
}
```

### Composing Function Decorators

```go
// Read bottom-to-top: the operation is timed, then retried, then logged
op := WithLogging("fetch-config",
    WithRetry(
        WithTiming("fetch",
            fetchConfig,
        ),
        3, time.Second,
    ),
)

op(ctx) // execute the decorated operation
```

This is directly analogous to function composition in TypeScript:

```typescript
// TypeScript higher-order functions
const op = withLogging("fetch-config",
    withRetry(
        withTiming("fetch",
            fetchConfig
        ),
        3, 1000
    )
);
```

The Go version is identical in structure. The difference is that Go's named function types make the decorator signatures explicit and type-checked, while TypeScript relies on generic type inference.

### Your notes
<!-- User adds insights here during learning -->


---

## Interface Decorators: Wrapping Implementations

When you wrap a struct that implements an interface, you create a new struct that holds the original as a field, implements the same interface, and adds behavior around each method call. This is the classic object-oriented decorator, and it's common in Go even though Go isn't object-oriented.

### The `io.Reader` / `io.Writer` Chain

The Go standard library's I/O system is the canonical example of interface decorators:

```go
// Start with a base reader
file, _ := os.Open("data.gz")

// Wrap with buffering (reduces syscalls)
buffered := bufio.NewReader(file)

// Wrap with decompression
decompressed, _ := gzip.NewReader(buffered)

// Wrap with a byte limit
limited := io.LimitReader(decompressed, 1024*1024) // max 1MB

// Each wrapper implements io.Reader
// Each adds behavior without modifying the others
// You can compose them in any order (though some orders make more sense)
```

Every reader in this chain implements `io.Reader`. Each one wraps another `io.Reader` and adds a specific capability. This is pure decoration -- no inheritance, no abstract classes, just interface satisfaction and delegation.

### Building a Custom Decorator

Here's a counting reader that tracks bytes read -- a useful production tool for monitoring data transfer:

```go
type CountingReader struct {
    reader    io.Reader
    bytesRead int64
}

func NewCountingReader(r io.Reader) *CountingReader {
    return &CountingReader{reader: r}
}

func (cr *CountingReader) Read(p []byte) (int, error) {
    n, err := cr.reader.Read(p)
    cr.bytesRead += int64(n)
    return n, err
}

func (cr *CountingReader) BytesRead() int64 {
    return cr.bytesRead
}
```

`CountingReader` implements `io.Reader`, so it can be used anywhere a reader is expected. It can wrap any reader. It composes with all other reader decorators. This is the power of the pattern.

### When Interface Decorators Shine

Interface decorators are the right choice when:

- The interface has **multiple methods** that all need decoration (logging every method of a Repository interface)
- You need to **maintain state** across calls (counting bytes, tracking latency percentiles)
- The decorator needs to be **configurable** (struct fields for thresholds, feature flags)
- You want **compile-time guarantees** that the decorator satisfies the interface

For single-method interfaces (like `http.Handler`), function decorators are simpler. For multi-method interfaces (like a `UserRepository` with `Create`, `Get`, `Update`, `Delete`), struct-based decorators are more maintainable.

### Your notes
<!-- User adds insights here during learning -->


---

## Cross-Language Comparison

The decorator pattern exists in every language, but the mechanics differ significantly.

| Aspect | Go | TypeScript | Rust |
|--------|-----|-----------|------|
| **Primary mechanism** | Function wrapping + interfaces | Higher-order functions, class decorators (TC39 Stage 3) | Newtype pattern, tower middleware, trait impl wrappers |
| **HTTP middleware** | `func(http.Handler) http.Handler` | Express `(req, res, next) => {}` | tower `Layer` + `Service` traits |
| **Function decoration** | Named function types | Generic higher-order functions | `Fn` trait wrappers (less common) |
| **Interface decoration** | Struct wrapping + implicit satisfaction | Class wrapping + explicit implements | Struct wrapping + explicit `impl Trait` |
| **Composition** | Manual chain or helper function | `compose()` utility, pipe operator proposal | `ServiceBuilder` in tower, combinator chains |
| **Type safety** | Interface satisfaction checked at compile time | Type inference, but decorators can break types | Full type safety via trait bounds |
| **Runtime cost** | Interface dispatch (indirect call) | Closure allocation + prototype chain | Zero-cost with monomorphization (tower) |

### TypeScript Decorators vs Go Middleware

TypeScript's TC39 decorators (`@decorator` syntax) operate at the class/method level and are primarily metaprogramming -- they modify or replace class elements at definition time. Go's middleware pattern operates at runtime -- each call goes through the wrapper chain. They solve similar problems (adding cross-cutting concerns) but at different levels:

```typescript
// TypeScript: decoration at definition time
class UserService {
    @log          // adds logging to this method
    @cache(300)   // adds caching with 300s TTL
    async getUser(id: string): Promise<User> { ... }
}

// Go: decoration at wiring time
handler := withLogging(withCache(300, getUserHandler))
```

### Rust's Tower Middleware

Rust's `tower` crate uses traits (`Service` and `Layer`) to build middleware stacks. It's more complex than Go's approach but provides zero-cost abstractions through monomorphization:

```rust
// Rust/tower: middleware via Service trait
let svc = ServiceBuilder::new()
    .layer(TimeoutLayer::new(Duration::from_secs(10)))
    .layer(RateLimitLayer::new(100, Duration::from_secs(1)))
    .service(MyHandler);
```

The core idea is identical to Go -- wrap a service with layers of behavior. The difference is that Rust's type system can eliminate the runtime dispatch cost that Go pays for interface calls.

### Your notes
<!-- User adds insights here during learning -->


---

## Standard Library Examples

Go's standard library uses the decorator pattern extensively. Recognizing these patterns helps you design your own decorators.

### `net/http` Middleware

- `http.StripPrefix` -- removes a path prefix before passing to the next handler
- `http.TimeoutHandler` -- wraps a handler with a timeout
- `http.MaxBytesHandler` -- limits request body size
- `http.AllowQuerySemicolons` -- normalizes query parameters

Each of these takes an `http.Handler` and returns an `http.Handler`. They compose freely.

### `io` Wrappers

- `io.LimitReader` -- caps bytes read
- `io.TeeReader` -- splits reads to a writer (like Unix `tee`)
- `io.NopCloser` -- adds a no-op `Close` method to a Reader
- `bufio.NewReader` / `bufio.NewWriter` -- adds buffering
- `gzip.NewReader` / `gzip.NewWriter` -- adds compression
- `cipher.StreamReader` / `cipher.StreamWriter` -- adds encryption

### `context` Decorators

Context itself is a decorator chain:

```go
ctx := context.Background()                           // base
ctx = context.WithValue(ctx, requestIDKey, "abc-123") // decorator: add value
ctx, cancel := context.WithTimeout(ctx, 5*time.Second) // decorator: add deadline
```

Each `context.With*` function wraps the parent context with additional behavior. The child context delegates to the parent for values it doesn't have, deadlines it doesn't override, etc.

### `net` Connection Decorators

- `tls.Client` / `tls.Server` -- wrap `net.Conn` with TLS
- `net.Pipe` -- creates a synchronous, in-memory connection pair (useful for testing decorators)

### Your notes
<!-- User adds insights here during learning -->


---

## Anti-Patterns and Pitfalls

### Too Many Layers (Onion Debugging)

When a request goes through 8 middleware layers and something goes wrong, stack traces become a nightmare. Each layer adds a frame, and the actual error is buried deep.

**Mitigation:**
- Keep middleware focused -- one concern per middleware
- Use structured logging with request IDs so you can trace through layers
- Consider collapsing closely-related middleware (auth + authorization can be one layer)
- Add middleware names to error wrapping: `fmt.Errorf("auth middleware: %w", err)`

### Decorators That Break the Contract

A decorator must honor the behavioral contract of the interface it wraps, not just the type signature. A caching decorator for a `Repository.Get` method that returns stale data after a `Delete` call technically satisfies the interface but breaks the semantic contract.

**Rule of thumb:** A consumer should not be able to tell whether it's talking to the real implementation or a decorated one, except for the specific behavior the decorator adds.

### Modifying Shared State

Middleware that modifies the `*http.Request` directly can cause race conditions if the request is shared (e.g., in fan-out patterns). Always clone the request before modifying it:

```go
// Wrong: modifies the original request
func badMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        r.Header.Set("X-Request-ID", uuid.New().String()) // mutates shared state!
        next.ServeHTTP(w, r)
    })
}

// Correct: clone before modifying
func goodMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        r2 := r.Clone(r.Context())
        r2.Header.Set("X-Request-ID", uuid.New().String())
        next.ServeHTTP(w, r2)
    })
}
```

### ResponseWriter Interface Loss

As mentioned earlier, wrapping `ResponseWriter` can break `http.Flusher`, `http.Hijacker`, and other optional interfaces. This is one of Go's most common middleware bugs.

### Order-Dependent Bugs

If your auth middleware reads the request body to verify a signature, and your logging middleware also reads the body, whoever goes first consumes it. The second reader gets an empty body. You need to buffer and restore the body:

```go
body, _ := io.ReadAll(r.Body)
r.Body = io.NopCloser(bytes.NewReader(body)) // restore for next handler
```

### Your notes
<!-- User adds insights here during learning -->


---

## When to Use Each Form

| Situation | Form | Why |
|-----------|------|-----|
| HTTP request/response processing | `func(http.Handler) http.Handler` | Standard convention, composes with all Go web frameworks |
| Adding behavior to a single function | Named function type decorator | Simpler than an interface when there's only one method |
| Adding behavior to a multi-method interface | Struct wrapper implementing the interface | Type safety across all methods, can maintain state |
| I/O processing pipeline | `io.Reader`/`io.Writer` wrappers | Composes with entire standard library I/O ecosystem |
| Cross-cutting concerns (logging, metrics) | Any form, depending on what you're wrapping | Match the form to what you're decorating |

The decorator pattern is one of the most practical patterns in Go. Unlike some GoF patterns that feel forced in Go, decoration is natural -- it falls directly out of Go's interface system and first-class functions. The standard library is full of it because it works.

### Your notes
<!-- User adds insights here during learning -->
