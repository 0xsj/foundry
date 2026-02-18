# Builder Pattern -- Go

## The Problem Builder Solves

You need to construct an object that has many configuration options. Some are required, some are optional, some have sensible defaults, and some depend on each other. The naive approach is a constructor with a long parameter list:

```go
// This is painful. What does each bool mean? What happens if you swap timeout and maxRetries?
func NewServer(addr string, port int, timeout time.Duration, maxRetries int, enableTLS bool, certFile string, keyFile string, logger *log.Logger, middleware []Middleware, maxConns int, readTimeout time.Duration, writeTimeout time.Duration) *Server
```

This has well-documented problems:

1. **Positional ambiguity** -- you swap two `int` arguments and the compiler shrugs
2. **Optional parameter explosion** -- Go has no default arguments, so you either take everything or create multiple constructor variants (`NewServer`, `NewServerWithTLS`, `NewServerWithTLSAndLogger`...)
3. **Validation deferred** -- when do you check that `certFile` is required when `enableTLS` is true? At construction? After? Never?
4. **Readability** -- calling code is opaque. Reviewers have to count parameters.

If you come from TypeScript, you solved this with an options object:

```typescript
// TypeScript: options object — clean, named, defaults via spread
const server = new Server({
  addr: "localhost",
  port: 8080,
  timeout: 30_000,
  tls: { cert: "./cert.pem", key: "./key.pem" },
});
```

Go doesn't have keyword arguments or optional parameters. But it has two idioms that solve this problem differently, each with distinct tradeoffs: **Functional Options** and **Fluent Builders**.

### When you will actually encounter this

- Configuring HTTP servers, database connections, cache clients
- Building SQL queries, API requests, CLI commands
- Constructing ETL pipelines, middleware chains, test fixtures
- Defining gRPC service options, logging configurations
- Any constructor that has grown beyond 3-4 parameters

### Your notes
<!-- User adds insights here during learning -->


---

## Functional Options: The Go Way

### The Core Idea

Functional Options is the idiomatic Go approach for constructable objects with many optional settings. It was popularized by Rob Pike and Dave Cheney and is used extensively in the standard library and virtually every major Go library.

The pattern has three parts:

1. **An unexported config struct** that holds all settings
2. **An `Option` type** that is a function modifying the config
3. **A constructor** that takes required params + variadic options

```go
// The option type: a function that mutates config
type Option func(*Config)

// Option constructors: exported functions that return Options
func WithTimeout(d time.Duration) Option {
    return func(c *Config) {
        c.timeout = d
    }
}

func WithLogger(l *log.Logger) Option {
    return func(c *Config) {
        c.logger = l
    }
}

// Constructor: required params first, then variadic options
func NewServer(addr string, opts ...Option) *Server {
    // Start with defaults
    cfg := Config{
        timeout:    30 * time.Second,
        maxConns:   100,
        logger:     log.Default(),
    }

    // Apply each option
    for _, opt := range opts {
        opt(&cfg)
    }

    return &Server{config: cfg, addr: addr}
}
```

Calling code reads like a sentence:

```go
srv := NewServer("localhost:8080",
    WithTimeout(60 * time.Second),
    WithTLS("cert.pem", "key.pem"),
    WithLogger(customLogger),
)
```

### Why This Works So Well in Go

**Self-documenting.** Each `With*` function names the option. No positional ambiguity. Reviewers know what each argument does without checking the constructor signature.

**Backward-compatible.** Adding a new option is a non-breaking change -- you add a new `With*` function. Existing callers are unaffected. This is huge for library APIs.

**Composable.** Options are just functions, so you can create preset bundles:

```go
func ProductionDefaults() Option {
    return func(c *Config) {
        c.timeout = 30 * time.Second
        c.maxConns = 500
        c.enableMetrics = true
    }
}

srv := NewServer(addr, ProductionDefaults(), WithLogger(prodLogger))
```

**Testable.** In tests, you only specify what matters:

```go
// Test doesn't care about timeout, TLS, or logging
srv := NewServer("localhost:0", WithMaxConns(1))
```

### The Validation Question

One weakness of basic functional options: each `With*` function silently mutates config. If someone passes `WithTimeout(-5 * time.Second)`, when does the error surface?

**Option 1: Validate at Build time.** The constructor checks the final config:

```go
func NewServer(addr string, opts ...Option) (*Server, error) {
    cfg := defaultConfig()
    for _, opt := range opts {
        opt(&cfg)
    }
    if cfg.timeout <= 0 {
        return nil, fmt.Errorf("timeout must be positive, got %v", cfg.timeout)
    }
    return &Server{config: cfg, addr: addr}, nil
}
```

**Option 2: Return errors from options.** Change the Option type:

```go
type Option func(*Config) error

func WithTimeout(d time.Duration) Option {
    return func(c *Config) error {
        if d <= 0 {
            return fmt.Errorf("timeout must be positive, got %v", d)
        }
        c.timeout = d
        return nil
    }
}

func NewServer(addr string, opts ...Option) (*Server, error) {
    cfg := defaultConfig()
    for _, opt := range opts {
        if err := opt(&cfg); err != nil {
            return nil, fmt.Errorf("invalid option: %w", err)
        }
    }
    return &Server{config: cfg, addr: addr}, nil
}
```

Option 2 is better for library code because it reports the specific bad option, not just a general "config invalid." Option 1 is simpler for internal code.

### Standard Library Examples

The standard library uses functional options implicitly. `http.Server` uses struct literal configuration (another valid approach), but libraries like `google.golang.org/grpc` use functional options extensively:

```go
// gRPC server with functional options
srv := grpc.NewServer(
    grpc.MaxRecvMsgSize(4 << 20),
    grpc.UnaryInterceptor(loggingInterceptor),
    grpc.StreamInterceptor(streamInterceptor),
)
```

### Your notes
<!-- User adds insights here during learning -->


---

## Fluent Builder: Method Chaining

### When Functional Options Are Not Enough

Functional options work best when you are configuring a single object. But some construction processes are inherently sequential -- you are building something step by step, and each step depends on previous steps. SQL queries are the canonical example:

```go
// This reads like a SQL query
query := NewQuery().
    Select("name", "email").
    From("users").
    Where("active = ?", true).
    Where("role = ?", "admin").
    OrderBy("name", Asc).
    Limit(10).
    Build()
```

Here, the Builder pattern uses method chaining -- each method returns the builder so calls can be chained. The final `Build()` method validates and produces the result.

### The Implementation

```go
type QueryBuilder struct {
    selects  []string
    table    string
    wheres   []whereClause
    orderBys []orderClause
    limit    int
    offset   int
    err      error  // Captures first error
}

type whereClause struct {
    condition string
    args      []any
}

// Each method returns *QueryBuilder for chaining
func (qb *QueryBuilder) Select(columns ...string) *QueryBuilder {
    if qb.err != nil {
        return qb  // Short-circuit on previous error
    }
    if len(columns) == 0 {
        qb.err = fmt.Errorf("select requires at least one column")
        return qb
    }
    qb.selects = append(qb.selects, columns...)
    return qb
}

func (qb *QueryBuilder) From(table string) *QueryBuilder {
    if qb.err != nil {
        return qb
    }
    qb.table = table
    return qb
}
```

### Error Handling in Builders

There are two schools of thought:

**Accumulate errors, check at Build():**

```go
func (qb *QueryBuilder) Build() (string, []any, error) {
    if qb.err != nil {
        return "", nil, qb.err
    }
    if qb.table == "" {
        return "", nil, fmt.Errorf("FROM clause is required")
    }
    // ... assemble the query
}
```

**Fail fast with first error:**

Same as above -- the `qb.err != nil` check at the top of each method means the first error short-circuits all subsequent calls. The chain still executes syntactically, but each method is a no-op after the first failure.

Both approaches preserve the fluent interface. The user checks for errors once at `Build()` time, not after each step.

### Pointer Receiver Trap

This is a critical Go gotcha. Builder methods **must** use pointer receivers. With value receivers, each method gets a copy, and the original is never modified:

```go
// WRONG: value receiver -- each call modifies a copy
func (qb QueryBuilder) Select(columns ...string) QueryBuilder {
    qb.selects = append(qb.selects, columns...)
    return qb  // Returns modified copy, original unchanged
}

// Chaining breaks:
q := NewQuery()
q.Select("name")  // Modifies a copy, q is unchanged
q.From("users")   // q still has no selects
```

With pointer receivers, all methods modify the same builder:

```go
// CORRECT: pointer receiver -- modifies the actual builder
func (qb *QueryBuilder) Select(columns ...string) *QueryBuilder {
    qb.selects = append(qb.selects, columns...)
    return qb
}
```

### Your notes
<!-- User adds insights here during learning -->


---

## Functional Options vs Fluent Builder

| Aspect | Functional Options | Fluent Builder |
|--------|-------------------|----------------|
| **Best for** | Configuring an object | Building something step by step |
| **Ordering** | Options are order-independent | Steps may have logical order |
| **Validation** | At construction or per-option | At Build() time |
| **Composability** | Preset bundles via combining options | Not easily composable |
| **Go idiom** | Yes -- the standard approach | Less common but valid for DSLs |
| **Backward compat** | Adding options is non-breaking | Adding methods is non-breaking |
| **Error handling** | Return error from constructor | Return error from Build() |
| **Thread safety** | Options are pure functions | Builder is mutable state |
| **Use in stdlib** | gRPC, Zap, many libraries | strings.Builder, bytes.Buffer |

**Rule of thumb:** Use functional options for constructors. Use fluent builders when you are building a domain-specific language (query builders, pipeline builders, request builders).

### Your notes
<!-- User adds insights here during learning -->


---

## Immutable Builders

Some builders create a new instance at each step instead of mutating the original. This is safer for concurrent use and allows branching:

```go
// Immutable builder -- each method returns a NEW builder
func (qb QueryBuilder) Where(cond string, args ...any) QueryBuilder {
    // Copy the wheres slice to avoid sharing
    newWheres := make([]whereClause, len(qb.wheres), len(qb.wheres)+1)
    copy(newWheres, qb.wheres)
    newWheres = append(newWheres, whereClause{condition: cond, args: args})

    return QueryBuilder{
        selects:  qb.selects,
        table:    qb.table,
        wheres:   newWheres,
        orderBys: qb.orderBys,
        limit:    qb.limit,
        offset:   qb.offset,
    }
}

// This enables branching:
base := NewQuery().Select("name", "email").From("users")
activeUsers := base.Where("active = ?", true)
adminUsers := base.Where("role = ?", "admin")
// base, activeUsers, and adminUsers are all independent
```

The tradeoff is allocation overhead -- each step creates a new struct and copies slices. For builders used in hot paths, this matters. For configuration or query building, it usually doesn't.

### Your notes
<!-- User adds insights here during learning -->


---

## Go Standard Library: strings.Builder

`strings.Builder` is a fluent builder used for efficient string concatenation. It is the standard replacement for `bytes.Buffer` when building strings:

```go
var b strings.Builder
b.WriteString("SELECT ")
b.WriteString("name, email")
b.WriteString(" FROM users")
b.WriteString(" WHERE active = true")
query := b.String()
```

Key design decisions in `strings.Builder`:
- Uses pointer receiver (`*Builder`) -- mutations happen in place
- `String()` method is zero-copy (returns the internal buffer directly)
- Cannot be copied after first write (runtime panic) -- this prevents accidental sharing
- No `Build()` method -- you call `String()` when done

The "cannot be copied" invariant is enforced at runtime with `noCopy`:

```go
type Builder struct {
    addr *Builder // points to self, detects copies
    buf  []byte
}
```

If you copy a `Builder` after writing, the address won't match, and the next write panics. This is a deliberate design choice to prevent subtle aliasing bugs.

### Your notes
<!-- User adds insights here during learning -->


---

## Cross-Language Comparison

### TypeScript: Builder Classes

TypeScript builders tend to be class-based with method chaining, similar to Go fluent builders. But TypeScript has a key advantage -- optional parameters and default values:

```typescript
// TypeScript: options object makes simple builders unnecessary
interface ServerOptions {
  port?: number;       // optional with ?
  timeout?: number;    // default can be set via ??
  tls?: { cert: string; key: string };
}

function createServer(addr: string, opts: ServerOptions = {}) {
  const port = opts.port ?? 8080;
  const timeout = opts.timeout ?? 30_000;
  // ...
}
```

For simple cases, TypeScript's options objects eliminate the need for builders entirely. Builders in TypeScript are used for genuinely complex construction (query builders, test data builders, request builders).

### Rust: Typestate Builders

Rust takes builders to another level with the typestate pattern. The type system enforces that required steps are completed before `build()` can be called:

```rust
// Rust: builder with typestate -- won't compile without required steps
let server = ServerBuilder::new("localhost")
    .port(8080)           // returns ServerBuilder<HasPort>
    .handler(my_handler)  // returns ServerBuilder<HasHandler>
    .build();             // Only available when HasPort AND HasHandler

// This won't compile:
// ServerBuilder::new("localhost").build()  // Error: build() not available
```

Go cannot do this at compile time. You validate at `Build()` runtime. Rust's approach catches errors earlier but adds type complexity.

### Python: Named Arguments

Python solves the "too many constructor parameters" problem with named arguments and default values:

```python
# Python: named arguments make builders rarely needed
server = Server(
    addr="localhost",
    port=8080,
    timeout=30,
    tls_cert="cert.pem",  # optional, defaults to None
)
```

Like TypeScript, Python's language features make simple builders unnecessary. Builders in Python appear mainly for complex DSLs (SQLAlchemy's query builder, for example).

### Your notes
<!-- User adds insights here during learning -->


---

## Anti-Patterns

### 1. Builder With Too Many Required Parameters

If your builder has 5 required fields that must be set before `Build()` works, the builder isn't adding value. Required parameters should be constructor arguments:

```go
// BAD: everything is optional but most things are required
srv := NewServerBuilder().
    WithAddr("localhost").    // required -- why is this an option?
    WithPort(8080).           // required
    WithHandler(mux).         // required
    Build()

// GOOD: required params in constructor, optional in options
srv := NewServer("localhost:8080", mux,
    WithTimeout(60 * time.Second),
    WithTLS("cert.pem", "key.pem"),
)
```

### 2. Builder That Doesn't Validate

A builder that silently produces invalid objects is worse than no builder. If `Build()` doesn't validate, you have pushed the error from construction time to usage time -- the exact problem builders are supposed to prevent:

```go
// BAD: Build() always succeeds, produces broken server
func (b *ServerBuilder) Build() *Server {
    return &Server{config: b.config}  // No validation!
}

// GOOD: Build() validates and returns error
func (b *ServerBuilder) Build() (*Server, error) {
    if b.config.addr == "" {
        return nil, fmt.Errorf("address is required")
    }
    if b.config.enableTLS && b.config.certFile == "" {
        return nil, fmt.Errorf("TLS enabled but no certificate file provided")
    }
    return &Server{config: b.config}, nil
}
```

### 3. Exposing Builder Internals

Returning a pointer to the builder's internal config lets callers modify it after construction, breaking encapsulation:

```go
// BAD: returns pointer to internal state
func (b *ServerBuilder) Config() *Config {
    return &b.config  // Caller can mutate this!
}

// GOOD: return a copy
func (b *ServerBuilder) Config() Config {
    return b.config
}
```

### 4. Functional Option With Side Effects

Options should configure the target object, not reach out and modify global state:

```go
// BAD: option has side effects
func WithDebug() Option {
    return func(c *Config) {
        c.debug = true
        log.SetFlags(log.Lshortfile)  // Modifies global logger!
        os.Setenv("DEBUG", "1")       // Modifies environment!
    }
}

// GOOD: option only touches config
func WithDebug() Option {
    return func(c *Config) {
        c.debug = true
    }
}
```

### Your notes
<!-- User adds insights here during learning -->


---

## When to Use Each Approach

```
Need to configure a struct/service with optional params?
  -> Functional Options

Building a DSL (queries, pipelines, requests)?
  -> Fluent Builder

Only 1-3 optional params?
  -> Config struct with sensible defaults (simplest approach)

Need compile-time guarantees on required fields?
  -> Use Go's type system (required = constructor params)
  -> Or switch to Rust :)
```

### The Simplest Approach: Config Struct

Before reaching for functional options, consider whether a simple config struct is enough:

```go
type Config struct {
    Timeout  time.Duration
    MaxConns int
    Logger   *log.Logger
}

func NewServer(addr string, cfg Config) *Server {
    if cfg.Timeout == 0 {
        cfg.Timeout = 30 * time.Second
    }
    if cfg.MaxConns == 0 {
        cfg.MaxConns = 100
    }
    if cfg.Logger == nil {
        cfg.Logger = log.Default()
    }
    return &Server{addr: addr, config: cfg}
}
```

This works well when zero values are distinguishable from "not set." It breaks down when zero is a valid setting (e.g., `MaxConns: 0` means "no limit" vs "use default").

### Your notes
<!-- User adds insights here during learning -->


---

## Key Takeaways

1. **Functional options** are the idiomatic Go approach for complex constructors. Learn this pattern cold -- you will use it and see it everywhere.
2. **Fluent builders** are for DSLs and sequential construction. SQL queries, HTTP requests, and pipeline definitions are natural fits.
3. **Validate at Build() time.** A builder that produces invalid objects is worse than no builder.
4. **Pointer receivers on builder methods.** Value receivers silently break chaining -- one of the most common Go builder bugs.
5. **Required params go in the constructor, optional params go in options.** If everything is optional, your API is probably underspecified.
6. **Options are composable.** Create preset bundles for common configurations (production defaults, test defaults).
7. **Consider the simple approach first.** If a config struct with zero-value defaults covers your needs, don't over-engineer.

### Your notes
<!-- User adds insights here during learning -->
