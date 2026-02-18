# Rust Reference -- Factory Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Programming Language](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/).
> Covers: trait objects, `impl Trait`, `From`/`Into` traits, associated types, object safety.

---

## Trait Objects and Dynamic Dispatch

Source: [Rust Reference -- Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html)

A trait object is an opaque value of another type that implements a set of traits. The set of traits is made up of an object-safe base trait plus any number of auto traits.

Trait objects are written as `dyn Trait` (or `dyn Trait + AutoTrait1 + AutoTrait2`).

```rust
// Trait object behind a Box (owned, heap-allocated)
let storage: Box<dyn Storage> = Box::new(InMemoryStorage::new());

// Trait object behind a reference (borrowed)
let storage: &dyn Storage = &in_memory;

// With additional bounds
let storage: Box<dyn Storage + Send + Sync> = Box::new(InMemoryStorage::new());
```

### Memory Layout

A trait object pointer is a **fat pointer** consisting of two machine words:
- A pointer to the data
- A pointer to the vtable

The vtable contains:
- Size and alignment of the concrete type
- The destructor (`drop`)
- Function pointers for each trait method

```
Box<dyn Storage>
┌─────────────┬─────────────┐
│  data ptr   │  vtable ptr │
└──────┬──────┴──────┬──────┘
       │             │
       ▼             ▼
┌──────────┐  ┌──────────────┐
│ concrete │  │ drop()       │
│ data on  │  │ get()        │
│ heap     │  │ set()        │
└──────────┘  │ delete()     │
              │ name()       │
              └──────────────┘
```

### Object Safety

Source: [Rust Reference -- Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is object-safe if all of the following hold:

1. The trait does not require `Self: Sized`
2. All associated functions satisfy one of:
   - Have a `where Self: Sized` bound (opt-out of object safety requirement)
   - Have a receiver (first parameter is `self`, `&self`, `&mut self`, or a type that dereferences to `Self`)
   - Have no type parameters (no generics on individual methods)
   - Do not use `Self` except in the receiver position

| Allowed | Not Allowed | Reason |
|---------|-------------|--------|
| `fn get(&self, key: &str) -> String` | `fn serialize<T>(&self, val: &T)` | Generic type parameter |
| `fn clone_box(&self) -> Box<dyn Trait>` | `fn clone(&self) -> Self` | Returns `Self` by value |
| `fn name(&self) -> &str` | `fn compare(&self, other: &Self)` | `Self` in non-receiver position |

**Workaround for `Self` returns:**

```rust
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

impl<T: Clone + Cloneable + 'static> Cloneable for T {
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(self.clone())
    }
}
```

---

## `impl Trait` in Return Position

Source: [Rust Reference -- impl Trait](https://doc.rust-lang.org/reference/types/impl-trait.html)

`impl Trait` in return position means "the function returns some type that implements `Trait`, but the caller cannot know which specific type."

```rust
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}
```

### Rules and Constraints

1. **One concrete type per function:** The return type is a single, specific type that the compiler infers. Different branches cannot return different concrete types.

```rust
// DOES NOT COMPILE:
fn create(kind: &str) -> impl Display {
    match kind {
        "a" => TypeA,   // concrete type A
        "b" => TypeB,   // concrete type B -- ERROR: different type
    }
}
```

2. **Opaque type alias:** The concrete type is hidden from the caller. The caller can only use methods from the declared trait.

3. **No `dyn` coercion needed:** Unlike trait objects, `impl Trait` incurs no vtable overhead. The compiler monomorphizes the calling code.

4. **Auto traits leak:** If the concrete type is `Send`, then `impl Trait` is also `Send`, even if not explicitly declared.

### Comparison Table

| Feature | `impl Trait` (return) | `Box<dyn Trait>` |
|---------|----------------------|------------------|
| Dispatch | Static | Dynamic |
| Allocation | Stack | Heap |
| Size known at compile time | Yes (by compiler) | No (fat pointer) |
| Multiple concrete types | No | Yes |
| Caller can name the type | No | No |
| Auto trait propagation | Automatic | Must be explicit |

---

## `From` and `Into` Conversion Traits

Source: [std::convert::From](https://doc.rust-lang.org/std/convert/trait.From.html), [std::convert::Into](https://doc.rust-lang.org/std/convert/trait.Into.html)

### `From<T>`

```rust
pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}
```

- Used for **infallible** conversions
- Implementing `From<T> for U` automatically provides `Into<U> for T`
- Must not fail -- for fallible conversions, use `TryFrom`

### `Into<T>`

```rust
pub trait Into<T>: Sized {
    fn into(self) -> T;
}
```

- Prefer implementing `From` rather than `Into` (you get `Into` for free)
- Use `Into` as a bound in function signatures for flexibility:

```rust
fn connect(config: impl Into<ConnectionConfig>) -> Connection {
    let config = config.into();
    // ...
}
```

### `TryFrom<T>` and `TryInto<T>`

Source: [std::convert::TryFrom](https://doc.rust-lang.org/std/convert/trait.TryFrom.html)

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

- For **fallible** conversions
- Implementing `TryFrom<T> for U` automatically provides `TryInto<U> for T`
- The `Error` associated type specifies the error kind

### Standard Library `From` Implementations (Selected)

| From | To | Notes |
|------|----|-------|
| `&str` | `String` | `String::from("hello")` |
| `Vec<u8>` | `String` | Only via `TryFrom` (UTF-8 validation) |
| `i32` | `i64` | Widening, always safe |
| `[T; N]` | `Vec<T>` | Array to vec |
| `&[T]` | `Vec<T>` where `T: Clone` | Slice to vec |
| `Box<T>` | `T` | Unboxing via `*` |
| `T` | `Box<T>` | Boxing |

---

## Associated Types in Traits

Source: [Rust Reference -- Associated Types](https://doc.rust-lang.org/reference/items/associated-items.html#associated-types)

Associated types are type placeholders within a trait definition that implementors must specify.

```rust
trait Factory {
    type Product;
    type Error;

    fn create(&self) -> Result<Self::Product, Self::Error>;
}
```

### Associated Types vs Generic Parameters

| Feature | Associated Type | Generic Parameter |
|---------|----------------|-------------------|
| Syntax | `trait T { type Item; }` | `trait T<Item> { }` |
| Implementations per type | One | Many |
| Caller specifies? | No (fixed by impl) | Yes (at call site) |
| Use when | Each impl has exactly one output type | Same impl might work with multiple types |

```rust
// Associated type: Iterator yields ONE specific type
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Generic: FromIterator can collect FROM any iterator
trait FromIterator<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self;
}
```

### Associated Types with Bounds

```rust
trait ConnectionFactory {
    type Connection: Send + Sync;
    type Config: Default;
    type Error: std::error::Error;

    fn create(&self, config: &Self::Config) -> Result<Self::Connection, Self::Error>;
}
```

---

## The Newtype Pattern

Source: [The Rust Programming Language -- Newtype](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)

A newtype is a tuple struct with a single field, used to create a distinct type from an existing one.

```rust
struct Meters(f64);
struct Seconds(f64);
struct MetersPerSecond(f64);

impl Meters {
    fn per(self, time: Seconds) -> MetersPerSecond {
        MetersPerSecond(self.0 / time.0)
    }
}
```

Newtypes are relevant to factories because:
1. **Type-safe construction:** `Port(8080)` vs bare `u16`
2. **Implement foreign traits:** Newtype lets you impl `From` for types you don't own
3. **Zero-cost abstraction:** Newtypes are erased at compile time (same memory layout)

```rust
// Can't impl From<String> for Vec<u8> (both foreign)
// But can wrap:
struct JsonBytes(Vec<u8>);

impl From<String> for JsonBytes {
    fn from(s: String) -> Self {
        JsonBytes(s.into_bytes())
    }
}
```

---

## Default Trait

Source: [std::default::Default](https://doc.rust-lang.org/std/default/trait.Default.html)

```rust
pub trait Default: Sized {
    fn default() -> Self;
}
```

`Default` is a factory trait for creating "zero value" or "sensible default" instances. It integrates with:

- `#[derive(Default)]` for automatic implementation
- `Option::unwrap_or_default()`
- `HashMap::entry(key).or_default()`
- Builder patterns as the starting point

```rust
#[derive(Default)]
struct ServerConfig {
    host: String,        // Default: ""
    port: u16,           // Default: 0
    max_connections: u32, // Default: 0
    tls: bool,           // Default: false
}

// Custom Default with sensible values:
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            max_connections: 100,
            tls: false,
        }
    }
}
```

---

## Quick Reference: Factory-Related Traits

| Trait | Purpose | Fallible? | Standard Use |
|-------|---------|-----------|--------------|
| `From<T>` | Convert `T` into `Self` | No | `String::from("hello")` |
| `TryFrom<T>` | Convert `T` into `Self` (may fail) | Yes | `u8::try_from(256i32)` |
| `Into<T>` | Convert `Self` into `T` | No | `"hello".into(): String` |
| `TryInto<T>` | Convert `Self` into `T` (may fail) | Yes | `256i32.try_into(): Result<u8, _>` |
| `Default` | Create default instance | No | `Vec::<i32>::default()` |
| `FromStr` | Parse from string | Yes | `"127.0.0.1".parse::<IpAddr>()` |
| `FromIterator<T>` | Collect from iterator | No | `iter.collect::<Vec<_>>()` |
| `Clone` | Duplicate an instance | No | `config.clone()` |
