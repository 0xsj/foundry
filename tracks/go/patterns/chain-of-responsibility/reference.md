# Go Reference -- Chain of Responsibility Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), standard library documentation,
> and common Go middleware conventions for the `chain-of-responsibility` module.
> Covers: handler interfaces, `http.Handler` chain composition, linked-list vs slice-based
> chains, `context.Context` for request-scoped data, and error propagation patterns.

---

## Handler Interface Patterns

Source: [Effective Go -- Interfaces](https://go.dev/doc/effective_go#interfaces)

### Minimal Handler Interface

The simplest Chain of Responsibility handler in Go uses a single-method interface:

```go
type Handler interface {
    Handle(ctx context.Context, req Request) (Response, error)
}
```

### Linked-List Handler Interface

The classic GoF pattern adds a `SetNext` method to create the chain:

```go
type Handler interface {
    Handle(ctx context.Context, req Request) (Response, error)
    SetNext(handler Handler) Handler
}
```

**Convention:** `SetNext` returns the *argument* (not the receiver) to enable fluent chaining:

```go
a.SetNext(b).SetNext(c)  // returns c, but chain is a -> b -> c
```

### Base Handler Embedding

Go provides chain traversal via struct embedding rather than abstract base classes:

```go
type BaseHandler struct {
    next Handler
}

func (b *BaseHandler) SetNext(h Handler) Handler {
    b.next = h
    return h
}

func (b *BaseHandler) Handle(ctx context.Context, req Request) (Response, error) {
    if b.next != nil {
        return b.next.Handle(ctx, req)
    }
    return Response{}, fmt.Errorf("end of chain: unhandled request")
}
```

Concrete handlers embed `BaseHandler` and call `b.BaseHandler.Handle(ctx, req)` to delegate.

---

## `http.Handler` Chain Composition

Source: [net/http -- Handler](https://pkg.go.dev/net/http#Handler)

### The Canonical Middleware Signature

```go
type Middleware func(http.Handler) http.Handler
```

This signature is used by the standard library, chi, gorilla/mux, alice, and nearly every Go HTTP framework.

### Composition Function

```go
func Chain(final http.Handler, mw ...Middleware) http.Handler {
    for i := len(mw) - 1; i >= 0; i-- {
        final = mw[i](final)
    }
    return final
}
```

**Why reverse order:** The first middleware in the argument list should be the outermost wrapper. Applying in reverse ensures `mw[0]` wraps everything else.

### Standard Library Middleware Pattern

```go
func withMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        // Pre-processing (before inner handler)
        // ...

        // Decision point: call next or short-circuit
        if shouldContinue {
            next.ServeHTTP(w, r)
        } else {
            http.Error(w, "rejected", statusCode)
            return
        }

        // Post-processing (after inner handler)
        // ...
    })
}
```

### `http.HandlerFunc` Adapter

Source: [net/http -- HandlerFunc](https://pkg.go.dev/net/http#HandlerFunc)

```go
type HandlerFunc func(ResponseWriter, *Request)

func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

Converts a function into an `http.Handler`. This adapter is essential for the middleware pattern -- without it, every middleware would need to define a named struct with a `ServeHTTP` method.

---

## Short-Circuiting Patterns

### HTTP Middleware Short-Circuit

```go
func withAuth(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        token := r.Header.Get("Authorization")
        if token == "" {
            w.WriteHeader(http.StatusUnauthorized)
            json.NewEncoder(w).Encode(map[string]string{"error": "missing token"})
            return  // chain stops here
        }
        next.ServeHTTP(w, r)
    })
}
```

**Key rule:** If a middleware writes to `ResponseWriter` and returns without calling `next.ServeHTTP`, the chain stops. No downstream handler runs.

### Slice-Based Short-Circuit

```go
func ExecuteChain(ctx context.Context, req *Request, handlers []HandlerFunc) (*Response, error) {
    for _, h := range handlers {
        resp, err := h(ctx, req)
        if err != nil {
            return nil, err       // error: stop chain
        }
        if resp != nil {
            return resp, nil      // handled: stop chain
        }
        // nil, nil: continue to next handler
    }
    return nil, ErrUnhandled
}
```

| Return value | Meaning | Chain behavior |
|-------------|---------|----------------|
| `(nil, err)` | Handler rejected request | Stop, return error |
| `(resp, nil)` | Handler produced response | Stop, return response |
| `(nil, nil)` | Handler passed | Continue to next |

---

## Linked-List vs Slice-Based Chains

| Feature | Linked List | Slice |
|---------|-------------|-------|
| **Construction** | `a.SetNext(b).SetNext(c)` | `[]Handler{a, b, c}` |
| **Inspection** | Walk the chain: O(n) | Index into slice: O(1) |
| **Insertion** | Between two nodes: O(1) | At index: O(n) amortized |
| **Removal** | Relink neighbors: O(1) | Shift elements: O(n) |
| **Runtime modification** | Easy but error-prone (cycles) | Easy and safe |
| **Go idiomaticity** | Less common | More idiomatic |
| **Serialization** | Difficult | Straightforward |

### Slice-Based: Preferred in Go

Go developers prefer slices over linked lists for nearly everything. The slice-based chain is:

- Easier to debug (print the slice, see all handlers)
- Easier to test (construct a slice, run it)
- Easier to modify at runtime (append, slice operations)
- Cache-friendly (contiguous memory for the slice header, though handlers are pointers)

### When Linked List Is Better

- When you need O(1) insertion/removal in the middle of the chain during processing
- When porting from Java/C# where the linked-list form is standard
- When handlers need to reference their specific successor (not just "the next one")

---

## Context for Request-Scoped Data

Source: [context -- Package context](https://pkg.go.dev/context)

Handlers in a chain often need to pass data downstream (e.g., authenticated user, request ID, trace span). Go's `context.Context` is the standard mechanism:

```go
// Handler adds data to context
func withAuth(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        user := authenticate(r)
        ctx := context.WithValue(r.Context(), userKey, user)
        next.ServeHTTP(w, r.WithContext(ctx))
    })
}

// Downstream handler reads from context
func handleRequest(w http.ResponseWriter, r *http.Request) {
    user := r.Context().Value(userKey).(*User)
    // ...
}
```

### Context Key Best Practices

| Practice | Reason |
|----------|--------|
| Use unexported type for key | Prevents collisions: `type ctxKey struct{}` |
| Provide accessor functions | `func UserFromContext(ctx) *User` hides the key |
| Never store mutable data | Context values should be immutable; use for read-only request-scoped data |
| Never store optional data without checking | Always handle the case where `ctx.Value` returns `nil` |

```go
type ctxKey struct{}

var userKey = ctxKey{}

func WithUser(ctx context.Context, u *User) context.Context {
    return context.WithValue(ctx, userKey, u)
}

func UserFromContext(ctx context.Context) (*User, bool) {
    u, ok := ctx.Value(userKey).(*User)
    return u, ok
}
```

---

## Error Propagation in Chains

### HTTP Middleware Errors

HTTP middleware communicates errors by writing to the `ResponseWriter`:

```go
// Error response helpers
func writeError(w http.ResponseWriter, code int, msg string) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(code)
    json.NewEncoder(w).Encode(map[string]string{"error": msg})
}
```

There is no `error` return value in `http.Handler.ServeHTTP`. Errors are communicated via HTTP status codes and response bodies. This is a design decision in `net/http` -- handlers own their error formatting.

### Non-HTTP Chain Errors

For non-HTTP chains, return errors directly:

```go
type Handler func(ctx context.Context, req *Request) error

// First error stops the chain
func RunChain(ctx context.Context, req *Request, handlers ...Handler) error {
    for _, h := range handlers {
        if err := h(ctx, req); err != nil {
            return fmt.Errorf("handler failed: %w", err)
        }
    }
    return nil
}
```

### Error Wrapping Convention

Wrap errors with handler identity for debugging:

```go
func (h *AuthHandler) Handle(ctx context.Context, req *Request) error {
    if err := h.validate(req); err != nil {
        return fmt.Errorf("auth handler: %w", err)
    }
    return nil
}
```

This produces error chains like:

```
chain execution: auth handler: token expired
```

---

## Common Standard Library Chains

### `io.Reader` / `io.Writer` Wrapping

Source: [io package](https://pkg.go.dev/io)

The `io` package uses chained readers/writers that are structurally similar to Chain of Responsibility:

```go
// Chain: compressed -> encrypted -> buffered -> file
var w io.Writer = file
w = bufio.NewWriter(w)
w = encrypt(w, key)
w = gzip.NewWriter(w)
```

Each writer processes data and passes it to the next. While this is technically more of a Decorator/Pipeline, the composition mechanism is the same.

### `http.ServeMux` as a Router Chain

Source: [net/http -- ServeMux](https://pkg.go.dev/net/http#ServeMux)

`ServeMux` implements `http.Handler` and routes requests to registered patterns. It's a simple Chain of Responsibility: patterns are checked in order (longest match wins), and the first match handles the request.

```go
mux := http.NewServeMux()
mux.HandleFunc("/api/v1/", apiHandler)    // matches /api/v1/*
mux.HandleFunc("/health", healthHandler)   // matches /health exactly
mux.HandleFunc("/", defaultHandler)        // catches everything else
```

---

## Third-Party Chain Libraries

### alice (justinas/alice)

Popular middleware chaining library:

```go
chain := alice.New(withLogging, withAuth, withRateLimit)
handler := chain.Then(finalHandler)
```

### chi (go-chi/chi)

Router with built-in middleware support:

```go
r := chi.NewRouter()
r.Use(middleware.Logger)
r.Use(middleware.Recoverer)
r.Use(middleware.RealIP)
r.Get("/", handler)
```

### negroni (urfave/negroni)

Middleware-focused HTTP library:

```go
n := negroni.New()
n.Use(negroni.NewLogger())
n.Use(negroni.NewRecovery())
n.UseHandler(router)
```

All three follow the `func(http.Handler) http.Handler` convention, differing only in ergonomics.
