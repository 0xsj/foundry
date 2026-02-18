# Structs, Methods & Enums — Rust

## Structs

### Named-Field Structs

A struct is a named, fixed-shape composite type. Every field has a name and a type. The compiler knows the exact layout at compile time.

```rust
struct HttpRequest {
    method: String,
    path: String,
    body: Option<String>,
    content_length: usize,
}
```

Constructing one requires all fields (no zero values, no partial initialization):

```rust
let req = HttpRequest {
    method: String::from("POST"),
    path: String::from("/api/users"),
    body: Some(String::from("{\"name\": \"alice\"}")),
    content_length: 18,
};
```

The compiler will refuse to compile if you leave a field out. This is unlike Go, where unset fields get zero values silently. Rust forces you to be explicit.

**Struct update syntax** — create a new struct from an existing one, overriding specific fields:

```rust
let redirect = HttpRequest {
    path: String::from("/api/v2/users"),
    ..req  // take all remaining fields from req
};
// Note: req is MOVED into redirect (String fields aren't Copy).
// If all fields were Copy types, req would still be usable.
```

This is superficially similar to JavaScript's spread (`{ ...obj, key: val }`) but with an important difference: fields that aren't `Copy` are *moved*, not copied. After `..req`, the original `req` is partially moved and cannot be used as a whole.

### Tuple Structs

A struct with unnamed positional fields. Use when the name of the type provides the meaning, not the field names.

```rust
struct Milliseconds(u64);
struct Bytes(usize);
struct UserId(u32);

let timeout = Milliseconds(5000);
let user = UserId(42);

// Access by position:
println!("timeout: {}ms", timeout.0);
```

This is the **newtype pattern** — wrapping a primitive to give it a distinct type. `Milliseconds` and `Bytes` both wrap `u64`, but you cannot accidentally pass one where the other is expected. The compiler enforces the distinction.

```rust
fn set_timeout(ms: Milliseconds) { ... }

set_timeout(Bytes(1024));  // compile error: expected Milliseconds, got Bytes
set_timeout(5000);         // compile error: expected Milliseconds, got integer
set_timeout(Milliseconds(5000));  // correct
```

This eliminates an entire class of bugs at zero runtime cost. The wrapper has no overhead — it compiles to the same machine code as the bare `u64`.

### Unit Structs

A struct with no fields. Used as a marker, a type-level token, or a zero-size type for generics.

```rust
struct Unauthenticated;
struct Authenticated;
struct DatabaseMigrationMarker;
```

You'll see these more when you reach traits and generics — they're often used as phantom type parameters to encode state in the type system.

### Your notes
<!-- -->


---

## impl Blocks

Methods and associated functions for a type are defined in `impl` blocks. A type can have multiple `impl` blocks (useful for organizing code or implementing separate traits).

### The Three Receivers

The receiver is the first parameter of a method. It determines how the method interacts with the instance.

| Receiver | Syntax | What it means |
|---|---|---|
| Immutable borrow | `&self` | Read-only access. Caller keeps ownership. Most common. |
| Mutable borrow | `&mut self` | Read-write access. Caller keeps ownership. |
| Owned (consuming) | `self` | Takes ownership. Instance is consumed. Use for builders or conversions. |

```rust
struct Connection {
    host: String,
    port: u16,
    connected: bool,
}

impl Connection {
    // Associated function (no receiver) — like a static method.
    // Called as: Connection::new("localhost", 5432)
    fn new(host: &str, port: u16) -> Connection {
        Connection {
            host: host.to_string(),
            port,
            connected: false,
        }
    }

    // &self — read only. Does not modify the connection.
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    // &mut self — mutates state. Caller must have a mut binding.
    fn connect(&mut self) {
        self.connected = true;
        println!("connected to {}", self.address());
    }

    // self — consumes the connection, returns a new type.
    // After calling this, the original Connection is gone.
    fn into_tls(self) -> TlsConnection {
        TlsConnection { inner: self }
    }
}

struct TlsConnection {
    inner: Connection,
}
```

Usage:

```rust
let mut conn = Connection::new("db.internal", 5432);
println!("{}", conn.address());  // &self — no mut needed
conn.connect();                   // &mut self — need let mut
let tls = conn.into_tls();       // self — conn is moved, no longer usable
// conn.address();  // compile error: value moved
```

### When to Use Which

- **`&self`**: default. Reading data, computing derived values, checking state. The vast majority of methods.
- **`&mut self`**: modifying internal state. Setting fields, accumulating data, changing status.
- **`self`**: irreversible operations. Builder pattern (consume partial config, return more complete version), type conversions, cleanup methods where you want to prevent further use.

### Associated Functions (No Receiver)

Functions inside `impl` without a `self` parameter are associated functions — constructors, factories, conversion utilities. The convention for the primary constructor is `new`:

```rust
impl Connection {
    fn new(host: &str, port: u16) -> Connection { ... }
    fn from_url(url: &str) -> Result<Connection, ParseError> { ... }
    fn default_dev() -> Connection { Connection::new("localhost", 5432) }
}
```

Called with `::` syntax: `Connection::new(...)`. This is equivalent to Go's package-level constructor functions (`NewConnection(...)`) or TypeScript's static methods (`Connection.new(...)`).

### Multiple impl Blocks

Rust allows splitting a type's methods across multiple `impl` blocks. This is commonly used to separate core methods from trait implementations, or to group related methods:

```rust
impl Connection {
    // Core lifecycle
    fn new(host: &str, port: u16) -> Connection { ... }
    fn connect(&mut self) { ... }
    fn disconnect(&mut self) { ... }
}

impl Connection {
    // Query helpers
    fn is_connected(&self) -> bool { self.connected }
    fn address(&self) -> String { format!("{}:{}", self.host, self.port) }
}
```

### Your notes
<!-- -->


---

## Enums

Enums are Rust's most powerful type. Every language has enums, but Rust's enums are **algebraic data types** (ADTs) — each variant can carry different data. This makes them much more expressive than C-style enums or TypeScript string unions.

### C-Like Enums (No Data)

The familiar variant: named alternatives, no associated data. Under the hood, stored as an integer discriminant.

```rust
#[derive(Debug, PartialEq, Clone)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}
```

### Enums with Data

This is where Rust enums diverge from every other mainstream language. Each variant can carry different shapes of data:

```rust
#[derive(Debug)]
enum ConfigValue {
    String(String),              // tuple variant — one String field
    Integer(i64),                // tuple variant — one i64 field
    Float(f64),
    Boolean(bool),
    List(Vec<ConfigValue>),      // recursive — contains more ConfigValues
    Null,                        // unit variant — no data
    Object {                     // struct variant — named fields
        keys: Vec<String>,
        values: Vec<ConfigValue>,
    },
}
```

This one enum represents an entire JSON-like value type. Compare this to how you'd model it in TypeScript:

```typescript
// TypeScript: discriminated union
type ConfigValue =
  | { kind: 'string'; value: string }
  | { kind: 'integer'; value: number }
  | { kind: 'list'; items: ConfigValue[] }
  | { kind: 'null' }
  // ...
```

TypeScript's discriminated unions are manually constructed — you choose the discriminant field name and check it yourself. Rust builds the discriminant in automatically. The compiler knows which variant is active and enforces exhaustive checking.

**Memory layout:** Rust enums use a tagged union. The compiler picks the smallest representation that can hold the largest variant. The tag (discriminant) identifies the active variant.

```
ConfigValue::Null  — just a discriminant (1 byte typically, may be padded)
ConfigValue::Boolean(true) — discriminant + 1 byte
ConfigValue::Integer(42) — discriminant + 8 bytes
ConfigValue::String(...) — discriminant + 24 bytes (String = ptr + len + cap)
```

The enum's size is `max(variant sizes) + discriminant`, plus any padding for alignment.

### Option<T> — The Null-Safe Option

`Option<T>` is a standard library enum:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

There is no `null` in Rust. If a value might be absent, it's `Option<T>`. The compiler forces you to handle both cases before accessing the value. This eliminates null pointer exceptions at compile time.

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("alice"))
    } else {
        None
    }
}

let name = find_user(1);

// You cannot use name directly — it might be None.
// The compiler requires handling:

// Option 1: match
match name {
    Some(n) => println!("Found: {n}"),
    None => println!("Not found"),
}

// Option 2: if let (when you only care about Some)
if let Some(n) = find_user(2) {
    println!("Found: {n}");
}

// Option 3: unwrap_or (provide a default)
let display = find_user(3).unwrap_or(String::from("anonymous"));

// Option 4: map (transform the inner value if present)
let upper = find_user(1).map(|n| n.to_uppercase());
```

Compare to Go:

```go
// Go: return (string, bool) or (string, error)
func findUser(id int) (string, bool) {
    if id == 1 { return "alice", true }
    return "", false
}
name, ok := findUser(1)
if !ok { ... }
// Go can't enforce that you check 'ok'. Rust can.
```

In Go you can accidentally use the zero value without checking the bool. In Rust, the compiler won't let you access the inner value without unwrapping `Option`.

### Result<T, E> — Explicit Error Handling

`Result<T, E>` is the error-handling enum:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Functions that can fail return `Result`. Callers must handle both cases. No exceptions, no hidden control flow.

```rust
fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>().map_err(|e| format!("invalid port '{}': {}", s, e))
}

// Handling a Result:
match parse_port("8080") {
    Ok(port) => println!("port: {port}"),
    Err(e) => eprintln!("error: {e}"),
}

// The ? operator — propagate errors up the call stack
fn start_server(port_str: &str) -> Result<(), String> {
    let port = parse_port(port_str)?;  // if Err, returns early with the error
    println!("listening on port {port}");
    Ok(())
}
```

The `?` operator is syntactic sugar for:

```rust
let port = match parse_port(port_str) {
    Ok(p) => p,
    Err(e) => return Err(e.into()),
};
```

### Pattern Matching on Enums

`match` is how you destructure enums. It is exhaustive — the compiler requires you to handle every variant.

```rust
#[derive(Debug)]
enum WebhookEvent {
    UserCreated { user_id: u64, email: String },
    UserDeleted(u64),
    OrderPlaced { order_id: u64, total_cents: u64, user_id: u64 },
    PaymentFailed { order_id: u64, reason: String },
    Ping,
}

fn handle_event(event: WebhookEvent) {
    match event {
        WebhookEvent::UserCreated { user_id, email } => {
            println!("new user {user_id}: {email}");
        }
        WebhookEvent::UserDeleted(id) => {
            println!("deleted user {id}");
        }
        WebhookEvent::OrderPlaced { order_id, total_cents, user_id } => {
            println!("order {order_id} for user {user_id}: ${}", total_cents / 100);
        }
        WebhookEvent::PaymentFailed { order_id, reason } => {
            eprintln!("payment failed for order {order_id}: {reason}");
        }
        WebhookEvent::Ping => {}  // must handle this or use a wildcard
    }
}
```

If you add a new variant to `WebhookEvent` later, every `match` on it will fail to compile until you handle the new case. This is **exhaustive pattern matching** — it's the compiler enforcing that your dispatch logic stays in sync with your data model.

**Wildcard patterns:**

```rust
match event {
    WebhookEvent::Ping => {},
    _ => handle_as_business_event(event),  // catch-all
}
```

**Guards — conditional matching:**

```rust
match event {
    WebhookEvent::OrderPlaced { total_cents, .. } if total_cents > 100_000 => {
        // high-value order — trigger fraud review
        flag_for_review();
    }
    WebhookEvent::OrderPlaced { order_id, .. } => {
        process_normal(order_id);
    }
    _ => {}
}
```

**`if let` — match a single variant:**

```rust
// When you only care about one variant, if let is cleaner:
if let WebhookEvent::UserCreated { user_id, email } = event {
    send_welcome_email(user_id, &email);
}

// Equivalent to:
match event {
    WebhookEvent::UserCreated { user_id, email } => send_welcome_email(user_id, &email),
    _ => {}
}
```

**`while let` — loop until a pattern stops matching:**

```rust
let mut queue: Vec<Option<WebhookEvent>> = get_queue();
while let Some(Some(event)) = queue.pop() {
    handle_event(event);
}
```

### Nested Pattern Matching

Patterns can nest arbitrarily deep:

```rust
enum Notification {
    Email { to: String, subject: String, body: Option<String> },
    Push { device_token: String, priority: Priority },
    Sms(String),
}

enum Priority { High, Normal, Low }

fn should_retry(n: &Notification) -> bool {
    match n {
        Notification::Push { priority: Priority::High, .. } => true,
        Notification::Email { body: Some(_), .. } => true,
        _ => false,
    }
}
```

The patterns bind names, test values, and destructure nested enums in a single expression. No chains of `if/else`, no null checks.

### Your notes
<!-- -->


---

## Derive Macros

`#[derive(...)]` automatically generates trait implementations. These are compile-time code generation — no runtime overhead.

| Derive | What it adds | When to use |
|---|---|---|
| `Debug` | `{:?}` formatting | Almost always — enables logging and debugging |
| `Clone` | `.clone()` method | When you need to explicitly duplicate a value |
| `Copy` | Implicit bitwise copy | Only for small, stack-only types (can't have String/Vec) |
| `PartialEq` | `==` and `!=` operators | Whenever you need to compare for equality |
| `Eq` | Marker for total equality | When PartialEq is also reflexive (not NaN) |
| `PartialOrd` | `<`, `>`, `<=`, `>=` | When ordering makes sense (enum variant order) |
| `Ord` | Total ordering | For BTreeMap keys, sorting |
| `Hash` | HashMap key usage | Derive with PartialEq for HashMap/HashSet keys |
| `Default` | `Default::default()` constructor | When a sensible zero-value exists |

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct ServiceConfig {
    host: String,
    port: u16,
    timeout_ms: u64,
    retries: u8,
}

// Default provides sensible zero values:
let config = ServiceConfig::default();
// ServiceConfig { host: "", port: 0, timeout_ms: 0, retries: 0 }

// Common pattern: override specific fields from default:
let config = ServiceConfig {
    host: String::from("api.internal"),
    port: 8080,
    ..ServiceConfig::default()
};
```

### Deriving PartialOrd on Enums

Variant declaration order determines ordering. Declare them from "smallest" to "largest":

```rust
#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Severity {
    Info,    // smallest
    Warning,
    Error,
    Fatal,   // largest
}

assert!(Severity::Info < Severity::Error);
assert!(Severity::Fatal > Severity::Warning);
```

### When You Can't Derive

Sometimes you need a custom implementation:

- **Debug**: when you want to redact sensitive fields (passwords, tokens)
- **PartialEq**: when equality has domain-specific meaning
- **Default**: when the zero value isn't a valid or sensible state

```rust
impl std::fmt::Debug for ApiCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiCredentials")
            .field("key_id", &self.key_id)
            .field("secret", &"[REDACTED]")  // never log secrets
            .finish()
    }
}
```

### Your notes
<!-- -->


---

## The Newtype Pattern

Wrap a primitive type in a tuple struct to give it a distinct identity. This is one of Rust's most useful patterns for domain modeling.

```rust
struct RequestId(String);
struct UserId(u64);
struct TeamId(u64);
struct DurationMs(u64);
struct Percent(f64);
```

Without newtypes:

```rust
// Which u64 is the user? Which is the team? The compiler has no idea.
fn transfer_ownership(user: u64, team: u64) { ... }

transfer_ownership(team_id, user_id);  // compiles fine, silently wrong
```

With newtypes:

```rust
fn transfer_ownership(user: UserId, team: TeamId) { ... }

transfer_ownership(team_id, user_id);  // compile error: expected UserId, got TeamId
```

The cost is zero. A `UserId(42)` compiles to the same bytes as a bare `42`. The type distinction only exists at compile time.

To add methods, implement them on the wrapper:

```rust
impl DurationMs {
    fn from_seconds(s: u64) -> DurationMs { DurationMs(s * 1000) }
    fn as_seconds(&self) -> u64 { self.0 / 1000 }
    fn is_expired(&self, elapsed: DurationMs) -> bool { elapsed.0 >= self.0 }
}

let timeout = DurationMs::from_seconds(30);
```

To make the newtype work transparently with the underlying type's traits (like `Display`, `Add`, etc.), implement them manually or use the `Deref` trait.

### Your notes
<!-- -->


---

## Comparison to Go and TypeScript

### Go: Structs and Interfaces vs Rust: Structs and Enums + Traits

Go uses structs for data, interfaces for behavior. Rust uses structs for data, traits for behavior — but adds enums as a first-class tool for modeling alternatives.

In Go, modeling a "command" that could be one of several types usually requires interfaces and type assertions:

```go
type Command interface {
    Execute() error
}

type DeployCommand struct { Service string; Version string }
type RollbackCommand struct { Service string }
type RestartCommand struct { Service string; Force bool }

// Type-switching is unidiomatic and not exhaustive:
switch cmd := c.(type) {
case *DeployCommand: ...
case *RollbackCommand: ...
// Forgot RestartCommand — compiler doesn't care
}
```

In Rust, enums model this directly:

```rust
enum Command {
    Deploy { service: String, version: String },
    Rollback { service: String },
    Restart { service: String, force: bool },
}

match cmd {
    Command::Deploy { service, version } => ...,
    Command::Rollback { service } => ...,
    Command::Restart { service, force } => ...,
    // Forgot a variant? Compile error. Exhaustiveness enforced.
}
```

### Go: Embedding vs Rust: Composition

Go has embedding — a struct can embed another struct's methods directly:

```go
type Base struct { ID int; CreatedAt time.Time }
type User struct { Base; Email string }  // User gets Base's methods
```

Rust has no embedding. You use explicit fields and either delegate manually or implement `Deref`. The Rust way is composition — prefer clear field access over implicit promotion:

```rust
struct Base { id: u64, created_at: u64 }
struct User { base: Base, email: String }

impl User {
    fn id(&self) -> u64 { self.base.id }  // explicit delegation
}
```

### TypeScript: Discriminated Unions vs Rust Enums

TypeScript's discriminated unions are the closest equivalent to Rust enums, but they're manually constructed:

```typescript
// TypeScript: you pick the discriminant field name and manage it
type WebhookEvent =
  | { kind: 'user_created'; userId: number; email: string }
  | { kind: 'user_deleted'; userId: number }
  | { kind: 'ping' }

function handleEvent(event: WebhookEvent) {
  switch (event.kind) {
    case 'user_created': break;
    // TypeScript checks exhaustiveness only if you use 'never' trick
  }
}
```

```rust
// Rust: the discriminant is implicit. Exhaustiveness is always enforced.
enum WebhookEvent {
    UserCreated { user_id: u64, email: String },
    UserDeleted { user_id: u64 },
    Ping,
}
```

TypeScript's `narrowing` is the type system inferring which union member is active after a check. Rust's pattern matching is more powerful — it destructures and binds in a single expression.

### TypeScript: Classes vs Rust: impl Blocks

TypeScript (and Go, Python, etc.) attach methods to types via class syntax or struct methods. Rust separates the data (`struct`) from the behavior (`impl`). The effect is similar but Rust's separation is explicit:

```typescript
class Connection {
    constructor(private host: string, private port: number) {}
    address(): string { return `${this.host}:${this.port}`; }
    connect(): void { ... }
}
```

```rust
struct Connection { host: String, port: u16 }

impl Connection {
    fn new(host: &str, port: u16) -> Connection { ... }
    fn address(&self) -> String { format!("{}:{}", self.host, self.port) }
    fn connect(&mut self) { ... }
}
```

The separation means you can add methods to types you don't own — as long as either the type or the trait is defined in your crate. This is how Rust extends without inheritance.

### Builder Pattern (Preview)

The consuming `self` receiver enables a builder pattern without mutable references:

```rust
// We'll cover this in depth in the patterns module.
// Preview: each method takes ownership and returns a modified value.

struct RequestBuilder { method: String, path: String, body: Option<String> }

impl RequestBuilder {
    fn new() -> Self { ... }
    fn method(mut self, m: &str) -> Self { self.method = m.to_string(); self }
    fn path(mut self, p: &str) -> Self { self.path = p.to_string(); self }
    fn body(mut self, b: &str) -> Self { self.body = Some(b.to_string()); self }
    fn build(self) -> HttpRequest { ... }
}

let req = RequestBuilder::new()
    .method("POST")
    .path("/api/users")
    .body("{\"name\":\"alice\"}")
    .build();
```

Each intermediate value is consumed. The builder is always in a valid state. No `Option<String>` fields that might be accidentally left empty.

### Your notes
<!-- -->
