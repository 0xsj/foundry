# Interfaces & Traits — Rust

## What Traits Are

A trait is a named collection of method signatures (and optionally, default implementations) that a type can commit to. When a type implements a trait, it promises that those methods exist and behave according to the contract.

If you're coming from Go, this is the closest analogue to Go's interface. The key difference: **Go interfaces are satisfied implicitly** — any type with the right methods satisfies an interface, with no declaration needed. **Rust traits require an explicit `impl` block**. You must say "this type implements this trait."

```rust
// Define the trait
trait Notifier {
    fn send(&self, message: &str) -> Result<(), String>;
}

// Implement it for a concrete type
struct EmailNotifier {
    address: String,
}

impl Notifier for EmailNotifier {
    fn send(&self, message: &str) -> Result<(), String> {
        println!("Email to {}: {}", self.address, message);
        Ok(())
    }
}
```

Compare to Go:

```go
// Go: no explicit impl declaration — just provide the method
type Notifier interface {
    Send(message string) error
}

type EmailNotifier struct { Address string }

func (e EmailNotifier) Send(message string) error {
    fmt.Printf("Email to %s: %s\n", e.Address, message)
    return nil
}
// EmailNotifier automatically satisfies Notifier — no explicit statement
```

Both approaches achieve the same polymorphism. Go's implicit satisfaction is convenient for retrofit (make an existing type satisfy a new interface without modifying it). Rust's explicit `impl` is easier to audit — you can grep for `impl Notifier` and immediately find every implementor. The orphan rule (covered below) is what makes Rust's explicit approach safe.

### Your notes
<!-- -->


---

## Default Method Implementations

Traits can provide default implementations for methods. Implementing types can use the default, override it, or both.

```rust
trait Notifier {
    // Required method — no default, every implementor must provide it
    fn send(&self, message: &str) -> Result<(), String>;

    // Default method — implementors can use this or override it
    fn send_urgent(&self, message: &str) -> Result<(), String> {
        // By default, urgent = send with a prefix
        self.send(&format!("[URGENT] {}", message))
    }

    fn name(&self) -> &str {
        "unknown"
    }
}

struct SlackNotifier {
    channel: String,
}

impl Notifier for SlackNotifier {
    // Must implement the required method
    fn send(&self, message: &str) -> Result<(), String> {
        println!("#{}: {}", self.channel, message);
        Ok(())
    }

    // Override the urgent behavior for Slack — @channel mention
    fn send_urgent(&self, message: &str) -> Result<(), String> {
        self.send(&format!("<!channel> {}", message))
    }

    // Uses the default name() — returns "unknown"
}
```

Default implementations can call other methods in the same trait, even required ones. This is how `Iterator` provides 70+ default methods (`map`, `filter`, `collect`, etc.) on top of a single required method (`next`).

```rust
// You only implement next() — you get everything else for free
impl Iterator for MyRange {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.current < self.end {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
    // .map(), .filter(), .take(), .collect() etc. all just work
}
```

### Your notes
<!-- -->


---

## Trait Bounds

Trait bounds let you write functions that accept any type as long as it implements certain traits. This is the primary mechanism for generic, reusable code.

### Basic Bounds

```rust
// This function works with any type T that implements Notifier
fn notify_all<T: Notifier>(notifiers: &[T], message: &str) {
    for n in notifiers {
        if let Err(e) = n.send(message) {
            eprintln!("failed to send: {e}");
        }
    }
}
```

The `T: Notifier` syntax reads "T must implement Notifier." The compiler monomorphizes this — it generates a separate, optimized version for each concrete `T` you actually use. No runtime overhead.

### Multiple Bounds

Use `+` to require multiple traits:

```rust
use std::fmt;

// T must implement both Notifier and Display
fn send_and_log<T: Notifier + fmt::Display>(n: &T, message: &str) {
    println!("Sending via {n}...");
    let _ = n.send(message);
}
```

### `where` Clause

When bounds get complex, `where` keeps signatures readable:

```rust
// Inline — gets hard to read with many bounds
fn send_batch<T: Notifier + fmt::Debug + Clone + Send>(notifiers: &[T], msgs: &[String]) { ... }

// where clause — cleaner
fn send_batch<T>(notifiers: &[T], msgs: &[String])
where
    T: Notifier + fmt::Debug + Clone + Send,
{
    // ...
}
```

Both forms are equivalent. Use `where` whenever the inline version obscures the function's purpose.

### Your notes
<!-- -->


---

## impl Trait vs dyn Trait: Static vs Dynamic Dispatch

This is one of Rust's most important distinctions, and it's unique to Rust in this form. You have two ways to use traits as abstractions: compile-time (static) and runtime (dynamic).

### `impl Trait` — Static Dispatch

`impl Trait` in a parameter position means "some specific type that implements this trait, determined at compile time."

```rust
// Argument position: caller passes any Notifier
fn alert(notifier: &impl Notifier, msg: &str) {
    notifier.send(msg).unwrap();
}

// Return position: caller gets a specific (opaque) type back
fn make_default_notifier() -> impl Notifier {
    EmailNotifier { address: String::from("ops@example.com") }
}
```

**What actually happens:** The compiler monomorphizes. `alert(&email_notifier, "hi")` compiles to a specific function call for `EmailNotifier`. `alert(&sms_notifier, "hi")` compiles to a different specific function call. Zero overhead — the method call is resolved at compile time and can be inlined.

**Limitation:** All values passed to a function must be the same concrete type. You cannot mix `EmailNotifier` and `SmsNotifier` in a `Vec<impl Notifier>`.

```rust
// This does NOT work — mixed types
let notifiers: Vec<impl Notifier> = vec![
    EmailNotifier { ... },
    SmsNotifier { ... },  // compile error: expected EmailNotifier
];
```

### `dyn Trait` — Dynamic Dispatch

`dyn Trait` is a trait object — a pointer to a type that implements the trait, with a vtable for runtime dispatch.

```rust
// A heterogeneous collection of notifiers
let notifiers: Vec<Box<dyn Notifier>> = vec![
    Box::new(EmailNotifier { address: String::from("ops@example.com") }),
    Box::new(SmsNotifier { number: String::from("+15551234567") }),
    Box::new(WebhookNotifier { url: String::from("https://hooks.example.com/alert") }),
];

for n in &notifiers {
    n.send("system alert").unwrap();
}
```

**What actually happens:** Each `Box<dyn Notifier>` is a fat pointer — two words: a pointer to the data and a pointer to the vtable. The vtable holds function pointers for each trait method. When you call `n.send(...)`, it dereferences the vtable and calls through the function pointer. One indirection cost per call.

**When to use which:**

| | `impl Trait` | `dyn Trait` |
|---|---|---|
| Performance | Zero overhead (monomorphized) | One vtable indirection per call |
| Flexibility | Homogeneous (single concrete type per call site) | Heterogeneous (mix types at runtime) |
| Compile time | Longer (generates code for each concrete type) | Shorter (one codepath) |
| Return type | Can return `impl Trait` from a function | Requires `Box<dyn>` or `&dyn` for returns |
| Binary size | Larger (code for each instantiation) | Smaller (one codepath) |

**Default to `impl Trait`** for bounds on generic functions. Use `dyn Trait` when you need a heterogeneous collection or a plugin/callback system where you don't know the types at compile time.

### Trait Object Ownership: `&dyn` vs `Box<dyn>`

```rust
// Borrow a trait object — caller retains ownership, lifetime limited to caller's scope
fn send_once(n: &dyn Notifier, msg: &str) {
    n.send(msg).unwrap();
}

// Box the trait object — takes ownership, can store in structs, send across threads
struct NotificationQueue {
    pending: Vec<Box<dyn Notifier>>,
}

impl NotificationQueue {
    fn add(&mut self, n: Box<dyn Notifier>) {
        self.pending.push(n);
    }
}
```

Use `&dyn Trait` when you're borrowing for the duration of a function call. Use `Box<dyn Trait>` when you need to store the trait object, return it, or pass ownership.

### Your notes
<!-- -->


---

## Object Safety

Not every trait can be used as a trait object (`dyn Trait`). A trait is **object-safe** if the compiler can build a vtable for it. The rules:

1. **No generic methods.** A method like `fn process<T>(&self, item: T)` cannot go in a vtable — the vtable would need an infinite number of entries (one per type T).
2. **No `Self` by value in return position** unless it's behind a pointer. `fn clone_self(&self) -> Self` makes the trait non-object-safe.
3. **No associated functions without `self`.** (They can be excluded with `where Self: Sized`.)

```rust
// NOT object-safe — generic method
trait Serializer {
    fn serialize<T: serde::Serialize>(&self, value: T) -> String;
}
// Box<dyn Serializer> would not compile

// Object-safe version — use &dyn Any or a concrete type
trait Serializer {
    fn serialize_str(&self, value: &str) -> String;
    fn serialize_json(&self, value: &serde_json::Value) -> String;
}
```

The compiler will tell you clearly if a trait is not object-safe when you try to use `dyn Trait`:
```
error[E0038]: the trait `Serializer` cannot be made into an object
```

### Your notes
<!-- -->


---

## Supertraits

A supertrait is a trait that another trait requires. If trait `B` has supertrait `A`, then any type implementing `B` must also implement `A`.

```rust
use std::fmt;

// Any Auditable type must also implement Display (so it can be logged)
trait Auditable: fmt::Display {
    fn audit_id(&self) -> &str;
    fn log_change(&self, field: &str, old: &str, new: &str) {
        // Can call self.to_string() because Display is guaranteed
        println!("[AUDIT] {} {} changed: {} -> {}", self, self.audit_id(), old, new);
    }
}

struct UserRecord {
    id: String,
    name: String,
    email: String,
}

// Must implement Display (the supertrait) first
impl fmt::Display for UserRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User({})", self.id)
    }
}

// Now can implement Auditable
impl Auditable for UserRecord {
    fn audit_id(&self) -> &str {
        &self.id
    }
}
```

Supertraits express "to be X, you must also be Y." It is not inheritance — `UserRecord` doesn't inherit any fields or automatic behavior from `Auditable`. It's a **constraint**: the compiler enforces that every `Auditable` implementor also implements `Display`.

This is different from Go, where you'd embed an interface or just use multiple interfaces as separate parameters. In Rust, supertraits bundle requirements together at the trait definition level.

### Your notes
<!-- -->


---

## Associated Types vs Generic Traits

There are two ways to make a trait parameterized over a type. The choice has real consequences.

### Generic Trait

A type can implement the trait multiple times with different type parameters.

```rust
trait Converter<T> {
    fn convert(&self, input: T) -> String;
}

struct Formatter;

// Can implement for multiple T values
impl Converter<i64> for Formatter {
    fn convert(&self, input: i64) -> String { input.to_string() }
}

impl Converter<bool> for Formatter {
    fn convert(&self, input: bool) -> String {
        if input { "yes".to_string() } else { "no".to_string() }
    }
}

let f = Formatter;
f.convert(42_i64);   // calls the i64 impl
f.convert(true);     // calls the bool impl
```

Use generic traits when a type should be able to implement the same trait behavior for multiple different input/output types.

### Associated Types

A type can only implement the trait once. The associated type is fixed when you implement the trait.

```rust
trait Repository {
    type Item;          // what this repo stores
    type Error;         // what errors it can produce

    fn find_by_id(&self, id: u64) -> Result<Option<Self::Item>, Self::Error>;
    fn save(&mut self, item: Self::Item) -> Result<(), Self::Error>;
}

struct UserRepository {
    store: Vec<User>,
}

impl Repository for UserRepository {
    type Item = User;
    type Error = String;

    fn find_by_id(&self, id: u64) -> Result<Option<User>, String> {
        Ok(self.store.iter().find(|u| u.id == id).cloned())
    }

    fn save(&mut self, item: User) -> Result<(), String> {
        self.store.push(item);
        Ok(())
    }
}
```

The benefit: callers who have a `T: Repository` can refer to `T::Item` and `T::Error` without extra type parameters. Compare:

```rust
// Generic trait: every use site needs an extra type parameter
fn load_or_default<T, R: Repository<T>>(repo: &R, id: u64) -> T { ... }

// Associated type: cleaner — Item is part of the trait contract
fn load_or_default<R: Repository>(repo: &R, id: u64) -> R::Item { ... }
```

**Rule of thumb:** Use associated types when the relationship is one-to-one (a repository has exactly one item type). Use generic traits when the relationship is many-to-many (a formatter can convert many different types).

The `Iterator` trait uses an associated type (`type Item`) because an iterator has exactly one item type. A `From` trait is generic (`From<T>`) because a type can convert from many different sources.

### Your notes
<!-- -->


---

## The Orphan Rule

You can only implement a trait for a type if:
- You own (defined in your crate) **the trait**, OR
- You own **the type**

You cannot implement an external trait for an external type. This is the **orphan rule**.

```rust
// Your crate defines Notifier. Implementing for Vec (external type) — OK because you own the trait.
impl Notifier for Vec<String> { ... }

// Vec is an external type, Display is an external trait — NOT OK.
impl std::fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ... }
}
// error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

**Why it exists:** Without the orphan rule, two crates could both implement `Display for Vec<String>` with conflicting behavior. The compiler wouldn't know which one to use. The orphan rule makes trait implementations globally coherent.

**Working around it:** The newtype pattern. Wrap the external type in a new struct, then implement whatever you want on the wrapper.

```rust
struct MyVec(Vec<String>);

impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}
```

### Your notes
<!-- -->


---

## Common Standard Library Traits

### `Display` and `Debug`

```rust
use std::fmt;

struct ApiError {
    code: u16,
    message: String,
}

// Display: human-readable, for end users and logs
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API error {}: {}", self.code, self.message)
    }
}

// Debug: auto-generated is fine for most types — use #[derive(Debug)]
// Manual impl useful when you want to redact sensitive fields
impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiError")
            .field("code", &self.code)
            .field("message", &self.message)
            .finish()
    }
}
```

`Display` uses `{}` in format strings. `Debug` uses `{:?}`. Implement `Display` for types that appear in user-facing output. Derive or implement `Debug` for everything — it's required by many libraries and the test framework.

### `From` and `Into`

`From<T>` defines a conversion from `T` to your type. `Into<T>` is automatically derived from `From` — you get it for free.

```rust
#[derive(Debug)]
struct UserId(u64);

impl From<u64> for UserId {
    fn from(n: u64) -> UserId {
        UserId(n)
    }
}

// From gives you Into for free:
let id: UserId = 42_u64.into();     // uses the Into derived from From
let id = UserId::from(42_u64);      // same thing, explicit

// Pattern: use Into in function signatures to accept both native and converted types
fn find_user(id: impl Into<UserId>) -> Option<String> {
    let id = id.into();
    // ...
    None
}

find_user(42_u64);        // caller passes u64, function converts
find_user(UserId(42));    // caller passes UserId directly
```

`From` is also how the `?` operator works with errors. If your error type implements `From<OtherError>`, the `?` operator will call `.into()` on `OtherError` to convert it automatically.

### `Default`

```rust
#[derive(Debug, Default)]
struct ServiceConfig {
    host: String,       // defaults to ""
    port: u16,          // defaults to 0
    timeout_ms: u64,    // defaults to 0
    max_retries: u8,    // defaults to 0
}

// Useful pattern: override just the fields you care about
let config = ServiceConfig {
    host: String::from("api.internal"),
    port: 8080,
    ..ServiceConfig::default()
};
```

For types where the zero-value defaults don't make sense, implement `Default` manually instead of deriving:

```rust
impl Default for ServiceConfig {
    fn default() -> Self {
        ServiceConfig {
            host: String::from("localhost"),
            port: 8080,
            timeout_ms: 5000,
            max_retries: 3,
        }
    }
}
```

### `Clone` and `Copy`

`Clone` is explicit duplication: you call `.clone()`. It can be expensive (allocates heap memory for `String`, `Vec`, etc.).

`Copy` is implicit bitwise duplication: values are copied automatically when assigned or passed, like integers. `Copy` requires `Clone`, and requires that all fields are also `Copy`. A type with a `String` or `Vec` cannot be `Copy`.

```rust
// Copy: small, stack-only types
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point { x: f64, y: f64 }

let p1 = Point { x: 1.0, y: 2.0 };
let p2 = p1;  // copy, p1 still usable
println!("{:?} {:?}", p1, p2);

// Clone only: contains a String (heap-allocated)
#[derive(Debug, Clone)]
struct ServiceEndpoint { host: String, port: u16 }

let e1 = ServiceEndpoint { host: String::from("api"), port: 8080 };
let e2 = e1.clone();  // explicit clone required
// e1 still owned — not moved because we used .clone()
```

### `PartialEq`, `Eq`, `Hash`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RequestId(String);

// With PartialEq: == and != operators work
// With Eq: satisfies total equality (reflexive, symmetric, transitive — no NaN)
// With Hash: can be used as HashMap or HashSet key
let mut seen: std::collections::HashSet<RequestId> = std::collections::HashSet::new();
seen.insert(RequestId(String::from("req-001")));
```

Derive `PartialEq` to enable equality checks. Add `Eq` if equality is total (no NaN-like values). Add `Hash` whenever you derive `Eq` and the type will be used as a map key.

### `Iterator`

`Iterator` is the most powerful trait in the standard library — one required method, 70+ default methods.

```rust
struct CountDown {
    current: u32,
}

impl Iterator for CountDown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.current == 0 {
            None
        } else {
            let val = self.current;
            self.current -= 1;
            Some(val)
        }
    }
}

let cd = CountDown { current: 5 };
let doubled: Vec<u32> = cd.map(|n| n * 2).collect();  // [10, 8, 6, 4, 2]
```

Implementing `Iterator` gives you `map`, `filter`, `fold`, `flat_map`, `take`, `skip`, `enumerate`, `zip`, `chain`, `sum`, `product`, `collect`, `any`, `all`, `find`, `min`, `max`, and dozens more.

### Operator Overloading via `std::ops`

```rust
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 { x: f64, y: f64 }

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

let a = Vec2 { x: 1.0, y: 2.0 };
let b = Vec2 { x: 3.0, y: 4.0 };
let c = a + b;  // calls Add::add
```

Other operator traits: `Sub`, `Mul`, `Div`, `Neg`, `Index`, `IndexMut`, `Deref`, `DerefMut`.

### Your notes
<!-- -->


---

## Derivable Traits

The `#[derive(...)]` macro generates trait implementations automatically. These are the commonly derived traits and when to use them:

| Trait | Derive when | Notes |
|-------|-------------|-------|
| `Debug` | Almost always | Required by test framework, logging |
| `Clone` | When you may need to duplicate | Explicit `.clone()` call |
| `Copy` | Small, stack-only types | Implicit bitwise copy; cannot have String/Vec |
| `PartialEq` | When `==` should work | Sufficient for most use cases |
| `Eq` | When equality is total | Requires PartialEq; signals "no NaN-like cases" |
| `PartialOrd` | When ordering makes sense | Enables `<`, `>`, etc. |
| `Ord` | Total ordering | For BTreeMap keys, `.sort()` |
| `Hash` | For use as HashMap/HashSet key | Must also derive `Eq` |
| `Default` | When a sensible zero-value exists | `ServiceConfig::default()` |

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct WebhookKey {
    endpoint_id: String,
    event_type: String,
}
```

Manual implementation is needed when:
- `Debug` should redact sensitive fields (API keys, passwords, tokens)
- `PartialEq` has domain-specific equality rules
- `Default` should return a non-zero-value default (e.g., retry count = 3, not 0)
- `Display` — there's no derive, you always write this manually

### Your notes
<!-- -->


---

## Blanket Implementations

A blanket implementation implements a trait for any type satisfying some bounds. The standard library uses this extensively.

```rust
// std::convert implements this:
// impl<T, U: From<T>> Into<U> for T { ... }
//
// Translation: for any T and U, if U implements From<T>,
// then T automatically implements Into<U>.
//
// This is why deriving From gives you Into for free.
```

You can write your own blanket implementations:

```rust
use std::fmt;

trait Loggable {
    fn log(&self);
}

// Any type that implements Display also implements Loggable
impl<T: fmt::Display> Loggable for T {
    fn log(&self) {
        println!("[LOG] {self}");
    }
}

// Now every Display type has .log():
42_i32.log();
String::from("hello").log();
```

Blanket impls are powerful but should be used carefully in library code — they can cause conflicts with other implementations.

### Your notes
<!-- -->


---

## Comparison: Go Interfaces vs Rust Traits

| | Go | Rust |
|--|----|----|
| Satisfaction | Implicit (structural) | Explicit (`impl Trait for Type`) |
| Default methods | No (Go 1.18+: none in interfaces) | Yes — rich defaults |
| Static dispatch | All Go method calls are at least one indirection | `impl Trait` / generics — zero overhead |
| Dynamic dispatch | Interface value (type + pointer) | `dyn Trait` (fat pointer + vtable) |
| Object safety | N/A (Go interfaces are always dynamic) | Explicit requirement for `dyn Trait` |
| Orphan rule | None — any package can implement any interface | Strict — own the trait or the type |
| Composition | Interface embedding | Supertraits |
| Generics | Type parameters with constraints (`[T Notifier]`) | Type parameters with trait bounds (`T: Notifier`) |

**The key mental model shift:** In Go, you write the interface and types "accidentally" implement it. In Rust, you design the trait and types deliberately opt in. This costs one line of code per implementor, but it means you can search the codebase for every implementor, the compiler catches missing implementations, and the orphan rule prevents ecosystem-wide conflicts.

Neither approach is universally better. Go's implicit satisfaction is excellent for adapter patterns and testing (mock types in tests automatically satisfy interfaces). Rust's explicit satisfaction is better for large codebases where you want to audit what satisfies a contract.

### Your notes
<!-- -->
