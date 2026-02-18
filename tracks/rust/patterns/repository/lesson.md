# Repository Pattern -- Rust

## The Problem Repository Solves

Every non-trivial application talks to storage. A user service reads and writes to PostgreSQL. An audit logger appends to an event store. A feature flag system reads from Redis. A config manager reads from a YAML file on disk.

The naive approach couples your domain logic directly to your storage implementation:

```rust
use std::collections::HashMap;

fn get_user(db: &HashMap<String, User>, id: &str) -> Option<&User> {
    db.get(id)
}

fn save_user(db: &mut HashMap<String, User>, user: User) {
    db.insert(user.id.clone(), user);
}
```

This seems fine until you need to:
- **Test** your domain logic without a running database
- **Swap** from PostgreSQL to DynamoDB because your scaling requirements changed
- **Add caching** in front of your database reads without touching every call site
- **Mock** storage in integration tests to simulate failure scenarios

The Repository pattern fixes this by **putting a trait between your domain logic and your storage**. Your service depends on the trait, not on the concrete storage. Implementations can be swapped, stacked, and tested independently.

In Rust, repository traits interact deeply with the ownership system, error handling, and async runtime. This makes Rust repositories more nuanced to design than in Go or TypeScript -- but also more powerful, because the type system encodes constraints that are invisible in other languages.

### Your notes
<!-- -->


---

## Repository Trait Design: The Core Abstraction

The simplest useful repository trait looks like this:

```rust
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    fn save(&mut self, user: &User) -> Result<(), RepoError>;
    fn delete(&mut self, id: &str) -> Result<bool, RepoError>;
    fn list_all(&self) -> Result<Vec<User>, RepoError>;
}
```

This is a **concrete** repository trait -- it knows about `User`. The alternative is a **generic** repository trait that works with any entity type. Both are valid; the choice is a genuine architectural decision.

### Concrete vs Generic Repository Traits

| Approach | Definition | Pros | Cons |
|----------|-----------|------|------|
| **Concrete** | `trait UserRepository { fn find_by_id(...) -> User }` | Domain-specific methods (`find_by_email`), clear intent, easy to understand | One trait per entity type, some boilerplate |
| **Generic** | `trait Repository<T> { fn find_by_id(...) -> T }` | Reuse across entities, consistent API | Can't express entity-specific queries, forced uniformity |
| **Associated types** | `trait Repository { type Entity; fn find(...) -> Self::Entity }` | One impl per type (no ambiguity), clean ergonomics | Less flexible than generics if you need multi-entity repos |

**The pragmatic choice:** Use concrete traits for your domain repositories. They let you define methods like `find_by_email`, `list_active_users`, `count_by_role` -- things that are meaningful for *that specific entity*. A generic `Repository<T>` trait forces you into a lowest-common-denominator CRUD interface.

That said, a generic trait works well as a **base** that concrete traits extend:

```rust
/// Base operations every repository supports
pub trait Repository {
    type Entity;
    type Id;
    type Error;

    fn find_by_id(&self, id: &Self::Id) -> Result<Option<Self::Entity>, Self::Error>;
    fn save(&mut self, entity: &Self::Entity) -> Result<(), Self::Error>;
    fn delete(&mut self, id: &Self::Id) -> Result<bool, Self::Error>;
}

/// User-specific repository adds domain queries
pub trait UserRepository: Repository<Entity = User, Id = String, Error = RepoError> {
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    fn list_active(&self) -> Result<Vec<User>, RepoError>;
}
```

> **Coming from Go?** Go interfaces are implicitly satisfied, so you naturally have small interfaces (`io.Reader`, `io.Writer`) composed at use sites. In Rust, you explicitly implement traits, which encourages bigger, more purposeful interfaces. The supertrait pattern above (`UserRepository: Repository`) is Rust's version of Go's interface embedding.

> **Coming from TypeScript?** This is similar to TypeScript's `interface UserRepository extends Repository<User>`. The key difference: Rust's associated types (`type Entity`) are resolved at the impl site, not at the call site. There's exactly one `Entity` type per implementation, which prevents a whole class of type confusion bugs.

### Your notes
<!-- -->


---

## Ownership in Repository Return Types

This is where Rust repositories diverge sharply from Go and TypeScript. In Go, you return a pointer (`*User`) and the garbage collector handles lifetime. In TypeScript, you return an object and never think about it. In Rust, every return type encodes an ownership decision.

### The Three Options

```rust
// Option 1: Return owned values (clone from storage)
fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;

// Option 2: Return references (borrow from storage)
fn find_by_id(&self, id: &str) -> Result<Option<&User>, RepoError>;

// Option 3: Return smart pointers (shared ownership)
fn find_by_id(&self, id: &str) -> Result<Option<Arc<User>>, RepoError>;
```

| Approach | When | Tradeoff |
|----------|------|----------|
| **Owned** (`User`) | Default choice. Works with any backend. | Clones data on every read. Cost is negligible for most entities. |
| **Reference** (`&User`) | In-memory repos only. The borrow ties to the repo's lifetime. | Zero-cost reads, but you can't use the value after the repo is dropped or mutated. Lifetime annotations get complex. |
| **Arc** (`Arc<User>`) | When multiple parts of the system hold the same entity. | Shared ownership without cloning. Small ref-count overhead. |

**The pragmatic choice:** Return owned values. It's the simplest, it works across all backends (you can't return a reference to a database row), and the clone cost is irrelevant for 99% of domain entities. Save `&T` returns for hot-path in-memory caches where profiling shows cloning is expensive.

```rust
use std::collections::HashMap;

struct User {
    id: String,
    email: String,
    active: bool,
}

// Returning owned values -- the default and best starting point
struct InMemoryUserRepo {
    users: HashMap<String, User>,
}

impl InMemoryUserRepo {
    fn find_by_id(&self, id: &str) -> Option<User> {
        // .cloned() would require User: Clone. Instead, we rebuild:
        self.users.get(id).map(|u| User {
            id: u.id.clone(),
            email: u.email.clone(),
            active: u.active,
        })
    }
}
```

The lifetime complexity of returning references becomes apparent quickly:

```rust
// This trait is hard to use in practice:
trait RefUserRepo {
    fn find_by_id(&self, id: &str) -> Option<&User>;
    //                                          ^--- borrows from self
    //
    // Problem: while this borrow is alive, you can't call any &mut self methods.
    // This means you can't read a user and then save a modified version in the
    // same scope without careful restructuring.
}
```

### Your notes
<!-- -->


---

## Error Handling: Custom Enum vs Box\<dyn Error\>

Every repository operation can fail. The question is how to represent that failure.

### Approach 1: Custom Error Enum (Recommended)

```rust
#[derive(Debug)]
pub enum RepoError {
    NotFound { entity: String, id: String },
    DuplicateKey { entity: String, id: String },
    ConnectionFailed { source: String },
    SerializationError { detail: String },
    Internal { message: String },
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::NotFound { entity, id } => {
                write!(f, "{} not found: {}", entity, id)
            }
            RepoError::DuplicateKey { entity, id } => {
                write!(f, "duplicate {} key: {}", entity, id)
            }
            RepoError::ConnectionFailed { source } => {
                write!(f, "connection failed: {}", source)
            }
            RepoError::SerializationError { detail } => {
                write!(f, "serialization error: {}", detail)
            }
            RepoError::Internal { message } => {
                write!(f, "internal error: {}", message)
            }
        }
    }
}
```

**Why this is better:** Callers can `match` on specific error variants. "Not found" is different from "connection failed" -- the caller might retry one but not the other. With `Box<dyn Error>`, you lose that structure and resort to string matching.

### Approach 2: Box\<dyn Error\> (Quick and Dirty)

```rust
type RepoResult<T> = Result<T, Box<dyn std::error::Error>>;

trait UserRepository {
    fn find_by_id(&self, id: &str) -> RepoResult<Option<User>>;
}
```

This is fine for prototypes but becomes a maintenance problem in production. Every error is opaque -- callers must downcast or string-match to handle specific failures.

> **Coming from Go?** Go's `error` interface is similar to `Box<dyn Error>` -- both are type-erased. But Go encourages sentinel errors (`var ErrNotFound = errors.New(...)`) and `errors.Is/As` for matching. Rust's enum approach is more powerful because `match` is exhaustive -- the compiler tells you if you forgot a case.

> **Coming from TypeScript?** TypeScript typically uses class hierarchies for errors (`class NotFoundError extends Error`). Rust's enum is the same idea, but flat rather than hierarchical, and enforced at compile time through exhaustive matching.

### Your notes
<!-- -->


---

## In-Memory Implementation with HashMap

The in-memory implementation is your first repository backend. It's valuable beyond testing -- it serves as a reference implementation that makes the trait's contract crystal clear.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub active: bool,
}

pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    fn save(&mut self, user: &User) -> Result<(), RepoError>;
    fn delete(&mut self, id: &str) -> Result<bool, RepoError>;
    fn list_active(&self) -> Result<Vec<User>, RepoError>;
}

pub struct InMemoryUserRepo {
    users: HashMap<String, User>,
}

impl InMemoryUserRepo {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserRepository for InMemoryUserRepo {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        Ok(self.users.get(id).cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        // Linear scan -- fine for in-memory, a real DB would use an index
        Ok(self.users.values().find(|u| u.email == email).cloned())
    }

    fn save(&mut self, user: &User) -> Result<(), RepoError> {
        self.users.insert(user.id.clone(), user.clone());
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        Ok(self.users.remove(id).is_some())
    }

    fn list_active(&self) -> Result<Vec<User>, RepoError> {
        Ok(self.users.values().filter(|u| u.active).cloned().collect())
    }
}
```

Notice that every method returns `Result`, even though the in-memory implementation can never fail. This is by design -- the trait represents the contract for *all* backends, including ones that do I/O. An in-memory impl that returns `Ok(...)` for everything is the simplest correct implementation.

### Your notes
<!-- -->


---

## Async Repository Traits

Most real repository implementations talk to a database, an HTTP API, or a file system -- all of which are async. Rust's `async` in traits was stabilized in Rust 1.75, but there are caveats around `dyn` dispatch.

### Using `async fn` in Traits (Rust 1.75+)

```rust
pub trait UserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    async fn save(&mut self, user: &User) -> Result<(), RepoError>;
}
```

This works for **generic** usage (`fn service<R: UserRepository>(repo: R)`), but you **cannot** use `dyn UserRepository` with async methods directly. The return type of an `async fn` is an opaque `impl Future`, which is not object-safe.

### The `async_trait` Workaround

For trait objects (`Box<dyn UserRepository>`), use the `async_trait` crate:

```rust
// With async_trait crate:
// #[async_trait::async_trait]
// pub trait UserRepository {
//     async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
// }
//
// This macro desugars to:
// pub trait UserRepository {
//     fn find_by_id<'a>(&'a self, id: &'a str)
//         -> Pin<Box<dyn Future<Output = Result<Option<User>, RepoError>> + Send + 'a>>;
// }
```

The `Pin<Box<dyn Future>>` enables dynamic dispatch at the cost of a heap allocation per call. For most repository operations (which involve I/O anyway), this overhead is negligible.

### Send + Sync Bounds for Async

When a repository is used across async tasks, it needs thread-safety bounds:

```rust
// This repository can be shared across tasks (Arc<dyn UserRepository>)
pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError>;
    fn save(&mut self, user: &User) -> Result<(), RepoError>;
}
```

Without `Send + Sync`, you cannot wrap the repository in `Arc` for sharing across tokio tasks. The in-memory `HashMap` implementation is already `Send + Sync` (assuming `User` is), so this just works. But a `Rc<RefCell<...>>` implementation would fail these bounds.

> **Coming from Go?** Goroutines have no type-level thread-safety checking. If you pass a map to multiple goroutines, you get a runtime panic if you're lucky, silent corruption if you're not. Rust's `Send + Sync` bounds catch this at compile time.

> **Coming from TypeScript?** Node.js is single-threaded, so this isn't a concern. If you move to Deno or a worker-based model, similar issues arise, but TypeScript has no type-level mechanism to prevent them.

### Your notes
<!-- -->


---

## Generic Repository Trait: Associated Types vs Type Parameters

When building a repository system for multiple entity types, you choose between generics and associated types.

### Generic Type Parameters

```rust
trait Repository<T, Id> {
    fn find(&self, id: &Id) -> Result<Option<T>, RepoError>;
    fn save(&mut self, entity: &T) -> Result<(), RepoError>;
    fn delete(&mut self, id: &Id) -> Result<bool, RepoError>;
}

// A single struct can implement Repository for multiple entities:
struct InMemoryStore {
    users: HashMap<String, User>,
    orders: HashMap<String, Order>,
}

impl Repository<User, String> for InMemoryStore {
    fn find(&self, id: &String) -> Result<Option<User>, RepoError> {
        Ok(self.users.get(id).cloned())
    }
    fn save(&mut self, entity: &User) -> Result<(), RepoError> {
        self.users.insert(entity.id.clone(), entity.clone());
        Ok(())
    }
    fn delete(&mut self, id: &String) -> Result<bool, RepoError> {
        Ok(self.users.remove(id).is_some())
    }
}
```

### Associated Types

```rust
trait Repository {
    type Entity: Clone;
    type Id: Eq + std::hash::Hash;
    type Error;

    fn find(&self, id: &Self::Id) -> Result<Option<Self::Entity>, Self::Error>;
    fn save(&mut self, entity: &Self::Entity) -> Result<(), Self::Error>;
    fn delete(&mut self, id: &Self::Id) -> Result<bool, Self::Error>;
}
```

With associated types, each struct implements the trait **once** -- `InMemoryUserRepo` is a repository for `User`, period. With generics, one struct can be a repository for multiple entity types.

| Aspect | Generic `Repository<T>` | Associated Type `Repository` |
|--------|------------------------|------------------------------|
| Multiple entity types per impl | Yes | No (one impl per struct) |
| Object safety | No (`dyn Repository<T>` needs `T` specified) | Yes (`dyn Repository<Entity=User>`) |
| Ergonomics | More type annotations at call sites | Cleaner, fewer turbofish |
| Typical use | Multi-entity stores | Dedicated per-entity repos |

### Your notes
<!-- -->


---

## Testing with Mock Repositories

One of the primary reasons to use the Repository pattern is testability. Your domain logic depends on the trait, and you inject a mock implementation in tests.

```rust
// The service depends on the trait, not the implementation
struct UserService<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> UserService<R> {
    fn new(repo: R) -> Self {
        Self { repo }
    }

    fn deactivate_user(&mut self, id: &str) -> Result<(), RepoError> {
        let user = self.repo.find_by_id(id)?
            .ok_or(RepoError::NotFound {
                entity: "User".to_string(),
                id: id.to_string(),
            })?;

        let deactivated = User {
            active: false,
            ..user
        };
        self.repo.save(&deactivated)
    }
}

// In tests: just use the InMemoryUserRepo
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deactivate_user() {
        let mut repo = InMemoryUserRepo::new();
        let user = User {
            id: "u1".to_string(),
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
            active: true,
        };
        repo.save(&user).unwrap();

        let mut service = UserService::new(repo);
        service.deactivate_user("u1").unwrap();

        // Verify through the repo
        let updated = service.repo.find_by_id("u1").unwrap().unwrap();
        assert!(!updated.active);
    }

    #[test]
    fn test_deactivate_nonexistent_user() {
        let repo = InMemoryUserRepo::new();
        let mut service = UserService::new(repo);

        let result = service.deactivate_user("u999");
        assert!(matches!(result, Err(RepoError::NotFound { .. })));
    }
}
```

Notice we don't need a mocking framework. The in-memory implementation *is* the mock. This is the idiomatic Rust approach -- real implementations of the trait that happen to use simple in-memory storage. You get full type safety and can inspect state directly.

For more complex scenarios (simulating errors, counting calls), you build a purpose-built test double:

```rust
struct FailingRepo {
    fail_on: String,
}

impl UserRepository for FailingRepo {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        if id == self.fail_on {
            Err(RepoError::ConnectionFailed {
                source: "simulated failure".to_string(),
            })
        } else {
            Ok(None)
        }
    }

    fn save(&mut self, _user: &User) -> Result<(), RepoError> {
        Err(RepoError::ConnectionFailed {
            source: "simulated failure".to_string(),
        })
    }

    // ... other methods
    fn find_by_email(&self, _email: &str) -> Result<Option<User>, RepoError> { Ok(None) }
    fn delete(&mut self, _id: &str) -> Result<bool, RepoError> { Ok(false) }
    fn list_active(&self) -> Result<Vec<User>, RepoError> { Ok(vec![]) }
}
```

### Your notes
<!-- -->


---

## The Decorator Pattern: Stacking Repositories

One of the most powerful uses of repository traits is **decoration** -- wrapping one repository with another that adds cross-cutting concerns like caching, logging, or metrics.

```rust
struct CachedUserRepo<R: UserRepository> {
    inner: R,
    cache: HashMap<String, User>,
}

impl<R: UserRepository> CachedUserRepo<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }
}

impl<R: UserRepository> UserRepository for CachedUserRepo<R> {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepoError> {
        // Check cache first
        if let Some(user) = self.cache.get(id) {
            return Ok(Some(user.clone()));
        }
        // Fall through to inner repo
        self.inner.find_by_id(id)
    }

    fn save(&mut self, user: &User) -> Result<(), RepoError> {
        // Write through: save to backing store, then update cache
        self.inner.save(user)?;
        self.cache.insert(user.id.clone(), user.clone());
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, RepoError> {
        self.cache.remove(id);
        self.inner.delete(id)
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        // Cache doesn't index by email -- pass through
        self.inner.find_by_email(email)
    }

    fn list_active(&self) -> Result<Vec<User>, RepoError> {
        self.inner.list_active()
    }
}
```

This composes cleanly:

```rust
// Production: Postgres -> Cache -> Service
// let repo = CachedUserRepo::new(PostgresUserRepo::new(pool));

// Test: InMemory -> Cache -> Service (tests caching behavior)
// let repo = CachedUserRepo::new(InMemoryUserRepo::new());

// Test: InMemory -> Service (tests business logic)
// let repo = InMemoryUserRepo::new();
```

The `CachedUserRepo<R>` is generic over `R: UserRepository`, so it works with any backend. This is composition through generics -- the Rust equivalent of Go's interface-based middleware pattern or TypeScript's class-based decorator pattern.

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go | TypeScript |
|--------|------|----|------------|
| **Abstraction** | Trait (`trait UserRepo`) | Interface (`type UserRepo interface`) | Interface or abstract class |
| **Dispatch** | Static (generics) or dynamic (`dyn Trait`) | Always dynamic (interfaces) | Always dynamic |
| **Return ownership** | Explicit: owned, borrowed, or shared (`Arc`) | Pointer (`*User`) + GC | Object reference + GC |
| **Error handling** | `Result<T, RepoError>` enum | `(User, error)` tuple | `throw` or `Promise<Result>` |
| **Mocking** | Implement the trait directly | Implement the interface directly | Class implementing interface, or jest mock |
| **Async** | `async fn` in traits (1.75+) or `async_trait` | Goroutines (no async/await) | `async/await` with `Promise` |
| **Thread safety** | `Send + Sync` bounds, compile-time checked | `sync.Mutex` for shared state, runtime panics | Single-threaded (Node.js) |
| **Composition** | Generic structs (`CachedRepo<R>`) | Struct embedding + delegation | Class extending/wrapping |
| **Object safety** | Compiler-enforced rules | Not a concept | Not a concept |
| **Popular ORMs** | `diesel`, `sea-orm`, `sqlx` | `gorm`, `sqlx`, `ent` | `TypeORM`, `Prisma`, `Drizzle` |

### Key Differences

**Rust vs Go:** Go's implicit interface satisfaction makes it trivially easy to swap implementations -- any type with matching methods satisfies the interface. Rust requires explicit `impl Trait for Type`, which is more ceremony but catches mismatches at the definition site rather than the use site. Go repositories always return heap-allocated values; Rust lets you choose.

**Rust vs TypeScript:** TypeScript repositories (e.g., TypeORM's `Repository<User>`) use class-based OOP with inheritance. Rust uses trait-based composition. TypeScript's `class CachedRepo extends PostgresRepo` creates tight coupling to the parent class; Rust's `CachedRepo<R: UserRepo>` works with any implementation.

**The ownership difference matters most for caching.** In Go/TypeScript, returning a cached value is trivial -- the runtime handles memory. In Rust, you must decide: clone the cached value (safe, slightly wasteful), return a reference (fast, but borrows the cache), or use `Arc` (shared ownership, small overhead).

### Your notes
<!-- -->


---

## Rust-Specific Concerns

### Lifetimes in Repository Return Types

If your repository trait returns references, lifetimes propagate through everything:

```rust
trait UserRepo {
    fn find_by_id<'a>(&'a self, id: &str) -> Result<Option<&'a User>, RepoError>;
    // The returned &User borrows from &self.
    // While this borrow is alive, no &mut self method can be called.
}
```

This creates friction in practice:

```rust
fn process(repo: &mut dyn UserRepo) {
    let user = repo.find_by_id("u1").unwrap().unwrap(); // borrows repo
    // repo.save(&modified_user); // ERROR: cannot borrow as mutable
    //                            // because it's already borrowed as immutable
}
```

The fix is to clone the value or restructure the code to drop the borrow before mutating:

```rust
fn process(repo: &mut impl UserRepo) {
    let user = repo.find_by_id("u1").unwrap().unwrap().clone(); // clone releases borrow
    let modified = User { active: false, ..user };
    repo.save(&modified).unwrap(); // now OK
}
```

### Object Safety Constraints

If you want `dyn UserRepository` (for runtime polymorphism), your trait must be object-safe:

```rust
// NOT object safe -- generic method
trait BadRepo {
    fn find_by_field<T: std::fmt::Display>(&self, field: &str, value: T) -> Option<User>;
}

// Object safe -- concrete types only
trait GoodRepo {
    fn find_by_field(&self, field: &str, value: &str) -> Option<User>;
}
```

This means you cannot have generic query methods on a trait-object repository. If you need both generic methods and dynamic dispatch, use the `where Self: Sized` escape hatch to exclude specific methods from the vtable:

```rust
trait UserRepo {
    fn find_by_id(&self, id: &str) -> Option<User>;

    // Available only through concrete types, not through &dyn UserRepo
    fn find_by_field<T: std::fmt::Display>(&self, field: &str, value: T) -> Option<User>
    where
        Self: Sized;
}
```

### Your notes
<!-- -->


---

## Preview: Related Patterns

The Repository pattern connects to several other patterns you'll encounter:

- **Unit of Work** (covered later): Groups multiple repository operations into a single transactional unit. In Rust, this interacts with ownership in interesting ways -- who owns the transaction?
- **CQRS** (Command Query Responsibility Segregation): Separate read and write repositories. In Rust, this maps naturally to `&self` (query) vs `&mut self` (command) methods.
- **Strategy pattern** (covered in strategy module): Repository backends are essentially strategies for storage. The caching decorator is the strategy pattern applied to data access.
- **Factory pattern**: A repository factory creates the right backend based on configuration. In Rust, this typically returns `Box<dyn UserRepository>`.
- **Domain-Driven Design**: Repositories are a core DDD building block. They enforce aggregate boundaries -- you can only access an entity through its repository, not by reaching into another aggregate's internals.

### Your notes
<!-- -->
