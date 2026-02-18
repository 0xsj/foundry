# Rust Reference — Chain of Responsibility Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/)
> for the `chain-of-responsibility` module. Covers: trait objects, Box, Option, recursive types,
> object safety, Send + Sync, dynamic dispatch.

---

## Trait Objects for Polymorphic Chains

Source: [Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)

A trait object (`dyn Trait`) is an opaque type that implements a set of traits. Trait objects are accessed behind a pointer — `&dyn Trait`, `Box<dyn Trait>`, or `Arc<dyn Trait>`.

```rust
trait Handler {
    fn handle(&self, data: &str) -> Result<String, String>;
}

// Trait object behind a Box — owned, heap-allocated
let handler: Box<dyn Handler> = Box::new(ConcreteHandler::new());

// Trait object behind a reference — borrowed
fn process(handler: &dyn Handler, data: &str) -> Result<String, String> {
    handler.handle(data)
}
```

### Vtable Layout

A trait object is a fat pointer: two machine words.

| Word | Contains |
|------|----------|
| First | Pointer to the data (the concrete type instance) |
| Second | Pointer to the vtable (table of function pointers for the trait methods) |

Each method call on a `dyn Trait` performs an indirect function call through the vtable. Typical overhead: ~2-5ns per call on modern hardware.

### Object Safety Rules

Source: [Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is **object safe** (usable as `dyn Trait`) if ALL of these hold:

| Rule | Explanation | Example Violation |
|------|-------------|-------------------|
| No `Self: Sized` bound on the trait | `dyn Trait` is unsized | `trait Foo: Sized {}` |
| No generic type parameters on methods | Vtable cannot have infinite entries | `fn handle<T>(&self, item: T)` |
| No methods return `Self` | Concrete type unknown behind `dyn` | `fn clone(&self) -> Self` |
| No associated constants | Not yet supported for trait objects | `const ID: u32;` |
| Methods use `&self`, `&mut self`, or `self: Box<Self>` | Dispatch needs a receiver | `fn handle()` (no self) |

**Workaround for `Clone`-like patterns:**

```rust
trait Handler {
    fn handle(&self, req: &Request) -> Response;

    // Object-safe clone: returns Box<dyn Handler> instead of Self
    fn clone_handler(&self) -> Box<dyn Handler>;
}
```

---

## Box<T> — Heap Allocation

Source: [std::boxed::Box](https://doc.rust-lang.org/std/boxed/struct.Box.html)

`Box<T>` allocates `T` on the heap and provides owned access. When the `Box` is dropped, the heap memory is freed.

```rust
let handler: Box<dyn Handler> = Box::new(AuthHandler { /* ... */ });
```

### Key Properties

| Property | Value |
|----------|-------|
| Size of `Box<T>` | One pointer (8 bytes on 64-bit) |
| Size of `Box<dyn Trait>` | Two pointers (16 bytes — data + vtable) |
| Deref | `Box<T>` implements `Deref<Target = T>` |
| Move semantics | Moving a `Box` moves the pointer, not the data |
| Drop | Calls `T::drop()`, then frees heap memory |

### Box for Recursive Types

Recursive types have no finite size without indirection:

```rust
// Error: recursive type `Chain` has infinite size
struct Chain {
    next: Option<Chain>,
}

// Fix: Box provides indirection (a pointer has fixed size)
struct Chain {
    next: Option<Box<Chain>>,
}
```

This is fundamental to Chain of Responsibility: each handler holds an optional next handler.

---

## Option<T> — Optional Next Handler

Source: [std::option::Option](https://doc.rust-lang.org/std/option/enum.Option.html)

`Option<T>` represents either `Some(T)` or `None`. In handler chains, it signals "there may or may not be a next handler."

```rust
struct Handler {
    next: Option<Box<dyn Handler>>,
}

impl Handler {
    fn delegate(&self, req: &Request) -> Response {
        match &self.next {
            Some(next) => next.handle(req),
            None => Response::default(), // End of chain
        }
    }
}
```

### Relevant Methods

| Method | Signature | Use in Chains |
|--------|-----------|---------------|
| `is_some()` | `fn is_some(&self) -> bool` | Check if chain continues |
| `as_ref()` | `fn as_ref(&self) -> Option<&T>` | Borrow next handler without consuming |
| `as_deref()` | `fn as_deref(&self) -> Option<&T::Target>` | For `Option<Box<T>>`, get `Option<&T>` |
| `map()` | `fn map<U, F>(self, f: F) -> Option<U>` | Transform the handler if present |
| `and_then()` | `fn and_then<U, F>(self, f: F) -> Option<U>` | Chain handler operations |

### Option<Box<dyn Trait>> Pattern

The idiomatic type for "optional next handler" in Rust chains:

```rust
type NextHandler = Option<Box<dyn Handler>>;

struct AuthHandler {
    next: NextHandler,
}

struct RateLimitHandler {
    next: NextHandler,
}
```

Memory layout of `Option<Box<T>>`:
- `Some(box)`: pointer to heap-allocated T
- `None`: null pointer

Due to null pointer optimization, `Option<Box<T>>` is the same size as `Box<T>` (one pointer). The `None` variant uses the null pointer representation.

---

## Dynamic Dispatch vs Static Dispatch

Source: [Rust Book - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)

### Dynamic Dispatch (Trait Objects)

```rust
fn run_chain(handler: &dyn Handler, req: &Request) -> Response {
    handler.handle(req)  // Indirect call through vtable
}
```

- Method call goes through vtable pointer
- One copy of the function in the binary
- Can be used with heterogeneous collections (`Vec<Box<dyn Handler>>`)
- Prevents inlining and some optimizations

### Static Dispatch (Generics / Monomorphization)

```rust
fn run_chain<H: Handler>(handler: &H, req: &Request) -> Response {
    handler.handle(req)  // Direct call, can be inlined
}
```

- Compiler generates a specialized function for each concrete type
- Direct call, can be inlined
- Homogeneous only (all elements same concrete type)
- Larger binary (one copy per concrete type)

### Cost Comparison

| Aspect | Dynamic (`dyn Handler`) | Static (`H: Handler`) |
|--------|------------------------|----------------------|
| Call overhead | ~2-5ns (vtable lookup) | 0ns (direct call / inlined) |
| Binary size | Smaller (one copy) | Larger (per-type copy) |
| Compilation | Faster | Slower (monomorphization) |
| Flexibility | Heterogeneous chains | Homogeneous or nested generic types |
| Inlining | No | Yes |

---

## Send and Sync for Threaded Chains

Source: [Rust Book - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)

### Send

A type is `Send` if it can be transferred to another thread. Required when a handler chain is moved into a thread or spawned task.

```rust
// Handler chain that can be sent to another thread
let chain: Box<dyn Handler + Send> = build_chain();
std::thread::spawn(move || {
    chain.handle(&request);
});
```

### Sync

A type is `Sync` if it can be referenced from multiple threads simultaneously. Required when a handler chain is shared (e.g., behind `Arc`).

```rust
// Handler chain shared across threads
let chain: Arc<dyn Handler + Send + Sync> = Arc::new(build_chain());

for _ in 0..4 {
    let chain = Arc::clone(&chain);
    std::thread::spawn(move || {
        chain.handle(&request);
    });
}
```

### Common Types and Their Bounds

| Type | Send | Sync | Notes |
|------|------|------|-------|
| `String`, `Vec<T>`, `Box<T>` | Yes (if T: Send) | Yes (if T: Sync) | Owned data is safe |
| `Rc<T>` | **No** | **No** | Reference counting is not atomic |
| `Arc<T>` | Yes (if T: Send + Sync) | Yes (if T: Send + Sync) | Atomic reference counting |
| `RefCell<T>` | Yes (if T: Send) | **No** | Runtime borrow checking is not thread-safe |
| `Mutex<T>` | Yes (if T: Send) | Yes (if T: Send) | Lock provides synchronized access |
| `Cell<T>` | Yes (if T: Send) | **No** | Interior mutability without thread safety |

### Adding Bounds to Handler Traits

```rust
// Basic handler — single-threaded only
trait Handler {
    fn handle(&self, req: &Request) -> Response;
}

// Thread-safe handler
trait Handler: Send + Sync {
    fn handle(&self, req: &Request) -> Response;
}

// Or constrain at the usage site
fn build_chain() -> Box<dyn Handler + Send + Sync> { /* ... */ }
```

---

## Closure Types for Functional Chains

Source: [Rust Reference - Closure Types](https://doc.rust-lang.org/reference/types/closure.html)

### Fn Trait Hierarchy

| Trait | Captures | Called | Use Case |
|-------|----------|--------|----------|
| `Fn(&self)` | By shared reference | Multiple times | Read-only handlers |
| `FnMut(&mut self)` | By mutable reference | Multiple times | Stateful handlers (counters, accumulators) |
| `FnOnce(self)` | By value (consuming) | Once | One-shot handlers |

For handler chains, `Fn` is most common (handlers are called repeatedly). Use `FnMut` when handlers need to update internal state (rate limit counters, etc.).

```rust
// Read-only handler chain
type HandlerFn = Box<dyn Fn(&Request) -> Result<Response, Error>>;

// Stateful handler chain (needs &mut self on the pipeline)
type StatefulHandlerFn = Box<dyn FnMut(&Request) -> Result<Response, Error>>;
```

### Closures Capturing State

```rust
let max_size: usize = 1024;

// This closure captures max_size by reference (implements Fn)
let size_check = move |req: &Request| -> Result<(), Error> {
    if req.body.len() > max_size {
        Err(Error::TooLarge)
    } else {
        Ok(())
    }
};
```

The `move` keyword moves `max_size` into the closure, giving it a `'static` lifetime. Without `move`, the closure borrows `max_size` and cannot outlive the scope where `max_size` is defined.

---

## Result<T, E> for Short-Circuit Chains

Source: [std::result::Result](https://doc.rust-lang.org/std/result/enum.Result.html)

The `?` operator provides automatic chain short-circuiting:

```rust
fn execute_chain(handlers: &[HandlerFn], req: &mut Request) -> Result<(), Error> {
    for handler in handlers {
        handler(req)?;  // If any handler returns Err, stop immediately
    }
    Ok(())
}
```

### The ? Operator

`?` applied to a `Result`:
- If `Ok(val)`: unwraps to `val`, execution continues
- If `Err(e)`: converts the error (via `From` trait if needed) and returns early

This makes `Result` the natural return type for handler chains — each handler either succeeds (chain continues) or fails (chain stops with error).

### Error Conversion in Chains

Different handlers may produce different error types. Use a common error enum or `Box<dyn Error>`:

```rust
#[derive(Debug)]
enum ChainError {
    Auth(String),
    RateLimit { retry_after: u64 },
    Validation(Vec<String>),
    Internal(String),
}

// Each handler returns Result<(), ChainError>
// The ? operator works uniformly across all handlers
```

---

## References

- [Rust Book - Trait Objects and Dynamic Dispatch](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
- [std::boxed::Box](https://doc.rust-lang.org/std/boxed/struct.Box.html)
- [Rust Book - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Tower Service Trait](https://docs.rs/tower/latest/tower/trait.Service.html) (ecosystem middleware pattern)
- [Refactoring Guru - Chain of Responsibility in Rust](https://refactoring.guru/design-patterns/chain-of-responsibility/rust/example)
