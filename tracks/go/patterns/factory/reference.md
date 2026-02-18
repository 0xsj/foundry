# Go Reference -- Factory Pattern

> Extracted from [Effective Go](https://go.dev/doc/effective_go),
> [Go Specification](https://go.dev/ref/spec), and
> [Standard Library Documentation](https://pkg.go.dev/std)
> for the `factory` module. Covers: constructor conventions, interface-based polymorphism,
> registration patterns, init() semantics.

---

## Constructor Function Convention

Source: [Effective Go - Constructors and composite literals](https://go.dev/doc/effective_go#composite_literals)

Go does not have constructors. The idiomatic replacement is a package-level function named `New` or `NewXxx` that returns a configured instance.

### Naming Rules

| Package | Constructor | Returns | Notes |
|---------|------------|---------|-------|
| `ring` | `ring.New(n)` | `*Ring` | Single exported type: use `New` |
| `bufio` | `bufio.NewReader(rd)` | `*Reader` | Multiple exported types: use `NewXxx` |
| `bufio` | `bufio.NewWriter(w)` | `*Writer` | |
| `http` | `http.NewRequest(method, url, body)` | `*Request, error` | |
| `sync` | `N/A` | `sync.Mutex{}` | Zero value is useful: no constructor needed |

### Convention Summary

- If a package exports only one type, use `New()`.
- If a package exports multiple types, use `NewXxx()` for each.
- If the zero value of a type is valid and useful, no constructor is needed (e.g., `sync.Mutex`, `bytes.Buffer`).
- Constructors return either `*T` or `(T, error)` -- never `T` by value unless the type is small and immutable.

```go
// From bufio package
func NewReader(rd io.Reader) *Reader {
    return NewReaderSize(rd, defaultBufSize)
}

func NewReaderSize(rd io.Reader, size int) *Reader {
    // Is it already a Reader?
    b, ok := rd.(*Reader)
    if ok && len(b.buf) >= size {
        return b
    }
    if size < minReadBufferSize {
        size = minReadBufferSize
    }
    r := new(Reader)
    r.reset(make([]byte, size), rd)
    return r
}
```

---

## Interfaces and Implicit Satisfaction

Source: [Go Specification - Interface types](https://go.dev/ref/spec#Interface_types)

> An interface type defines a type set. A variable of interface type can store a value of any type that is in the type set of the interface. Such a type is said to *implement* the interface.

### Key Properties for Factory Pattern

| Property | Implication for Factories |
|----------|--------------------------|
| Interfaces are satisfied implicitly | Implementations don't need to declare `implements` |
| Interface defined at consumer site | The factory package defines the interface, not the implementations |
| Empty interface (`any`) | Can represent any value -- used in generic registries |
| Type assertion | `value.(ConcreteType)` recovers concrete type from interface |
| Type switch | `switch v := value.(type)` dispatches on concrete type |

```go
// Interface defined by the package that USES it
type Writer interface {
    Write(p []byte) (n int, err error)
}

// Any type with this method satisfies Writer -- no declaration needed
type MyBuffer struct {
    data []byte
}

func (b *MyBuffer) Write(p []byte) (n int, err error) {
    b.data = append(b.data, p...)
    return len(p), nil
}
```

### Interface Embedding

Interfaces can be composed by embedding:

```go
type ReadWriter interface {
    Reader
    Writer
}
```

This is relevant for abstract factories where the factory interface may embed other interfaces.

---

## The `init()` Function

Source: [Go Specification - Package initialization](https://go.dev/ref/spec#Package_initialization)

> A package with no imports is initialized by assigning initial values to all its package-level variables and then calling all `init` functions defined in the source, in the order they appear in the source, after evaluating all variable initializers in the package.

### Semantics

| Rule | Detail |
|------|--------|
| Multiple `init()` per file | Allowed; executed in order of appearance |
| Multiple `init()` per package | Allowed; files processed in alphabetical order |
| Execution order | Package-level vars first, then `init()` functions |
| Cross-package order | Imports are initialized first (depth-first) |
| Cannot be called or referenced | `init` is not an identifier that can be used elsewhere |
| Runs before `main()` | All `init()` across all packages complete before `main()` starts |

### Registration Pattern

The `init()` function is the standard mechanism for self-registering drivers:

```go
// From database/sql
func Register(name string, driver driver.Driver) {
    driversMu.Lock()
    defer driversMu.Unlock()
    if driver == nil {
        panic("sql: Register driver is nil")
    }
    if _, dup := drivers[name]; dup {
        panic("sql: Register called twice for driver " + name)
    }
    drivers[name] = driver
}
```

### Blank Import

Source: [Go Specification - Import declarations](https://go.dev/ref/spec#Import_declarations)

> If an explicit period (.) or blank (_) appears as the PackageName, the import is still evaluated for side effects (initialization of package-level variables and execution of init functions).

```go
import _ "image/png"  // only init() runs; no exported names available
```

This is the standard way to register a driver/plugin without directly referencing its exports.

---

## `database/sql` Driver Registration (Complete Reference)

Source: [database/sql package](https://pkg.go.dev/database/sql)

### Registration API

```go
// Register makes a database driver available by the provided name.
// If Register is called twice with the same name or if driver is nil, it panics.
func Register(name string, driver driver.Driver)

// Drivers returns a sorted list of the names of the registered drivers.
func Drivers() []string

// Open opens a database specified by its database driver name and
// a driver-specific data source name.
func Open(driverName, dataSourceName string) (*DB, error)
```

### Driver Interface

```go
// driver.Driver is the interface that must be implemented by a database driver.
type Driver interface {
    // Open returns a new connection to the database.
    Open(name string) (Conn, error)
}
```

### Full Flow

1. Driver package defines a type satisfying `driver.Driver`
2. Driver's `init()` calls `sql.Register("drivername", &MyDriver{})`
3. Application imports driver with blank import: `import _ "github.com/lib/pq"`
4. Application calls `sql.Open("drivername", dsn)` which looks up the registered driver
5. `sql.Open` returns `*sql.DB` (a pool) that uses the driver to create connections

---

## `image` Package Registration

Source: [image package](https://pkg.go.dev/image)

### Registration API

```go
// RegisterFormat registers an image format for use by Decode.
func RegisterFormat(name, magic string, decode func(io.Reader) (Image, error), decodeConfig func(io.Reader) (Config, error))
```

### Auto-Detection Factory

```go
// Decode decodes an image that has been encoded in a registered format.
// It sniffs the first bytes of the input to determine the format.
func Decode(r io.Reader) (Image, string, error)
```

This is a factory pattern where the factory auto-detects the correct implementation by examining the input data (magic bytes), rather than requiring the caller to specify which format.

---

## `sync.Once` for Singleton Factories

Source: [sync.Once documentation](https://pkg.go.dev/sync#Once)

> Once is an object that will perform exactly one action.
>
> A Once must not be copied after first use.

```go
type Once struct {
    // contains filtered or unexported fields
}

func (o *Once) Do(f func())
```

### Guarantees

| Guarantee | Detail |
|-----------|--------|
| Exactly once | `f` is called exactly once, even across goroutines |
| Happens-before | The completion of `f` "happens before" the return of any `Do` call |
| Blocking | Other goroutines calling `Do` block until `f` completes |
| Panic handling | If `f` panics, `Do` considers it completed (won't retry) |

```go
var (
    instance *Config
    once     sync.Once
)

func GetConfig() *Config {
    once.Do(func() {
        instance = loadConfigFromDisk()
    })
    return instance
}
```

---

## Functional Options Pattern

Source: [Rob Pike - Self-referential functions and the design of options](https://commandcenter.blogspot.com/2014/01/self-referential-functions-and-design-of.html), [Dave Cheney - Functional options for friendly APIs](https://dave.cheney.net/2014/10/17/functional-options-for-friendly-apis)

### Pattern Structure

```go
type Option func(*T)

func New(required string, opts ...Option) *T {
    t := &T{
        required: required,
        // defaults
    }
    for _, opt := range opts {
        opt(t)
    }
    return t
}
```

### Variant: Options with Errors

```go
type Option func(*T) error

func New(required string, opts ...Option) (*T, error) {
    t := &T{required: required}
    for _, opt := range opts {
        if err := opt(t); err != nil {
            return nil, fmt.Errorf("applying option: %w", err)
        }
    }
    return t, nil
}
```

### Standard Library Usage

| Package | Function | Options Pattern |
|---------|----------|----------------|
| `net/http` | `http.NewServeMux` | Not options, but similar via `Handle`/`HandleFunc` |
| `google.golang.org/grpc` | `grpc.NewServer` | `grpc.ServerOption` |
| `go.uber.org/zap` | `zap.New` | `zap.Option` |
| `github.com/spf13/cobra` | Various | Uses builder-style but similar concept |

---

## Type Assertions and Type Switches

Source: [Go Specification - Type assertions](https://go.dev/ref/spec#Type_assertions)

These are essential for factory patterns where you need to recover the concrete type from an interface.

### Type Assertion

```go
// Panics if x is not a *PostgresStore
store := x.(*PostgresStore)

// Safe form: comma-ok
store, ok := x.(*PostgresStore)
if !ok {
    // x is not a *PostgresStore
}
```

### Type Switch

```go
switch v := x.(type) {
case *PostgresStore:
    // v is *PostgresStore
case *SQLiteStore:
    // v is *SQLiteStore
default:
    // unknown type
}
```

### Relevance to Factories

Type assertions are used:
- In factory methods to validate or unwrap concrete types
- In registries to convert `any` back to specific factory types
- In decorator/wrapper factories to check if an object already has a capability
