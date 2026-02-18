# Go Reference -- Builder Pattern

> Extracted from the Go standard library documentation, Rob Pike's "Self-referential functions and the design of options" (2014),
> and Dave Cheney's "Functional options for friendly APIs" (2014).
> Covers: functional options, strings.Builder, method chaining, variadic functions.

---

## Functional Options Pattern

Source: [Rob Pike -- Self-referential functions](https://commandcenter.blogspot.com/2014/01/self-referential-functions-and-design-of.html), [Dave Cheney -- Functional options](https://dave.cheney.net/2014/10/17/functional-options-for-friendly-apis)

### Canonical Form

The functional options pattern uses a function type as the option mechanism:

```go
// Option is a function that configures a Server.
type Option func(*Server)

// NewServer creates a Server with the given address and applies options.
func NewServer(addr string, options ...Option) *Server {
    srv := &Server{addr: addr}
    for _, o := range options {
        o(srv)
    }
    return srv
}

// WithTimeout returns an Option that sets the server timeout.
func WithTimeout(d time.Duration) Option {
    return func(s *Server) {
        s.timeout = d
    }
}
```

### Variants

**Error-returning options:**

```go
type Option func(*Config) error

func NewServer(addr string, opts ...Option) (*Server, error) {
    cfg := defaultConfig()
    for _, opt := range opts {
        if err := opt(&cfg); err != nil {
            return nil, err
        }
    }
    return &Server{config: cfg}, nil
}
```

**Interface-based options** (used by gRPC):

```go
type ServerOption interface {
    apply(*serverOptions)
}

type funcServerOption struct {
    f func(*serverOptions)
}

func (fso *funcServerOption) apply(o *serverOptions) {
    fso.f(o)
}

func newFuncServerOption(f func(*serverOptions)) *funcServerOption {
    return &funcServerOption{f: f}
}
```

The interface-based variant allows type safety and prevents external packages from implementing options with arbitrary function types.

### Conventions

| Convention | Description |
|---|---|
| `With*` prefix | Standard naming for option constructors: `WithTimeout`, `WithLogger`, `WithTLS` |
| Unexported config | The config struct fields are unexported; only `With*` functions can set them |
| Required params first | Constructor takes required params before `...Option` |
| Sensible defaults | Unset options use reasonable defaults |
| Composable presets | Bundle common option sets: `ProductionDefaults()`, `TestDefaults()` |

---

## strings.Builder

Source: [Go standard library -- strings.Builder](https://pkg.go.dev/strings#Builder)

### Type Definition

```go
type Builder struct {
    addr *Builder // of receiver, to detect copies by value
    buf  []byte
}
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `Grow` | `func (b *Builder) Grow(n int)` | Pre-allocate at least n bytes of capacity |
| `Len` | `func (b *Builder) Len() int` | Number of accumulated bytes |
| `Cap` | `func (b *Builder) Cap() int` | Capacity of underlying byte slice |
| `Reset` | `func (b *Builder) Reset()` | Reset to empty |
| `String` | `func (b *Builder) String() string` | Return accumulated string (zero-copy) |
| `Write` | `func (b *Builder) Write(p []byte) (int, error)` | Append bytes |
| `WriteByte` | `func (b *Builder) WriteByte(c byte) error` | Append single byte |
| `WriteRune` | `func (b *Builder) WriteRune(r rune) (int, error)` | Append single rune |
| `WriteString` | `func (b *Builder) WriteString(s string) (int, error)` | Append string |

### Key Properties

- All methods use **pointer receivers** (`*Builder`)
- **Zero value is ready to use** -- no constructor needed
- `String()` is **zero-copy** -- returns the internal buffer as a string without allocation
- **Copying after write panics** -- the `addr` field detects copies via `b.addr != b`
- Implements `io.Writer`, `io.ByteWriter`, `io.StringWriter`

### Usage Example

```go
var b strings.Builder
b.Grow(100) // Pre-allocate if size is known
b.WriteString("Hello, ")
b.WriteString("world!")
s := b.String() // "Hello, world!"
```

### Copy Detection

```go
var b1 strings.Builder
b1.WriteString("hello")
b2 := b1       // Copy after write
b2.WriteString(" world") // PANIC: strings: illegal use of non-zero Builder copied by value
```

---

## Method Chaining in Go

Source: [Go specification -- Method declarations](https://go.dev/ref/spec#Method_declarations)

### Pointer vs Value Receivers

Method chaining requires returning the receiver. The receiver type determines mutation behavior:

```go
// Pointer receiver: modifies original, chaining works
func (b *Builder) SetName(name string) *Builder {
    b.name = name
    return b
}

// Value receiver: modifies copy, chaining appears to work but doesn't
func (b Builder) SetName(name string) Builder {
    b.name = name
    return b // Returns modified copy
}
```

### Rules for Builder Methods

| Rule | Reason |
|------|--------|
| Use pointer receiver `*T` | Ensures mutations apply to the original |
| Return `*T` for chaining | Enables `b.X().Y().Z()` syntax |
| Check for accumulated errors | Short-circuit on first failure |
| Keep `Build()` as the terminal method | Signals "construction is complete" |

### Method Set Implications

A pointer receiver method is only in the method set of `*T`, not `T`. This means:

```go
var b Builder           // Value type
b.SetName("test")       // OK: Go automatically takes &b
(&b).SetName("test")    // Explicit: same thing

var bp *Builder = &Builder{}
bp.SetName("test")      // OK: pointer receiver on pointer
```

When passing a builder to an interface, only `*Builder` satisfies an interface requiring pointer receiver methods:

```go
type Configurable interface {
    SetName(string) *Builder
}

var _ Configurable = &Builder{}  // OK
// var _ Configurable = Builder{} // Compile error
```

---

## Variadic Functions

Source: [Go specification -- Function types](https://go.dev/ref/spec#Function_types), [Passing arguments to ... parameters](https://go.dev/ref/spec#Passing_arguments_to_..._parameters)

### Syntax

```go
func f(a int, opts ...Option) { }
```

- The final parameter can be variadic, indicated by `...T`
- Inside the function, the variadic parameter has type `[]T`
- A variadic function can be called with zero or more arguments for that parameter

### Passing Slices to Variadic Functions

```go
opts := []Option{WithTimeout(5 * time.Second), WithLogger(l)}
srv := NewServer("addr", opts...)  // Note the ... suffix
```

### Combining Required and Optional

```go
// Required params come first, variadic options last
func NewServer(addr string, handler http.Handler, opts ...Option) *Server
```

The variadic parameter must be the last parameter. You cannot have:

```go
// INVALID: variadic must be last
func f(opts ...Option, name string) { }  // Compile error
```

---

## Related Standard Library Types

### bytes.Buffer

Similar to `strings.Builder` but works with `[]byte` and supports reading:

```go
var buf bytes.Buffer
buf.WriteString("hello")
buf.Write([]byte{' '})
buf.WriteString("world")
data := buf.Bytes()  // []byte
text := buf.String() // string (allocates)
```

| Feature | strings.Builder | bytes.Buffer |
|---------|-----------------|--------------|
| Primary output | `String()` (zero-copy) | `Bytes()` (zero-copy) |
| Supports reading | No | Yes (`Read`, `ReadByte`, `ReadRune`) |
| Copy safety | Panics on copy | No protection |
| Implements | `io.Writer` | `io.Reader`, `io.Writer`, `io.ReaderFrom` |
| Use when | Building strings | Building bytes, or need read-back |

### http.Request Construction

`http.NewRequest` and `http.NewRequestWithContext` use a traditional constructor pattern, not functional options:

```go
req, err := http.NewRequestWithContext(ctx, "POST", url, body)
req.Header.Set("Content-Type", "application/json")
req.Header.Set("Authorization", "Bearer "+token)
```

Headers and other properties are set after construction via direct field access. This is an alternative to the builder pattern that works when the object's fields are exported and self-validating.

### exec.Cmd

`exec.Command` creates a command with positional args, then you configure it via struct fields:

```go
cmd := exec.Command("git", "log", "--oneline")
cmd.Dir = "/path/to/repo"
cmd.Env = append(os.Environ(), "GIT_PAGER=cat")
cmd.Stdout = &buf
err := cmd.Run()
```

This is the "config struct with exported fields" approach. Works well when configuration is simple and all fields are independent.

---

## Notable Open Source Examples

| Library | Pattern | Option Type |
|---------|---------|-------------|
| `google.golang.org/grpc` | Interface-based options | `grpc.ServerOption` |
| `go.uber.org/zap` | Functional options | `zap.Option` |
| `github.com/spf13/cobra` | Fluent builder | Method chaining on `Command` |
| `database/sql` | Config struct | `sql.Open(driver, dsn)` |
| `net/http` | Exported fields | `http.Server{Addr: ..., Handler: ...}` |
| `github.com/jmoiron/sqlx` | Fluent builder | Query builder methods |

---

## Further Reading

- [Rob Pike -- Self-referential functions and the design of options (2014)](https://commandcenter.blogspot.com/2014/01/self-referential-functions-and-design-of.html)
- [Dave Cheney -- Functional options for friendly APIs (2014)](https://dave.cheney.net/2014/10/17/functional-options-for-friendly-apis)
- [Go Blog -- The strings.Builder type](https://pkg.go.dev/strings#Builder)
- [Go by Example -- Variadic Functions](https://gobyexample.com/variadic-functions)
- [Go Specification -- Method declarations](https://go.dev/ref/spec#Method_declarations)
