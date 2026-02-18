# Rust Reference — Structs, Methods & Enums

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/) and
> [The Rust Book](https://doc.rust-lang.org/book/) for the `structs-methods-and-enums`
> module. Covers: struct types, enumerated types, implementations, and derive macros.

---

## Struct Types

Source: [reference/types/struct.html](https://doc.rust-lang.org/reference/types/struct.html)

A **struct type** is a nominal type defined with a name and zero or more fields. There are
three forms:

### Named-Field Struct

```rust
struct Point {
    x: f64,
    y: f64,
}
```

- Each field has a name and a type
- Fields are private to the defining module by default; use `pub` to expose them
- All fields must be initialized in a struct expression (no default zero-filling)
- Field shorthand: if a variable matches a field name, you can write just the name

```rust
let x = 1.0;
let y = 2.0;
let p = Point { x, y };   // shorthand: equivalent to Point { x: x, y: y }
```

### Struct Update Syntax

A struct expression may end with `..expr` to fill in remaining fields from another
struct of the same type:

```rust
let p2 = Point { x: 3.0, ..p };  // x overridden; y taken from p
```

Fields that are moved (non-`Copy`) transfer ownership. After the update expression,
the source struct may be partially moved — its overridden fields remain valid but
the moved fields do not.

### Tuple Struct

```rust
struct Meters(f64);
struct Pair(i32, i32);
```

- Fields accessed by position: `.0`, `.1`, etc.
- Otherwise identical to named-field structs (can have `pub` fields, methods, derives)
- The newtype pattern: a single-field tuple struct wrapping another type, used to
  enforce type distinctions without runtime cost

### Unit Struct

```rust
struct Marker;
struct PhantomEvent;
```

- Zero fields, zero size (ZST — zero-sized type)
- Has exactly one value: `Marker`
- Commonly used as trait implementations, type-level markers, and phantom types

---

## Enumerated Types

Source: [reference/types/enum.html](https://doc.rust-lang.org/reference/types/enum.html)

An **enum type** is a nominal, discriminated union type. Each variant is one of three forms:

### Unit Variant

No associated data. Like a C enum constant.

```rust
enum Direction { North, South, East, West }
```

### Tuple Variant

One or more unnamed fields (like a tuple struct).

```rust
enum Shape {
    Circle(f64),           // radius
    Rectangle(f64, f64),   // width, height
}
```

### Struct Variant

Named fields (like a named-field struct).

```rust
enum Message {
    Move { x: i32, y: i32 },
    Write { text: String },
}
```

### Memory Layout

An enum is stored as a tagged union. The compiler selects the minimum representation
that can hold all variants. For enums whose variants carry no data, the discriminant
is stored directly (typically a `u8`). For data-carrying variants, the layout is the
discriminant plus the largest variant's data, aligned to the strictest field.

The Rust reference does not stabilize the exact layout — it may be optimized. For
example, `Option<&T>` is guaranteed to have the same size as `&T` because a `null`
pointer represents `None` (null pointer optimization).

```
size_of::<Option<&str>>() == size_of::<&str>()  // guaranteed
```

### Discriminants

Discriminant values can be explicitly set for unit variants:

```rust
#[repr(u8)]
enum Status {
    Active = 1,
    Inactive = 2,
    Suspended = 100,
}
```

Without `repr`, discriminants are assigned sequentially from 0 but the representation
is unspecified. With `repr(u8)`, the layout is guaranteed.

---

## Implementations

Source: [reference/items/implementations.html](https://doc.rust-lang.org/reference/items/implementations.html)

An **implementation** (`impl`) associates functions and constants with a type. There are
two kinds:

### Inherent Implementations

```rust
impl Point {
    // Associated function — no receiver. Called as Point::new(...)
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    // Method — has a receiver parameter
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
```

- A type may have multiple inherent `impl` blocks
- Inherent methods are in scope whenever the type is in scope (no `use` needed)
- Functions in `impl` blocks are namespaced to the type: `Point::new`, not bare `new`

### Method Receivers

The first parameter of a method is the **receiver**. It determines the calling syntax
and the access level:

| Receiver form | Type | Notes |
|---|---|---|
| `self` | `Self` | Moves ownership into the method |
| `mut self` | `Self` | Moves ownership, allows mutation |
| `&self` | `&Self` | Shared borrow — read-only access |
| `&mut self` | `&mut Self` | Mutable borrow — read-write access |
| `self: Box<Self>` | `Box<Self>` | Consuming a boxed instance |
| `self: Rc<Self>` | `Rc<Self>` | Consuming a reference-counted instance |

Methods are called with `.` syntax on values. Associated functions (no receiver)
are called with `::` syntax.

```rust
let p = Point::new(1.0, 2.0);   // associated function
let d = p.distance(&origin);     // method — &self
```

### Trait Implementations

```rust
impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

A type can implement any number of traits, each in its own `impl` block.
The **orphan rule** applies: at least one of the type or the trait must be
defined in the current crate.

---

## Pattern Matching

Source: [reference/expressions/match-expr.html](https://doc.rust-lang.org/reference/expressions/match-expr.html)

### Match Expression

```rust
match scrutinee {
    pattern1 => expression1,
    pattern2 if guard => expression2,
    _ => fallback,
}
```

- **Exhaustive**: the compiler requires all possible values to be covered
- Arms are checked in declaration order; the first matching arm executes
- The `_` wildcard matches any value and binds nothing
- Match arms can bind variables, destructure structs, enums, and tuples

### Pattern Forms

| Pattern | Example | Matches |
|---|---|---|
| Wildcard | `_` | anything |
| Literal | `42`, `"hello"`, `true` | exact value |
| Variable | `x` | anything, binds to `x` |
| Tuple | `(a, b)` | tuple of matching patterns |
| Tuple struct | `Point(x, y)` | tuple struct |
| Struct | `Point { x, y }` | named-field struct |
| Enum variant | `Some(x)`, `None` | enum variant, optionally binding |
| Or | `1 \| 2 \| 3` | any of the listed patterns |
| Range | `1..=5` | inclusive range |
| Guard | `x if x > 0` | pattern plus condition |
| Reference | `&val` | dereferences and matches |
| `..` | `Point { x, .. }` | ignore remaining fields |
| `@` binding | `n @ 1..=5` | bind while testing |

### Struct Patterns

```rust
let Point { x, y } = p;           // destructure into bindings
let Point { x: a, y: b } = p;    // rename while destructuring
let Point { x, .. } = p;          // ignore y
```

### Enum Patterns

```rust
match value {
    Some(x) => println!("{x}"),   // binds inner value
    None => println!("empty"),
}

match event {
    Event::Click { x, y } => ..., // struct variant
    Event::Key(code) => ...,       // tuple variant
    Event::Close => ...,           // unit variant
}
```

### `if let` and `while let`

Source: [reference/expressions/if-expr.html](https://doc.rust-lang.org/reference/expressions/if-expr.html)

```rust
// if let — match a single refutable pattern
if let Some(x) = opt {
    println!("{x}");
} else {
    println!("none");
}

// while let — loop while pattern matches
while let Some(item) = queue.pop() {
    process(item);
}

// let...else — bind or diverge
let Some(x) = opt else { return; };
// x is now in scope and is definitely Some's inner value
```

---

## Derive Macros

Source: [reference/attributes/derive.html](https://doc.rust-lang.org/reference/attributes/derive.html)

The `derive` attribute generates trait implementations. The compiler expands it to
equivalent hand-written `impl` blocks before type-checking.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct Config {
    host: String,
    port: u16,
}
```

### Standard Derivable Traits

| Trait | Crate | Requirements on fields | Notes |
|---|---|---|---|
| `Debug` | `core` | all fields: `Debug` | Required for `{:?}` formatting |
| `Clone` | `core` | all fields: `Clone` | Required for `Copy`; enables `.clone()` |
| `Copy` | `core` | all fields: `Copy`; type not `Drop` | Enables implicit bitwise copies |
| `PartialEq` | `core` | all fields: `PartialEq` | `==` and `!=`; reflexivity not required |
| `Eq` | `core` | all fields: `Eq`; type: `PartialEq` | Marker — total equality (reflexive) |
| `PartialOrd` | `core` | all fields: `PartialOrd` | `<`, `>`, `<=`, `>=` |
| `Ord` | `core` | all fields: `Ord`; type: `Eq` + `PartialOrd` | Total ordering; needed for `BTreeMap` keys |
| `Hash` | `core` | all fields: `Hash` | Needed for `HashMap`/`HashSet` keys (with `Eq`) |
| `Default` | `core` | all fields: `Default` | Generates zero-like constructor |

### PartialOrd on Enums

For enums, `PartialOrd` compares by discriminant value first (declaration order),
then lexicographically by fields if both variants are identical:

```rust
#[derive(PartialEq, PartialOrd)]
enum Priority { Low, Medium, High }

assert!(Priority::Low < Priority::High);     // by variant order
```

### Default

`Default::default()` returns a "zero value" for the type. For structs, each field
gets its own `Default` value. The derived implementation calls `Default::default()`
on each field:

```rust
#[derive(Default)]
struct Config {
    host: String,   // ""
    port: u16,      // 0
    debug: bool,    // false
}

let c = Config::default();
// Config { host: "", port: 0, debug: false }
```

You can also implement `Default` manually when the zero value needs to be non-trivial:

```rust
impl Default for Config {
    fn default() -> Self {
        Config {
            host: String::from("localhost"),
            port: 8080,
            debug: false,
        }
    }
}
```

---

## Struct Expressions and Field Access

Source: [reference/expressions/struct-expr.html](https://doc.rust-lang.org/reference/expressions/struct-expr.html)

### Construction

```rust
// Named fields
let p = Point { x: 1.0, y: 2.0 };

// Field shorthand (when variable name matches field name)
let x = 1.0;
let p = Point { x, y: 2.0 };

// Struct update from another value of the same type
let p2 = Point { x: 3.0, ..p };
```

### Field Access

```rust
let p = Point { x: 1.0, y: 2.0 };
let x = p.x;                   // field access
let y = p.y;

// Tuple struct access
let m = Meters(42.0);
let v = m.0;
```

### Destructuring in Let

```rust
let Point { x, y } = p;      // binds x and y
let (a, b, c) = (1, 2, 3);   // tuple destructuring
```

---

## Standard Library Enums

### `Option<T>`

Source: [std::option](https://doc.rust-lang.org/std/option/)

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

Key methods:

| Method | Signature | Returns |
|---|---|---|
| `is_some` | `(&self) -> bool` | `true` if `Some` |
| `is_none` | `(&self) -> bool` | `true` if `None` |
| `unwrap` | `(self) -> T` | inner value, **panics** on `None` |
| `unwrap_or` | `(self, default: T) -> T` | inner or default |
| `unwrap_or_else` | `(self, f: impl FnOnce() -> T) -> T` | inner or computed default |
| `map` | `(self, f: impl FnOnce(T) -> U) -> Option<U>` | transform inner value |
| `and_then` | `(self, f: impl FnOnce(T) -> Option<U>) -> Option<U>` | flatMap |
| `ok_or` | `(self, err: E) -> Result<T, E>` | convert to `Result` |
| `as_ref` | `(&self) -> Option<&T>` | borrow without moving |
| `as_deref` | `(&self) -> Option<&T::Target>` | e.g., `Option<String>` → `Option<&str>` |

### `Result<T, E>`

Source: [std::result](https://doc.rust-lang.org/std/result/)

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Key methods:

| Method | Signature | Returns |
|---|---|---|
| `is_ok` | `(&self) -> bool` | `true` if `Ok` |
| `is_err` | `(&self) -> bool` | `true` if `Err` |
| `unwrap` | `(self) -> T` | inner value, **panics** on `Err` |
| `unwrap_err` | `(self) -> E` | error value, **panics** on `Ok` |
| `map` | `(self, f: impl FnOnce(T) -> U) -> Result<U, E>` | transform `Ok` value |
| `map_err` | `(self, f: impl FnOnce(E) -> F) -> Result<T, F>` | transform `Err` value |
| `and_then` | `(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<U, E>` | chain fallible ops |
| `ok` | `(self) -> Option<T>` | discard error, convert to `Option` |
| `?` operator | — | propagate `Err` to calling function |

---

## The `?` Operator

Source: [reference/expressions/operator-expr.html#the-question-mark-operator](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator)

The `?` operator is only allowed in functions that return `Result` or `Option`.

For `Result<T, E>`:

```rust
fn foo() -> Result<T, E> {
    let x = some_result?;
    // Desugars to:
    // let x = match some_result {
    //     Ok(v) => v,
    //     Err(e) => return Err(e.into()),
    // };
}
```

The `.into()` call means the error type is automatically converted if the function's
error type implements `From<E>`. This is how libraries compose: each layer converts
errors from lower layers via `From` impls.

For `Option<T>`:

```rust
fn foo() -> Option<T> {
    let x = some_option?;
    // Desugars to:
    // let x = match some_option {
    //     Some(v) => v,
    //     None => return None,
    // };
}
```
