# Builder Pattern -- Rust

## The Problem Builder Solves

Rust has no default parameters, no function overloading, and no named arguments. When a struct has 8 fields, 5 of which are optional, you have a problem. In TypeScript you would pass an options object with optional fields. In Go you would use functional options. In Rust, you reach for the Builder pattern.

Consider a production HTTP server configuration:

```rust
struct ServerConfig {
    address: String,
    port: u16,
    max_connections: usize,
    timeout_secs: u64,
    tls_cert_path: Option<String>,
    tls_key_path: Option<String>,
    cors_origins: Vec<String>,
    request_body_limit: usize,
}
```

Without a builder, constructing this requires specifying every field:

```rust
let config = ServerConfig {
    address: "0.0.0.0".to_string(),
    port: 8080,
    max_connections: 1024,
    timeout_secs: 30,
    tls_cert_path: None,
    tls_key_path: None,
    cors_origins: vec![],
    request_body_limit: 1024 * 1024,
};
```

Every caller must know all the fields, their types, and their sensible defaults. Add a field and every call site breaks. The builder pattern fixes this by providing a fluent API where you only set what you care about.

But Rust's ownership system makes the builder pattern more interesting than in other languages. You get to choose between consuming builders (take `self`) and borrowing builders (take `&mut self`), and Rust's type system lets you encode required-field constraints **at compile time** through typestates -- something no garbage-collected language can do.

### Your notes
<!-- -->


---

## Simple Builder: The Basics

The simplest builder stores optional fields and produces the target struct on `build()`. The `build()` method returns `Result` because validation can fail.

```rust
struct DatabasePool {
    host: String,
    port: u16,
    max_connections: u32,
    min_connections: u32,
    connection_timeout_secs: u64,
    idle_timeout_secs: u64,
    database: String,
    ssl_mode: bool,
}

struct DatabasePoolBuilder {
    host: Option<String>,
    port: u16,
    max_connections: u32,
    min_connections: u32,
    connection_timeout_secs: u64,
    idle_timeout_secs: u64,
    database: Option<String>,
    ssl_mode: bool,
}

impl DatabasePoolBuilder {
    fn new() -> Self {
        Self {
            host: None,
            port: 5432,
            max_connections: 10,
            min_connections: 1,
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            database: None,
            ssl_mode: false,
        }
    }

    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    fn database(mut self, db: &str) -> Self {
        self.database = Some(db.to_string());
        self
    }

    fn ssl(mut self, enabled: bool) -> Self {
        self.ssl_mode = enabled;
        self
    }

    fn build(self) -> Result<DatabasePool, String> {
        let host = self.host.ok_or("host is required")?;
        let database = self.database.ok_or("database is required")?;

        if self.min_connections > self.max_connections {
            return Err(format!(
                "min_connections ({}) cannot exceed max_connections ({})",
                self.min_connections, self.max_connections
            ));
        }

        Ok(DatabasePool {
            host,
            port: self.port,
            max_connections: self.max_connections,
            min_connections: self.min_connections,
            connection_timeout_secs: self.connection_timeout_secs,
            idle_timeout_secs: self.idle_timeout_secs,
            database,
            ssl_mode: self.ssl_mode,
        })
    }
}
```

Usage:

```rust
let pool = DatabasePoolBuilder::new()
    .host("db.internal.prod")
    .database("users")
    .max_connections(50)
    .ssl(true)
    .build()?;
```

**Key decisions in this design:**

1. **`build()` returns `Result`** -- required fields (`host`, `database`) are `Option<String>` in the builder, validated at build time. This is a runtime check. We will see how typestates move this to compile time.

2. **Methods take `self` (consuming)** -- each method takes ownership and returns `Self`. This prevents using a partially-configured builder after calling `build()`. More on this choice shortly.

3. **Sensible defaults** -- `port`, `max_connections`, timeouts all have defaults. Callers only override what they need.

> **Coming from TypeScript?** This is roughly equivalent to:
> ```typescript
> interface DatabasePoolOptions {
>   host: string;        // required
>   database: string;    // required
>   port?: number;       // default 5432
>   maxConnections?: number; // default 10
> }
> function createPool(options: DatabasePoolOptions): DatabasePool
> ```
> TypeScript solves this with optional properties on an interface. Rust can't do that because structs don't have optional fields, and there are no default parameter values. The builder pattern is the idiomatic workaround.

### Your notes
<!-- -->


---

## Consuming vs Borrowing Builders

This is the first real design decision when writing a builder in Rust. It comes down to what `self` you take in the setter methods.

### Consuming Builder (`self`)

Each method takes ownership and returns `Self`:

```rust
fn host(mut self, host: &str) -> Self {
    self.host = Some(host.to_string());
    self
}
```

**Pros:**
- Clean chaining: `Builder::new().host("x").port(5432).build()`
- After `build()`, the builder is consumed -- no accidental reuse
- The compiler prevents use-after-build

**Cons:**
- Cannot build multiple configs from one builder
- Cannot conditionally set fields across multiple statements without rebinding

```rust
// This works:
let config = Builder::new().host("x").port(5432).build()?;

// This requires rebinding:
let mut builder = Builder::new();
builder = builder.host("x");
if use_ssl {
    builder = builder.ssl(true);
}
let config = builder.build()?;
```

### Borrowing Builder (`&mut self`)

Each method takes a mutable reference and returns `&mut Self`:

```rust
fn host(&mut self, host: &str) -> &mut Self {
    self.host = Some(host.to_string());
    self
}
```

**Pros:**
- Can build multiple configs from one builder
- Conditional setting is natural -- just call or don't call
- Familiar from Java/C# builders

**Cons:**
- Chaining across statements can hit lifetime issues
- Builder remains usable after `build()` -- no compile-time "consumed" guarantee
- `build()` must clone or take `&self` (can't move out of a reference)

```rust
// Chaining works in a single expression:
let config = builder.host("x").port(5432).build()?;

// But this fails with lifetimes if you split across statements:
let b = builder.host("x");  // b: &mut Builder
let c = b.port(5432);       // c: &mut Builder (same reference)
// b and c are the same borrow -- fine within one chain,
// but confusing when stored in separate variables
```

### Which to Choose

| Criterion | Consuming (`self`) | Borrowing (`&mut self`) |
|-----------|-------------------|------------------------|
| Single-use builder | Preferred | Works |
| Reusable builder (template) | Cannot | Preferred |
| Conditional fields | Requires rebinding | Natural |
| Chaining ergonomics | Clean in single expression | Lifetime edge cases |
| Use-after-build safety | Compile-time error | Runtime responsibility |
| Standard library usage | `Command::new()`, `thread::Builder` | Less common |

**Rule of thumb:** Use consuming builders unless you need reusability. The standard library prefers consuming builders (`Command`, `thread::Builder`), and they provide stronger guarantees.

### Your notes
<!-- -->


---

## The `Default` Trait: Builder's Lightweight Alternative

Before reaching for a full builder, consider whether `Default` + struct update syntax is enough.

```rust
#[derive(Debug)]
struct RetryConfig {
    max_retries: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_factor: f64,
    jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 30_000,
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

// Usage: override only what you care about
let config = RetryConfig {
    max_retries: 5,
    max_delay_ms: 60_000,
    ..Default::default()
};
```

The `..Default::default()` syntax fills in remaining fields from the `Default` implementation. This is Rust's closest equivalent to TypeScript's `{ ...defaults, ...overrides }` spread.

### When Default Is Enough

- All fields have sensible defaults
- No required fields (everything has a default)
- No cross-field validation needed
- The struct is small (< 5 fields)
- Fields are all public or you control the module

### When You Need a Real Builder

- Some fields are required (no sensible default for `host`)
- Cross-field validation is needed (`min < max`)
- Fields are complex to construct (derived from other fields)
- You want compile-time enforcement of required fields (typestate)
- The API is public and ergonomics matter

> **Coming from Go?** The `Default` trait serves a similar role to Go's zero values, but with control. Go structs always have zero values (`""`, `0`, `false`, `nil`). Rust's `Default` trait lets you define what the "zero value" should be, and it's opt-in -- not every type has a `Default`.

### Your notes
<!-- -->


---

## Typestate Builder: Compile-Time Required Fields

This is where Rust's type system shines. A typestate builder uses generic type parameters to track which required fields have been set. The `build()` method is only available when all required fields are present. If you forget a required field, the code **does not compile**.

### The Core Idea

```rust
use std::marker::PhantomData;

// Marker types -- these exist only in the type system, no runtime cost
struct Missing;
struct Set;

struct ServerBuilder<Addr, Port> {
    address: Option<String>,
    port: Option<u16>,
    max_connections: usize,
    _state: PhantomData<(Addr, Port)>,
}
```

`PhantomData` tells the compiler "this struct conceptually contains an `(Addr, Port)` even though it doesn't store one." It has zero size at runtime.

### Building the Typestate Machine

```rust
impl ServerBuilder<Missing, Missing> {
    fn new() -> Self {
        Self {
            address: None,
            port: None,
            max_connections: 1024,
            _state: PhantomData,
        }
    }
}

// Setting address transitions Missing -> Set for the first type parameter
impl<Port> ServerBuilder<Missing, Port> {
    fn address(self, addr: &str) -> ServerBuilder<Set, Port> {
        ServerBuilder {
            address: Some(addr.to_string()),
            port: self.port,
            max_connections: self.max_connections,
            _state: PhantomData,
        }
    }
}

// Setting port transitions Missing -> Set for the second type parameter
impl<Addr> ServerBuilder<Addr, Missing> {
    fn port(self, port: u16) -> ServerBuilder<Addr, Set> {
        ServerBuilder {
            address: self.address,
            port: Some(port),
            max_connections: self.max_connections,
            _state: PhantomData,
        }
    }
}

// Optional fields work on any state
impl<Addr, Port> ServerBuilder<Addr, Port> {
    fn max_connections(mut self, n: usize) -> Self {
        self.max_connections = n;
        self
    }
}

// build() is only available when BOTH required fields are Set
impl ServerBuilder<Set, Set> {
    fn build(self) -> ServerConfig {
        ServerConfig {
            address: self.address.unwrap(),  // safe: type system guarantees it's set
            port: self.port.unwrap(),        // safe: type system guarantees it's set
            max_connections: self.max_connections,
        }
    }
}
```

Now the compiler enforces required fields:

```rust
// Compiles:
let config = ServerBuilder::new()
    .address("0.0.0.0")
    .port(8080)
    .build();

// Does NOT compile -- build() doesn't exist on ServerBuilder<Set, Missing>:
let config = ServerBuilder::new()
    .address("0.0.0.0")
    .build();  // error: no method named `build` found
```

The error message tells you exactly what is missing. This is a compile-time guarantee that no runtime check can match.

### The Cost of Typestates

- **Code duplication**: Each required field adds an `impl` block. With N required fields, you need N separate setter blocks plus the `build()` block. This scales poorly past 3-4 required fields.
- **Error messages**: The compiler says "no method named `build`" rather than "you forgot to set `port`." Custom error messages are possible but require more machinery.
- **Complexity**: Readers unfamiliar with the pattern may be confused by `PhantomData` and the generic parameters.

**When to use typestates:**
- Public APIs where misuse should be impossible
- Safety-critical builders (network config, crypto params)
- Builders with few required fields (2-3 max)
- Libraries where you control the API surface

**When NOT to use typestates:**
- Internal code where a `Result` from `build()` is fine
- Many required fields (the combinatorial explosion gets unwieldy)
- Rapid prototyping

> **This doesn't exist in Go or TypeScript.** Neither language has a type-level mechanism to enforce "this method is only callable after that method." Go's functional options always validate at runtime. TypeScript could approximate it with conditional types, but it is extremely unergonomic. This is a genuinely Rust-specific capability enabled by zero-cost PhantomData and monomorphization.

### Your notes
<!-- -->


---

## `derive_builder` and Macro Approaches

Writing builders by hand is repetitive. The Rust ecosystem has several crates that generate builders via derive macros.

### `derive_builder` Crate

```rust
// With derive_builder (add to Cargo.toml: derive_builder = "0.12")
use derive_builder::Builder;

#[derive(Builder)]
#[builder(setter(into))]
struct ServerConfig {
    #[builder(setter(into))]
    address: String,
    port: u16,
    #[builder(default = "1024")]
    max_connections: usize,
    #[builder(default)]
    tls_enabled: bool,
    #[builder(setter(each = "add_origin"))]
    cors_origins: Vec<String>,
}

// Generated usage:
let config = ServerConfigBuilder::default()
    .address("0.0.0.0")
    .port(8080)
    .add_origin("https://example.com".to_string())
    .add_origin("https://api.example.com".to_string())
    .build()?;
```

### What `derive_builder` Generates

The macro generates:
- A `ServerConfigBuilder` struct with all fields as `Option<T>`
- Setter methods that take `Into<T>` (so you can pass `&str` for `String` fields)
- A `build()` method that returns `Result<ServerConfig, String>`
- Support for `#[builder(default)]` to set defaults
- `each` attribute for collection fields (adds items one at a time)

### `typed-builder` Crate

An alternative that uses typestates under the hood:

```rust
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
struct ServerConfig {
    address: String,
    port: u16,
    #[builder(default = 1024)]
    max_connections: usize,
    #[builder(default)]
    tls_enabled: bool,
}

// Compile-time enforcement: address and port are required
let config = ServerConfig::builder()
    .address("0.0.0.0".to_string())
    .port(8080)
    .build();
```

### When to Use Macros vs Hand-Written Builders

| Criterion | Hand-written | `derive_builder` | `typed_builder` |
|-----------|-------------|-------------------|-----------------|
| Control | Full | Moderate | Moderate |
| Custom validation | Easy | Possible with `#[builder(build_fn)]` | Limited |
| Typestate enforcement | Manual | No (runtime `Result`) | Yes (compile-time) |
| Boilerplate | High | Minimal | Minimal |
| Learning | Educational | Hides mechanism | Hides mechanism |
| Dependencies | None | 1 crate + proc-macro | 1 crate + proc-macro |

**Recommendation:** Write builders by hand when learning or when you need custom validation logic. Use derive macros in production when the builder is straightforward and you want to reduce boilerplate.

### Your notes
<!-- -->


---

## Standard Library Builders

The Rust standard library uses the builder pattern extensively. Studying these builds intuition for idiomatic usage.

### `std::process::Command`

The most famous builder in the standard library. It constructs a child process:

```rust
use std::process::Command;

let output = Command::new("ls")
    .arg("-la")
    .arg("/tmp")
    .env("LANG", "en_US.UTF-8")
    .current_dir("/home")
    .spawn()
    .expect("failed to start process");
```

**Design notes:**
- `Command::new(program)` -- the program is required, so it goes in the constructor, not a setter
- `.arg()` can be called multiple times (accumulating, not replacing)
- `.spawn()` is the "build" step -- it returns `io::Result<Child>`
- The builder is **borrowing** (`&mut self`) -- you can reuse a `Command` to spawn multiple processes
- All methods return `&mut Command` for chaining

### `std::thread::Builder`

```rust
use std::thread;

let handler = thread::Builder::new()
    .name("worker-1".into())
    .stack_size(4 * 1024 * 1024)
    .spawn(|| {
        println!("running on custom thread");
    })
    .expect("failed to spawn thread");
```

**Design notes:**
- `Builder::new()` returns a builder with defaults
- `.name()` and `.stack_size()` are optional configuration
- `.spawn()` is the "build" step, taking a closure as the thread's work
- Returns `io::Result<JoinHandle<T>>`

### `std::fs::OpenOptions`

```rust
use std::fs::OpenOptions;

let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(false)
    .open("data.log")?;
```

**Design notes:**
- Boolean setters -- each flag is a method taking `bool`
- `.open()` is the "build" step
- Borrowing builder (`&mut self`) -- can reuse for multiple files
- Strong defaults (everything is `false`)

### Pattern Summary from the Standard Library

| Builder | Required in constructor | Optional setters | Build method | Reusable? |
|---------|----------------------|------------------|-------------|-----------|
| `Command` | program name | args, env, dir | `spawn()` | Yes |
| `thread::Builder` | none | name, stack_size | `spawn(closure)` | No (consuming) |
| `OpenOptions` | none | read, write, create, etc. | `open(path)` | Yes |
| `Regex` (regex crate) | pattern | case_insensitive, multi_line | `build()` | No |

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go | TypeScript |
|--------|------|----|------------|
| **The need** | No default params, no overloading | No default params, no overloading | Has optional params and object spread |
| **Primary pattern** | Builder struct with `build()` | Functional options (`With...` functions) | Options object with defaults |
| **Required field enforcement** | Typestate (compile-time) or `Result` (runtime) | Runtime validation only | TypeScript type system (`Required<T>`) |
| **Default values** | `Default` trait + struct update syntax | Zero values | Default parameter values, `?? defaultVal` |
| **Chaining** | `self` or `&mut self` return | Not idiomatic (options are variadic args) | `this` return (class-based) |
| **Immutability** | Builder consumed after `build()` | Options applied to pointer | Spread creates new object |
| **Macro support** | `derive_builder`, `typed-builder` | None needed (pattern is simple) | None needed |
| **Compile-time safety** | Typestate pattern | Not possible | Partial (conditional types, but unergonomic) |

### Go: Functional Options

```go
type ServerConfig struct {
    Address        string
    Port           int
    MaxConnections int
}

type Option func(*ServerConfig)

func WithPort(port int) Option {
    return func(c *ServerConfig) {
        c.Port = port
    }
}

func NewServer(address string, opts ...Option) *ServerConfig {
    cfg := &ServerConfig{Address: address, Port: 8080, MaxConnections: 1024}
    for _, opt := range opts {
        opt(cfg)
    }
    return cfg
}

// Usage:
// server := NewServer("0.0.0.0", WithPort(9090))
```

Go's functional options pattern is simpler -- a slice of functions that mutate a config struct. Required parameters go in the constructor. No intermediate builder struct needed. The tradeoff: no compile-time enforcement of required fields, and validation happens at runtime.

### TypeScript: Options Object

```typescript
interface ServerOptions {
    address: string;          // required
    port?: number;            // default 8080
    maxConnections?: number;  // default 1024
    tls?: { cert: string; key: string };
}

function createServer(options: ServerOptions): Server {
    const config = {
        port: 8080,
        maxConnections: 1024,
        ...options,
    };
    return new Server(config);
}

// Usage:
// const server = createServer({ address: "0.0.0.0", port: 9090 });
```

TypeScript's approach is the simplest -- optional properties on an interface, spread syntax for defaults. The type system enforces required fields at compile time (you cannot omit `address`). The tradeoff: no complex validation, no fluent chaining, no incremental construction.

### When Builder Is Overkill

If your struct has 3-4 fields and all have defaults, use `Default` + struct update syntax. If you only have one or two required fields, put them in the constructor (`::new(required_field)`) and use setters for the rest, as the standard library's `Command::new(program)` does.

The builder pattern earns its complexity when you have:
- Many optional fields (> 5)
- Complex cross-field validation
- A public API where ergonomics matter
- Required fields that should be compile-time enforced

### Your notes
<!-- -->


---

## Advanced: Generic Builders and Type-Safe Pipelines

Builders can leverage Rust's generics to produce differently-typed results based on configuration. This is more advanced than simple field collection.

```rust
// A connection builder that produces different connection types
// based on whether TLS is configured
struct PlainConnection { /* ... */ }
struct TlsConnection { /* ... */ }

struct ConnBuilder<Mode> {
    host: String,
    port: u16,
    _mode: PhantomData<Mode>,
}

struct Plain;
struct Tls {
    cert: String,
}

impl ConnBuilder<Plain> {
    fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            _mode: PhantomData,
        }
    }

    fn with_tls(self, cert: &str) -> ConnBuilder<Tls> {
        ConnBuilder {
            host: self.host,
            port: self.port,
            _mode: PhantomData,
        }
    }

    fn connect(self) -> PlainConnection {
        PlainConnection { /* ... */ }
    }
}

impl ConnBuilder<Tls> {
    fn connect(self) -> TlsConnection {
        TlsConnection { /* ... */ }
    }
}
```

The `connect()` method returns `PlainConnection` or `TlsConnection` depending on whether `.with_tls()` was called. The return type changes at compile time based on the builder's state. This pattern appears in real crates like `hyper` and `reqwest`.

### Your notes
<!-- -->


---

## Summary

| Approach | When to Use | Complexity |
|----------|-------------|------------|
| `Default` + struct update | All fields have defaults, no required fields | Low |
| Simple builder with `Result` | Some required fields, runtime validation is fine | Medium |
| Typestate builder | Public API, compile-time required field enforcement | High |
| `derive_builder` macro | Many fields, straightforward validation | Low (setup) |
| Generic/typestate builder | Build method should produce different types based on config | High |

The builder pattern in Rust is not just about ergonomics -- it is about encoding invariants into the type system. Start with the simplest approach that solves your problem and graduate to typestates only when the compile-time guarantee justifies the complexity.

### Your notes
<!-- -->
