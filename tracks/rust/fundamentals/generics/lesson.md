# Generics — Rust

## How Generics Work Under the Hood

### Monomorphization: Zero-Cost Is Literal

In most languages, "generics" means one of two things at runtime: either the language boxes everything into a common representation (Java's type erasure, Go pre-1.18 interfaces), or it uses runtime type information to dispatch correctly (some dynamic languages). Rust does neither.

Rust uses **monomorphization**: for every unique combination of type arguments you use, the compiler generates a completely separate, concrete version of that code. There is no shared runtime implementation. There are no virtual calls. The binary contains what looks exactly like hand-written, type-specific code.

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// When you call this with i32 and f64...
largest(&[34i32, 50, 25, 100]);
largest(&[3.5f64, 2.1, 9.9]);
```

The compiler emits two functions: `largest_i32` and `largest_f64`. Neither is generic at runtime. Both are fully inlined, type-specialized, identical in performance to what you'd write by hand. This is what "zero-cost abstraction" means — the abstraction costs nothing at runtime because it ceases to exist at runtime.

**Comparison to Go generics (1.18+):** Go's generics use a hybrid approach called GC-shape stenciling. Go creates one version per "GC shape" — roughly, one version per pointer layout. Multiple types with the same pointer shape share an implementation with runtime dictionary lookups for type-specific operations. This is simpler and compiles faster, but adds slight overhead for operations that differ by type (like calling methods). Rust's full monomorphization has no overhead but increases compile times and binary size.

**Comparison to TypeScript:** TypeScript generics are entirely erased at compile time. The JS runtime sees none of it — just plain objects and functions. TypeScript generics are a compile-time correctness tool, not a performance tool.

### Your notes
<!-- -->


---

## Generic Functions

### The Syntax

```rust
fn function_name<T>(param: T) -> T {
    param
}
```

`T` is a **type parameter** — a placeholder for a concrete type the caller provides. By convention, single-letter uppercase names (`T`, `U`, `K`, `V`) are used, though descriptive names are allowed (`Item`, `Error`).

Without any bounds, `T` is completely opaque — you can't do anything with it except move it around:

```rust
fn identity<T>(x: T) -> T {
    x  // can only return it; can't compare, print, or compute with it
}
```

### Trait Bounds

To actually use `T` in a meaningful way, you constrain it with **trait bounds**. A bound says: "T must implement this trait."

```rust
// T must implement Display to be formatted with {}
fn log_value<T: std::fmt::Display>(label: &str, value: T) {
    println!("[{}] {}", label, value);
}

log_value("timeout_ms", 5000);         // T = i32
log_value("service_name", "payments"); // T = &str
```

Without the `Display` bound, `println!("{}", value)` would fail to compile — the compiler can't guarantee that `T` has a `Display` implementation.

### Multiple Bounds with `+`

A type parameter can require multiple traits simultaneously:

```rust
use std::fmt::{Debug, Display};

fn log_and_return<T: Display + Debug + Clone>(value: T) -> T {
    println!("display: {}", value);
    println!("debug:   {:?}", value);
    value.clone()
}
```

`T: Display + Debug + Clone` means: T must implement all three. The type must satisfy every bound.

### Your notes
<!-- -->


---

## Where Clauses

When bounds get complex, inline syntax becomes hard to read. `where` clauses move the bounds after the function signature:

```rust
// Hard to read inline:
fn serialize_and_hash<T: serde::Serialize + std::hash::Hash + Clone + Send>(item: T) -> u64 { ... }

// Much cleaner with where:
fn serialize_and_hash<T>(item: T) -> u64
where
    T: serde::Serialize + std::hash::Hash + Clone + Send,
{
    // ...
    0 // placeholder
}
```

`where` clauses are required when:
- Bounds involve associated types: `T: Iterator<Item = String>`
- Multiple parameters each have bounds
- A bound applies to a compound type (e.g., `Vec<T>: Display`)

```rust
fn process_pairs<K, V>(pairs: Vec<(K, V)>) -> std::collections::HashMap<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    pairs.into_iter().collect()
}
```

**Style rule:** Prefer `where` clauses once you have more than two bounds or more than one generic parameter. The function signature communicates intent; `where` handles mechanics.

### Your notes
<!-- -->


---

## Generic Structs and Enums

### Generic Structs

Structs can be parameterized over one or more types:

```rust
// A simple typed wrapper
struct TypedId<T> {
    value: u64,
    _phantom: std::marker::PhantomData<T>,  // explained in the PhantomData section
}

// A key-value pair — two type parameters
struct CacheEntry<K, V> {
    key: K,
    value: V,
    expires_at: u64,
}

let entry: CacheEntry<String, Vec<u8>> = CacheEntry {
    key: String::from("session:abc123"),
    value: vec![1, 2, 3],
    expires_at: 9999999,
};
```

The struct definition does not require bounds — bounds are only needed on `impl` blocks or functions that actually use the constrained behavior.

### Methods on Generic Structs

```rust
struct Queue<T> {
    items: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

// Add a method only when T: Display
impl<T: std::fmt::Display> Queue<T> {
    pub fn print_all(&self) {
        for item in &self.items {
            println!("{}", item);
        }
    }
}
```

The `impl<T>` declares the type parameter for the block. Methods that need additional bounds use their own `where` clause or inline bounds.

### Generic Enums: Option and Result

You already use the two most important generic enums daily. Looking at their definitions shows how expressive generic enums are:

```rust
// The actual standard library definitions
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Option<T>` says: "there might be a value of type T, or there might be nothing." `Result<T, E>` says: "either success with a T, or failure with an E." Two type parameters, two failure modes, one elegant type.

You can define your own multi-variant generic enums:

```rust
// A value that's either immediately available or pending computation
enum Deferred<T> {
    Ready(T),
    Pending { task_id: u64 },
}
```

### Your notes
<!-- -->


---

## PhantomData: Type-Level Programming

### The Problem

Sometimes you want a struct to be "typed" by a type it doesn't actually store. This comes up when you're encoding type information for compile-time safety — preventing programmers from confusing, say, a user ID with a product ID even though both are `u64`.

```rust
struct UserId(u64);
struct ProductId(u64);

fn get_user(id: UserId) -> User { ... }

// Without typed IDs, this compiles:
let product_id: u64 = 42;
get_user(product_id); // wrong! passing a product ID to a user function
```

With newtypes, the above fails at compile time. But sometimes you want to be more flexible — encoding the "type" as a generic parameter rather than a dedicated newtype per entity.

### PhantomData to the Rescue

`PhantomData<T>` is a zero-sized type that tells the compiler "this struct logically contains a T, even though no T is stored." It exists purely to influence type checking and variance.

```rust
use std::marker::PhantomData;

// A typed identifier — same runtime representation as u64, but compile-time distinct
struct Id<Entity> {
    value: u64,
    _marker: PhantomData<Entity>,  // zero size; only affects type checker
}

// Marker types (never instantiated)
struct User;
struct Product;
struct Order;

type UserId    = Id<User>;
type ProductId = Id<Product>;
type OrderId   = Id<Order>;

fn find_user(id: UserId) -> &'static str { "user data" }

let user_id:    UserId    = Id { value: 1, _marker: PhantomData };
let product_id: ProductId = Id { value: 1, _marker: PhantomData };

find_user(user_id);     // OK
// find_user(product_id); // ERROR: expected Id<User>, found Id<Product>
```

Two IDs with the same runtime value, distinct at compile time. Zero overhead.

### When PhantomData Is Required

If you define `struct Wrapper<T>` but `T` doesn't appear in any field, the compiler rejects it — it doesn't know the variance semantics or drop behavior for `T`. `PhantomData<T>` tells the compiler you're logically holding a `T` and opts into standard variance rules.

### Your notes
<!-- -->


---

## Const Generics

### Arrays with Compile-Time Sizes

Rust's type system treats `[i32; 4]` and `[i32; 8]` as different types — the size is part of the type. This used to be awkward for writing generic array code. Const generics, stabilized in Rust 1.51, solve this.

```rust
// N is a const generic parameter — a compile-time integer
fn sum_array<T, const N: usize>(arr: [T; N]) -> T
where
    T: std::ops::Add<Output = T> + Default + Copy,
{
    arr.iter().fold(T::default(), |acc, &x| acc + x)
}

let result = sum_array([1u32, 2, 3, 4]);  // T = u32, N = 4
println!("{}", result);  // 10
```

The type `[T; N]` only exists when `N` is a compile-time constant. This is different from a runtime slice `&[T]`.

### Fixed-Size Buffers

Const generics shine for fixed-capacity data structures that avoid heap allocation:

```rust
// A stack-allocated ring buffer — capacity is baked into the type
struct RingBuffer<T, const CAP: usize> {
    data: [Option<T>; CAP],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Copy, const CAP: usize> RingBuffer<T, CAP> {
    pub fn new() -> Self {
        RingBuffer {
            data: [None; CAP],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.len == CAP {
            return false;  // full
        }
        self.data[self.tail] = Some(item);
        self.tail = (self.tail + 1) % CAP;
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let item = self.data[self.head].take();
        self.head = (self.head + 1) % CAP;
        self.len -= 1;
        item
    }

    pub fn capacity() -> usize { CAP }
    pub fn len(&self) -> usize { self.len }
}

// Different capacities = different types
let mut small: RingBuffer<u32, 8>  = RingBuffer::new();
let mut large: RingBuffer<u32, 64> = RingBuffer::new();
```

`RingBuffer<u32, 8>` and `RingBuffer<u32, 64>` are unrelated types. The capacity is encoded in the type, not stored at runtime.

### Const Generic Bounds

You can constrain const generics, though the syntax is a bit limited today:

```rust
// Ensure N is at least 1 (checked at compile time via where clause)
fn first_element<T: Copy, const N: usize>(arr: [T; N]) -> T
where
    [T; N]: Sized,  // technically always true; just for illustration
{
    arr[0]  // safe because N >= 1... but the compiler doesn't verify N > 0 yet
}
```

Full const generic arithmetic and bounds are still evolving in Rust. For now, `const N: usize` with `[T; N]` patterns are the main use case.

### Your notes
<!-- -->


---

## Turbofish Syntax

### When Type Inference Fails

Rust's type inference is powerful but occasionally needs help. When a function or method has a generic return type but no arguments constrain it, you must provide the type explicitly. The `::<Type>` syntax is called **turbofish**.

```rust
// parse() is generic over the return type — T: FromStr
let port: u16 = "8080".parse().unwrap();       // type from annotation
let port        = "8080".parse::<u16>().unwrap(); // turbofish

// collect() is generic over the collection type
let v: Vec<i32> = (1..=5).collect();           // type from annotation
let v = (1..=5).collect::<Vec<i32>>();         // turbofish
```

Both styles compile. Turbofish is useful when the type annotation would be far away from the call site, or when calling a method in a chain where an intermediate type needs to be specified.

### Turbofish on Free Functions

```rust
fn parse_config<T: std::str::FromStr>(value: &str) -> Option<T> {
    value.parse().ok()
}

// Explicit type via turbofish
let timeout = parse_config::<u32>("30");
let host    = parse_config::<String>("localhost");
```

### Turbofish on Struct Methods

```rust
let v = Vec::<String>::new();  // explicit type in Vec::new()
let set = std::collections::HashSet::<i32>::new();
```

The rule: turbofish goes **after the function/method name, before the arguments**, using `::< >`.

### When NOT to Use Turbofish

If the type can be inferred from a variable annotation or from how the return value is used downstream, prefer the annotation. Turbofish is for the cases where inference genuinely cannot determine the type.

### Your notes
<!-- -->


---

## Lifetime Parameters in Generics

Lifetimes are just another kind of generic parameter. A brief look — the memory module covers this in depth.

```rust
// 'a is a lifetime parameter: "T lives at least as long as 'a"
struct StrSplit<'a> {
    remainder: &'a str,
    delimiter: &'a str,
}

// Generic over both a type T and a lifetime 'a
struct Ref<'a, T: 'a> {
    reference: &'a T,
}
```

`T: 'a` is a **lifetime bound**: "whatever T is, it must live at least as long as `'a`." This appears in generic structs that hold references to `T`.

When you combine lifetime parameters with type parameters:

```rust
fn longest_with_announcement<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where
    T: std::fmt::Display,
{
    println!("Announcing: {}", ann);
    if x.len() > y.len() { x } else { y }
}
```

`'a` governs how long the returned reference lives. `T: Display` governs what `ann` can do. Both are generic parameters; both are resolved at compile time.

**Cross-reference:** [[fundamentals/rust/memory-and-ownership]] covers lifetime elision rules, explicit lifetime annotations, and when you actually need to write `'a` vs when the compiler infers it.

### Your notes
<!-- -->


---

## When NOT to Use Generics

Generics are not free. They have costs — and knowing when to avoid them is as important as knowing how to use them.

### Readability Cost

A function with four generic parameters and complex where clauses is hard to understand:

```rust
// Too generic — nobody wants to read this
fn process<T, U, F, G>(items: Vec<T>, f: F, g: G) -> Vec<U>
where
    T: Clone + Send + std::fmt::Debug,
    U: From<T> + serde::Serialize,
    F: Fn(T) -> Option<T>,
    G: Fn(T) -> U,
{
    // ...
}
```

If your generic function is harder to understand than two specific functions, write two specific functions. The language encourages generics; good engineering sometimes pushes back.

### Compile Time Cost

Each unique set of type arguments creates a new copy of the function in the compiler's work queue. A heavily generic codebase compiles noticeably slower than one that uses trait objects judiciously. This is real — the Rust ecosystem has ongoing discussions about managing compile times in large generic libraries.

**When to prefer `dyn Trait` over `impl Trait`:**
- Heterogeneous collections (you need `Vec<Box<dyn Processor>>`, not `Vec<Processor<T>>`)
- Plugin systems where concrete types are unknown at compile time
- When you're hitting compile time walls on a large codebase
- When the runtime cost of a vtable call is irrelevant (not a hot path)

### Binary Size Cost

Monomorphization increases binary size. `Vec<String>`, `Vec<i32>`, and `Vec<Vec<u8>>` each generate their own copy of all Vec methods. For embedded systems or WASM where binary size matters, this is a real concern.

### The Practical Rule

**Use generics when:**
- The abstraction has zero runtime cost and the compiler can inline aggressively
- You're writing library code that must work with any type the caller chooses
- The semantic constraint (trait bound) adds correctness guarantees

**Use trait objects (`dyn Trait`) when:**
- You have genuinely heterogeneous data in a collection
- You need runtime polymorphism (plugin architecture, strategies loaded from config)
- Compile time or binary size is a concern

**Use concrete types when:**
- You only have one or two callers and they're internal
- Generics make the code harder to understand without meaningful benefit
- You're in a hot path where inlining matters less than code clarity

### Your notes
<!-- -->


---

## Cross-Language Comparison

### Go Generics vs Rust Generics

Go added generics in 1.18. The designs are meaningfully different.

```go
// Go: type constraints are interface types
type Ordered interface {
    ~int | ~float64 | ~string  // union of underlying types
}

func Max[T Ordered](a, b T) T {
    if a > b { return a }
    return b
}
```

```rust
// Rust: trait bounds express the same constraint
fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

| Aspect | Go | Rust |
|---|---|---|
| Constraint mechanism | Interface with type unions | Trait bounds |
| Type union support | Yes (`~int \| ~float64`) | Limited (`num-traits` crate) |
| Monomorphization | Partial (GC-shape stenciling) | Full (per-type specialization) |
| Runtime overhead | Small (dictionary lookup in some cases) | Zero (fully monomorphized) |
| Compile time | Faster (less code generation) | Slower (full codegen per type) |
| Where clauses | No equivalent | `where T: Trait` |
| Const generics | No | Yes (`const N: usize`) |
| Variance | N/A (no PhantomData) | Manual (PhantomData) |

Go's constraints are simpler to write and reason about. Rust's trait system is more expressive — a trait can have associated types, default implementations, and blanket implementations that Go interfaces cannot.

### TypeScript Generics vs Rust Generics

TypeScript generics are structurally typed (duck typing). Rust generics are nominally typed (explicit trait implementations).

```typescript
// TypeScript: structural — T is constrained by shape
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
    return obj[key];
}

// Conditional types — no Rust equivalent
type NonNullable<T> = T extends null | undefined ? never : T;

// Mapped types — no Rust equivalent
type Readonly<T> = { readonly [K in keyof T]: T[K] };
```

```rust
// Rust: nominal — T must explicitly implement a trait
fn get_display<T: std::fmt::Display>(value: T) -> String {
    format!("{}", value)
}

// No conditional/mapped types — Rust uses associated types instead
trait Container {
    type Item;  // associated type
    fn get(&self) -> &Self::Item;
}
```

| Aspect | TypeScript | Rust |
|---|---|---|
| Typing style | Structural | Nominal |
| Conditional types | Yes (`T extends U ? A : B`) | No |
| Mapped types | Yes (`{ [K in keyof T]: ... }`) | No; use proc macros |
| Const generics | No | Yes |
| Erased at runtime | Fully erased | Fully monomorphized |
| Associated types | No (use generics) | Yes (trait associated types) |
| PhantomData equivalent | No need (structural typing) | Required for unused parameters |

TypeScript's conditional and mapped types enable sophisticated type-level transformations (the entire `type-fest` library). Rust doesn't have these, but associated types, const generics, and proc macros cover most of the same ground — differently.

### Your notes
<!-- -->
