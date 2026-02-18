# Interfaces & Traits — Rust Reference

Extracted from: [The Rust Reference — Traits](https://doc.rust-lang.org/reference/items/traits.html) and [The Rust Book — Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)

---

## Trait Definitions

A **trait** is a collection of methods defined for an unknown type `Self`. They can access other methods declared in the same trait.

```
Syntax:
Trait :
    unsafe? trait IDENTIFIER GenericParams? ( : TypeParamBounds? )? WhereClause? {
        InnerAttribute*
        AssociatedItem*
    }
```

### Trait Items

A trait body can contain:
- **Associated functions** — methods or free functions associated with the trait
- **Associated types** — type aliases bound within the trait
- **Associated constants** — constants defined within the trait

### Method Signatures

Trait methods use the same receiver syntax as `impl` blocks:

| Receiver | Syntax | Meaning |
|----------|--------|---------|
| Immutable borrow | `&self` | `self: &Self` |
| Mutable borrow | `&mut self` | `self: &mut Self` |
| Owned | `self` | `self: Self` |
| Arbitrary self types | `self: Box<Self>` | Explicit receiver type |

---

## Implementing a Trait

```
Syntax:
Implementation :
    InherentImpl | TraitImpl

TraitImpl :
    unsafe? impl GenericParams? !? TypePath for Type
        WhereClause?
    {
        InnerAttribute*
        AssociatedItem*
    }
```

A type implements a trait by providing definitions for all required items:

```rust
trait Summary {
    fn summarize(&self) -> String;

    fn preview(&self) -> String {  // default implementation
        format!("{}...", &self.summarize()[..20])
    }
}

struct Article { content: String }

impl Summary for Article {
    fn summarize(&self) -> String {
        self.content.clone()
    }
    // preview() uses default — no need to override
}
```

---

## Trait Bounds

### Syntax

```rust
fn func<T: Trait1 + Trait2>(arg: T) { ... }

// Equivalent with where clause
fn func<T>(arg: T) where T: Trait1 + Trait2 { ... }
```

### `?Sized` Bound

By default, generic type parameters have an implicit `Sized` bound (the type's size must be known at compile time). Use `?Sized` to opt out:

```rust
fn print_it<T: fmt::Display + ?Sized>(t: &T) {
    println!("{}", t);
}
// Now works with both sized types (String) and unsized types (&str, [u8])
```

### Lifetime Bounds in Trait Bounds

```rust
fn longest<'a, T: fmt::Display + 'a>(x: &'a str, y: &'a str, ann: T) -> &'a str {
    println!("Announcement: {ann}");
    if x.len() > y.len() { x } else { y }
}
```

---

## Associated Types

Associated types connect a type placeholder with a trait such that the trait methods can use those placeholder types in their signatures.

```rust
pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}
```

### Specifying Associated Types in Bounds

```rust
// Require that the iterator's Item is a String
fn collect_strings<I>(iter: I) -> Vec<String>
where
    I: Iterator<Item = String>,
{
    iter.collect()
}
```

### Multiple Associated Types

```rust
trait Graph {
    type Node;
    type Edge;

    fn add_node(&mut self, node: Self::Node);
    fn add_edge(&mut self, from: &Self::Node, to: &Self::Node, edge: Self::Edge);
}
```

---

## Generic Traits

Unlike associated types, generic traits allow multiple implementations for the same type:

```rust
trait From<T> {
    fn from(value: T) -> Self;
}

// A type can implement From<T> for many T values:
impl From<u32> for MyType { ... }
impl From<String> for MyType { ... }
impl From<&str> for MyType { ... }
```

---

## Default Implementations

Trait methods can have default implementations. Implementing types can override them.

```rust
trait Greeting {
    fn name(&self) -> &str;

    // Default: calls another trait method
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

Default methods cannot be called from within their own overriding implementation (no direct recursion through the default).

---

## `impl Trait` Syntax

### In Argument Position

`impl Trait` in argument position is syntactic sugar for an anonymous generic type parameter with a bound:

```rust
// These are equivalent:
fn notify(item: &impl Summary) { ... }
fn notify<T: Summary>(item: &T) { ... }
```

Each `impl Trait` in a different argument position introduces a separate type parameter. Two `impl Trait` arguments may be different types:

```rust
fn compare_summaries(a: &impl Summary, b: &impl Summary) { ... }
// a and b can be different types, both implementing Summary
```

### In Return Position

`impl Trait` in return position means "some specific type implementing this trait, determined by the implementation":

```rust
fn make_notifier() -> impl Notifier {
    EmailNotifier { address: String::from("ops@example.com") }
}
```

**Restriction:** A function with `-> impl Trait` can only return one concrete type. If different branches return different types, use `Box<dyn Trait>` instead.

```rust
// ERROR: can't return different types
fn make_notifier(use_email: bool) -> impl Notifier {
    if use_email {
        EmailNotifier { ... }  // one type
    } else {
        SmsNotifier { ... }    // different type — compile error
    }
}

// OK: use Box<dyn Trait>
fn make_notifier(use_email: bool) -> Box<dyn Notifier> {
    if use_email {
        Box::new(EmailNotifier { ... })
    } else {
        Box::new(SmsNotifier { ... })
    }
}
```

---

## Trait Objects

A **trait object** is a value of a dynamically dispatched type. Trait objects use the `dyn` keyword.

### Syntax

```
DynTraitType : dyn TypeParamBounds
```

```rust
&dyn Trait           // borrowed trait object
&mut dyn Trait       // mutable borrowed trait object
Box<dyn Trait>       // owned trait object (heap-allocated)
Arc<dyn Trait>       // reference-counted trait object (thread-safe)
```

### Object Safety

A trait is **object-safe** (can be used as `dyn Trait`) when all its methods are object-safe. A method is object-safe when it:
- Does not use `Self` as a return type (except behind a pointer: `Box<Self>`, `Arc<Self>`)
- Does not use generic type parameters
- Does not have `where Self: Sized` bounds (though this can be used to *exclude* a method from the vtable)

A trait with non-object-safe methods can still be used as a trait object if the non-object-safe methods are excluded with `where Self: Sized`:

```rust
trait MyTrait {
    fn normal_method(&self);  // object-safe

    fn generic_method<T>(&self, val: T)  // NOT object-safe
    where
        Self: Sized;  // excluded from dyn vtable — callers using dyn cannot call this
}
```

### Fat Pointer Layout

A trait object (`&dyn Trait` or `Box<dyn Trait>`) is a **fat pointer**: two machine words.
- Word 1: pointer to the data
- Word 2: pointer to the vtable

The vtable contains:
- Size and alignment of the concrete type
- Drop destructor function pointer
- Function pointer for each method in the trait

---

## Supertraits

A trait can require that implementors also implement other traits (supertraits):

```rust
trait Auditable: fmt::Display + fmt::Debug {
    fn audit_id(&self) -> &str;
}
```

Any type implementing `Auditable` must also implement `Display` and `Debug`. This is a **constraint**, not inheritance — no behavior is inherited, only requirements.

### Calling Supertrait Methods

Inside a trait definition, you can call methods from the supertrait without qualification:

```rust
trait Auditable: fmt::Display {
    fn audit_id(&self) -> &str;

    fn log_to_audit_trail(&self) {
        // self.to_string() is available because Display is a supertrait
        println!("AUDIT {} ({})", self.to_string(), self.audit_id());
    }
}
```

---

## The Orphan Rule (Coherence)

Rust enforces **coherence**: at most one implementation of a trait for a given type must exist globally. The orphan rule ensures this.

**Rule:** You may implement a trait for a type only if either the trait or the type is defined in the current crate.

```rust
// OK: you define the trait
trait MyTrait { ... }
impl MyTrait for Vec<u8> { ... }   // Vec is external, but MyTrait is yours

// OK: you define the type
struct MyType;
impl std::fmt::Display for MyType { ... }  // Display is external, but MyType is yours

// NOT OK: both are external
impl std::fmt::Display for Vec<u8> { ... }  // error[E0117]
```

**Workaround:** Newtype pattern — wrap the external type.

```rust
struct Wrapper(Vec<u8>);
impl std::fmt::Display for Wrapper { ... }  // OK: Wrapper is your type
```

---

## Blanket Implementations

A **blanket implementation** implements a trait for any type satisfying some bound:

```rust
// From the standard library:
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

This says: "for any types T and U, if U implements From<T>, then T automatically gets Into<U>."

```rust
impl<T: fmt::Display> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}
```

This says: "any type implementing Display automatically gets `.to_string()`."

---

## Standard Library Trait Quick Reference

| Trait | Path | Purpose | Derivable |
|-------|------|---------|-----------|
| `Debug` | `std::fmt::Debug` | `{:?}` formatting | Yes |
| `Display` | `std::fmt::Display` | `{}` formatting | No |
| `Clone` | `std::clone::Clone` | Explicit duplication | Yes |
| `Copy` | `std::marker::Copy` | Implicit bitwise copy | Yes |
| `PartialEq` | `std::cmp::PartialEq` | `==` and `!=` | Yes |
| `Eq` | `std::cmp::Eq` | Total equality marker | Yes |
| `PartialOrd` | `std::cmp::PartialOrd` | `<`, `>`, `<=`, `>=` | Yes |
| `Ord` | `std::cmp::Ord` | Total ordering | Yes |
| `Hash` | `std::hash::Hash` | HashMap/HashSet key | Yes |
| `Default` | `std::default::Default` | Zero-value constructor | Yes |
| `From<T>` | `std::convert::From` | Infallible conversion from T | No |
| `Into<T>` | `std::convert::Into` | Infallible conversion to T | Auto-from-From |
| `TryFrom<T>` | `std::convert::TryFrom` | Fallible conversion from T | No |
| `TryInto<T>` | `std::convert::TryInto` | Fallible conversion to T | Auto-from-TryFrom |
| `Iterator` | `std::iter::Iterator` | Sequence iteration | No |
| `IntoIterator` | `std::iter::IntoIterator` | Convert to iterator (enables `for` loops) | No |
| `Add` | `std::ops::Add` | `+` operator | No |
| `Sub` | `std::ops::Sub` | `-` operator | No |
| `Mul` | `std::ops::Mul` | `*` operator | No |
| `Index` | `std::ops::Index` | `[]` operator (read) | No |
| `Deref` | `std::ops::Deref` | `*` dereference | No |
| `Drop` | `std::ops::Drop` | Destructor (RAII cleanup) | No |
| `Send` | `std::marker::Send` | Safe to transfer across threads | Auto |
| `Sync` | `std::marker::Sync` | Safe to share reference across threads | Auto |
| `Sized` | `std::marker::Sized` | Compile-time known size | Auto |

---

## `From` and `Into` — Conversion Conventions

```rust
// Implementing From<T> for U automatically provides Into<U> for T
impl From<&str> for String {
    fn from(s: &str) -> String {
        s.to_owned()
    }
}

let s: String = "hello".into();  // uses the From impl above
let s = String::from("hello");   // equivalent
```

### `TryFrom` / `TryInto` — Fallible Conversions

```rust
use std::convert::TryFrom;

impl TryFrom<i64> for Port {
    type Error = String;

    fn try_from(value: i64) -> Result<Port, String> {
        if value >= 1 && value <= 65535 {
            Ok(Port(value as u16))
        } else {
            Err(format!("port {} out of range 1..=65535", value))
        }
    }
}

let port = Port::try_from(8080_i64)?;
```

---

## Iterator Trait

```rust
pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    // Provided methods (selected):
    fn map<B, F: FnMut(Self::Item) -> B>(self, f: F) -> Map<Self, F>;
    fn filter<P: FnMut(&Self::Item) -> bool>(self, predicate: P) -> Filter<Self, P>;
    fn fold<B, F: FnMut(B, Self::Item) -> B>(self, init: B, f: F) -> B;
    fn collect<B: FromIterator<Self::Item>>(self) -> B;
    fn count(self) -> usize;
    fn any<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool;
    fn all<F: FnMut(Self::Item) -> bool>(&mut self, f: F) -> bool;
    fn find<P: FnMut(&Self::Item) -> bool>(&mut self, predicate: P) -> Option<Self::Item>;
    fn enumerate(self) -> Enumerate<Self>;
    fn take(self, n: usize) -> Take<Self>;
    fn skip(self, n: usize) -> Skip<Self>;
    fn zip<U: IntoIterator>(self, other: U) -> Zip<Self, U::IntoIter>;
    fn chain<U: IntoIterator<Item = Self::Item>>(self, other: U) -> Chain<Self, U::IntoIter>;
    fn flat_map<U, F: FnMut(Self::Item) -> U>(self, f: F) -> FlatMap<Self, U, F>
        where U: IntoIterator;
    fn sum<S: Sum<Self::Item>>(self) -> S;
    fn product<P: Product<Self::Item>>(self) -> P;
    fn max(self) -> Option<Self::Item> where Self::Item: Ord;
    fn min(self) -> Option<Self::Item> where Self::Item: Ord;
    fn peekable(self) -> Peekable<Self>;
    // ... and many more
}
```

---

## Operator Overloading (`std::ops`)

```rust
use std::ops::Add;

// Implementing Add for a custom type
impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { ... }
}

// Generic version: add Vec2 with &Vec2
impl Add<&Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: &Vec2) -> Vec2 { ... }
}
```

Complete list of operator traits:

| Operator | Trait | Method |
|----------|-------|--------|
| `+` | `Add` | `add` |
| `-` | `Sub` | `sub` |
| `*` | `Mul` | `mul` |
| `/` | `Div` | `div` |
| `%` | `Rem` | `rem` |
| `-` (unary) | `Neg` | `neg` |
| `!` | `Not` | `not` |
| `&` | `BitAnd` | `bitand` |
| `\|` | `BitOr` | `bitor` |
| `^` | `BitXor` | `bitxor` |
| `+=` | `AddAssign` | `add_assign` |
| `[]` (read) | `Index` | `index` |
| `[]` (write) | `IndexMut` | `index_mut` |
| `*` (deref) | `Deref` | `deref` |
| `*` (deref mut) | `DerefMut` | `deref_mut` |

---

## Relevant RFC and Reference Links

- [The Rust Reference: Traits](https://doc.rust-lang.org/reference/items/traits.html)
- [The Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Rust Book: Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [std::fmt — Display and Debug](https://doc.rust-lang.org/std/fmt/index.html)
- [std::convert — From/Into/TryFrom/TryInto](https://doc.rust-lang.org/std/convert/index.html)
- [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [std::ops — Operator Traits](https://doc.rust-lang.org/std/ops/index.html)
- [RFC 0195: Associated Types](https://rust-lang.github.io/rfcs/0195-associated-items.html)
- [RFC 2071: impl Trait](https://rust-lang.github.io/rfcs/2071-impl-trait-existential-types.html)
