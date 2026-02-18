# Go Reference -- Decorator / Middleware Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), and standard library documentation
> for the `decorator` module. Covers: `http.Handler`, `http.HandlerFunc`, `io.Reader`/`io.Writer`
> wrappers, middleware conventions, and interface embedding for delegation.

---

## `http.Handler` Interface

Source: [net/http -- Handler](https://pkg.go.dev/net/http#Handler)

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}
```

A `Handler` responds to an HTTP request. `ServeHTTP` should write reply headers and data to the `ResponseWriter` and then return. Returning signals that the request is finished; it is not valid to use the `ResponseWriter` or read from the `Request.Body` after or concurrently with the completion of the `ServeHTTP` call.

### Key Behavioral Rules

| Rule | Description |
|------|-------------|
| No concurrent use after return | `ResponseWriter` and `Request.Body` must not be used after `ServeHTTP` returns |
| Panic recovery | The HTTP server recovers panics from handler goroutines and logs them; the connection is closed |
| Header mutation timing | `ResponseWriter.Header()` must be set before `WriteHeader()` or `Write()` is called |
| Single `WriteHeader` | Calling `WriteHeader` more than once is not an error but only the first call takes effect; `Write` implicitly calls `WriteHeader(200)` |

---

## `http.HandlerFunc` Type

Source: [net/http -- HandlerFunc](https://pkg.go.dev/net/http#HandlerFunc)

```go
type HandlerFunc func(ResponseWriter, *Request)

func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

The `HandlerFunc` type is an adapter to allow the use of ordinary functions as HTTP handlers. If `f` is a function with the appropriate signature, `HandlerFunc(f)` is a `Handler` that calls `f`.

### Usage Pattern

```go
// Convert a function to a Handler
mux.Handle("/path", http.HandlerFunc(myFunc))

// Shorthand (ServeMux provides HandleFunc)
mux.HandleFunc("/path", myFunc)
```

---

## `http.ResponseWriter` Interface

Source: [net/http -- ResponseWriter](https://pkg.go.dev/net/http#ResponseWriter)

```go
type ResponseWriter interface {
    Header() http.Header
    Write([]byte) (int, error)
    WriteHeader(statusCode int)
}
```

### Optional Interfaces

The underlying `ResponseWriter` may additionally implement these interfaces. Middleware wrappers must be careful to preserve access to these:

| Interface | Method | Purpose |
|-----------|--------|---------|
| `http.Flusher` | `Flush()` | Sends buffered data to client (required for SSE, streaming) |
| `http.Hijacker` | `Hijack() (net.Conn, *bufio.ReadWriter, error)` | Takes over the TCP connection (required for WebSockets) |
| `http.Pusher` | `Push(target string, opts *http.PushOptions) error` | HTTP/2 server push |
| `io.ReaderFrom` | `ReadFrom(r io.Reader) (n int64, err error)` | Efficient file serving via `sendfile` syscall |

---

## Standard Library Middleware Functions

Source: [net/http package](https://pkg.go.dev/net/http)

| Function | Signature | Behavior |
|----------|-----------|----------|
| `http.StripPrefix` | `func StripPrefix(prefix string, h Handler) Handler` | Removes prefix from URL path before forwarding |
| `http.TimeoutHandler` | `func TimeoutHandler(h Handler, dt time.Duration, msg string) Handler` | Wraps handler with a timeout; returns 503 on timeout |
| `http.MaxBytesHandler` | `func MaxBytesHandler(h Handler, n int64) Handler` | Limits request body to `n` bytes |
| `http.AllowQuerySemicolons` | `func AllowQuerySemicolons(h Handler) Handler` | Converts semicolons to ampersands in query strings |

### `http.StripPrefix` Implementation Pattern

```go
func StripPrefix(prefix string, h Handler) Handler {
    if prefix == "" {
        return h
    }
    return HandlerFunc(func(w ResponseWriter, r *Request) {
        p := strings.TrimPrefix(r.URL.Path, prefix)
        rp := strings.TrimPrefix(r.URL.RawPath, prefix)
        if len(p) < len(r.URL.Path) && (r.URL.RawPath == "" || len(rp) < len(r.URL.RawPath)) {
            r2 := new(Request)
            *r2 = *r
            r2.URL = new(url.URL)
            *r2.URL = *r.URL
            r2.URL.Path = p
            r2.URL.RawPath = rp
            h.ServeHTTP(w, r2)
        } else {
            NotFound(w, r)
        }
    })
}
```

Note: `StripPrefix` creates a shallow copy of the request (`*r2 = *r`) and a shallow copy of the URL before modifying. This avoids mutating the original request.

---

## Middleware Signature Convention

The canonical Go middleware type:

```go
type Middleware func(http.Handler) http.Handler
```

Composition reads inside-out but is typically written with a helper:

```go
// Manual composition (reads inside-out)
handler = middleware3(middleware2(middleware1(handler)))

// Chain helper (reads top-to-bottom)
func Chain(h http.Handler, mw ...Middleware) http.Handler {
    for i := len(mw) - 1; i >= 0; i-- {
        h = mw[i](h)
    }
    return h
}
```

### Execution Order

For a chain `[A, B, C]` wrapping handler `H`:

```
Request  → A.before → B.before → C.before → H → C.after → B.after → A.after → Response
```

The first middleware in the list is the outermost wrapper. It runs first on the way in and last on the way out.

---

## `io.Reader` Interface

Source: [io -- Reader](https://pkg.go.dev/io#Reader)

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

### Standard Library Reader Decorators

| Decorator | Constructor | Behavior |
|-----------|-------------|----------|
| `io.LimitedReader` | `io.LimitReader(r, n)` | Reads at most `n` bytes from `r` |
| `io.TeeReader` | `io.TeeReader(r, w)` | Writes all data read from `r` to `w` |
| `io.NopCloser` | `io.NopCloser(r)` | Wraps `Reader` with a no-op `Close` method, returning `ReadCloser` |
| `bufio.Reader` | `bufio.NewReader(r)` | Adds buffering (default 4096 bytes) |
| `gzip.Reader` | `gzip.NewReader(r)` | Adds gzip decompression |
| `cipher.StreamReader` | `cipher.StreamReader{S, R}` | Decrypts data read from `R` using stream cipher `S` |
| `io.SectionReader` | `io.NewSectionReader(ra, off, n)` | Reads a section of a `ReaderAt` |

### `io.LimitReader` Implementation

```go
func LimitReader(r Reader, n int64) Reader {
    return &LimitedReader{r, n}
}

type LimitedReader struct {
    R Reader // underlying reader
    N int64  // max bytes remaining
}

func (l *LimitedReader) Read(p []byte) (n int, err error) {
    if l.N <= 0 {
        return 0, EOF
    }
    if int64(len(p)) > l.N {
        p = p[0:l.N]
    }
    n, err = l.R.Read(p)
    l.N -= int64(n)
    return
}
```

---

## `io.Writer` Interface

Source: [io -- Writer](https://pkg.go.dev/io#Writer)

```go
type Writer interface {
    Write(p []byte) (n int, err error)
}
```

### Standard Library Writer Decorators

| Decorator | Constructor | Behavior |
|-----------|-------------|----------|
| `bufio.Writer` | `bufio.NewWriter(w)` | Adds buffering (default 4096 bytes); requires `Flush()` |
| `gzip.Writer` | `gzip.NewWriter(w)` | Adds gzip compression; requires `Close()` to flush |
| `io.MultiWriter` | `io.MultiWriter(w1, w2, ...)` | Writes to all writers simultaneously |
| `cipher.StreamWriter` | `cipher.StreamWriter{S, W}` | Encrypts data written to `W` using stream cipher `S` |

---

## Interface Embedding for Delegation

Source: [Go Specification -- Struct types](https://go.dev/ref/spec#Struct_types)

When building interface decorators, embedding the wrapped interface as an anonymous field provides automatic delegation for methods you don't need to override:

```go
type statusRecorder struct {
    http.ResponseWriter  // embedded: delegates Header(), Write() automatically
    statusCode int
}

// Only override the method you need to intercept
func (sr *statusRecorder) WriteHeader(code int) {
    sr.statusCode = code
    sr.ResponseWriter.WriteHeader(code)
}
```

### Embedding Rules for Decorators

| Rule | Effect |
|------|--------|
| Embedded interface promotes all methods | All methods of `http.ResponseWriter` are available on `statusRecorder` |
| Overriding promoted method | Defining `WriteHeader` on `statusRecorder` shadows the embedded version |
| Accessing embedded method explicitly | `sr.ResponseWriter.WriteHeader(code)` calls the original |
| Interface satisfaction | `statusRecorder` satisfies `http.ResponseWriter` because all methods exist (promoted + overridden) |

---

## `context` Decorator Pattern

Source: [context package](https://pkg.go.dev/context)

The `context` package implements decorators through constructor functions:

```go
func WithCancel(parent Context) (ctx Context, cancel CancelFunc)
func WithDeadline(parent Context, d time.Time) (Context, CancelFunc)
func WithTimeout(parent Context, timeout time.Duration) (Context, CancelFunc)
func WithValue(parent Context, key, val any) Context
```

Each function wraps the parent `Context` with additional behavior. The resulting context delegates `Value()` calls to the parent if the key doesn't match, and delegates `Done()` / `Deadline()` to whichever ancestor has the nearest deadline.

### Context Value Key Convention

```go
// Unexported type prevents collisions
type contextKey struct{}

var requestIDKey = contextKey{}

// Set
ctx = context.WithValue(ctx, requestIDKey, "abc-123")

// Get
id, ok := ctx.Value(requestIDKey).(string)
```

Using unexported types as keys prevents key collisions between packages. Using string keys is an anti-pattern because any package can accidentally use the same string.

---

## `http.Request.Clone`

Source: [net/http -- Request.Clone](https://pkg.go.dev/net/http#Request.Clone)

```go
func (r *Request) Clone(ctx context.Context) *Request
```

`Clone` returns a deep copy of `r` with its context changed to `ctx`. The provided `ctx` must be non-nil.

`Clone` only makes a shallow copy of the `Body` field. This is acceptable because the `Body` is an `io.ReadCloser`, and the caller is expected to not read from the original after cloning.

Middleware should use `Clone` (or manual shallow copy) before mutating request headers or URL:

```go
r2 := r.Clone(r.Context())
r2.Header.Set("X-Forwarded-For", clientIP)
next.ServeHTTP(w, r2)
```

---

## Common Middleware Patterns Reference

### Request ID Injection

```go
func WithRequestID(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        id := r.Header.Get("X-Request-ID")
        if id == "" {
            id = uuid.New().String()
        }
        ctx := context.WithValue(r.Context(), requestIDKey, id)
        w.Header().Set("X-Request-ID", id)
        next.ServeHTTP(w, r.WithContext(ctx))
    })
}
```

### Panic Recovery

```go
func WithRecovery(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        defer func() {
            if rec := recover(); rec != nil {
                log.Printf("panic recovered: %v\n%s", rec, debug.Stack())
                http.Error(w, "Internal Server Error", http.StatusInternalServerError)
            }
        }()
        next.ServeHTTP(w, r)
    })
}
```

### CORS Headers

```go
func WithCORS(origin string) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            w.Header().Set("Access-Control-Allow-Origin", origin)
            w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
            w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
            if r.Method == http.MethodOptions {
                w.WriteHeader(http.StatusNoContent)
                return
            }
            next.ServeHTTP(w, r)
        })
    }
}
```
