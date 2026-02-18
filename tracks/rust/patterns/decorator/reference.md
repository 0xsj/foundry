# Rust Reference -- Decorator / Middleware Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/),
> [std library docs](https://doc.rust-lang.org/std/), and
> [Tower docs](https://docs.rs/tower/)
> for the `decorator` module. Covers: newtype pattern, `Deref`/`DerefMut`, `io::Read`/`io::Write` adapters, trait-based composition, Tower Service/Layer.

---

## Newtype Pattern

Source: [The Rust Book - Using the Newtype Pattern](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types)

A newtype is a tuple struct with a single field that wraps another type:

```rust
struct Wrapper(Vec<String>);
```

### Primary Use Cases

| Use Case | Example | Motivation |
|----------|---------|------------|
| Orphan rule workaround | `struct Wrapper(Vec<String>)` then `impl fmt::Display for Wrapper` | Implement external trait on external type |
| Type distinction | `struct Meters(f64)`, `struct Seconds(f64)` | Prevent mixing semantically different values |
| Encapsulation | `struct DatabasePool(Pool)` | Hide inner type's full API |
| Decoration | `struct BufReader<R>(R, Buffer)` | Add behavior while preserving interface |

### Zero-Cost Guarantee

> The newtype pattern is a zero-cost abstraction: the wrapper type is optimized away at compile time, resulting in no runtime overhead.

The compiler guarantees `#[repr(transparent)]` semantics for single-field structs, meaning the newtype has the same memory layout as the inner type.

```rust
#[repr(transparent)]
struct Meters(f64);

// Meters has the exact same size and alignment as f64
assert_eq!(std::mem::size_of::<Meters>(), std::mem::size_of::<f64>());
```

---

## `Deref` and `DerefMut` Traits

Source: [Rust Reference - Deref](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-dereference-operator), [std::ops::Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html)

```rust
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

### Deref Coercion Rules

The compiler automatically applies `Deref` coercion in these contexts:

| Context | Coercion |
|---------|----------|
| `&T` where `T: Deref<Target = U>` | `&T` -> `&U` |
| `&mut T` where `T: DerefMut<Target = U>` | `&mut T` -> `&mut U` |
| `&mut T` where `T: Deref<Target = U>` | `&mut T` -> `&U` |

### When to Implement `Deref`

From the [API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html#only-smart-pointers-implement-deref-and-derefmut-c-deref):

> **Only smart pointers implement `Deref` and `DerefMut`.** The `Deref` traits are designed for implementing custom pointer types. Implementing them on a non-pointer type is confusing because:
>
> - Method resolution becomes implicit and hard to predict
> - Users cannot tell when a method is called on the wrapper vs the inner type
> - `DerefMut` allows bypassing the wrapper entirely

Standard library types that implement `Deref`:

| Type | Target | Reason |
|------|--------|--------|
| `Box<T>` | `T` | Smart pointer |
| `Rc<T>` | `T` | Reference-counted smart pointer |
| `Arc<T>` | `T` | Atomic reference-counted smart pointer |
| `Vec<T>` | `[T]` | Smart pointer to heap-allocated slice |
| `String` | `str` | Smart pointer to heap-allocated string |
| `MutexGuard<T>` | `T` | RAII guard (smart pointer semantics) |

### Anti-Pattern: Deref for Delegation

```rust
// WRONG: Using Deref to delegate methods
struct LoggingVec<T> {
    inner: Vec<T>,
}

impl<T> std::ops::Deref for LoggingVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> { &self.inner }
}
// LoggingVec now implicitly has push(), pop(), len(), etc.
// But push() bypasses any logging -- DerefMut goes straight to Vec
```

```rust
// CORRECT: Explicit delegation
struct LoggingVec<T> {
    inner: Vec<T>,
}

impl<T: std::fmt::Debug> LoggingVec<T> {
    fn push(&mut self, value: T) {
        println!("push: {:?}", value);
        self.inner.push(value);
    }
    fn len(&self) -> usize { self.inner.len() }
    // ... delegate explicitly
}
```

---

## `std::io::Read` Trait and Adapters

Source: [std::io::Read](https://doc.rust-lang.org/std/io/trait.Read.html)

```rust
pub trait Read {
    // Required
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    // Provided (adapter methods that return decorators)
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> { ... }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> { ... }
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> { ... }
    fn bytes(self) -> Bytes<Self> where Self: Sized { ... }
    fn chain<R: Read>(self, next: R) -> Chain<Self, R> where Self: Sized { ... }
    fn take(self, limit: u64) -> Take<Self> where Self: Sized { ... }
    fn by_ref(&mut self) -> &mut Self where Self: Sized { ... }
}
```

### Adapter Types (Decorators)

| Adapter | Source | Decorates | Added Behavior |
|---------|--------|-----------|----------------|
| `BufReader<R>` | `std::io::BufReader` | `R: Read` | Buffered reading, `lines()`, `read_line()` |
| `Take<R>` | `Read::take()` | `R: Read` | Limits bytes readable |
| `Chain<R1, R2>` | `Read::chain()` | Two `R: Read` | Reads R1 until EOF, then R2 |
| `Bytes<R>` | `Read::bytes()` | `R: Read` | Produces `Iterator<Item = Result<u8>>` |

### Ownership in I/O Chains

Adapter methods consume `self`:

```rust
fn take(self, limit: u64) -> Take<Self>
//       ^^^^
//       Takes ownership of the reader
```

Use `by_ref()` to borrow instead:

```rust
let mut file = File::open("data.txt")?;

// Read first 100 bytes without consuming the file handle
let first_100: Vec<u8> = file.by_ref().take(100).bytes()
    .collect::<Result<Vec<u8>, _>>()?;

// file is still usable here
let mut rest = String::new();
file.read_to_string(&mut rest)?;
```

---

## `std::io::Write` Trait and Adapters

Source: [std::io::Write](https://doc.rust-lang.org/std/io/trait.Write.html)

```rust
pub trait Write {
    // Required
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    // Provided
    fn write_all(&mut self, buf: &[u8]) -> Result<()> { ... }
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> { ... }
    fn by_ref(&mut self) -> &mut Self where Self: Sized { ... }
}
```

### Write Adapters

| Adapter | Decorates | Added Behavior |
|---------|-----------|----------------|
| `BufWriter<W>` | `W: Write` | Buffered writing, flushes on drop |
| `LineWriter<W>` | `W: Write` | Flushes after each newline |

---

## Trait Object Safety for Decorators

Source: [Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is object-safe (usable as `dyn Trait`) if all methods:

| Rule | Allowed | Not Allowed |
|------|---------|-------------|
| Receiver | `&self`, `&mut self`, `self: Box<Self>`, `self: Arc<Self>` | No receiver, `self` by value (usually) |
| Return type | Concrete types | `Self` (use `-> Box<dyn Trait>` instead) |
| Generics | None on methods | `fn foo<T>(&self, x: T)` |
| Where clauses | `where Self: Sized` (opts method out of vtable) | - |

For decorator patterns using `Box<dyn Handler>`, the `Handler` trait must be object-safe:

```rust
// Object-safe: can use as Box<dyn Handler>
trait Handler {
    fn handle(&self, req: &Request) -> Response;
    fn name(&self) -> &str;
}

// NOT object-safe: generic method
trait Handler {
    fn handle<R: Request>(&self, req: R) -> Response;  // Error: generic method
}
```

---

## Tower Service and Layer Traits

Source: [tower::Service](https://docs.rs/tower/latest/tower/trait.Service.html), [tower::Layer](https://docs.rs/tower-layer/latest/tower_layer/trait.Layer.html)

> Note: These are from the `tower` crate, not std. Included because they define the de facto standard for Rust middleware.

### Service Trait (simplified, sync version)

```rust
pub trait Service<Request> {
    type Response;
    type Error;

    fn call(&mut self, req: Request) -> Result<Self::Response, Self::Error>;
}
```

The actual `tower::Service` is async and includes a `poll_ready` method for backpressure:

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

### Layer Trait

```rust
pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

### ServiceBuilder (Stacking Layers)

```rust
use tower::ServiceBuilder;

let service = ServiceBuilder::new()
    .timeout(Duration::from_secs(30))
    .rate_limit(100, Duration::from_secs(1))
    .retry(policy)
    .service(my_handler);
```

`ServiceBuilder` applies layers top-to-bottom. The first layer added is the outermost (runs first on request, last on response).

### Key Tower Types

| Type | Role | Category |
|------|------|----------|
| `tower::Service` | Core service trait | Trait |
| `tower::Layer` | Wraps a service to produce a new service | Trait |
| `tower::ServiceBuilder` | Ergonomic layer stacking | Builder |
| `tower::timeout::Timeout<S>` | Timeout decorator | Service |
| `tower::limit::RateLimit<S>` | Rate limiting decorator | Service |
| `tower::retry::Retry<P, S>` | Retry with policy | Service |
| `tower::buffer::Buffer<S>` | Buffered service (backpressure) | Service |

---

## `Send` and `Sync` Bounds for Threaded Decorators

Source: [Rust Reference - Send and Sync](https://doc.rust-lang.org/reference/special-types-and-traits.html#send-and-sync)

| Trait | Meaning | Auto-implemented when |
|-------|---------|----------------------|
| `Send` | Type can be transferred to another thread | All fields are `Send` |
| `Sync` | Type can be shared between threads via `&T` | All fields are `Sync` |

### Common types and thread safety

| Type | `Send` | `Sync` | Use in decorators |
|------|--------|--------|-------------------|
| `String`, `Vec<T>` | Yes | Yes | Safe in any decorator |
| `Arc<T>` | Yes (if `T: Send + Sync`) | Yes (if `T: Send + Sync`) | Shared ownership across threads |
| `Mutex<T>` | Yes (if `T: Send`) | Yes (if `T: Send`) | Interior mutability across threads |
| `Cell<T>` | Yes (if `T: Send`) | **No** | Single-threaded interior mutability only |
| `RefCell<T>` | Yes (if `T: Send`) | **No** | Single-threaded interior mutability only |
| `Rc<T>` | **No** | **No** | Single-threaded shared ownership only |
| `AtomicU64` | Yes | Yes | Lock-free counters in decorators |

For `Box<dyn Handler + Send + Sync>` to work, the concrete type behind the trait object must implement both `Send` and `Sync`. If any field of your decorator is `Rc`, `Cell`, or `RefCell`, the decorator will not be `Send + Sync`.

---

## Interior Mutability for Stateful Decorators

Source: [std::cell](https://doc.rust-lang.org/std/cell/), [std::sync::atomic](https://doc.rust-lang.org/std/sync/atomic/)

When a decorator method takes `&self` but needs to mutate state (e.g., counting requests):

| Tool | Thread-safe | Blocking | Use case |
|------|-------------|----------|----------|
| `Cell<T>` | No | No | Simple values, single thread |
| `RefCell<T>` | No | No (panics) | Complex types, single thread |
| `AtomicU64` | Yes | No | Counters, flags |
| `Mutex<T>` | Yes | Yes (blocks) | Complex types, multi-thread |
| `RwLock<T>` | Yes | Yes | Read-heavy multi-thread access |

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct CountingDecorator<H> {
    inner: H,
    count: AtomicU64,
}

impl<H: Handler> Handler for CountingDecorator<H> {
    fn handle(&self, req: &str) -> String {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.handle(req)
    }
}
```
