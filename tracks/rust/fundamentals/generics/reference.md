# Rust Reference — Generics

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/items/generics.html) and
> [The Rust Programming Language](https://doc.rust-lang.org/book/ch10-00-generics.html) for the
> `generics` module. Covers: generic parameters, trait bounds, where clauses, monomorphization,
> const generics, turbofish, PhantomData, and lifetime bounds.

---

## Generic Parameters

Source: [reference/items/generics.html](https://doc.rust-lang.org/reference/items/generics.html)

Generic parameters can appear on functions, type aliases, structs, enums, traits, and impl blocks.
There are three kinds of generic parameter: **lifetime**, **type**, and **const**.

### Syntax

```
GenericParams:
    < >
  | < (GenericParam ,)* GenericParam ,? >

GenericParam:
    OuterAttribute* (
        LifetimeParam
      | TypeParam
      | ConstParam
    )

TypeParam:
    IDENTIFIER ( : TypeParamBounds? )? ( = Type )?

ConstParam:
    const IDENTIFIER : Type ( = Block | IDENTIFIER | -? LITERAL )?
```

### Type Parameters

```rust
fn identity<T>(x: T) -> T { x }

struct Pair<T> {
    first: T,
    second: T,
}

enum Either<L, R> {
    Left(L),
    Right(R),
}
```

- Type parameters are in scope for the entire item they parameterize.
- Each use of the function/type with a different concrete type is a distinct **instantiation**.
- By convention, single uppercase letters (`T`, `U`, `K`, `V`) or short names in CamelCase.

### Defaults

Type parameters may have defaults:

```rust
struct HashMap<K, V, S = std::collections::hash_map::RandomState> {
    // S defaults to RandomState
}
```

Defaults are used when the type argument is omitted at the use site.

---

## Trait Bounds

Source: [reference/trait-bounds.html](https://doc.rust-lang.org/reference/trait-bounds.html)

**Trait bounds** constrain which types a generic parameter accepts.

### Syntax

```
TypeParamBounds:
    TypeParamBound ( + TypeParamBound )* +?

TypeParamBound:
    Lifetime | TraitBound

TraitBound:
    ?? ForLifetimes? TypePath
  | ( ?? ForLifetimes? TypePath )
```

### Single Bound

```rust
fn print<T: std::fmt::Display>(value: T) {
    println!("{}", value);
}
```

### Multiple Bounds (`+`)

```rust
fn debug_and_display<T: std::fmt::Debug + std::fmt::Display>(value: T) {
    println!("{:?} / {}", value, value);
}
```

### `?Sized` — Relaxing the `Sized` Bound

All type parameters implicitly require `Sized`. Use `?Sized` to accept unsized types:

```rust
fn print_it<T: std::fmt::Display + ?Sized>(t: &T) {
    println!("{}", t);
}
print_it("a string slice");  // str is not Sized
```

### Bound on Associated Types

```rust
fn process_iter<I>(iter: I) -> Vec<String>
where
    I: Iterator,
    I::Item: std::fmt::Display,
{
    iter.map(|x| format!("{}", x)).collect()
}
```

---

## Where Clauses

Source: [reference/items/where-clauses.html](https://doc.rust-lang.org/reference/items/generics.html#where-clauses)

`where` clauses express bounds outside the angle brackets.

### Syntax

```
WhereClause:
    where ( WhereClauseItem , )* WhereClauseItem?

WhereClauseItem:
    LifetimeWhereClauseItem
  | TypeBoundWhereClauseItem

TypeBoundWhereClauseItem:
    ForLifetimes? Type : TypeParamBounds?
```

### Required for

1. **Bounds on associated types:**
   ```rust
   fn f<I: Iterator>(iter: I) where I::Item: Clone { ... }
   ```
2. **Bounds on compound types:**
   ```rust
   fn f<T>(v: Vec<T>) where Vec<T>: std::fmt::Debug { ... }
   ```
3. **Complex multi-parameter bounds** (readability):
   ```rust
   fn f<T, U>() where T: Clone + Send, U: Into<T> + 'static { ... }
   ```

Where clauses and inline bounds are semantically equivalent. The compiler treats both identically.

---

## Monomorphization

Source: [reference/items/generics.html#monomorphization](https://doc.rust-lang.org/reference/items/generics.html)

> The Rust Reference: "When a generic item is instantiated, the compiler creates specialized versions
> of the item with the type arguments substituted in. This process is called *monomorphization*."

- Each unique instantiation becomes a separate item in the compiled output.
- Monomorphization enables inlining and optimization of generic code as if it were hand-written for each type.
- Compile time and binary size increase with the number of distinct instantiations.

```rust
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T { a + b }

// These calls generate two distinct functions in the binary:
add(1i32, 2i32);   // add::<i32>
add(1.0f64, 2.0);  // add::<f64>
```

### Contrast with `dyn Trait` (Dynamic Dispatch)

| | Generics (`impl Trait`) | Trait objects (`dyn Trait`) |
|---|---|---|
| Dispatch | Static (direct call or inlined) | Dynamic (vtable pointer lookup) |
| Code generation | One copy per concrete type | One shared copy |
| Inline-able | Yes | No |
| Heterogeneous collections | No | Yes (`Vec<Box<dyn Trait>>`) |
| Object-safe trait required | No | Yes |
| `Sized` constraint | Required | Relaxed (`?Sized`) |

---

## Const Generics

Source: [reference/items/generics.html#const-generics](https://doc.rust-lang.org/reference/items/generics.html)

Const generic parameters allow values (not just types) as generic arguments. Stabilized in Rust 1.51.

### Allowed Types

As of Rust 1.79, const generic parameters may be any of:
- Integer types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- `bool`
- `char`

### Syntax

```rust
struct ArrayWrapper<T, const N: usize> {
    data: [T; N],
}

fn first<T: Copy, const N: usize>(arr: [T; N]) -> T {
    arr[0]
}
```

### Const Generic Expressions

The const parameter can be used in expressions within the type:

```rust
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}
```

### Default Const Parameters

```rust
struct Buffer<T, const N: usize = 1024> {
    data: [T; N],
}

let default_buf: Buffer<u8> = Buffer { data: [0u8; 1024] };  // N = 1024
let small_buf: Buffer<u8, 64> = Buffer { data: [0u8; 64] };  // N = 64
```

---

## Turbofish (`::<>`)

Source: [reference/expressions/call-expr.html](https://doc.rust-lang.org/reference/expressions/call-expr.html)

When generic type arguments cannot be inferred, they must be supplied explicitly using the **turbofish** syntax: `function_name::<Type>(args)`.

```rust
// Without turbofish — type inferred from context:
let v: Vec<i32> = (1..=5).collect();

// With turbofish — type supplied explicitly:
let v = (1..=5).collect::<Vec<i32>>();

// Turbofish on free functions:
"42".parse::<i32>().unwrap();

// Turbofish on associated functions:
let v = Vec::<String>::new();
```

### Rule

Turbofish syntax is `::<Types>` placed after the function or method name and before the argument list. It cannot be omitted when the compiler cannot infer the type — the error will say "type annotations needed."

---

## PhantomData

Source: [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)

`std::marker::PhantomData<T>` is a zero-sized marker type used to indicate that a struct is logically associated with type `T` without storing it.

### Size

```rust
use std::mem::size_of;
use std::marker::PhantomData;

assert_eq!(size_of::<PhantomData<u8>>(), 0);
assert_eq!(size_of::<PhantomData<Vec<String>>>(), 0);
// PhantomData always has zero size regardless of T
```

### When Required

A struct with an unused type parameter will not compile without `PhantomData`:

```rust
// ERROR: parameter `T` is never used
struct Dangerous<T> {
    value: u64,
}

// OK: PhantomData makes the unused parameter explicit
struct Safe<T> {
    value: u64,
    _marker: PhantomData<T>,
}
```

### Variance Implications

The choice of `PhantomData<T>` variant affects variance:

| Usage | Variance of T |
|---|---|
| `PhantomData<T>` | Covariant — `Safe<Dog>` can be used where `Safe<Animal>` is expected |
| `PhantomData<*mut T>` | Invariant — `Safe<T>` and `Safe<U>` are never compatible |
| `PhantomData<fn() -> T>` | Covariant in return position |
| `PhantomData<fn(T)>` | Contravariant |

For most use cases, `PhantomData<T>` (covariant) is correct.

### Drop Check

`PhantomData<T>` also participates in drop check — if your struct logically owns a `T` (will drop it), use `PhantomData<T>`. If it holds a reference to a T without owning it, use `PhantomData<&'a T>` or `PhantomData<*const T>`.

---

## Lifetime Bounds in Generics

Source: [reference/trait-bounds.html#lifetime-bounds](https://doc.rust-lang.org/reference/trait-bounds.html)

Lifetime bounds constrain how long a generic type must live.

### `T: 'a` — Outlives Bound

```rust
struct Ref<'a, T: 'a> {
    reference: &'a T,
}
```

`T: 'a` means "T must be valid for at least the lifetime `'a`." In practice: if T contains references, those references must live at least as long as `'a`.

### `T: 'static`

```rust
fn store_forever<T: 'static>(value: T) { ... }
```

`T: 'static` means T contains no borrowed references with finite lifetimes. Owned types like `String`, `Vec<T>`, and `i32` satisfy `'static`. `&'a str` does not (unless `'a = 'static`).

Note: `T: 'static` does not mean T lives forever — it means T is not constrained by a short-lived borrow. You can still drop a `T: 'static` early.

### Lifetime Parameters Combined with Type Parameters

```rust
fn longest_prefix<'a, T>(haystack: &'a str, needle: T) -> &'a str
where
    T: AsRef<str>,
{
    let n = needle.as_ref();
    if haystack.starts_with(n) { &haystack[..n.len()] } else { "" }
}
```

---

## Object Safety

Source: [reference/items/traits.html#object-safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is **object-safe** if it can be used as a `dyn Trait` (trait object). Generic methods on a trait make it **not** object-safe.

### Rules for Object Safety

A trait is object-safe if:
- It has no generic methods (methods parameterized by a type `<T>`)
- It has no `where Self: Sized` methods
- Its associated functions all have `&self` or `&mut self` as receiver

```rust
// Object-safe trait
trait Serializer {
    fn serialize(&self, data: &[u8]) -> Vec<u8>;
}

// NOT object-safe — generic method makes it non-dispatchable at runtime
trait BadTrait {
    fn process<T>(&self, value: T);  // can't put in Box<dyn BadTrait>
}
```

If you need to use a trait both as a generic bound and as a `dyn Trait`, keep generic methods in a separate trait or use associated types instead.

---

## Standard Library Generic Types (Reference)

### Commonly Used Generic Standard Library Types

| Type | Parameters | Description |
|---|---|---|
| `Vec<T>` | `T` | Growable array |
| `Option<T>` | `T` | Optional value |
| `Result<T, E>` | `T`, `E` | Fallible result |
| `Box<T>` | `T` | Heap-allocated value |
| `Rc<T>` | `T` | Reference-counted pointer |
| `Arc<T>` | `T: Send + Sync` | Atomic reference-counted pointer |
| `Cell<T>` | `T: Copy` | Interior mutability (copy types) |
| `RefCell<T>` | `T` | Interior mutability (runtime borrow check) |
| `HashMap<K, V, S>` | `K: Eq + Hash`, `V`, `S` | Hash map |
| `BTreeMap<K, V>` | `K: Ord`, `V` | Sorted map |
| `PhantomData<T>` | `T` | Zero-sized marker |

### Commonly Used Generic Traits

| Trait | Type Params | Meaning |
|---|---|---|
| `From<T>` | `T` | Infallible conversion from T |
| `Into<T>` | `T` | Infallible conversion into T |
| `TryFrom<T>` | `T` | Fallible conversion from T |
| `AsRef<T>` | `T: ?Sized` | Borrow as reference to T |
| `Fn(T) -> U` | `T`, `U` | Callable (immutable) |
| `Iterator<Item=T>` | `T` | Sequence of T |
| `Default` | — | Default value |
| `PartialOrd` | — | Partial ordering |
| `Add<Rhs>` | `Rhs` | Addition operator |
