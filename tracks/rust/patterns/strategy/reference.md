# Rust Reference — Strategy Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/)
> for the `strategy` module. Covers: traits, trait objects, generics, closures, object safety, dispatch.

---

## Trait Definition and Implementation

Source: [Rust Reference - Traits](https://doc.rust-lang.org/reference/items/traits.html)

A trait describes an abstract interface that types can implement. Trait methods may have default implementations.

```rust
trait Strategy {
    // Required method — implementors must provide this
    fn execute(&self, input: &[u8]) -> Vec<u8>;

    // Provided method — implementors MAY override this
    fn name(&self) -> &str {
        "unnamed"
    }
}
```

### Implementation Rules

```rust
struct ConcreteStrategy;

impl Strategy for ConcreteStrategy {
    fn execute(&self, input: &[u8]) -> Vec<u8> {
        input.to_vec()
    }
    // name() uses the default implementation
}
```

| Rule | Description |
|------|-------------|
| Orphan rule | You can implement a trait for a type only if the trait OR the type is defined in your crate |
| Coherence | There can be at most one implementation of a trait for any given type |
| No partial impl | All required methods must be implemented |
| Supertraits | `trait A: B {}` means any type implementing `A` must also implement `B` |

### Associated Types vs Generic Traits

```rust
// Generic trait — allows multiple impls per type
trait Convert<T> {
    fn convert(&self) -> T;
}

// Associated type — exactly one impl per type
trait IntoBytes {
    type Output;
    fn into_bytes(&self) -> Self::Output;
}
```

For strategy patterns, generic traits allow a type to serve as a strategy for multiple input types. Associated types constrain it to one.

---

## Trait Objects and Dynamic Dispatch

Source: [Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)

A trait object is an opaque value of another type that implements a set of traits. The set of traits is made up of an object safe base trait plus any number of auto traits.

### Syntax

```rust
// Reference to trait object (borrowed)
fn process(strategy: &dyn Strategy) { /* ... */ }

// Owned trait object (on the heap)
fn process_owned(strategy: Box<dyn Strategy>) { /* ... */ }

// Mutable reference
fn process_mut(strategy: &mut dyn Strategy) { /* ... */ }

// With lifetime
fn process_lt<'a>(strategy: &'a dyn Strategy) { /* ... */ }
```

### Fat Pointer Layout

A trait object reference (`&dyn Trait`) is a "fat pointer" consisting of two machine-word-sized values:

| Component | Size | Contains |
|-----------|------|----------|
| Data pointer | 1 word (8 bytes on 64-bit) | Points to the concrete value |
| Vtable pointer | 1 word (8 bytes on 64-bit) | Points to the vtable for this type's impl of the trait |

The vtable contains:
- Destructor (`drop`) function pointer
- Size and alignment of the concrete type
- Function pointers for each trait method, in declaration order

### Lifetime Bounds

```rust
// Default lifetime bound for Box<dyn Trait> is 'static
let s: Box<dyn Strategy>;          // means Box<dyn Strategy + 'static>

// For references, the default is the reference's lifetime
let s: &dyn Strategy;              // means &'a (dyn Strategy + 'a)

// Explicit lifetime bounds
let s: Box<dyn Strategy + 'a>;     // borrowed data inside strategy
let s: Box<dyn Strategy + Send>;   // safe to send across threads
let s: Box<dyn Strategy + Send + Sync>; // safe to share AND send
```

---

## Object Safety

Source: [Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is **object safe** if it can be used as a trait object (`dyn Trait`). The compiler enforces these rules:

### Requirements

| Requirement | Reason |
|-------------|--------|
| All supertraits must be object safe | The vtable must be constructable for all supertrait methods |
| `Sized` must not be a supertrait | Trait objects are `!Sized` by definition |
| No associated constants | Cannot be stored in vtable |
| No associated types with generic bounds | Cannot be resolved without concrete type |
| All associated functions must either be dispatchable from a trait object or explicitly non-dispatchable | Methods need a valid vtable slot |

### Dispatchable Methods

A method is dispatchable (can be called through `dyn Trait`) when:

1. It does not have generic type parameters (no `fn foo<T>(&self)`)
2. It takes `self` by some form of receiver: `&self`, `&mut self`, `self`, `Box<Self>`, `Rc<Self>`, `Arc<Self>`, `Pin<&Self>`, `Pin<&mut Self>`, `Pin<Box<Self>>`
3. It does not use `Self` in the signature except as the receiver (no `-> Self`, no `fn foo(other: Self)`)
4. It does not have a `where Self: Sized` bound (methods with this bound are excluded from the vtable)

### Examples

```rust
// OBJECT SAFE
trait ObjectSafe {
    fn process(&self, data: &[u8]) -> Vec<u8>;
    fn name(&self) -> &str;
}

// OBJECT SAFE — method with Self: Sized is excluded from vtable
trait AlsoSafe {
    fn process(&self, data: &[u8]) -> Vec<u8>;

    // This method won't be callable through &dyn AlsoSafe
    // but the trait is still object safe
    fn clone_box(&self) -> Box<Self>
    where
        Self: Sized;
}

// NOT OBJECT SAFE — generic method
trait NotSafe1 {
    fn process<T>(&self, data: T);
    // Reason: infinite vtable slots needed (one per monomorphization of T)
}

// NOT OBJECT SAFE — returns Self
trait NotSafe2 {
    fn duplicate(&self) -> Self;
    // Reason: size of Self unknown behind dyn pointer
}

// NOT OBJECT SAFE — Self parameter (not as receiver)
trait NotSafe3 {
    fn merge(&self, other: Self);
    // Reason: other could be a different concrete type
}
```

### Workarounds for Object Safety

```rust
// Problem: need Clone on trait objects
// Solution: clone_box pattern
trait CloneableStrategy: Strategy {
    fn clone_box(&self) -> Box<dyn CloneableStrategy>;
}

impl<T: Strategy + Clone + 'static> CloneableStrategy for T {
    fn clone_box(&self) -> Box<dyn CloneableStrategy> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneableStrategy> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
```

---

## Generics and Static Dispatch

Source: [Rust Book - Generic Types](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### Trait Bounds

```rust
// Bound on function
fn process<S: Strategy>(strategy: &S, data: &[u8]) -> Vec<u8> {
    strategy.execute(data)
}

// Multiple bounds
fn process<S: Strategy + Send + Sync>(strategy: &S, data: &[u8]) -> Vec<u8> {
    strategy.execute(data)
}

// Where clause (preferred for complex bounds)
fn process<S>(strategy: &S, data: &[u8]) -> Vec<u8>
where
    S: Strategy + Send + Sync + 'static,
{
    strategy.execute(data)
}

// impl Trait in argument position (syntactic sugar for simple generic)
fn process(strategy: &impl Strategy, data: &[u8]) -> Vec<u8> {
    strategy.execute(data)
}
```

### `impl Trait` vs Generics

| Feature | `impl Trait` | Generic `<T: Trait>` |
|---------|-------------|---------------------|
| Turbofish syntax | Not supported | `process::<ConcreteType>()` |
| Multiple params, same type | Each `impl` could be different | `<T>` forces same type |
| Return position | Opaque type (caller can't name it) | Caller chooses type |
| Complexity | Simpler | More flexible |

```rust
// These are different:
fn same_generic<T: Strategy>(a: &T, b: &T) { }     // a and b must be SAME type
fn different_impl(a: &impl Strategy, b: &impl Strategy) { } // a and b can be DIFFERENT types

// In return position:
fn make_strategy() -> impl Strategy { /* ... */ }  // Caller can't name the type
```

### Monomorphization

The compiler generates a specialized copy for each concrete type:

```rust
fn process<S: Strategy>(s: &S, data: &[u8]) -> Vec<u8> {
    s.execute(data)
}

// Calling with two different types:
let a = ConcreteA;
let b = ConcreteB;
process(&a, &data);  // compiler generates process_ConcreteA
process(&b, &data);  // compiler generates process_ConcreteB
```

---

## Closure Traits: Fn, FnMut, FnOnce

Source: [Rust Reference - Closure Types](https://doc.rust-lang.org/reference/types/closure.html)

Closures implement one or more of three traits, forming a hierarchy:

```
FnOnce  (can be called once — may consume captured values)
  ^
  |
FnMut   (can be called multiple times — may mutate captured values)
  ^
  |
Fn      (can be called multiple times — only reads captured values)
```

### Trait Definitions

```rust
pub trait FnOnce<Args> {
    type Output;
    fn call_once(self, args: Args) -> Self::Output;
}

pub trait FnMut<Args>: FnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Output;
}

pub trait Fn<Args>: FnMut<Args> {
    fn call(&self, args: Args) -> Self::Output;
}
```

### Which Trait Does a Closure Implement?

| Closure behavior | Implements | Example |
|-----------------|------------|---------|
| Doesn't capture anything | `Fn + FnMut + FnOnce` | `\|x\| x + 1` |
| Captures by shared reference | `Fn + FnMut + FnOnce` | `\|x\| x + captured` |
| Captures by mutable reference | `FnMut + FnOnce` | `\|\| count += 1` |
| Captures by move (and value is `Copy`) | `Fn + FnMut + FnOnce` | `move \|x\| x + copied_val` |
| Captures by move (consumes value) | `FnOnce` | `move \|\| drop(owned_val)` |

### Using Closures as Strategies

```rust
// Fn — can be called repeatedly, read-only access to captured state
fn with_validator(validate: impl Fn(&str) -> bool) { /* ... */ }

// FnMut — can be called repeatedly, may mutate captured state
fn with_accumulator(mut acc: impl FnMut(u64)) { /* ... */ }

// FnOnce — can be called exactly once, may consume captured state
fn with_finalizer(finalize: impl FnOnce() -> Vec<u8>) { /* ... */ }

// Boxed closure for storage
struct Handler {
    on_event: Box<dyn Fn(&str) + Send>,           // shareable
    on_complete: Option<Box<dyn FnOnce() + Send>>, // one-shot
}
```

### Closure Size and Allocation

| Closure Type | Size | Allocation |
|-------------|------|------------|
| Non-capturing | 0 bytes (ZST) | None |
| Captures by reference | Size of captured references | Stack |
| Captures by value (move) | Size of captured values | Stack |
| `Box<dyn Fn()>` | 2 words (fat pointer) | Heap |
| `fn()` (function pointer) | 1 word | None |

---

## Decision Matrix

### When to Use Each Dispatch Mechanism

| Criterion | Generics | Trait Objects | Closures | Enum |
|-----------|----------|---------------|----------|------|
| **Known at compile time** | Required | Not required | Either | Required |
| **Extensible** (open set) | Yes | Yes | Yes | No (closed) |
| **Heterogeneous collections** | No | Yes | Yes (boxed) | Yes |
| **Performance** | Best (inlined) | Good (vtable) | Good to Best | Best (no indirection) |
| **Binary size** | Larger | Smaller | Varies | Smallest |
| **Complexity** | Medium | Low | Low | Low |
| **Multiple methods** | Yes | Yes | Awkward | Yes |
| **State** | Yes | Yes | Captured | Yes (in variants) |
| **Thread safety** | Automatic with bounds | Need `+ Send + Sync` | Need bounds | Automatic |
| **Dynamic loading / plugins** | No | Yes | Yes (boxed) | No |

### Standard Library Examples

| Pattern | Std Lib Example | Dispatch |
|---------|----------------|----------|
| Sort comparator | `slice::sort_by(compare)` | Closure / Generic |
| Hashing | `HashMap<K, V, S: BuildHasher>` | Generic |
| I/O abstraction | `Read`, `Write` traits | Trait object or generic |
| Iterator adaptor | `Iterator::map(f)` | Generic (closure) |
| Error handling | `Box<dyn Error>` | Trait object |
| Allocator | `Vec<T, A: Allocator>` | Generic |
| Formatting | `fmt::Display`, `fmt::Debug` | Generic (monomorphized in macros) |

---

## Common Patterns

### Strategy with Builder

```rust
struct ClientBuilder<R = NoRetry> {
    retry: R,
    timeout_ms: u64,
}

impl ClientBuilder<NoRetry> {
    fn new() -> Self {
        Self { retry: NoRetry, timeout_ms: 5000 }
    }
}

impl<R> ClientBuilder<R> {
    fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    // Type-state: return a builder with a different strategy type
    fn with_retry<R2: RetryStrategy>(self, retry: R2) -> ClientBuilder<R2> {
        ClientBuilder {
            retry,
            timeout_ms: self.timeout_ms,
        }
    }

    fn build(self) -> Client<R>
    where
        R: RetryStrategy,
    {
        Client {
            retry: self.retry,
            timeout_ms: self.timeout_ms,
        }
    }
}
```

### Trait Object with Send + Sync

```rust
// For use across threads (e.g., in async runtime)
type SharedStrategy = Box<dyn Strategy + Send + Sync>;

// Arc for shared ownership across tasks
type ArcStrategy = std::sync::Arc<dyn Strategy + Send + Sync>;

struct AsyncService {
    strategy: ArcStrategy,
}
```

### Default Strategy via Generics

```rust
struct NoCompression;

impl CompressionStrategy for NoCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
    fn name(&self) -> &str { "none" }
}

// Default type parameter
struct Pipeline<C: CompressionStrategy = NoCompression> {
    compressor: C,
}

impl Pipeline {
    fn new() -> Self {
        Pipeline { compressor: NoCompression }
    }
}

impl<C: CompressionStrategy> Pipeline<C> {
    fn with_compression(compressor: C) -> Self {
        Pipeline { compressor }
    }
}
```

---

## References

- [The Rust Programming Language, Ch. 17.2 - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [The Rust Reference - Traits](https://doc.rust-lang.org/reference/items/traits.html)
- [The Rust Reference - Trait Objects](https://doc.rust-lang.org/reference/types/trait-object.html)
- [The Rust Reference - Closures](https://doc.rust-lang.org/reference/types/closure.html)
- [Rust by Example - Traits](https://doc.rust-lang.org/rust-by-example/trait.html)
- [The Rustonomicon - Representing Dynamically Sized Types](https://doc.rust-lang.org/nomicon/exotic-sizes.html#dynamically-sized-types-dsts)
