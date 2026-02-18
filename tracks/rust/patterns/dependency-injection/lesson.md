# Dependency Injection -- Rust

## Why Dependency Injection Exists

Every non-trivial service depends on other things: a database, an HTTP client, a logger, a cache. The question is how those dependencies get wired together.

The naive approach hard-codes them:

```rust
struct UserService;

impl UserService {
    fn get_user(&self, id: u64) -> String {
        // Directly constructs a PostgresPool inside the method
        let pool = PostgresPool::connect("postgres://localhost/mydb");
        pool.query_one("SELECT name FROM users WHERE id = $1", id)
    }
}
```

This is untestable (you need a running Postgres), inflexible (can't swap to MySQL), and tightly coupled (UserService knows connection string details). Dependency injection fixes this by **passing dependencies in from the outside** rather than creating them internally.

In languages with runtime reflection (Java/Spring, TypeScript/NestJS), DI often involves a runtime container that resolves dependencies by type at startup. Rust has no runtime reflection. Instead, Rust achieves DI through its type system -- generics and traits -- making dependencies explicit at compile time. This is arguably better: if it compiles, your dependency graph is valid.

### Your notes
<!-- -->


---

## How Rust Approaches DI Differently

The key insight: **Rust's trait system IS its dependency injection framework.**

| Language | DI Mechanism | Resolution Time | Type Safety |
|----------|-------------|-----------------|-------------|
| **Rust** | Generics + traits, trait objects | Compile time | Full -- mismatched deps won't compile |
| **Go** | Interface parameters, struct composition | Compile time | Full -- interface satisfaction checked at compile time |
| **TypeScript** | Constructor params, DI containers (InversifyJS, NestJS) | Runtime | Partial -- depends on decorators/types |
| **Java** | Spring/Guice annotations, runtime container | Runtime | Partial -- missing beans fail at startup |
| **Python** | Constructor params, dependency-injector lib | Runtime | None (unless using Protocol + mypy) |

Rust gives you two main approaches, and choosing between them is a genuine architectural decision:

1. **Generic parameters** (static dispatch) -- `struct Service<R: Repository>`
2. **Trait objects** (dynamic dispatch) -- `struct Service { repo: Box<dyn Repository> }`

Both achieve DI. They differ in performance, ergonomics, and flexibility.

### Your notes
<!-- -->


---

## Approach 1: Generic Parameters (Static Dispatch)

This is the most "Rusty" approach. You parameterize your service over its dependencies using trait bounds:

```rust
trait UserRepository {
    fn find_by_id(&self, id: u64) -> Option<String>;
    fn save(&self, id: u64, name: &str) -> Result<(), String>;
}

trait EmailSender {
    fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

struct UserService<R: UserRepository, E: EmailSender> {
    repo: R,
    email: E,
}

impl<R: UserRepository, E: EmailSender> UserService<R, E> {
    fn new(repo: R, email: E) -> Self {
        UserService { repo, email }
    }

    fn register(&self, id: u64, name: &str, email_addr: &str) -> Result<(), String> {
        self.repo.save(id, name)?;
        self.email.send(
            email_addr,
            "Welcome!",
            &format!("Hello {}, your account is ready.", name),
        )?;
        Ok(())
    }
}
```

**What happens at compile time:** The compiler monomorphizes `UserService` -- it generates a separate, specialized version for each concrete combination of `R` and `E`. If you have `PostgresRepo` and `SmtpSender`, the compiler creates `UserService<PostgresRepo, SmtpSender>` with direct function calls, no vtable indirection.

**Implications:**

| Aspect | Generic DI |
|--------|-----------|
| Performance | Zero-cost -- inlined like hand-written code |
| Binary size | Larger -- one copy per type combination |
| Flexibility | Fixed at compile time -- can't swap at runtime |
| Ergonomics | Type signatures get verbose with many deps |
| Testability | Excellent -- plug in mock types |

If you know your dependency graph at compile time (which is most applications), generics are the default choice. This is the equivalent of Go's interface-based DI, but enforced by the compiler rather than by convention.

### Your notes
<!-- -->


---

## Approach 2: Trait Objects (Dynamic Dispatch)

When you need runtime flexibility -- plugin systems, configuration-driven backends, or just simpler type signatures -- use trait objects:

```rust
struct UserService {
    repo: Box<dyn UserRepository>,
    email: Box<dyn EmailSender>,
}

impl UserService {
    fn new(repo: Box<dyn UserRepository>, email: Box<dyn EmailSender>) -> Self {
        UserService { repo, email }
    }

    fn register(&self, id: u64, name: &str, email_addr: &str) -> Result<(), String> {
        self.repo.save(id, name)?;
        self.email.send(
            email_addr,
            "Welcome!",
            &format!("Hello {}, your account is ready.", name),
        )?;
        Ok(())
    }
}
```

Now `UserService` is a concrete type -- no generics. Dependencies are resolved at runtime via vtable dispatch (like calling methods through an interface pointer in Go or a virtual method in C++).

**The tradeoff:**

| Aspect | Trait Object DI |
|--------|----------------|
| Performance | Small overhead (~1ns per call via vtable) |
| Binary size | Smaller -- one version of the service |
| Flexibility | Can swap dependencies at runtime |
| Ergonomics | Cleaner type signatures |
| Testability | Good -- Box::new(MockRepo) |

The performance cost is almost always negligible. If your service makes a database call that takes 2ms, a 1ns vtable lookup is noise. Use trait objects when they make the code clearer.

> **Coming from Go:** `Box<dyn Repository>` is similar to how Go interfaces work -- you pass concrete types that satisfy the interface, and dispatch happens through an interface table. The difference is that Rust makes you explicitly opt in to dynamic dispatch, whereas Go defaults to it.

> **Coming from TypeScript:** This is like accepting an interface type in a constructor parameter. The TS equivalent would be `constructor(private repo: UserRepository)` where `UserRepository` is an interface. The runtime behavior is similar -- method calls go through a lookup table.

### Your notes
<!-- -->


---

## Constructor Injection with `new()`

Rust doesn't have constructors in the OOP sense. The convention is a `new()` associated function that takes dependencies as parameters. This is constructor injection:

```rust
// All dependencies explicit in the constructor
impl<R: UserRepository, E: EmailSender> UserService<R, E> {
    fn new(repo: R, email: E) -> Self {
        UserService { repo, email }
    }
}

// Usage -- the "composition root"
fn main() {
    let repo = PostgresRepo::new("postgres://localhost/mydb");
    let email = SmtpSender::new("smtp://localhost:587");
    let service = UserService::new(repo, email);

    // service is fully wired and ready to use
}
```

This is the Rust equivalent of a composition root. All dependency wiring happens in one place (typically `main()` or an application builder), and the rest of the code just uses the injected dependencies.

**Builder pattern for many dependencies:**

When a service has 4+ dependencies, a builder pattern keeps construction readable:

```rust
struct AppService<R, E, C, L>
where
    R: UserRepository,
    E: EmailSender,
    C: CacheStore,
    L: Logger,
{
    repo: R,
    email: E,
    cache: C,
    logger: L,
}

// Builder avoids positional argument confusion
struct AppServiceBuilder<R, E, C, L> {
    repo: Option<R>,
    email: Option<E>,
    cache: Option<C>,
    logger: Option<L>,
}

impl<R: UserRepository, E: EmailSender, C: CacheStore, L: Logger>
    AppServiceBuilder<R, E, C, L>
{
    fn new() -> Self {
        AppServiceBuilder {
            repo: None,
            email: None,
            cache: None,
            logger: None,
        }
    }

    fn repo(mut self, repo: R) -> Self {
        self.repo = Some(repo);
        self
    }

    fn email(mut self, email: E) -> Self {
        self.email = Some(email);
        self
    }

    // ... similar for cache, logger

    fn build(self) -> Result<AppService<R, E, C, L>, String> {
        Ok(AppService {
            repo: self.repo.ok_or("repo is required")?,
            email: self.email.ok_or("email is required")?,
            cache: self.cache.ok_or("cache is required")?,
            logger: self.logger.ok_or("logger is required")?,
        })
    }
}
```

### Your notes
<!-- -->


---

## The `impl Trait` Pattern

Rust's `impl Trait` syntax provides a middle ground. In argument position, it's syntactic sugar for generics:

```rust
// These are equivalent:
fn process(repo: &impl UserRepository) { /* ... */ }
fn process<R: UserRepository>(repo: &R) { /* ... */ }
```

In return position, it hides the concrete type while preserving static dispatch:

```rust
fn create_repo(url: &str) -> impl UserRepository {
    PostgresRepo::new(url)
    // Caller only knows it gets "something implementing UserRepository"
    // but the compiler knows the concrete type for optimization
}
```

This is useful for factory functions where you want to abstract the return type without paying for dynamic dispatch. The limitation: you can only return one concrete type per function (no conditional returns of different types -- that requires `Box<dyn Trait>`).

### Your notes
<!-- -->


---

## Ownership and Lifetimes in DI

This is where Rust DI diverges significantly from other languages. You must decide who owns each dependency:

| Ownership Model | Syntax | When to Use |
|----------------|--------|-------------|
| **Owned** | `Box<dyn Trait>` or generic `T` | Service owns the dependency exclusively |
| **Shared** | `Arc<dyn Trait>` | Multiple services share the same dependency |
| **Borrowed** | `&dyn Trait` or `&T` | Short-lived usage, caller retains ownership |
| **Shared + Mutable** | `Arc<Mutex<dyn Trait>>` | Multiple services, dependency has mutable state |

**Owned (`Box<dyn Trait>`)** is the default. Each service owns its dependencies. Simple, no lifetime annotations:

```rust
struct Service {
    repo: Box<dyn Repository>,
}
```

**Shared (`Arc<dyn Trait>`)** when multiple services need the same dependency (e.g., a connection pool shared across services):

```rust
use std::sync::Arc;

struct ServiceA {
    pool: Arc<dyn ConnectionPool + Send + Sync>,
}

struct ServiceB {
    pool: Arc<dyn ConnectionPool + Send + Sync>,
}

// Both services share the same pool
let pool: Arc<dyn ConnectionPool + Send + Sync> = Arc::new(PostgresPool::new());
let svc_a = ServiceA { pool: Arc::clone(&pool) };
let svc_b = ServiceB { pool: Arc::clone(&pool) };
```

Note the `Send + Sync` bounds -- required when passing trait objects across threads. This is Rust forcing you to think about thread safety, which other languages let you discover at runtime.

**Borrowed (`&dyn Trait`)** for short-lived, scoped usage. Requires lifetime annotations:

```rust
struct Handler<'a> {
    repo: &'a dyn Repository,
}

// The handler cannot outlive the repository
```

This is rarely used for long-lived services because lifetime annotations propagate through your entire type hierarchy. Use owned or shared for application services.

### Your notes
<!-- -->


---

## Associated Types for Complex Dependency Graphs

When trait methods return types that depend on the implementation, use associated types:

```rust
trait Repository {
    type Error: std::fmt::Display;
    type Connection;

    fn connect(&self) -> Result<Self::Connection, Self::Error>;
    fn find_by_id(&self, conn: &Self::Connection, id: u64) -> Result<Option<String>, Self::Error>;
}

struct PostgresRepo {
    url: String,
}

struct PgConnection {
    // Simulated connection
    connected: bool,
}

struct PgError(String);

impl std::fmt::Display for PgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PgError: {}", self.0)
    }
}

impl Repository for PostgresRepo {
    type Error = PgError;
    type Connection = PgConnection;

    fn connect(&self) -> Result<PgConnection, PgError> {
        Ok(PgConnection { connected: true })
    }

    fn find_by_id(&self, conn: &PgConnection, id: u64) -> Result<Option<String>, PgError> {
        if !conn.connected {
            return Err(PgError("not connected".into()));
        }
        Ok(Some(format!("user_{}", id)))
    }
}
```

Associated types make the trait more precise: each implementation specifies its own error and connection types. The service using this repository gets type-safe access to the specific error type.

This is fundamentally different from Go, where `error` is a single interface, and TypeScript, where you'd use a generic parameter or union type. Rust's associated types lock the implementation to specific concrete types while keeping the trait abstract.

### Your notes
<!-- -->


---

## Testing with DI

This is the primary reason DI exists. With injected dependencies, you can test business logic in isolation:

```rust
// Production implementation
struct PostgresRepo;
impl UserRepository for PostgresRepo {
    fn find_by_id(&self, id: u64) -> Option<String> {
        // Real database query
        todo!()
    }
    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        todo!()
    }
}

// Test mock -- no database needed
struct MockRepo {
    users: std::collections::HashMap<u64, String>,
    save_called_with: std::cell::RefCell<Vec<(u64, String)>>,
}

impl MockRepo {
    fn new() -> Self {
        let mut users = std::collections::HashMap::new();
        users.insert(1, "Alice".to_string());
        users.insert(2, "Bob".to_string());
        MockRepo {
            users,
            save_called_with: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn assert_saved(&self, id: u64, name: &str) {
        let calls = self.save_called_with.borrow();
        assert!(
            calls.iter().any(|(i, n)| *i == id && n == name),
            "expected save({}, {:?}) to have been called",
            id, name
        );
    }
}

impl UserRepository for MockRepo {
    fn find_by_id(&self, id: u64) -> Option<String> {
        self.users.get(&id).cloned()
    }

    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        self.save_called_with.borrow_mut().push((id, name.to_string()));
        Ok(())
    }
}
```

Notice `RefCell` for interior mutability in the mock -- since `save` takes `&self` (not `&mut self`), we need `RefCell` to record calls. This is a common pattern for Rust mocks. The alternative is making trait methods take `&mut self`, but that forces all callers to hold a mutable reference, which is often impractical.

> **Coming from Go:** In Go, you'd use a struct with fields to record calls and a mutex for thread safety. The `RefCell` pattern is the single-threaded Rust equivalent. For multi-threaded tests, use `Arc<Mutex<Vec<...>>>`.

> **Coming from TypeScript:** This is like jest.fn() or sinon.spy(), but manual. Rust doesn't have a mocking runtime -- you build mock types explicitly. Crates like `mockall` can auto-generate mocks from traits if you want less boilerplate.

### Your notes
<!-- -->


---

## When to Use Generics vs Trait Objects

This is the key architectural decision in Rust DI. Here's the decision framework:

**Use generics when:**
- You know all implementations at compile time
- Performance matters (hot path, called millions of times)
- You want the compiler to monomorphize for optimization
- Your dependency graph is shallow (1-2 levels of generic params)

**Use trait objects when:**
- Implementations are selected at runtime (config file, feature flags)
- You need to store different implementations in the same collection
- Type signatures are getting unwieldy (4+ generic parameters)
- You're building a plugin system or extension points
- The extra indirection is negligible (I/O-bound code, infrequent calls)

**Hybrid approach** -- use generics internally, trait objects at the boundary:

```rust
// Internal processing -- generic for performance
fn process_batch<P: Processor>(processor: &P, items: &[Item]) -> Vec<Result> {
    items.iter().map(|item| processor.process(item)).collect()
}

// External API -- trait object for flexibility
struct Pipeline {
    processors: Vec<Box<dyn Processor>>,
}

impl Pipeline {
    fn run(&self, items: &[Item]) -> Vec<Vec<Result>> {
        self.processors.iter()
            .map(|p| process_batch(p.as_ref(), items))
            .collect()
    }
}
```

### Your notes
<!-- -->


---

## Anti-Patterns to Avoid

### 1. `Any` for Runtime DI

```rust
use std::any::Any;

struct Container {
    services: std::collections::HashMap<String, Box<dyn Any>>,
}

impl Container {
    fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.services.get(name)?.downcast_ref::<T>()
    }
}
```

This is a Java-style service locator in Rust. It compiles, but you lose all type safety -- `get()` returns `Option`, and a missing or wrong-typed service is a runtime error. You've thrown away Rust's main advantage. If you find yourself reaching for `Any`, restructure your dependency graph to use generics or trait objects instead.

### 2. Global Mutable State

```rust
use std::sync::Mutex;

// DO NOT DO THIS
static REPO: Mutex<Option<Box<dyn UserRepository + Send>>> = Mutex::new(None);

fn get_user(id: u64) -> Option<String> {
    let repo = REPO.lock().unwrap();
    repo.as_ref()?.find_by_id(id)
}
```

Global state makes testing nearly impossible (tests share state, run order matters), hides dependencies (functions appear to have no dependencies but secretly depend on global state), and introduces lock contention. Pass dependencies explicitly.

### 3. Over-constraining Trait Bounds

```rust
// Too many bounds -- most implementations won't need all of these
trait Repository: Clone + Debug + Send + Sync + 'static {
    fn find(&self, id: u64) -> Option<String>;
}
```

Only require bounds you actually use. `Send + Sync` are needed for cross-thread usage. `Clone` is rarely needed for a repository. `Debug` is nice but shouldn't be mandatory. Add bounds where they're needed, not on the trait definition itself.

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Concept | Rust | Go | TypeScript |
|---------|------|-----|------------|
| DI mechanism | Generics + traits | Interface parameters | Constructor params / DI container |
| Resolution time | Compile time | Compile time | Runtime |
| Default dispatch | Static (generics) | Dynamic (interfaces) | Dynamic (vtable/prototype chain) |
| Shared dependency | `Arc<dyn Trait>` | Pass pointer, or `sync.Pool` | Just pass the reference (GC handles it) |
| Mutable shared dep | `Arc<Mutex<dyn Trait>>` | `sync.Mutex` wrapper | No built-in -- use careful design |
| Testing | Manual mocks or `mockall` crate | Manual mocks or `gomock` | jest.mock, sinon, or manual |
| Container/framework | None standard (compile-time DI) | None standard (wire, fx) | InversifyJS, NestJS, tsyringe |

**Key Rust difference:** Ownership. In Go/TS, you don't think about who owns the dependency -- the GC handles it. In Rust, you must decide: `Box` (owned), `Arc` (shared), or `&` (borrowed). This is more work but eliminates an entire class of bugs around shared mutable state.

### Your notes
<!-- -->


---

## Practical Patterns

### Factory Functions

When construction is complex, use factory functions that return `impl Trait`:

```rust
fn create_repository(config: &Config) -> impl UserRepository {
    match config.db_type.as_str() {
        "postgres" => PostgresRepo::new(&config.db_url),
        // Can't return different types here -- all branches must return the same type
        _ => panic!("unsupported db type"),
    }
}

// For truly dynamic selection, return Box<dyn Trait>:
fn create_repository_dynamic(config: &Config) -> Box<dyn UserRepository> {
    match config.db_type.as_str() {
        "postgres" => Box::new(PostgresRepo::new(&config.db_url)),
        "sqlite" => Box::new(SqliteRepo::new(&config.db_path)),
        _ => panic!("unsupported db type"),
    }
}
```

### Composition Root

Wire everything together in one place:

```rust
fn main() {
    let config = Config::from_env();

    // Create dependencies
    let repo = PostgresRepo::new(&config.db_url);
    let email = SmtpSender::new(&config.smtp_url);
    let cache = RedisCache::new(&config.redis_url);

    // Wire services
    let user_service = UserService::new(repo, email);
    let order_service = OrderService::new(cache);

    // Start application with fully wired services
    start_server(user_service, order_service);
}
```

This is identical in spirit to Go's `main()` composition pattern and similar to the "module" setup in NestJS/InversifyJS -- but with zero runtime overhead and compile-time verification.

### Your notes
<!-- -->


---

## Summary

| Decision | Default Choice | Alternative |
|----------|---------------|-------------|
| DI approach | Generics (static dispatch) | Trait objects (dynamic dispatch) |
| Ownership | `Box<dyn Trait>` (owned) | `Arc<dyn Trait>` (shared) |
| Thread safety | Add `Send + Sync` when needed | Single-threaded: no bounds needed |
| Mocking | Manual mock structs | `mockall` crate for auto-generation |
| Composition | Constructor (`new()`) injection | Builder pattern for many deps |

The big takeaway: Rust doesn't need a DI framework because the type system IS the DI framework. Traits define contracts, generics or trait objects wire implementations, and the compiler verifies the graph. No runtime surprises.

### Your notes
<!-- -->
