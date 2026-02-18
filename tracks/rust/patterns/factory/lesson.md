# Factory Pattern -- Rust

## Why Rust Changes the Factory Conversation

In languages with class hierarchies -- Java, C#, TypeScript -- the factory pattern is about choosing which subclass to instantiate. You have a base class or interface, N implementations, and a function that picks the right one based on some input. The factory hides the `new ConcreteClass()` behind a method that returns the base type.

Rust has no classes. No inheritance. No `new` keyword. So the factory pattern must be rethought from first principles.

What Rust *does* have:
- **Traits** as the abstraction boundary (like Go interfaces, but declared explicitly)
- **Trait objects** (`Box<dyn Trait>`) for dynamic dispatch -- choose implementation at runtime
- **`impl Trait`** in return position for static dispatch -- compiler monomorphizes, zero overhead
- **Enums** as closed dispatch -- all variants known at compile time, exhaustive matching
- **`From`/`Into`** traits as the standard library's built-in factory protocol
- **Builder pattern** as the idiomatic way to handle complex construction

Each of these is a different kind of "factory," and the right choice depends on whether your set of implementations is open or closed, whether you need runtime flexibility, and how much you care about allocation.

| Mechanism | Dispatch | Allocation | Open/Closed | Use When |
|-----------|----------|------------|-------------|----------|
| `Box<dyn Trait>` | Dynamic (vtable) | Heap | Open | Plugin systems, runtime config, user-extensible |
| `impl Trait` return | Static (monomorphized) | Stack | Closed at call site | Factory knows the concrete type, caller doesn't need to |
| Enum dispatch | Static (match) | Stack | Closed | All variants known, exhaustive handling needed |
| `From`/`Into` | Static | Varies | Open (via impls) | Type conversions, natural transformations |
| Builder | Static | Stack until `.build()` | N/A | Complex construction with many optional fields |

### Your notes
<!-- -->


---

## Constructor Conventions in Rust

Before diving into factories, understand Rust's constructor conventions. There's no special constructor syntax -- just associated functions that return `Self`.

```rust
struct ConnectionPool {
    host: String,
    port: u16,
    max_connections: usize,
    timeout_ms: u64,
}

impl ConnectionPool {
    // The standard "constructor" -- by convention, called `new`
    fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            max_connections: 10,     // sensible defaults
            timeout_ms: 5000,
        }
    }

    // Named constructor for a specific configuration
    fn with_config(host: &str, port: u16, max_conn: usize, timeout_ms: u64) -> Self {
        Self {
            host: host.to_string(),
            port,
            max_connections: max_conn,
            timeout_ms,
        }
    }

    // Factory for common environments
    fn for_development() -> Self {
        Self::new("localhost", 5432)
    }

    fn for_production(host: &str) -> Self {
        Self::with_config(host, 5432, 100, 30_000)
    }
}
```

**Comparison with Go and TypeScript:**

| Language | Convention | Example |
|----------|-----------|---------|
| Rust | `Type::new()`, `Type::with_x()` | `ConnectionPool::new("localhost", 5432)` |
| Go | `NewType()`, `NewTypeWithX()` | `NewConnectionPool("localhost", 5432)` |
| TypeScript | `new Type()`, static methods | `new ConnectionPool("localhost", 5432)` |

In Go, constructors are package-level functions (`NewThing`). In Rust, they're associated functions on the type itself (`Thing::new`). This is a small difference that matters for discoverability -- in Rust, you always look at the type's `impl` block.

### Your notes
<!-- -->


---

## Factory Functions with `Box<dyn Trait>` -- Dynamic Dispatch

This is the closest to the classic factory pattern. A function takes some discriminant (a string, enum, config struct) and returns a trait object. The caller doesn't know -- and doesn't need to know -- the concrete type.

**When to use:** The set of implementations is open (plugins, user-extensible systems), or the concrete type isn't known until runtime (config-driven, feature flags).

```rust
use std::collections::HashMap;

trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
    fn delete(&mut self, key: &str) -> bool;
    fn name(&self) -> &str;
}

struct InMemoryStorage {
    data: HashMap<String, String>,
}

impl InMemoryStorage {
    fn new() -> Self {
        Self { data: HashMap::new() }
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
    fn name(&self) -> &str { "in-memory" }
}

struct FileStorage {
    base_path: String,
}

// ... implementation omitted for brevity -- see storage.rs

/// The factory function. Takes a config string, returns the right backend.
fn create_storage(backend: &str) -> Result<Box<dyn Storage>, String> {
    match backend {
        "memory" => Ok(Box::new(InMemoryStorage::new())),
        "file" => Ok(Box::new(FileStorage { base_path: "/tmp/store".into() })),
        other => Err(format!("unknown storage backend: {}", other)),
    }
}
```

**What happens at runtime:** `Box<dyn Storage>` is a fat pointer -- 2 words. One points to the data on the heap, the other points to the vtable (a table of function pointers for the trait methods). Every method call goes through the vtable -- an indirect jump. The cost is real but small (typically 1-2ns overhead per call).

**Object safety matters.** A trait can only be used as `dyn Trait` if it's object-safe. The main restrictions:
- No methods with generic type parameters (the vtable can't represent infinite monomorphizations)
- No methods that return `Self` by value (the vtable doesn't know the size)
- No methods with `where Self: Sized` bounds

If your factory trait has `fn serialize<T: serde::Serialize>(&self, value: &T)`, it's not object-safe. This is one of the most common "the compiler won't let me" moments with factories in Rust. We'll see how to work around it in the exercises.

### Your notes
<!-- -->


---

## Factory Functions with `impl Trait` -- Static Dispatch

When the factory function *knows* which concrete type it will return for a given input, but you don't want to expose that type to the caller, use `impl Trait` in return position.

```rust
trait Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8>;
    fn name(&self) -> &str;
}

struct Sha256Hasher;
struct Blake3Hasher;

impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        // placeholder: real impl would use sha2 crate
        let mut result = vec![0u8; 32];
        for (i, &b) in data.iter().enumerate() {
            result[i % 32] ^= b;
        }
        result
    }
    fn name(&self) -> &str { "sha256" }
}

impl Hasher for Blake3Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; 32];
        for (i, &b) in data.iter().enumerate() {
            result[i % 32] = result[i % 32].wrapping_add(b);
        }
        result
    }
    fn name(&self) -> &str { "blake3" }
}

// This does NOT compile -- each arm returns a different concrete type:
//   fn create_hasher(algo: &str) -> impl Hasher {
//       match algo {
//           "sha256" => Sha256Hasher,
//           "blake3" => Blake3Hasher,
//           _ => panic!(),
//       }
//   }
//
// `impl Trait` means "one specific type that the compiler will figure out."
// It does NOT mean "any type implementing the trait."

// This compiles -- each function returns exactly one type:
fn sha256_hasher() -> impl Hasher {
    Sha256Hasher
}

fn blake3_hasher() -> impl Hasher {
    Blake3Hasher
}
```

**Key insight:** `impl Trait` in return position is *not* dynamic dispatch. The compiler knows the exact type. It's sugar for "I'm returning a concrete type, but I'm not telling you which one -- just that it implements this trait." This means:

- No heap allocation (the value can live on the stack)
- No vtable indirection (direct function calls)
- But you can only return ONE concrete type per function

This is useful for creating wrappers, adapters, or iterators where the concrete type is an implementation detail. It's a factory in the sense that it hides the concrete type, but it's not a *runtime-chosen* factory.

**Comparison:** In Go, `func NewHasher() Hasher` always returns an interface (dynamic dispatch). There's no Go equivalent of `impl Trait` -- Go interfaces are always dynamically dispatched. TypeScript's return types can be narrowed with generics or overloads, but the runtime behavior is always the same.

### Your notes
<!-- -->


---

## Enum Dispatch -- The Rust-Idiomatic Factory

When all variants are known at compile time, enum dispatch is usually the best factory pattern in Rust. It avoids heap allocation, avoids vtable overhead, and gives you exhaustive matching.

```rust
enum Serializer {
    Json { pretty: bool },
    Csv { delimiter: char, has_header: bool },
    MessagePack,
}

impl Serializer {
    fn from_format(format: &str) -> Result<Self, String> {
        match format {
            "json" => Ok(Serializer::Json { pretty: false }),
            "json-pretty" => Ok(Serializer::Json { pretty: true }),
            "csv" => Ok(Serializer::Csv { delimiter: ',', has_header: true }),
            "tsv" => Ok(Serializer::Csv { delimiter: '\t', has_header: true }),
            "msgpack" => Ok(Serializer::MessagePack),
            other => Err(format!("unsupported format: {}", other)),
        }
    }

    fn serialize(&self, data: &[(&str, &str)]) -> Vec<u8> {
        match self {
            Serializer::Json { pretty } => {
                let mut out = String::from("{");
                for (i, (k, v)) in data.iter().enumerate() {
                    if i > 0 { out.push_str(","); }
                    if *pretty { out.push_str("\n  "); }
                    out.push_str(&format!(r#""{}":"{}""#, k, v));
                }
                if *pretty { out.push('\n'); }
                out.push('}');
                out.into_bytes()
            }
            Serializer::Csv { delimiter, has_header } => {
                let mut out = String::new();
                if *has_header {
                    out.push_str(&data.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(&delimiter.to_string()));
                    out.push('\n');
                }
                out.push_str(&data.iter().map(|(_, v)| *v).collect::<Vec<_>>().join(&delimiter.to_string()));
                out.into_bytes()
            }
            Serializer::MessagePack => {
                // Simplified: real impl would use rmp-serde
                let count = data.len() as u8;
                let mut bytes = vec![0x80 | count]; // fixmap marker
                for (k, v) in data {
                    bytes.push(k.len() as u8);
                    bytes.extend_from_slice(k.as_bytes());
                    bytes.push(v.len() as u8);
                    bytes.extend_from_slice(v.as_bytes());
                }
                bytes
            }
        }
    }

    fn content_type(&self) -> &str {
        match self {
            Serializer::Json { .. } => "application/json",
            Serializer::Csv { .. } => "text/csv",
            Serializer::MessagePack => "application/x-msgpack",
        }
    }
}
```

**Trade-off: enum vs trait object.**

| | Enum dispatch | Trait object dispatch |
|---|---|---|
| **Variant set** | Closed (all known at compile time) | Open (new types can be added) |
| **Adding a variant** | Must modify the enum + all match arms | Just impl the trait for a new type |
| **Memory** | Stack, size of largest variant | Heap allocation per instance |
| **Dispatch cost** | Branch prediction (usually cheap) | Vtable lookup (1-2ns overhead) |
| **Exhaustiveness** | Compiler checks all match arms | No compile-time exhaustiveness |
| **When to use** | Known, stable set of variants | Plugin systems, user-extensible code |

**The `strum` or manual `FromStr` pattern** is common for enum factories:

```rust
impl std::str::FromStr for Serializer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_format(s)
    }
}

// Now you can do:
// let s: Serializer = "json".parse().unwrap();
```

### Your notes
<!-- -->


---

## `From`/`Into` -- The Standard Library's Factory Protocol

Rust's `From` and `Into` traits are the language's built-in convention for type conversions that can't fail, and `TryFrom`/`TryInto` for conversions that can. These are factory patterns hiding in plain sight.

```rust
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
    max_connections: u32,
}

// From a connection string
impl From<&str> for DatabaseConfig {
    fn from(conn_str: &str) -> Self {
        // Simplified parsing: "host:port/database"
        let parts: Vec<&str> = conn_str.splitn(2, '/').collect();
        let (host_port, database) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (parts[0], "default")
        };

        let hp: Vec<&str> = host_port.splitn(2, ':').collect();
        let host = hp[0].to_string();
        let port = hp.get(1).and_then(|p| p.parse().ok()).unwrap_or(5432);

        Self {
            host,
            port,
            database: database.to_string(),
            max_connections: 10,
        }
    }
}

// From a tuple (common in tests)
impl From<(&str, u16, &str)> for DatabaseConfig {
    fn from((host, port, db): (&str, u16, &str)) -> Self {
        Self {
            host: host.to_string(),
            port,
            database: db.to_string(),
            max_connections: 10,
        }
    }
}

// Usage:
// let cfg = DatabaseConfig::from("localhost:5432/myapp");
// let cfg: DatabaseConfig = ("localhost", 5432, "myapp").into();
```

**`TryFrom` for fallible construction:**

```rust
use std::convert::TryFrom;
use std::num::ParseIntError;

#[derive(Debug)]
struct Port(u16);

impl TryFrom<&str> for Port {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let n: u16 = s.parse().map_err(|e: ParseIntError| e.to_string())?;
        if n == 0 {
            return Err("port 0 is reserved".to_string());
        }
        Ok(Port(n))
    }
}
```

**Why this matters:** `From`/`Into` integrate with the `?` operator, `.into()` chaining, and generic bounds like `fn connect(config: impl Into<DatabaseConfig>)`. This means your factory works seamlessly with Rust's error handling and type inference ergonomics.

**Comparison:**
- Go has no equivalent -- you write explicit conversion functions
- TypeScript can approximate with constructor overloads or static factory methods
- Python uses `__init__` with flexible arguments or `@classmethod` factories
- Scala uses implicit conversions (similar spirit, different mechanism)

### Your notes
<!-- -->


---

## Factory Traits with Associated Types

When you need a factory that creates instances of a *related* type -- not just any type, but a type that's specific to the factory -- associated types are the tool.

```rust
trait ConnectionFactory {
    type Connection;
    type Config;
    type Error;

    fn create(&self, config: &Self::Config) -> Result<Self::Connection, Self::Error>;
    fn validate_config(&self, config: &Self::Config) -> Result<(), Self::Error>;
}

// Postgres implementation
struct PostgresFactory;

struct PgConnection {
    host: String,
    connected: bool,
}

struct PgConfig {
    host: String,
    port: u16,
    ssl: bool,
}

impl ConnectionFactory for PostgresFactory {
    type Connection = PgConnection;
    type Config = PgConfig;
    type Error = String;

    fn create(&self, config: &PgConfig) -> Result<PgConnection, String> {
        if config.host.is_empty() {
            return Err("host is required".into());
        }
        Ok(PgConnection {
            host: format!("{}:{}", config.host, config.port),
            connected: true,
        })
    }

    fn validate_config(&self, config: &PgConfig) -> Result<(), String> {
        if config.port == 0 {
            return Err("port must be non-zero".into());
        }
        Ok(())
    }
}
```

**Associated types vs generics on the trait:**

```rust
// Associated type: ONE connection type per factory
trait ConnectionFactory {
    type Connection;
    fn create(&self) -> Self::Connection;
}

// Generic parameter: factory could produce MANY connection types
trait GenericFactory<C> {
    fn create(&self) -> C;
}
```

Use associated types when each factory implementation produces exactly one kind of connection. Use generics when the same factory could produce different types.

**This is the Abstract Factory pattern** in GoF terms. Rust's associated types make it more ergonomic than languages where you need separate generic parameters or nested generics.

### Your notes
<!-- -->


---

## The Builder Pattern -- Rust's Preferred Complex Construction

When a type has many optional configuration fields, Rust idiom uses the builder pattern rather than telescoping constructors or option structs. This is a specialized factory for a single type.

```rust
struct HttpClient {
    base_url: String,
    timeout_ms: u64,
    max_retries: u32,
    headers: Vec<(String, String)>,
    follow_redirects: bool,
}

struct HttpClientBuilder {
    base_url: String,
    timeout_ms: u64,
    max_retries: u32,
    headers: Vec<(String, String)>,
    follow_redirects: bool,
}

impl HttpClientBuilder {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            timeout_ms: 30_000,
            max_retries: 3,
            headers: Vec::new(),
            follow_redirects: true,
        }
    }

    fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    fn build(self) -> Result<HttpClient, String> {
        if self.base_url.is_empty() {
            return Err("base_url is required".into());
        }
        Ok(HttpClient {
            base_url: self.base_url,
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            headers: self.headers,
            follow_redirects: self.follow_redirects,
        })
    }
}

// Usage:
// let client = HttpClientBuilder::new("https://api.example.com")
//     .timeout_ms(5000)
//     .max_retries(5)
//     .header("Authorization", "Bearer token123")
//     .build()?;
```

**Why `mut self` not `&mut self`:** Taking ownership (`mut self`) means each method returns the builder, enabling chaining. It also means you can't accidentally reuse a builder after calling `.build()`. Some builders use `&mut self` and return `&mut Self` instead -- this allows reuse but requires a separate `.build()` that clones. The `reqwest` crate uses `&mut self`; the `derive_builder` crate generates `mut self`.

**Standard library examples:**
- `std::process::Command` -- builder for spawning processes
- `std::thread::Builder` -- builder for thread configuration
- `std::fs::OpenOptions` -- builder for file open configuration

### Your notes
<!-- -->


---

## Registration and Plugin Patterns

For truly open systems where new implementations are added without modifying the factory, Rust uses a registry pattern.

```rust
use std::collections::HashMap;

type StorageConstructor = Box<dyn Fn(&str) -> Box<dyn Storage>>;

struct StorageRegistry {
    constructors: HashMap<String, StorageConstructor>,
}

impl StorageRegistry {
    fn new() -> Self {
        Self { constructors: HashMap::new() }
    }

    fn register<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn(&str) -> Box<dyn Storage> + 'static,
    {
        self.constructors.insert(name.to_string(), Box::new(constructor));
    }

    fn create(&self, name: &str, config: &str) -> Result<Box<dyn Storage>, String> {
        self.constructors
            .get(name)
            .map(|ctor| ctor(config))
            .ok_or_else(|| format!("no storage registered for: {}", name))
    }
}

// Usage:
// let mut registry = StorageRegistry::new();
// registry.register("memory", |_cfg| Box::new(InMemoryStorage::new()));
// registry.register("file", |cfg| Box::new(FileStorage::new(cfg)));
//
// let store = registry.create("memory", "")?;
```

For compile-time registration (like Go's `init()` functions), the `inventory` crate provides a mechanism where implementations register themselves via a macro, and the registry is assembled at link time. This is advanced and rarely needed, but it exists.

### Your notes
<!-- -->


---

## Decision Guide: Which Factory Mechanism?

```
Do you know all implementations at compile time?
├── YES: Are there more than 3-4 variants?
│   ├── YES: Enum dispatch (or consider if you need a trait at all)
│   └── NO: Enum dispatch
├── NO: Is the factory for a single type with many config options?
│   ├── YES: Builder pattern
│   └── NO: Will the factory be used across thread boundaries?
│       ├── YES: Box<dyn Trait + Send + Sync>
│       └── NO: Box<dyn Trait>
└── MAYBE: Start with enum dispatch, refactor to trait objects when needed
```

**Real-world standard library examples:**

| Factory | Mechanism | Why |
|---------|-----------|-----|
| `File::open(path)` | Associated function returning `Result<File>` | Single type, simple construction |
| `TcpStream::connect(addr)` | Associated function with `ToSocketAddrs` bound | `impl Trait` parameter as factory input |
| `Vec::from(slice)` | `From` trait | Natural conversion |
| `Command::new(program)` | Builder pattern | Many optional args |
| `io::BufReader::new(reader)` | Wrapper factory (decorator) | Adds buffering to any `Read` |

### Your notes
<!-- -->


---

## Common Pitfalls

1. **Object safety violation:** You design a trait with a generic method, then try to use `Box<dyn Trait>`. The compiler refuses. Fix: erase the generic with a trait object parameter, or use enum dispatch instead.

2. **Lifetime issues:** Your factory returns `&dyn Trait` referencing local data. The reference outlives the data. Fix: return `Box<dyn Trait>` (owned) instead of borrowing.

3. **Missing `Send + Sync`:** Your factory returns `Box<dyn Trait>` and you try to send it to another thread. Fix: `Box<dyn Trait + Send + Sync>`.

4. **Infinite `From` recursion:** `impl From<A> for B` calls `B::from(a)` which calls `A::into()` which calls `B::from(a)`. This stack overflows. Fix: implement the conversion directly without delegating to the reciprocal trait.

5. **Over-abstracting:** Creating a `Box<dyn Trait>` factory when you have 3 known variants. An enum is simpler, faster, and gives exhaustive matching. Don't reach for dynamic dispatch by default.

> **See also:**
> - [[fundamentals/interfaces-and-traits]] -- Trait objects and dynamic dispatch
> - [[fundamentals/generics]] -- Static dispatch and monomorphization
> - [[patterns/strategy]] -- Strategy uses factory-like creation
> - [[patterns/builder]] -- Builder as specialized factory

### Your notes
<!-- -->
