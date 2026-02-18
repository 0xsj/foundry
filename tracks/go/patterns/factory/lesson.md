# Factory Pattern -- Go

## The Problem Factory Solves

You need to create objects, but the calling code shouldn't know (or care) which concrete type it gets back. The creation logic depends on configuration, runtime conditions, environment variables, or user input -- and it's complex enough that scattering `if config.Type == "postgres" { ... }` across your codebase will become a maintenance nightmare.

The Factory pattern centralizes object creation behind a function or interface, so callers work with abstractions while the factory handles the messy details of picking and configuring the right implementation.

This is not academic. If you've used `sql.Open("postgres", connStr)` in Go, you've used a factory. The `sql` package doesn't know anything about PostgreSQL -- a driver registered itself at init time, and `sql.Open` looks it up by name. That's the factory pattern in the standard library.

### Why this matters more in Go than you'd expect

Coming from TypeScript, you might think factories are over-engineering. In JS/TS, you can just return an object literal that satisfies a shape -- no classes, no constructors, no ceremony:

```typescript
// TypeScript: just return the right shape
function createStorage(type: string): Storage {
  if (type === "memory") return { get: (k) => cache[k], set: (k, v) => cache[k] = v };
  if (type === "redis") return new RedisStorage(config);
  throw new Error(`unknown storage: ${type}`);
}
```

Go doesn't have classes, constructors, or object literals that satisfy interfaces. Instead, Go has **constructor functions** (`NewXxx`), **interface-based polymorphism**, and a **registration pattern** using `init()` that is uniquely powerful. The factory pattern in Go is idiomatic, lightweight, and appears everywhere once you know what to look for.

### Where Factory shows up in production

- **Storage backends**: Create the right storage (S3, GCS, local filesystem) from config
- **Database connections**: `sql.Open` returns `*sql.DB` regardless of the underlying driver
- **HTTP clients**: Create pre-configured clients with different auth, retry, and timeout policies
- **Message brokers**: Connect to Kafka, RabbitMQ, or NATS based on environment
- **Notification dispatchers**: Create email, SMS, push, or webhook senders from channel config
- **Serialization**: Create JSON, protobuf, or msgpack encoders based on content type
- **Logger backends**: Create structured loggers for stdout, files, or remote services

### Your notes
<!-- User adds insights here during learning -->


---

## Simple Factory: Go's Constructor Functions

### The `NewXxx` Convention

Go doesn't have constructors, but it has a universal convention: functions named `NewXxx` that return a configured instance. This is the simplest form of factory -- a function that encapsulates creation logic.

```go
// Simple constructor -- not really a "pattern", just good Go
func NewPostgresStore(connStr string) (*PostgresStore, error) {
    db, err := sql.Open("postgres", connStr)
    if err != nil {
        return nil, fmt.Errorf("connecting to postgres: %w", err)
    }
    return &PostgresStore{db: db}, nil
}
```

This becomes a factory when the return type is an **interface** instead of a concrete type, and the function decides which implementation to create:

```go
// Factory function -- returns interface, hides concrete type
func NewStore(cfg StoreConfig) (Store, error) {
    switch cfg.Type {
    case "postgres":
        return newPostgresStore(cfg.ConnectionString)
    case "sqlite":
        return newSQLiteStore(cfg.FilePath)
    case "memory":
        return newMemoryStore()
    default:
        return nil, fmt.Errorf("unknown store type: %q", cfg.Type)
    }
}
```

The caller gets a `Store` interface and never imports the postgres, sqlite, or memory packages. That's the decoupling.

### When a simple switch is enough

Not every factory needs a registry pattern or abstract factory. If you have 3-5 well-known implementations that change infrequently, a switch statement in a `NewXxx` function is perfectly fine. Over-engineering creation logic is a real risk -- the goal is to make the right tradeoff for your codebase's actual complexity.

**Use a simple switch factory when:**
- The set of implementations is small and stable
- All implementations are in the same module or closely related modules
- You're the only consumer (no third-party plugins)

**Graduate to a registry when:**
- Third parties need to add implementations (database drivers, plugins)
- Implementations are in separate modules with separate build dependencies
- You want to add new implementations without touching the factory code

### Your notes
<!-- -->


---

## Factory Method: Interface-Based Creation

### The Core Idea

Factory Method is when you define an **interface for creating objects**, not just an interface for the objects themselves. This is a level of indirection above the simple factory: instead of one function with a switch, you have multiple factory implementations that each know how to create one family of objects.

Why bother? Because sometimes the *creation process itself* varies significantly between implementations. Creating a PostgreSQL connection pool involves TCP connections, TLS negotiation, and connection health checks. Creating an SQLite connection means opening a file and setting pragmas. These aren't just different constructors -- they're fundamentally different initialization sequences.

```go
// The product interface -- what gets created
type Cache interface {
    Get(ctx context.Context, key string) ([]byte, error)
    Set(ctx context.Context, key string, value []byte, ttl time.Duration) error
    Delete(ctx context.Context, key string) error
}

// The factory interface -- how things get created
type CacheFactory interface {
    Create(cfg CacheConfig) (Cache, error)
    HealthCheck(ctx context.Context) error
}
```

Now each backend implements both interfaces:

```go
// Redis factory knows how to create Redis caches
type redisCacheFactory struct {
    pool *redis.Pool
}

func (f *redisCacheFactory) Create(cfg CacheConfig) (Cache, error) {
    // Redis-specific: pick the right database, set serialization format
    return &redisCache{
        pool:   f.pool,
        prefix: cfg.KeyPrefix,
        db:     cfg.Namespace,
    }, nil
}

func (f *redisCacheFactory) HealthCheck(ctx context.Context) error {
    conn := f.pool.Get()
    defer conn.Close()
    _, err := conn.Do("PING")
    return err
}
```

```go
// Memcached factory has different creation needs entirely
type memcachedCacheFactory struct {
    client *memcache.Client
}

func (f *memcachedCacheFactory) Create(cfg CacheConfig) (Cache, error) {
    // Memcached-specific: consistent hashing, compression threshold
    return &memcachedCache{
        client:     f.client,
        prefix:     cfg.KeyPrefix,
        compress:   cfg.MaxValueSize > 1024,
    }, nil
}
```

### Comparison with TypeScript

In TypeScript, Factory Method typically uses abstract classes:

```typescript
// TypeScript Factory Method
abstract class CacheFactory {
  abstract create(cfg: CacheConfig): Cache;
  abstract healthCheck(): Promise<void>;
}

class RedisCacheFactory extends CacheFactory {
  create(cfg: CacheConfig): Cache { /* ... */ }
  async healthCheck(): Promise<void> { /* ... */ }
}
```

In Go, you skip the inheritance hierarchy. The interface is implicit -- any struct with `Create` and `HealthCheck` methods is a `CacheFactory`. No `extends`, no `implements`, no base class. This makes it trivial to add new factories in separate packages that don't even know about each other.

### Comparison with Rust

Rust achieves the same thing with trait objects:

```rust
// Rust Factory Method via traits
trait CacheFactory {
    fn create(&self, cfg: &CacheConfig) -> Result<Box<dyn Cache>, Error>;
    fn health_check(&self) -> Result<(), Error>;
}

impl CacheFactory for RedisCacheFactory {
    fn create(&self, cfg: &CacheConfig) -> Result<Box<dyn Cache>, Error> { /* ... */ }
    fn health_check(&self) -> Result<(), Error> { /* ... */ }
}
```

The Rust version is more explicit about ownership (`Box<dyn Cache>` vs Go's implicit heap allocation), but the structural idea is identical.

### Your notes
<!-- -->


---

## The Registry Pattern: Go's Unique Strength

### How `init()` Enables Plugin-Style Factories

This is where Go's factory story gets genuinely interesting and differs from most other languages. The `init()` function runs automatically when a package is imported, and you can use it to register implementations with a central registry. This is how `database/sql` works, how `image` decoders work, and how many Go frameworks handle plugins.

```go
// registry.go -- central registry
package storage

import (
    "fmt"
    "sync"
)

// Factory function type
type Factory func(config map[string]string) (Store, error)

var (
    mu        sync.RWMutex
    factories = make(map[string]Factory)
)

// Register adds a factory to the registry. Called from init() in driver packages.
func Register(name string, factory Factory) {
    mu.Lock()
    defer mu.Unlock()
    if _, exists := factories[name]; exists {
        panic(fmt.Sprintf("storage: duplicate driver registration: %s", name))
    }
    factories[name] = factory
}

// New creates a Store by looking up the registered factory.
func New(driver string, config map[string]string) (Store, error) {
    mu.RLock()
    factory, exists := factories[driver]
    mu.RUnlock()
    if !exists {
        return nil, fmt.Errorf("storage: unknown driver %q (forgotten import?)", driver)
    }
    return factory(config)
}
```

```go
// s3/driver.go -- self-registering driver
package s3

import "myapp/storage"

func init() {
    storage.Register("s3", func(config map[string]string) (storage.Store, error) {
        return &S3Store{
            bucket: config["bucket"],
            region: config["region"],
        }, nil
    })
}
```

```go
// main.go -- consumer only needs a blank import
package main

import (
    "myapp/storage"
    _ "myapp/storage/s3"    // blank import triggers init() registration
    _ "myapp/storage/gcs"   // add drivers with just an import
)

func main() {
    store, err := storage.New("s3", map[string]string{
        "bucket": "my-uploads",
        "region": "us-east-1",
    })
    // store is a storage.Store -- we never imported s3 directly
}
```

### How this mirrors `database/sql`

This is exactly how Go's standard library works:

```go
import (
    "database/sql"
    _ "github.com/lib/pq"           // registers "postgres" driver
    _ "github.com/go-sql-driver/mysql" // registers "mysql" driver
)

db, err := sql.Open("postgres", connStr)  // factory lookup by name
```

The `_` import exists solely to trigger `init()`, which calls `sql.Register`. The `database/sql` package has zero knowledge of PostgreSQL or MySQL. Drivers are added by third parties without modifying the standard library. This is the open/closed principle implemented through Go's package system.

### Thread safety matters

Notice the `sync.RWMutex` in the registry. Registration typically happens during `init()` (before `main()` runs, single-threaded), but lookups happen at runtime (potentially concurrent). The mutex ensures safety if someone registers drivers lazily.

In practice, you'll see two approaches:
1. **`init()` only**: Registration happens before `main()`, no mutex needed for reads after startup
2. **Lazy registration**: Registration can happen anytime, mutex required for both reads and writes

The standard library uses approach 1 but includes the mutex for correctness.

### Your notes
<!-- -->


---

## Abstract Factory: Families of Related Objects

### When You Need More Than One Object

Abstract Factory solves a different problem than simple factory: you need to create **families of related objects** that must be compatible with each other. A database layer doesn't just need a connection -- it needs a connection, a query builder, and a migrator that all understand the same SQL dialect.

```go
// Abstract factory interface -- creates families of related objects
type DatabaseKit interface {
    CreateConnection(cfg ConnectionConfig) (Connection, error)
    CreateQueryBuilder() QueryBuilder
    CreateMigrator(conn Connection) Migrator
}

// PostgreSQL family
type postgresKit struct{}

func (k *postgresKit) CreateConnection(cfg ConnectionConfig) (Connection, error) {
    return &pgConnection{host: cfg.Host, port: cfg.Port}, nil
}
func (k *postgresKit) CreateQueryBuilder() QueryBuilder {
    return &pgQueryBuilder{quoteChar: '"'}  // PostgreSQL uses double quotes
}
func (k *postgresKit) CreateMigrator(conn Connection) Migrator {
    return &pgMigrator{conn: conn, lockTable: "schema_migrations"}
}

// SQLite family
type sqliteKit struct{}

func (k *sqliteKit) CreateConnection(cfg ConnectionConfig) (Connection, error) {
    return &sqliteConnection{path: cfg.FilePath}, nil
}
func (k *sqliteKit) CreateQueryBuilder() QueryBuilder {
    return &sqliteQueryBuilder{quoteChar: '`'}  // SQLite uses backticks
}
func (k *sqliteKit) CreateMigrator(conn Connection) Migrator {
    return &sqliteMigrator{conn: conn, useForeignKeys: true}
}
```

The key constraint: you should never mix a `pgQueryBuilder` with a `sqliteConnection`. The abstract factory ensures consistency by creating all related objects together.

### When to use Abstract Factory vs simple factory

| Situation | Use |
|-----------|-----|
| One object type, multiple implementations | Simple factory (`NewStore(config)`) |
| Multiple related objects that must match | Abstract factory (`DatabaseKit`) |
| Plugin/driver registration | Registry pattern |
| Creation needs complex configuration | Builder + factory combo |

Abstract factory is genuinely rare in Go codebases. Most Go code uses simple factories or the registry pattern. When you see an abstract factory, it's usually in database abstraction layers, UI toolkit backends (like different rendering engines), or cross-platform system abstractions.

### Your notes
<!-- -->


---

## Go-Specific Factory Idioms

### Functional Options Pattern

The functional options pattern is a factory technique unique to Go (well, any language with first-class functions, but Go popularized it). Instead of a config struct with dozens of fields, you pass option functions that modify the object being created:

```go
type Server struct {
    host         string
    port         int
    readTimeout  time.Duration
    writeTimeout time.Duration
    maxConns     int
    tls          *tls.Config
    logger       Logger
}

type Option func(*Server)

func WithPort(port int) Option {
    return func(s *Server) { s.port = port }
}

func WithTimeouts(read, write time.Duration) Option {
    return func(s *Server) {
        s.readTimeout = read
        s.writeTimeout = write
    }
}

func WithTLS(cfg *tls.Config) Option {
    return func(s *Server) { s.tls = cfg }
}

func WithLogger(l Logger) Option {
    return func(s *Server) { s.logger = l }
}

// NewServer is a factory with functional options
func NewServer(host string, opts ...Option) *Server {
    s := &Server{
        host:         host,
        port:         8080,            // sensible defaults
        readTimeout:  30 * time.Second,
        writeTimeout: 30 * time.Second,
        maxConns:     100,
        logger:       defaultLogger,
    }
    for _, opt := range opts {
        opt(s)
    }
    return s
}

// Usage
srv := NewServer("0.0.0.0",
    WithPort(9090),
    WithTimeouts(10*time.Second, 10*time.Second),
    WithTLS(tlsConfig),
)
```

This is a factory because it centralizes creation with defaults and lets callers customize without knowing the internal structure. The pattern is used extensively in Go libraries: `grpc.NewServer`, `zap.New`, `http.NewServeMux` (in newer Go APIs).

**Why this works better than a config struct in some cases:**
- Self-documenting: `WithPort(9090)` is clearer than `Config{Port: 9090}`
- Backward compatible: adding a new option doesn't change the function signature
- Validation: each option function can validate its own input
- Composability: you can define preset option bundles (`ProductionDefaults()` that returns `[]Option`)

**When a config struct is better:**
- When you have many required fields (functional options make everything optional)
- When you need to serialize/deserialize the configuration (YAML, JSON)
- When the config is passed across package boundaries frequently

### Singleton Factory with `sync.Once`

Sometimes a factory should return the same instance every time. This is the singleton pattern, and in Go, `sync.Once` is the idiomatic way to implement it:

```go
var (
    defaultClient     *http.Client
    defaultClientOnce sync.Once
)

func DefaultHTTPClient() *http.Client {
    defaultClientOnce.Do(func() {
        defaultClient = &http.Client{
            Timeout: 30 * time.Second,
            Transport: &http.Transport{
                MaxIdleConns:        100,
                MaxIdleConnsPerHost: 10,
                IdleConnTimeout:     90 * time.Second,
            },
        }
    })
    return defaultClient
}
```

This is thread-safe, lazy (only creates when first called), and guaranteed to run exactly once. You'll see this pattern in any Go package that provides a default instance: loggers, HTTP clients, connection pools, metric registries.

### Your notes
<!-- -->


---

## Standard Library Factories

Go's standard library is full of factory patterns. Knowing these helps you recognize the pattern and understand when to apply it in your own code.

### `sql.Open` -- Registry-Based Factory

```go
// Registration (in driver package)
func init() {
    sql.Register("postgres", &PostgresDriver{})
}

// Factory lookup (in application code)
db, err := sql.Open("postgres", "host=localhost dbname=myapp")
```

`sql.Open` doesn't create a connection -- it creates a `*sql.DB` which is a connection pool factory. The actual connections are created lazily by the registered driver. This is a two-level factory: the registry creates the pool, the pool creates connections.

### `http.NewRequest` -- Simple Factory

```go
req, err := http.NewRequest("GET", "https://api.example.com/users", nil)
```

Returns a concrete type (`*http.Request`), but the function centralizes request creation with validation, default headers, and URL parsing. It's a factory in the sense that it encapsulates construction complexity.

### `json.NewDecoder` / `json.NewEncoder` -- Wrapping Factory

```go
decoder := json.NewDecoder(r.Body)    // Factory wraps an io.Reader
encoder := json.NewEncoder(w)         // Factory wraps an io.Writer
```

These factories take an interface (`io.Reader`, `io.Writer`) and return a configured decoder/encoder. The pattern is: take a low-level abstraction, return a higher-level one.

### `image.Decode` -- Registry + Auto-Detection Factory

```go
import (
    "image"
    _ "image/png"   // register PNG decoder
    _ "image/jpeg"  // register JPEG decoder
)

img, format, err := image.Decode(file)  // auto-detects format from magic bytes
```

This is the registry pattern with auto-detection. Each decoder registers magic bytes (file signatures), and `image.Decode` reads the first few bytes to pick the right decoder. No switch statement, no caller decision -- the data itself determines the factory.

### Your notes
<!-- -->


---

## When NOT to Use Factory

### Over-Engineering Signals

The factory pattern has real costs: indirection makes code harder to trace, registries can hide dependencies, and abstract factories add complexity that may never pay off. Watch for these signals that you're over-engineering:

**You only have one implementation.** If `NewStore` always returns a `PostgresStore`, the factory adds indirection without value. Just use `NewPostgresStore` directly and introduce the factory when a second implementation actually appears.

**The creation logic is trivial.** If your factory is just `return &Thing{field: value}`, a factory function adds a function call for no reason. Direct initialization is fine:

```go
// Don't do this -- pointless indirection
func NewConfig(path string) *Config {
    return &Config{Path: path}
}

// Just do this
cfg := &Config{Path: "/etc/myapp/config.yaml"}
```

**You're using a factory to hide a global.** If your factory is really just `return globalInstance`, you have a singleton, not a factory. Be honest about it.

**YAGNI applies.** If you're building a factory "in case we need to swap implementations later" but there's no concrete plan for a second implementation, you're speculating. The interface can be extracted later when the need actually arises -- Go's implicit interfaces make this a low-cost refactor.

### The right amount of factory

| Scenario | Recommendation |
|----------|---------------|
| One implementation, simple init | Direct construction: `&Thing{...}` |
| One implementation, complex init | Constructor: `NewThing(config)` |
| Multiple implementations, stable set | Factory function with switch |
| Multiple implementations, extensible | Registry pattern |
| Families of related objects | Abstract factory (rare) |

### Your notes
<!-- -->


---

## Summary: Factory Variants at a Glance

| Variant | Go Idiom | When to Use | Standard Library Example |
|---------|----------|-------------|--------------------------|
| Simple factory | `NewXxx(config) (Interface, error)` | 2-5 implementations, stable | `http.NewRequest` |
| Factory method | Factory interface with `Create()` | Complex creation logic that varies | `hash.Hash` constructors |
| Abstract factory | Interface returning multiple related types | Families that must stay compatible | `database/sql` (driver creates conn + stmt + tx) |
| Registry | `Register(name, factory)` + `init()` | Plugin architecture, extensible | `sql.Register`, `image.RegisterFormat` |
| Functional options | `NewXxx(required, ...Option)` | Complex configuration with defaults | `grpc.NewServer`, `zap.New` |
| Singleton factory | `sync.Once` + package-level var | Shared expensive resources | `http.DefaultClient` |

### Your notes
<!-- -->
