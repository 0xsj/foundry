# Rust Reference -- Builder Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/),
> [std library docs](https://doc.rust-lang.org/std/),
> and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
> for the `builder` module. Covers: Default trait, struct update syntax, PhantomData, Command, thread::Builder, typestate pattern.

---

## The `Default` Trait

Source: [std::default::Default](https://doc.rust-lang.org/std/default/trait.Default.html)

The `Default` trait provides a function to create a default value for a type.

### Definition

```rust
pub trait Default: Sized {
    fn default() -> Self;
}
```

### Deriving Default

`#[derive(Default)]` generates an implementation that calls `Default::default()` on each field:

```rust
#[derive(Default)]
struct Config {
    retries: u32,        // default: 0
    verbose: bool,       // default: false
    name: String,        // default: ""
    tags: Vec<String>,   // default: vec![]
    limit: Option<u32>,  // default: None
}
```

### Default Values for Standard Types

| Type | Default Value |
|------|--------------|
| `bool` | `false` |
| `u8`, `u16`, `u32`, `u64`, `u128`, `usize` | `0` |
| `i8`, `i16`, `i32`, `i64`, `i128`, `isize` | `0` |
| `f32`, `f64` | `0.0` |
| `char` | `'\x00'` |
| `String` | `""` (empty string) |
| `Vec<T>` | `vec![]` (empty vec) |
| `Option<T>` | `None` |
| `HashMap<K, V>` | `HashMap::new()` (empty map) |
| `()` | `()` |

### Manual Default Implementation

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            retries: 3,          // custom default, not 0
            verbose: false,
            name: "unnamed".to_string(),
            tags: vec![],
            limit: Some(100),    // custom default, not None
        }
    }
}
```

### Default with Enums

```rust
#[derive(Default)]
enum LogLevel {
    Debug,
    #[default]
    Info,       // <-- marked as the default variant
    Warn,
    Error,
}
```

The `#[default]` attribute (stabilized in Rust 1.62) marks which variant is the default. Without it, `#[derive(Default)]` on an enum is a compile error.

---

## Struct Update Syntax

Source: [Rust Reference - Struct expressions](https://doc.rust-lang.org/reference/expressions/struct-expr.html)

The struct update syntax `..expr` fills in remaining fields from another struct value:

```rust
let default = Config::default();
let custom = Config {
    retries: 5,
    verbose: true,
    ..default       // fill remaining fields from `default`
};
```

### Rules

| Rule | Description |
|------|-------------|
| Must be last | `..expr` must be the final element in the struct expression |
| Same type | The base expression must be the same struct type |
| Moves non-Copy fields | If a field is not `Copy`, it is moved from the base |
| Partial move | After `..base`, fields that were moved cannot be accessed on `base` |

```rust
// After struct update, moved fields are no longer accessible:
let base = Config { name: "original".to_string(), ..Default::default() };
let updated = Config { retries: 10, ..base };
// base.retries is still accessible (u32 is Copy)
// base.name is NOT accessible (String was moved into updated)
```

### With Default

The most common pattern combines struct update syntax with `Default`:

```rust
let config = Config {
    retries: 5,
    ..Default::default()
};
```

This is the idiomatic "partial construction" pattern in Rust -- the equivalent of TypeScript's `{ ...defaults, ...overrides }`.

---

## `std::marker::PhantomData`

Source: [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)

`PhantomData<T>` is a zero-sized type that tells the compiler "this struct is conceptually related to type `T`."

### Purpose

```rust
use std::marker::PhantomData;

// Without PhantomData, the compiler warns about unused type parameter
struct Builder<State> {
    data: String,
    _state: PhantomData<State>,  // zero-sized, exists only in the type system
}
```

### Size and Layout

| Type | Size |
|------|------|
| `PhantomData<T>` (for any `T`) | 0 bytes |
| `PhantomData<()>` | 0 bytes |
| `(PhantomData<A>, PhantomData<B>)` | 0 bytes |
| Struct containing only `PhantomData` fields | 0 bytes |

`PhantomData` adds zero runtime overhead. It is purely a compile-time construct.

### Variance

`PhantomData<T>` makes the containing type covariant over `T`. For typestate builders, this is the correct behavior -- the state markers are not actually stored or referenced.

```rust
// Covariant: PhantomData<T> acts as if it owns a T
struct Builder<State> {
    _state: PhantomData<State>,  // covariant over State
}

// If you need invariance (rare for builders):
// PhantomData<fn(T) -> T>     -- invariant
// PhantomData<*const T>       -- covariant (same as PhantomData<T>)
// PhantomData<*mut T>         -- invariant
// PhantomData<fn(T)>          -- contravariant
```

For typestate builders, `PhantomData<State>` (covariant) is correct. The state markers are empty types used purely for type-level bookkeeping.

### Common Typestate Markers

```rust
// Empty marker types
struct NotConfigured;
struct Configured;

// Or use a sealed mod for better namespacing:
mod state {
    pub struct Missing;
    pub struct Set;
}
```

---

## `std::process::Command`

Source: [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html)

A process builder, providing fine-grained control over how a new process should be spawned.

### API Surface

```rust
impl Command {
    // Constructor -- program is required
    pub fn new<S: AsRef<OsStr>>(program: S) -> Command;

    // Argument methods (accumulating)
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Command;
    pub fn args<I, S>(&mut self, args: I) -> &mut Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;

    // Environment
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Command;
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Command;
    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Command;
    pub fn env_clear(&mut self) -> &mut Command;

    // Working directory
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Command;

    // I/O configuration
    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command;
    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command;
    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command;

    // Build/execute methods
    pub fn spawn(&mut self) -> io::Result<Child>;
    pub fn output(&mut self) -> io::Result<Output>;
    pub fn status(&mut self) -> io::Result<ExitStatus>;
}
```

### Key Design Points

| Aspect | Design |
|--------|--------|
| Self type in setters | `&mut self` (borrowing -- reusable) |
| Return type from setters | `&mut Command` (for chaining) |
| Required fields | Program name in constructor |
| Accumulating fields | `arg()` adds to list (does not replace) |
| Build methods | `spawn()`, `output()`, `status()` |
| Error handling | Returns `io::Result` |

### Usage Patterns

```rust
use std::process::Command;

// Single chain
let output = Command::new("git")
    .arg("log")
    .arg("--oneline")
    .arg("-5")
    .output()?;

// Reusable builder
let mut cmd = Command::new("cargo");
cmd.arg("test");
cmd.current_dir("/path/to/project");

// Run once
let status = cmd.status()?;

// Can reuse (borrowing builder)
cmd.arg("--release");
let status2 = cmd.status()?;
```

---

## `std::thread::Builder`

Source: [std::thread::Builder](https://doc.rust-lang.org/std/thread/struct.Builder.html)

Thread factory which can be used to configure the properties of a new thread.

### API Surface

```rust
impl Builder {
    pub fn new() -> Builder;
    pub fn name(self, name: String) -> Builder;
    pub fn stack_size(self, size: usize) -> Builder;
    pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}
```

### Key Design Points

| Aspect | Design |
|--------|--------|
| Self type in setters | `self` (consuming -- single use) |
| Return type from setters | `Builder` (owned, for chaining) |
| Required fields | None (all optional) |
| Build method | `spawn(closure)` -- combines build + execute |
| Error handling | Returns `io::Result<JoinHandle<T>>` |

Note that `thread::Builder` uses **consuming** setters (takes `self`, not `&mut self`), unlike `Command`. This means the builder cannot be reused after `spawn()`.

---

## `std::fs::OpenOptions`

Source: [std::fs::OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)

Options and flags which can be used to configure how a file is opened.

### API Surface

```rust
impl OpenOptions {
    pub fn new() -> OpenOptions;

    pub fn read(&mut self, read: bool) -> &mut OpenOptions;
    pub fn write(&mut self, write: bool) -> &mut OpenOptions;
    pub fn append(&mut self, append: bool) -> &mut OpenOptions;
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions;
    pub fn create(&mut self, create: bool) -> &mut OpenOptions;
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions;

    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File>;
}
```

### Key Design Points

| Aspect | Design |
|--------|--------|
| Self type in setters | `&mut self` (borrowing -- reusable) |
| Setter parameters | All `bool` flags |
| Build method | `open(path)` takes `&self` (non-consuming) |
| Reusable | Yes -- same options can open multiple files |

---

## Rust API Guidelines: Builder Pattern

Source: [Rust API Guidelines C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder)

### Recommended Conventions

| Guideline | Description |
|-----------|-------------|
| Name the builder `ThingBuilder` | e.g., `CommandBuilder`, `ServerConfigBuilder` |
| Constructor returns builder | `Thing::builder()` or `ThingBuilder::new()` |
| Setters return `&mut Self` or `Self` | Choose one consistently within a builder |
| `build()` validates and returns `Result` | Unless all configurations are valid |
| `build()` consumes or borrows | Consuming is preferred for single-use; borrowing for reusable |
| Document defaults | Every field with a default should say what the default is |
| Use `Into<T>` for string params | `fn name(mut self, name: impl Into<String>) -> Self` |

### `Into<T>` Convention

```rust
// Allows passing &str, String, Cow<str>, etc.
fn host(mut self, host: impl Into<String>) -> Self {
    self.host = Some(host.into());
    self
}

// Usage:
builder.host("localhost")              // &str
builder.host(String::from("localhost")) // String
builder.host(hostname_var)             // any impl Into<String>
```

### Builder Method Naming

| Pattern | Example | When |
|---------|---------|------|
| `with_X(value)` | `.with_timeout(30)` | Setting a value |
| `X(value)` | `.timeout(30)` | Setting a value (shorter, common in std) |
| `set_X(value)` | `.set_timeout(30)` | Less idiomatic in Rust |
| `add_X(value)` | `.add_header(k, v)` | Accumulating into a collection |
| `enable_X()` / `disable_X()` | `.enable_tls()` | Boolean flags |
| `X(bool)` | `.tls(true)` | Boolean flags (std style) |

---

## References

- [The Rust Programming Language, Ch. 7 - Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust API Guidelines - C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder)
- [std::default::Default](https://doc.rust-lang.org/std/default/trait.Default.html)
- [std::marker::PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)
- [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html)
- [std::thread::Builder](https://doc.rust-lang.org/std/thread/struct.Builder.html)
- [std::fs::OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)
- [Rust Design Patterns - Builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
