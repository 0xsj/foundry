# Go Reference -- Strategy Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), and standard library documentation
> for the `strategy` module. Covers: interface definitions, interface satisfaction,
> function types, method sets, and standard library strategy examples.

---

## Interface Types

Source: [Go Specification -- Interface types](https://go.dev/ref/spec#Interface_types)

An interface type defines a **type set**. A variable of interface type can store a value of any type that is in the type set of the interface. Such a type is said to *implement* the interface.

An interface type is specified by a list of interface elements. An interface element is either a method or a type element.

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

A type `T` implements an interface `I` if `T` is an element of the type set of `I`. As a shorthand, we say that `T` implements `I`.

### Basic Interfaces

An interface whose type set can be defined entirely by a list of methods is called a **basic interface**.

```go
type ReadWriter interface {
    Read(p []byte) (n int, err error)
    Write(p []byte) (n int, err error)
}
```

Every type that has the methods `Read` and `Write` (with the exact signatures) implements `ReadWriter`. This includes `*os.File`, `*bytes.Buffer`, `*net.TCPConn`, etc.

### Interface Embedding

An interface can embed other interface types:

```go
type ReadWriteCloser interface {
    ReadWriter
    Close() error
}
```

The resulting interface has the union of all embedded methods. Duplicate methods (from multiple embedded interfaces) must have identical signatures.

---

## Interface Satisfaction Rules

Source: [Go Specification -- Implementing an interface](https://go.dev/ref/spec#Implementing_an_interface)

A type `T` implements an interface `I` if:

1. `T` is not an interface and is an element of the type set of `I`
2. `T` is an interface and the type set of `T` is a subset of the type set of `I`

### Method Sets

Source: [Go Specification -- Method sets](https://go.dev/ref/spec#Method_sets)

The **method set** of a type determines the interfaces that the type implements and the methods that can be called using a receiver of that type.

| Type | Method Set |
|------|------------|
| Defined type `T` | All methods declared with receiver type `T` |
| Pointer to defined type `*T` | All methods declared with receiver `T` or `*T` |
| Interface type | The intersection of method sets of the types in its type set |

This means:

- `*T` always has a superset of `T`'s methods
- If an interface requires a pointer-receiver method, only `*T` satisfies it
- Value types cannot satisfy interfaces that require pointer-receiver methods

```go
type Flusher interface {
    Flush() error
}

type Buffer struct{ data []byte }

func (b *Buffer) Flush() error { b.data = nil; return nil }

var _ Flusher = &Buffer{}    // OK: *Buffer has Flush
// var _ Flusher = Buffer{}  // COMPILE ERROR: Buffer does not have Flush
```

### Compile-Time Interface Guard

Idiomatic Go uses a blank variable assignment to verify interface satisfaction at compile time:

```go
var _ io.ReadWriter = (*MyType)(nil)
```

This has zero runtime cost. The compiler verifies that `*MyType` implements `io.ReadWriter` and discards the variable.

---

## Function Types

Source: [Go Specification -- Function types](https://go.dev/ref/spec#Function_types)

A function type denotes the set of all functions with the same parameter and result types:

```go
type HandlerFunc func(ResponseWriter, *Request)
type BackoffFunc func(attempt int) time.Duration
type CompareFunc func(a, b string) int
```

Function types are first-class values in Go. They can be:
- Assigned to variables
- Passed as arguments
- Returned from functions
- Stored in struct fields
- Used in maps and slices

### Function Types as Interface Implementors

A function type can have methods, allowing it to satisfy interfaces:

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}

type HandlerFunc func(ResponseWriter, *Request)

// HandlerFunc satisfies Handler
func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

This adapter pattern allows callers to provide either a struct with state or a simple function. It appears throughout the standard library.

---

## Interface-Based vs Function-Based Strategy

| Criterion | Interface | Function Type |
|-----------|-----------|---------------|
| **Number of methods** | Multiple methods | Single method |
| **State management** | Struct fields (explicit) | Closure capture (implicit) |
| **Testability** | Mock struct implementing interface | Mock function matching signature |
| **Discoverability** | Method set visible on type | Signature only, no named methods |
| **Composition** | Interface embedding | Function composition / chaining |
| **Standard library precedent** | `io.Reader`, `sort.Interface`, `http.Handler` | `http.HandlerFunc`, `sort.Slice`, `slices.SortFunc` |
| **Runtime cost** | Interface dispatch (itab lookup + indirect call) | Direct function call (no dispatch) |
| **IDE support** | "Find implementations" works | No way to find all matching functions |

### Decision Rule

Use **interfaces** when:
- Strategy has 2+ methods
- Strategy requires initialization, cleanup, or lifecycle management
- You want IDE support for finding implementations
- The strategy represents a major component (storage backend, transport layer)

Use **function types** when:
- Strategy is a single behavior
- Implementations are simple (one-liners or small closures)
- You want inline anonymous function support
- The strategy is a callback, hook, or comparator

Use **the adapter pattern** (function type implementing interface) when:
- You want to support both approaches
- Simple cases should be easy, complex cases should be possible

---

## Standard Library Strategy Patterns

### `sort.Interface` -- Interface-Based Sorting Strategy

Source: [sort package](https://pkg.go.dev/sort)

```go
type Interface interface {
    Len() int
    Less(i, j int) bool
    Swap(i, j int)
}
```

The `Less` method is the comparison strategy. The sort algorithm is fixed; the ordering is pluggable.

After Go 1.18, the function-based alternative became preferred:

```go
// Function-based sort (Go 1.21+)
slices.SortFunc(s, func(a, b T) int { ... })

// Older generic sort
sort.Slice(s, func(i, j int) bool { ... })
```

### `io.Reader` / `io.Writer` -- Data Source/Sink Strategy

Source: [io package](https://pkg.go.dev/io)

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}
```

Every function accepting `io.Reader` or `io.Writer` uses the strategy pattern. The data source or destination is pluggable.

Implementations in the standard library:

| Implementation | Package | Description |
|----------------|---------|-------------|
| `*os.File` | `os` | File system |
| `*bytes.Buffer` | `bytes` | In-memory buffer |
| `*strings.Reader` | `strings` | String as reader |
| `*bufio.Reader` | `bufio` | Buffered reading |
| `*gzip.Reader` | `compress/gzip` | Decompression |
| `*tls.Conn` | `crypto/tls` | TLS connection |
| `*http.Response.Body` | `net/http` | HTTP response |

### `http.Handler` -- Request Processing Strategy

Source: [net/http package](https://pkg.go.dev/net/http)

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}

type HandlerFunc func(ResponseWriter, *Request)

func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

Each registered handler is a strategy for processing requests matching a pattern. Middleware wraps handlers (decorator + strategy).

### `crypto.Signer` -- Cryptographic Signing Strategy

Source: [crypto package](https://pkg.go.dev/crypto)

```go
type Signer interface {
    Public() PublicKey
    Sign(rand io.Reader, digest []byte, opts SignerOpts) (signature []byte, err error)
}
```

The signing algorithm (RSA, ECDSA, Ed25519) is the strategy. TLS, x509 certificate generation, and JWT signing all accept `crypto.Signer`.

### `encoding.TextMarshaler` / `encoding.TextUnmarshaler` -- Serialization Strategy

Source: [encoding package](https://pkg.go.dev/encoding)

```go
type TextMarshaler interface {
    MarshalText() (text []byte, err error)
}

type TextUnmarshaler interface {
    UnmarshalText(text []byte) error
}
```

JSON, XML, YAML, and TOML encoders check for these interfaces to allow types to control their own serialization. The serialization format is the strategy.

---

## Options Pattern (Functional Options)

Source: [Dave Cheney -- Functional options for friendly APIs](https://dave.cheney.net/2014/10/17/functional-options-for-friendly-apis)

Functional options use function-type strategies for configuration:

```go
type Option func(*Server)

func WithTimeout(d time.Duration) Option {
    return func(s *Server) {
        s.timeout = d
    }
}

func WithLogger(l *slog.Logger) Option {
    return func(s *Server) {
        s.logger = l
    }
}

func NewServer(addr string, opts ...Option) *Server {
    s := &Server{addr: addr, timeout: 30 * time.Second}
    for _, opt := range opts {
        opt(s)
    }
    return s
}
```

Each `Option` is a strategy for modifying a default configuration. This pattern is ubiquitous in Go libraries (gRPC, Zap, OpenTelemetry).

---

## Interface Internals

Source: [Go Data Structures: Interfaces (Russ Cox)](https://research.swtch.com/interfaces)

An interface value is represented as a two-word pair:

```
+--------+--------+
|  itab  |  data  |
+--------+--------+
```

- **itab**: Pointer to the interface table. Contains the concrete type descriptor and a table of function pointers for each method in the interface.
- **data**: Pointer to the concrete value (or the value itself if it fits in one pointer).

### itab Structure

```
itab:
  inter: *interfaceType  // points to the interface type descriptor
  _type: *_type           // points to the concrete type descriptor
  hash:  uint32           // copy of _type.hash for fast type switches
  fun:   [1]uintptr       // method table (variable size, one entry per interface method)
```

The `fun` array contains pointers to the concrete type's method implementations, in the order of the interface's method set. When you call `iface.Method()`, the runtime:

1. Loads the itab pointer from the interface value
2. Loads the method pointer from `itab.fun[methodIndex]`
3. Calls the method with the data pointer as the receiver

### itab Caching

The runtime maintains a global hash table of computed itabs. Once an itab for (interface type, concrete type) pair is computed, it's cached and reused. The cost is paid once per pair, not per call.

### Nil Interface vs Nil Concrete Value

An interface value is nil only when both `itab` and `data` are nil:

```
Nil interface:       itab=nil, data=nil   -> == nil is true
Non-nil with nil value: itab=&itab{...}, data=nil -> == nil is false
```

This is why returning a nil pointer through an interface creates a non-nil interface.

---

## Related Documentation

- [Go Specification -- Interfaces](https://go.dev/ref/spec#Interface_types)
- [Go Specification -- Method sets](https://go.dev/ref/spec#Method_sets)
- [Go Specification -- Function types](https://go.dev/ref/spec#Function_types)
- [Effective Go -- Interfaces](https://go.dev/doc/effective_go#interfaces)
- [Go Blog -- The Laws of Reflection](https://go.dev/blog/laws-of-reflection)
- [Go Data Structures: Interfaces](https://research.swtch.com/interfaces)
- [Go Wiki -- CodeReviewComments (interfaces)](https://go.dev/wiki/CodeReviewComments#interfaces)
