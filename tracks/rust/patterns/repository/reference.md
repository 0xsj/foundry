# Rust Reference -- Repository Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/)
> for the `repository` module. Covers: trait abstractions for data access,
> associated types vs generics, object safety for repository traits,
> error handling patterns, async traits, and `Send + Sync` bounds.

---

## Trait Object Patterns for Repository Abstraction

Source: [Rust Book - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

Repositories use traits to abstract storage. The two dispatch models:

### Static Dispatch (Generics)

```rust
struct UserService<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> UserService<R> {
    fn new(repo: R) -> Self {
        Self { repo }
    }
}
```

- Monomorphization: compiler generates a specialized `UserService` for each `R`
- Zero overhead: no vtable lookup, methods can be inlined
- Constraint: `R` is fixed at compile time -- cannot swap implementations at runtime

### Dynamic Dispatch (Trait Objects)

```rust
struct UserService {
    repo: Box<dyn UserRepository>,
}

impl UserService {
    fn new(repo: Box<dyn UserRepository>) -> Self {
        Self { repo }
    }
}
```

- Fat pointer: `Box<dyn UserRepository>` is `[data_ptr, vtable_ptr]` (16 bytes on 64-bit)
- Vtable call on every method invocation (~1-3ns overhead)
- Flexible: implementation can be swapped at runtime (factory pattern, config-driven)
- Requires trait to be **object safe**

### When to Use Which

| Criterion | Static (`<R: Repo>`) | Dynamic (`Box<dyn Repo>`) |
|-----------|---------------------|--------------------------|
| Backend chosen at compile time | Preferred | Works |
| Backend chosen at runtime (config) | Cannot | Required |
| Heterogeneous collection of repos | Cannot | Required |
| Performance-critical inner loop | Preferred | Acceptable |
| Binary size concern | Larger (monomorphization) | Smaller |
| Simplicity of type signatures | More annotations | Simpler at call sites |

---

## Associated Types vs Generics on Traits

Source: [Rust Book - Associated Types](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#specifying-placeholder-types-in-trait-definitions-with-associated-types)

### Associated Types

```rust
pub trait Repository {
    type Entity: Clone;
    type Id: Eq + std::hash::Hash;
    type Error: std::fmt::Debug;

    fn find(&self, id: &Self::Id) -> Result<Option<Self::Entity>, Self::Error>;
    fn save(&mut self, entity: &Self::Entity) -> Result<(), Self::Error>;
    fn delete(&mut self, id: &Self::Id) -> Result<bool, Self::Error>;
    fn list(&self) -> Result<Vec<Self::Entity>, Self::Error>;
}
```

**Rules:**
- Each implementing type chooses the associated types **once**
- No ambiguity: `InMemoryUserRepo` has exactly one `Entity` type
- Trait is object-safe when used as `dyn Repository<Entity=User, Id=String, Error=RepoError>`
- Bounds on associated types (`Entity: Clone`) are enforced on all implementations

### Generic Type Parameters

```rust
pub trait Repository<T: Clone, Id: Eq + std::hash::Hash> {
    fn find(&self, id: &Id) -> Result<Option<T>, RepoError>;
    fn save(&mut self, entity: &T) -> Result<(), RepoError>;
    fn delete(&mut self, id: &Id) -> Result<bool, RepoError>;
}
```

**Rules:**
- A single type can implement `Repository<User, String>` AND `Repository<Order, Uuid>`
- Requires specifying type parameters at every use site
- Trait object must specify parameters: `dyn Repository<User, String>`

### Comparison Table

| Feature | Associated Types | Generic Parameters |
|---------|-----------------|-------------------|
| Implementations per type | One | Multiple |
| Syntax at use site | `R: Repository` | `R: Repository<User, String>` |
| Object safety | Yes (with specified types) | Yes (with specified types) |
| Type inference | Better (fewer annotations) | More annotations needed |
| Flexibility | Less (one entity per repo) | More (multi-entity repos) |
| Typical use | Dedicated repos | Generic stores |

---

## Object Safety for Repository Traits

Source: [Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A repository trait must be object-safe if you want to use `dyn Repository`. The key constraints:

### Object-Safe Repository

```rust
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    fn save(&mut self, user: &User) -> Result<(), RepoError>;
    fn delete(&mut self, id: &str) -> Result<bool, RepoError>;
    fn list_active(&self) -> Result<Vec<User>, RepoError>;
}

// This works:
fn make_repo() -> Box<dyn UserRepository> {
    Box::new(InMemoryUserRepo::new())
}
```

### Non-Object-Safe Patterns (and Fixes)

```rust
// PROBLEM 1: Generic method
trait BadRepo {
    fn query<F: Fn(&User) -> bool>(&self, predicate: F) -> Vec<User>;
    // Cannot create vtable: infinite possible monomorphizations of F
}

// FIX: Use trait object or concrete type for the predicate
trait FixedRepo {
    fn query(&self, predicate: &dyn Fn(&User) -> bool) -> Vec<User>;
    // Now object-safe: predicate is a trait object, not a generic
}

// PROBLEM 2: Method returns Self
trait BadRepo2 {
    fn clone_repo(&self) -> Self;
    // Size of Self unknown behind dyn pointer
}

// FIX: Return Box<dyn Trait> instead
trait FixedRepo2 {
    fn clone_repo(&self) -> Box<dyn FixedRepo2>;
}

// PROBLEM 3: Method takes Self as non-receiver parameter
trait BadRepo3 {
    fn merge(&self, other: Self);
}

// FIX: Use trait object parameter
trait FixedRepo3 {
    fn merge(&self, other: &dyn FixedRepo3);
}
```

### Excluding Methods with `where Self: Sized`

```rust
trait UserRepository {
    // Available through &dyn UserRepository
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;

    // NOT available through &dyn UserRepository, but trait is still object-safe
    fn batch_query<F: Fn(&User) -> bool>(&self, f: F) -> Vec<User>
    where
        Self: Sized;
}
```

---

## Error Handling with Enum Types

Source: [Rust Book - Recoverable Errors](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

### Standard Repository Error Enum

```rust
#[derive(Debug)]
pub enum RepoError {
    NotFound { entity: String, id: String },
    DuplicateKey { entity: String, id: String },
    ConnectionFailed { source: String },
    SerializationError { detail: String },
    InvalidInput { field: String, reason: String },
    Internal { message: String },
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(f, "{} not found: {}", entity, id),
            Self::DuplicateKey { entity, id } => write!(f, "duplicate {} key: {}", entity, id),
            Self::ConnectionFailed { source } => write!(f, "connection failed: {}", source),
            Self::SerializationError { detail } => write!(f, "serialization error: {}", detail),
            Self::InvalidInput { field, reason } => write!(f, "invalid {}: {}", field, reason),
            Self::Internal { message } => write!(f, "internal error: {}", message),
        }
    }
}

// Implement std::error::Error for interoperability with ? operator
// and Box<dyn Error>
impl std::error::Error for RepoError {}
```

### thiserror-Style Pattern (Without the Crate)

The `thiserror` crate generates `Display` and `Error` impls from attributes. Here is the equivalent manual pattern:

```rust
#[derive(Debug)]
pub enum RepoError {
    NotFound { entity: String, id: String },
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(f, "{} not found: {}", entity, id),
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::Parse(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for RepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            _ => None,
        }
    }
}

// From impls for ergonomic ? operator usage
impl From<std::io::Error> for RepoError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::num::ParseIntError> for RepoError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::Parse(e)
    }
}
```

### Error Conversion with `?` Operator

```rust
// With From<io::Error> for RepoError, this just works:
fn read_from_file(path: &str) -> Result<Vec<User>, RepoError> {
    let data = std::fs::read_to_string(path)?;  // io::Error -> RepoError::Io
    // parse data...
    Ok(vec![])
}
```

---

## Async Traits for Repositories

Source: [Rust Blog - Async fn in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)

### Native `async fn` in Traits (Rust 1.75+)

```rust
pub trait UserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    async fn save(&mut self, user: &User) -> Result<(), RepoError>;
}

// Works with generics:
async fn process<R: UserRepository>(repo: &R) {
    let user = repo.find_by_id("u1").await;
}

// Does NOT work with trait objects:
// let repo: Box<dyn UserRepository> = ...;  // ERROR: not object-safe
```

### `async_trait` Macro Desugaring

```rust
// What async_trait generates (conceptually):
pub trait UserRepository {
    fn find_by_id<'a>(
        &'a self,
        id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<User>, RepoError>> + Send + 'a>
    >;
}

// Now Box<dyn UserRepository> works -- the return type is a concrete trait object
```

### Send Bounds

```rust
// Default async_trait: futures are Send (can cross thread boundaries)
// #[async_trait]
// trait Repo: Send + Sync {
//     async fn find(&self, id: &str) -> Option<User>;
// }

// Non-Send variant (for single-threaded runtimes):
// #[async_trait(?Send)]
// trait Repo {
//     async fn find(&self, id: &str) -> Option<User>;
// }
```

| Variant | Bound | Use case |
|---------|-------|----------|
| `#[async_trait]` | `Send` on futures | Multi-threaded runtimes (tokio default) |
| `#[async_trait(?Send)]` | No `Send` bound | Single-threaded runtimes, `Rc`-based repos |

---

## Send + Sync Bounds

Source: [Rust Reference - Send and Sync](https://doc.rust-lang.org/reference/special-types-and-traits.html#send-and-sync)

### When Repositories Need Thread Safety

```rust
// Shared across async tasks with Arc
let repo: Arc<dyn UserRepository + Send + Sync> = Arc::new(InMemoryUserRepo::new());

// Passed to a spawned task
// tokio::spawn(async move {
//     let user = repo.find_by_id("u1").await;
// });
```

### Auto-Trait Implementation Rules

| Type | Send | Sync | Reason |
|------|------|------|--------|
| `HashMap<String, User>` | Yes (if User: Send) | Yes (if User: Sync) | Owning container |
| `Rc<T>` | No | No | Non-atomic reference count |
| `Arc<T>` | Yes (if T: Send + Sync) | Yes (if T: Send + Sync) | Atomic reference count |
| `RefCell<T>` | Yes (if T: Send) | No | Not safe to share |
| `Mutex<T>` | Yes (if T: Send) | Yes (if T: Send) | Provides interior mutability safely |
| `RwLock<T>` | Yes (if T: Send + Sync) | Yes (if T: Send + Sync) | Read-write locking |

### Interior Mutability for Shared Repos

```rust
use std::sync::{Arc, RwLock};

// Thread-safe shared repository
struct SharedUserRepo {
    users: RwLock<HashMap<String, User>>,
}

impl SharedUserRepo {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        let users = self.users.read().unwrap();
        Ok(users.get(id).cloned())
    }

    fn save(&self, user: &User) -> Result<(), RepoError> {
        let mut users = self.users.write().unwrap();
        users.insert(user.id.clone(), user.clone());
        Ok(())
    }
}

// Can be wrapped in Arc and shared:
// let repo = Arc::new(SharedUserRepo { users: RwLock::new(HashMap::new()) });
```

Note the trait signature change: `save` takes `&self` instead of `&mut self` because `RwLock` provides interior mutability. This is a common pattern for shared async repositories.

---

## Common Repository Trait Patterns

### CRUD Trait with Bounds

```rust
pub trait CrudRepository {
    type Entity: Clone + std::fmt::Debug;
    type Id: Clone + Eq + std::hash::Hash + std::fmt::Display;

    fn find(&self, id: &Self::Id) -> Result<Option<Self::Entity>, RepoError>;
    fn save(&mut self, entity: &Self::Entity) -> Result<(), RepoError>;
    fn delete(&mut self, id: &Self::Id) -> Result<bool, RepoError>;
    fn exists(&self, id: &Self::Id) -> Result<bool, RepoError> {
        // Default implementation
        Ok(self.find(id)?.is_some())
    }
    fn count(&self) -> Result<usize, RepoError>;
}
```

### Repository Factory

```rust
fn create_user_repo(backend: &str) -> Box<dyn UserRepository> {
    match backend {
        "memory" => Box::new(InMemoryUserRepo::new()),
        // "postgres" => Box::new(PostgresUserRepo::new(pool)),
        // "redis" => Box::new(RedisUserRepo::new(client)),
        _ => panic!("unknown backend: {}", backend),
    }
}
```

### Decorator Composition

```rust
// Type-safe composition chain through generics
struct LoggingRepo<R: UserRepository> {
    inner: R,
    prefix: String,
}

impl<R: UserRepository> UserRepository for LoggingRepo<R> {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        println!("[{}] find_by_id({})", self.prefix, id);
        let result = self.inner.find_by_id(id);
        println!("[{}] find_by_id({}) -> {:?}", self.prefix, id,
            result.as_ref().map(|r| r.is_some()));
        result
    }
    // ... delegate all other methods with logging
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        self.inner.find_by_email(email)
    }
    fn save(&mut self, user: &User) -> Result<(), RepoError> {
        self.inner.save(user)
    }
    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        self.inner.delete(id)
    }
    fn list_active(&self) -> Result<Vec<User>, RepoError> {
        self.inner.list_active()
    }
}

// Usage: let repo = LoggingRepo { inner: CachedRepo::new(InMemoryUserRepo::new()), prefix: "user-repo".into() };
```

---

## References

- [The Rust Programming Language, Ch. 10 - Generics and Traits](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [The Rust Programming Language, Ch. 17.2 - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [The Rust Reference - Traits](https://doc.rust-lang.org/reference/items/traits.html)
- [The Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
- [The Rust Reference - Send and Sync](https://doc.rust-lang.org/reference/special-types-and-traits.html#send-and-sync)
- [Rust Blog - Async fn in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html)
- [async-trait crate](https://crates.io/crates/async-trait)
- [diesel ORM](https://diesel.rs/)
- [sea-orm](https://www.sea-ql.org/SeaORM/)
- [sqlx](https://github.com/launchbadge/sqlx)
