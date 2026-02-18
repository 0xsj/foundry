# Rust Reference -- Dependency Injection

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/)
> for the `dependency-injection` module. Covers: trait bounds, generics, trait objects, dynamic dispatch, object safety, Arc, Send, Sync.

---

## Trait Bounds for DI

Source: [Rust Reference - Trait and lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html)

Trait bounds constrain generic type parameters, ensuring only types implementing specified traits can be used. This is the compile-time mechanism for dependency injection.

```rust
// Bound on struct definition
struct Service<R: Repository> {
    repo: R,
}

// Bound on impl block
impl<R: Repository> Service<R> {
    fn new(repo: R) -> Self {
        Service { repo }
    }
}

// Multiple bounds with + syntax
fn process<R: Repository + Clone + Send>(repo: R) { /* ... */ }

// where clause for readability
fn process<R>(repo: R)
where
    R: Repository + Clone + Send,
{
    // ...
}
```

### Bound Placement

| Location | Syntax | Effect |
|----------|--------|--------|
| Struct definition | `struct S<T: Trait>` | All uses of `S` require `T: Trait` |
| Impl block | `impl<T: Trait> S<T>` | Methods only available when `T: Trait` |
| Function | `fn f<T: Trait>(t: T)` | Caller must provide type satisfying `Trait` |
| Where clause | `where T: Trait` | Same as inline bound, better readability |

### Supertraits

```rust
// Repository requires Display -- every implementor must also implement Display
trait Repository: std::fmt::Display {
    fn find(&self, id: u64) -> Option<String>;
}
```

Supertraits compose requirements. If your DI trait needs additional capabilities, express them as supertraits rather than adding bounds at every use site.

---

## Trait Objects and Dynamic Dispatch

Source: [Rust Reference - Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html)

A trait object is an opaque value of another type that implements a set of traits. Methods are called via a vtable (virtual dispatch table).

```rust
// Trait object behind a Box (heap-allocated, owned)
let repo: Box<dyn Repository> = Box::new(PostgresRepo::new());

// Trait object behind a reference (borrowed)
let repo: &dyn Repository = &postgres_repo;

// Trait object behind Arc (shared ownership)
let repo: Arc<dyn Repository> = Arc::new(PostgresRepo::new());
```

### Object Safety

Source: [Rust Reference - Object safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is object-safe (can be used as `dyn Trait`) if all its methods satisfy:

| Rule | Allowed | Not Allowed |
|------|---------|-------------|
| `self` parameter | `&self`, `&mut self`, `self: Box<Self>`, `self: Arc<Self>` | No `self` parameter |
| Return type | Concrete types, `Box<dyn OtherTrait>` | `Self` (unless `Sized` bound) |
| Generic methods | Not allowed in object-safe traits | `fn method<T>(&self, t: T)` |
| Associated functions | Must have `self` parameter | `fn new() -> Self` (no self) |

```rust
// Object-safe -- can be used as dyn Repository
trait Repository {
    fn find(&self, id: u64) -> Option<String>;
    fn save(&self, id: u64, name: &str) -> Result<(), String>;
}

// NOT object-safe -- generic method
trait Serializer {
    fn serialize<T: serde::Serialize>(&self, value: &T) -> Vec<u8>;
    // Cannot use as dyn Serializer because of the generic parameter T
}

// NOT object-safe -- returns Self
trait Clonable {
    fn clone_self(&self) -> Self;
    // Cannot use as dyn Clonable because return type is Self
}
```

**Workaround for Self-returning methods:**

```rust
trait Repository {
    fn find(&self, id: u64) -> Option<String>;

    // Return a trait object instead of Self
    fn clone_box(&self) -> Box<dyn Repository>;
}
```

---

## `impl Trait` Syntax

Source: [Rust Reference - Impl trait type](https://doc.rust-lang.org/reference/types/impl-trait.html)

### Argument Position (APIT)

Syntactic sugar for a generic type parameter with a trait bound:

```rust
// These are equivalent
fn log(logger: &impl Logger) { /* ... */ }
fn log<L: Logger>(logger: &L) { /* ... */ }
```

Each call site can use a different concrete type. The compiler monomorphizes.

### Return Position (RPIT)

The function returns some concrete type that implements the trait, but the caller doesn't know which:

```rust
fn create_logger() -> impl Logger {
    FileLogger::new("/var/log/app.log")
    // Caller sees: impl Logger
    // Compiler knows: FileLogger
}
```

**Limitation:** All return paths must return the same concrete type:

```rust
// COMPILE ERROR -- two different concrete types
fn create_logger(use_file: bool) -> impl Logger {
    if use_file {
        FileLogger::new("/var/log/app.log")
    } else {
        StdoutLogger::new()  // Different type!
    }
}

// Fix: use Box<dyn Logger> for runtime polymorphism
fn create_logger(use_file: bool) -> Box<dyn Logger> {
    if use_file {
        Box::new(FileLogger::new("/var/log/app.log"))
    } else {
        Box::new(StdoutLogger::new())
    }
}
```

---

## Send and Sync

Source: [Rust std - Send](https://doc.rust-lang.org/std/marker/trait.Send.html), [Sync](https://doc.rust-lang.org/std/marker/trait.Sync.html)

When passing dependencies across threads (common in async services), trait objects must be `Send + Sync`:

```rust
// Required for multi-threaded usage
trait Repository: Send + Sync {
    fn find(&self, id: u64) -> Option<String>;
}

// Or on the trait object
struct Service {
    repo: Box<dyn Repository + Send + Sync>,
}

// Arc requires Send + Sync for the inner type
struct SharedService {
    repo: Arc<dyn Repository + Send + Sync>,
}
```

| Marker | Meaning | Auto-derived? |
|--------|---------|---------------|
| `Send` | Type can be transferred to another thread | Yes, if all fields are Send |
| `Sync` | Type can be shared between threads via `&T` | Yes, if all fields are Sync |
| `Send + Sync` | Both -- can be shared and transferred | Most types satisfy this |

**Types that are NOT Send/Sync:**

| Type | Send? | Sync? | Why |
|------|-------|-------|-----|
| `Rc<T>` | No | No | Non-atomic reference counting |
| `RefCell<T>` | Yes | No | Non-thread-safe interior mutability |
| `*mut T` | No | No | Raw pointer, no guarantees |
| `Arc<T>` | Yes (if T: Send + Sync) | Yes (if T: Send + Sync) | Atomic reference counting |

---

## Arc for Shared Dependencies

Source: [Rust std - Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)

`Arc<T>` (Atomically Reference Counted) enables shared ownership across threads:

```rust
use std::sync::Arc;

let pool: Arc<dyn ConnectionPool + Send + Sync> = Arc::new(PostgresPool::new());

// Clone creates a new reference, not a deep copy
let pool_clone = Arc::clone(&pool);

// Both references point to the same pool
// Arc is dropped when the last reference goes out of scope
```

### Arc + Mutex for Mutable Shared Dependencies

```rust
use std::sync::{Arc, Mutex};

struct MetricsCollector {
    counts: Mutex<HashMap<String, u64>>,
}

impl MetricsCollector {
    fn increment(&self, key: &str) {
        let mut counts = self.counts.lock().unwrap();
        *counts.entry(key.to_string()).or_insert(0) += 1;
    }
}

// Shared across services
let metrics: Arc<MetricsCollector> = Arc::new(MetricsCollector {
    counts: Mutex::new(HashMap::new()),
});
```

### Arc vs Box for Trait Objects

| Container | Ownership | Clone behavior | Thread-safe? | Use case |
|-----------|-----------|---------------|--------------|----------|
| `Box<dyn T>` | Single owner | Cannot clone (unless T: Clone) | If T: Send | One service owns the dependency |
| `Arc<dyn T>` | Shared | Cheap clone (ref count) | If T: Send + Sync | Multiple services share the dependency |
| `Rc<dyn T>` | Shared | Cheap clone (ref count) | No | Single-threaded shared ownership |

---

## Mock Testing Patterns

### Manual Mocks

```rust
struct MockRepo {
    // Pre-configured responses
    responses: HashMap<u64, String>,
    // Call tracking
    calls: RefCell<Vec<String>>,
}

impl Repository for MockRepo {
    fn find(&self, id: u64) -> Option<String> {
        self.calls.borrow_mut().push(format!("find({})", id));
        self.responses.get(&id).cloned()
    }
}
```

### Configurable Mock with Closures

```rust
struct MockRepo {
    find_fn: Box<dyn Fn(u64) -> Option<String>>,
    save_fn: Box<dyn Fn(u64, &str) -> Result<(), String>>,
}

impl Repository for MockRepo {
    fn find(&self, id: u64) -> Option<String> {
        (self.find_fn)(id)
    }

    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        (self.save_fn)(id, name)
    }
}

// Usage in tests
let mock = MockRepo {
    find_fn: Box::new(|id| {
        if id == 1 { Some("Alice".into()) } else { None }
    }),
    save_fn: Box::new(|_, _| Ok(())),
};
```

### Interior Mutability for Assertions

```rust
use std::cell::RefCell;

struct SpyRepo {
    save_calls: RefCell<Vec<(u64, String)>>,
}

impl Repository for SpyRepo {
    fn find(&self, _id: u64) -> Option<String> { None }

    fn save(&self, id: u64, name: &str) -> Result<(), String> {
        self.save_calls.borrow_mut().push((id, name.to_string()));
        Ok(())
    }
}

// In test:
let spy = SpyRepo { save_calls: RefCell::new(vec![]) };
service.register(1, "Alice", &spy);
assert_eq!(spy.save_calls.borrow().len(), 1);
```

---

## Associated Types vs Generic Traits

Source: [Rust Book - Associated Types](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#specifying-placeholder-types-in-trait-definitions-with-associated-types)

| Feature | Associated Type | Generic Trait |
|---------|----------------|---------------|
| Syntax | `trait T { type Item; }` | `trait T<Item> { }` |
| Implementations per type | One | Multiple (one per generic param) |
| Use case | "A type has exactly one X" | "A type can work with many X" |
| Object safety | Object-safe (with constraints) | Makes object safety harder |

```rust
// Associated type -- each repo has ONE error type
trait Repository {
    type Error;
    fn find(&self, id: u64) -> Result<Option<String>, Self::Error>;
}

// Generic trait -- a converter can work with MANY types
trait Converter<T> {
    fn convert(&self, input: &str) -> Result<T, String>;
}
```

For DI, associated types are preferred when each implementation naturally pairs with specific types (its own error type, connection type, etc.). Generic traits are for when the same implementation should work with multiple type parameters.

---

## Summary Table: DI Patterns

| Pattern | Mechanism | Dispatch | Flexibility | Ergonomics | Performance |
|---------|-----------|----------|-------------|------------|-------------|
| `struct S<T: Trait>` | Generics | Static | Compile-time only | Verbose with many params | Zero-cost |
| `struct S { t: Box<dyn Trait> }` | Trait object | Dynamic | Runtime swappable | Clean signatures | ~1ns overhead/call |
| `struct S { t: Arc<dyn Trait> }` | Shared trait object | Dynamic | Runtime + shared | Clean, cloneable | Arc overhead + vtable |
| `fn f(t: &impl Trait)` | impl Trait | Static | Compile-time | Concise | Zero-cost |
| `fn f() -> impl Trait` | Opaque return | Static | Hides concrete type | Concise | Zero-cost |
