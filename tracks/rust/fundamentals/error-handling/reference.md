# Rust Reference — Error Handling

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Programming Language (Book)](https://doc.rust-lang.org/book/ch09-00-error-handling.html),
> and [std library docs](https://doc.rust-lang.org/std/) for the `error-handling` module.
> Covers: `Result<T, E>`, `Option<T>`, the `?` operator, `std::error::Error`, `From` trait,
> panic semantics, and common error-handling crates.

---

## `Result<T, E>`

Source: [std::result](https://doc.rust-lang.org/std/result/index.html)

`Result<T, E>` is an enum defined in the standard library and automatically imported via the prelude:

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

- `T`: the success value type
- `E`: the error value type
- Both variants are in scope without qualification: `Ok(value)`, `Err(error)`

### Key Methods

| Method | Signature | Description |
|---|---|---|
| `is_ok` | `(&self) -> bool` | Returns `true` if `Ok` |
| `is_err` | `(&self) -> bool` | Returns `true` if `Err` |
| `ok` | `(self) -> Option<T>` | Converts `Ok(v)` to `Some(v)`, `Err(_)` to `None` |
| `err` | `(self) -> Option<E>` | Converts `Err(e)` to `Some(e)`, `Ok(_)` to `None` |
| `unwrap` | `(self) -> T` | Returns `T` or panics if `Err` |
| `expect` | `(self, msg: &str) -> T` | Returns `T` or panics with `msg` if `Err` |
| `unwrap_or` | `(self, default: T) -> T` | Returns `T` or `default` if `Err` |
| `unwrap_or_else` | `(self, f: FnOnce(E) -> T) -> T` | Returns `T` or calls `f(e)` if `Err` |
| `unwrap_or_default` | `(self) -> T where T: Default` | Returns `T` or `T::default()` if `Err` |
| `map` | `(self, f: FnOnce(T) -> U) -> Result<U, E>` | Transforms `Ok` value |
| `map_err` | `(self, f: FnOnce(E) -> F) -> Result<T, F>` | Transforms `Err` value |
| `map_or` | `(self, default: U, f: FnOnce(T) -> U) -> U` | Maps `Ok` or returns default |
| `map_or_else` | `(self, default: FnOnce(E) -> U, f: FnOnce(T) -> U) -> U` | Maps `Ok` or calls default closure on `Err` |
| `and` | `(self, res: Result<U, E>) -> Result<U, E>` | Returns `res` if `Ok`, else `Err` |
| `and_then` | `(self, f: FnOnce(T) -> Result<U, E>) -> Result<U, E>` | Chains fallible operations |
| `or` | `(self, res: Result<T, F>) -> Result<T, F>` | Returns `self` if `Ok`, else `res` |
| `or_else` | `(self, f: FnOnce(E) -> Result<T, F>) -> Result<T, F>` | Recovers from error |
| `transpose` | `(self) -> Option<Result<T, E>>` where `T: Option<U>` | Converts `Result<Option<T>, E>` to `Option<Result<T, E>>` |

### Collecting `Result`s from an Iterator

`Iterator::collect()` can produce `Result<Vec<T>, E>` — it short-circuits on the first `Err`:

```rust
let v: Result<Vec<i32>, _> = vec!["1", "2", "3"].iter()
    .map(|s| s.parse::<i32>())
    .collect();
// Ok([1, 2, 3])

let v: Result<Vec<i32>, _> = vec!["1", "bad", "3"].iter()
    .map(|s| s.parse::<i32>())
    .collect();
// Err(ParseIntError)
```

---

## `Option<T>`

Source: [std::option](https://doc.rust-lang.org/std/option/index.html)

`Option<T>` represents an optional value. Defined in the prelude:

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

### Key Methods

| Method | Signature | Description |
|---|---|---|
| `is_some` | `(&self) -> bool` | Returns `true` if `Some` |
| `is_none` | `(&self) -> bool` | Returns `true` if `None` |
| `unwrap` | `(self) -> T` | Returns `T` or panics if `None` |
| `expect` | `(self, msg: &str) -> T` | Returns `T` or panics with `msg` if `None` |
| `unwrap_or` | `(self, default: T) -> T` | Returns `T` or `default` if `None` |
| `unwrap_or_else` | `(self, f: FnOnce() -> T) -> T` | Returns `T` or calls `f()` if `None` |
| `unwrap_or_default` | `(self) -> T where T: Default` | Returns `T` or `T::default()` if `None` |
| `map` | `(self, f: FnOnce(T) -> U) -> Option<U>` | Transforms `Some` value |
| `map_or` | `(self, default: U, f: FnOnce(T) -> U) -> U` | Maps `Some` or returns default |
| `and_then` | `(self, f: FnOnce(T) -> Option<U>) -> Option<U>` | Chains optional operations |
| `or` | `(self, optb: Option<T>) -> Option<T>` | Returns `self` if `Some`, else `optb` |
| `or_else` | `(self, f: FnOnce() -> Option<T>) -> Option<T>` | Returns `self` if `Some`, else calls `f()` |
| `filter` | `(self, predicate: FnOnce(&T) -> bool) -> Option<T>` | Returns `None` if predicate is false |
| `ok_or` | `(self, err: E) -> Result<T, E>` | Converts `Some(v)` to `Ok(v)`, `None` to `Err(err)` |
| `ok_or_else` | `(self, err: FnOnce() -> E) -> Result<T, E>` | Like `ok_or` but lazy |
| `flatten` | `(self) -> Option<T>` where `T: Option<U>` | Flattens `Option<Option<T>>` to `Option<T>` |
| `take` | `(&mut self) -> Option<T>` | Takes the value out, leaves `None` |
| `replace` | `(&mut self, value: T) -> Option<T>` | Replaces value, returns old |
| `as_ref` | `(&self) -> Option<&T>` | Converts `&Option<T>` to `Option<&T>` |

---

## The `?` Operator

Source: [reference/expressions/operator-expr.html#the-question-mark-operator](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator)

`?` is a postfix operator. Applied to an expression of type `Result<T, E>` or `Option<T>`:

### On `Result<T, E>`

```
expr?
```

Desugars to (approximately):

```rust
match expr {
    Ok(val) => val,
    Err(e) => return Err(From::from(e)),
}
```

- `From::from(e)` converts the error `e` from its source type to the function's return error type.
- Requires the enclosing function to return `Result<_, F>` where `From<E>` is implemented for `F`.

### On `Option<T>`

```
expr?
```

Desugars to:

```rust
match expr {
    Some(val) => val,
    None => return None,
}
```

- Requires the enclosing function to return `Option<_>`.

### Return Type Requirements

The `?` operator can only be used inside functions that return `Result` or `Option`. The `Residual` trait (unstable) governs this generically, but in stable Rust the requirements are:

| `?` applied to | Required function return type |
|---|---|
| `Result<T, E>` | `Result<_, F>` where `F: From<E>` |
| `Option<T>` | `Option<_>` |

### `?` in `main`

`main` may return `Result<(), E>` where `E: Debug`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ? works here
    Ok(())
}
```

If `main` returns `Err(e)`, the runtime prints `"{:?}", e` to stderr and exits with code 1.

---

## `std::error::Error` Trait

Source: [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }

    // deprecated:
    // fn description(&self) -> &str
    // fn cause(&self) -> Option<&dyn Error>
}
```

### Supertraits

`Error` requires `Debug + Display`:
- `Debug`: for `{:?}` formatting (usually derived)
- `Display`: for `{}` formatting (must be implemented manually — this is the human-readable message)

### `source()`

Returns the lower-level error that caused this error, if any. Used by error reporters to print the full error chain.

```rust
impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            _ => None,
        }
    }
}
```

### Automatic Blanket Impls

The standard library provides:

```rust
// Any Box<dyn Error + ...> can be created from any Error type
impl<E: Error + Send + Sync + 'static> From<E> for Box<dyn Error + Send + Sync>
impl<E: Error + 'static> From<E> for Box<dyn Error>
```

This is why `?` works when the function returns `Result<_, Box<dyn Error>>`: any error type is automatically boxed.

---

## `From` and `Into` Traits

Source: [std::convert::From](https://doc.rust-lang.org/std/convert/trait.From.html)

```rust
pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}
```

`From<T>` for `U` means "a `U` can be created from a `T`." The `?` operator calls `From::from(e)` to convert error types.

Implementing `From<SourceError>` for `AppError` enables:

```rust
fn parse_config() -> Result<Config, AppError> {
    let file = std::fs::read_to_string("cfg")?;  // io::Error -> AppError via From
    Ok(Config::default())
}
```

### Rules

- `From` implies `Into`: implementing `From<A>` for `B` automatically gives `B::into()` on `A`.
- Cannot implement `From<A> for A` for foreign types (orphan rules).
- The blanket impl `From<T> for T` is provided by the standard library.

---

## Panic and Unwinding

Source: [reference/runtime.html#panics](https://doc.rust-lang.org/reference/runtime.html#panics),
[std::panic](https://doc.rust-lang.org/std/panic/index.html)

### `panic!`

```rust
panic!("message");
panic!("formatted: {}", value);
```

Triggers a panic. By default, Rust unwinds the stack (running destructors) before aborting. This can be changed to an immediate abort with `panic = "abort"` in `Cargo.toml`.

### Panic Hooks

`std::panic::set_hook` installs a custom panic handler (for logging):

```rust
std::panic::set_hook(Box::new(|info| {
    eprintln!("panic: {}", info);
}));
```

### `catch_unwind`

`std::panic::catch_unwind` catches a panic from a closure:

```rust
let result = std::panic::catch_unwind(|| {
    panic!("oh no");
});
assert!(result.is_err());
```

This is primarily for FFI boundaries and plugin systems where foreign code might panic. Using it as a general error handling mechanism is an anti-pattern.

### Panic in Tests

In tests, `#[should_panic]` asserts that a test body panics:

```rust
#[test]
#[should_panic(expected = "index out of bounds")]
fn test_out_of_bounds() {
    let v = vec![1, 2, 3];
    let _ = v[10];
}
```

---

## `thiserror` Crate

Source: [docs.rs/thiserror](https://docs.rs/thiserror/latest/thiserror/)

Version: `thiserror = "2"` (current major version as of late 2024)

`thiserror` provides derive macros for `std::error::Error`, `Display`, and `From`.

### Attribute Reference

| Attribute | Applied to | Effect |
|---|---|---|
| `#[error("message")]` | Variant | Generates `Display` impl for this variant |
| `#[from]` | Field | Generates `From<FieldType>` for the enum; also sets `source()` |
| `#[source]` | Field | Sets this field as the `source()` without generating `From` |
| `#[error(transparent)]` | Variant with single field | Delegates `Display` and `source()` to the field |

### Format String Syntax

In `#[error("...")]` format strings:
- `{0}` — first tuple field
- `{field_name}` — named struct field
- `{0:?}` — debug format of first tuple field
- `{source}` — if the variant has a `source` field, `{source}` refers to it

### Examples

```rust
use thiserror::Error;

// Tuple variant — positional field reference
#[derive(Debug, Error)]
#[error("parse error at position {0}: {1}")]
struct ParseError(usize, String);

// Struct variant — named field references
#[derive(Debug, Error)]
#[error("rate limit exceeded: {count} requests in {window_secs}s")]
struct RateLimitError {
    count: u32,
    window_secs: u64,
}

// Enum with From conversion
#[derive(Debug, Error)]
enum ServiceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error at {path}: {source}")]
    Config {
        path: String,
        #[source]
        source: ConfigError,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),  // delegate everything to inner
}
```

---

## `anyhow` Crate

Source: [docs.rs/anyhow](https://docs.rs/anyhow/latest/anyhow/)

Version: `anyhow = "1"`

`anyhow::Error` is a dynamically typed error that wraps any `Box<dyn Error + Send + Sync + 'static>`.

### Type Aliases

```rust
// anyhow provides:
type Result<T, E = Error> = std::result::Result<T, E>;
```

So `anyhow::Result<T>` = `std::result::Result<T, anyhow::Error>`.

### Key Functions and Methods

| Item | Signature | Description |
|---|---|---|
| `anyhow!` | macro | Creates an `anyhow::Error` from a format string |
| `bail!` | macro | Returns `Err(anyhow!(...))` — like `return Err(anyhow!(...))` |
| `ensure!` | macro | `ensure!(condition, "msg")` — like `assert!` but returns `Err` |
| `.context(msg)` | `Result<T>` method | Wraps error with a message string (eager) |
| `.with_context(\|\| msg)` | `Result<T>` method | Wraps error with a closure (lazy) |
| `.downcast::<E>()` | `Error` method | Attempts to downcast to concrete type `E` (consumes) |
| `.downcast_ref::<E>()` | `Error` method | Downcasts to `&E` without consuming |
| `.downcast_mut::<E>()` | `Error` method | Downcasts to `&mut E` |
| `.source()` | `Error` method | Returns the underlying cause, if any |
| `.chain()` | `Error` method | Iterator over the error chain |
| `.root_cause()` | `Error` method | The deepest error in the chain (no source) |

### Example

```rust
use anyhow::{anyhow, bail, ensure, Context, Result};

fn load_port(path: &str) -> Result<u16> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path))?;

    let trimmed = contents.trim();
    ensure!(!trimmed.is_empty(), "port file is empty");

    let port = trimmed.parse::<u16>()
        .with_context(|| format!("'{}' is not a valid port number", trimmed))?;

    ensure!(port >= 1024, anyhow!("port {} is privileged (< 1024)", port));

    Ok(port)
}
```

### `anyhow::Error` vs `Box<dyn std::error::Error>`

| Feature | `anyhow::Error` | `Box<dyn Error>` |
|---|---|---|
| Context wrapping | `.context()` / `.with_context()` | Not available |
| Downcasting | `.downcast_ref::<E>()` (ergonomic) | `.downcast_ref::<E>()` (same, less ergonomic API) |
| Error chain iteration | `.chain()` | Manual `.source()` traversal |
| `Send + Sync` | Always | Only if specified in the type (`Box<dyn Error + Send + Sync>`) |
| Display | Full chain via `{:#}` | Only top-level |
| Stack capture | Optional (feature flag) | No |

---

## `std::io::Error` and `io::ErrorKind`

Source: [std::io::Error](https://doc.rust-lang.org/std/io/struct.Error.html)

`std::io::Error` is the most commonly encountered error type. It has a `kind()` method returning `ErrorKind`:

```rust
use std::io::ErrorKind;

match std::fs::read_to_string("file.txt") {
    Ok(s) => println!("{}", s),
    Err(e) => match e.kind() {
        ErrorKind::NotFound       => eprintln!("file not found"),
        ErrorKind::PermissionDenied => eprintln!("permission denied"),
        ErrorKind::TimedOut       => eprintln!("timed out"),
        _                         => eprintln!("other I/O error: {}", e),
    },
}
```

### Creating `io::Error`

```rust
// From a kind
let e = std::io::Error::new(std::io::ErrorKind::InvalidData, "bad checksum");

// From a string (stable since Rust 1.74)
let e = std::io::Error::other("unexpected EOF in header");

// From another error
let e = std::io::Error::new(std::io::ErrorKind::Other, custom_error);
```

---

## Printing Error Chains

Source: [std::error::Error source()](https://doc.rust-lang.org/std/error/trait.Error.html#method.source)

To print an error and all its causes:

```rust
fn print_error_chain(e: &dyn std::error::Error) {
    eprintln!("error: {}", e);
    let mut source = e.source();
    while let Some(cause) = source {
        eprintln!("caused by: {}", cause);
        source = cause.source();
    }
}
```

With `anyhow`, alternate format `{:#}` prints the full chain separated by `: `:

```rust
eprintln!("{:#}", anyhow_error);
// "failed to load config: I/O error: No such file or directory (os error 2)"
```

---

## `Cargo.toml` Error Handling Dependencies

```toml
[dependencies]
thiserror = "2"    # for library error types (derive macros)
anyhow = "1"       # for application error handling (ergonomic)

# Optional: for structured error reporting in CLI tools
# miette = "5"    # rich diagnostics with source spans
```

### Feature Flags

```toml
# anyhow: capture backtraces on error creation (nightly only for std::backtrace)
anyhow = { version = "1", features = ["backtrace"] }
```
