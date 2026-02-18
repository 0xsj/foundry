# Error Handling — Rust

## The Philosophy: Make Failure Explicit

Rust has no exceptions. There is no `try/catch`, no unchecked exception propagating up the call stack, no runtime surprise. Errors are values. They appear in function signatures. The compiler forces you to deal with them.

This is a deliberate design decision. In a language without a garbage collector, stack unwinding from exceptions is expensive and hard to make safe. More importantly, the Rust designers believe that **errors should be part of the API contract** — if a function can fail, that should be visible at the call site.

The mechanism is two enums built into the language:

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Option<T>` represents a value that may or may not be present. `Result<T, E>` represents an operation that may succeed (returning `T`) or fail (returning an error `E`).

**Comparison to Go:** Go has multiple return values with the `(value, error)` convention. Rust uses `Result<T, E>` — a single return value that encodes both possibilities. The key difference: in Go, you can ignore the error return value without a compiler warning. In Rust, ignoring a `Result` produces a compiler warning by default, and calling `.unwrap()` on an `Err` value panics at runtime.

**Comparison to TypeScript:** TypeScript `throw`/`catch` looks like Java. The `neverthrow` library offers Result types, but they're opt-in. Rust forces this discipline by default.

### Your notes
<!-- -->


---

## Result Basics

### Creating and Matching

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("division by zero"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e)     => println!("Error: {}", e),
    }
}
```

The exhaustive `match` on `Result` is the most explicit way to handle it. You must handle both arms.

### unwrap, expect, and Their Dangers

When you're certain an operation will succeed — in tests, in prototyping, or after explicit validation — you can extract the value with `unwrap()` or `expect()`:

```rust
let config_str = std::fs::read_to_string("config.toml")
    .expect("config.toml must exist at startup");  // panics with this message if Err

let port: u16 = "8080".parse().unwrap();  // panics with generic message if Err
```

`expect` is almost always better than `unwrap`: the message tells you *why* you expected success, which makes panics debuggable.

**When are these acceptable?**
- `unwrap()` on `Option::None` in tests — test panics are fine, they surface failures
- `expect()` on startup configuration that is truly required to proceed
- Parsing hardcoded literals that cannot fail (e.g., `"8080".parse::<u16>().unwrap()`)

**When are they not acceptable?**
- Any user-provided input
- Network calls, file I/O, database operations
- Any code path that might reasonably fail in production

### unwrap_or and unwrap_or_else

Provide fallback values instead of panicking:

```rust
let timeout_ms: u64 = std::env::var("TIMEOUT_MS")
    .unwrap_or_else(|_| String::from("5000"))  // closure called only on Err
    .parse()
    .unwrap_or(5000);  // fallback value on parse failure

// unwrap_or_default uses the type's Default impl
let port: u16 = parse_port("abc").unwrap_or_default();  // 0 for u16
```

`unwrap_or_else` accepts a closure — useful when the fallback value is expensive to compute (lazy evaluation).

### Your notes
<!-- -->


---

## The `?` Operator

The `?` operator is the idiomatic way to propagate errors up the call stack. It is syntactic sugar for a common pattern.

```rust
// Without ?
fn read_username_from_file() -> Result<String, std::io::Error> {
    let mut file = match std::fs::File::open("username.txt") {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}

// With ?
use std::io::Read;

fn read_username_from_file() -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open("username.txt")?;  // early return on Err
    let mut username = String::new();
    file.read_to_string(&mut username)?;                  // early return on Err
    Ok(username)
}
```

`?` does three things:
1. On `Ok(v)`: unwraps to `v`, execution continues
2. On `Err(e)`: converts the error with `From::from(e)`, then `return Err(converted_error)`
3. Works with `Option` too: `None` becomes `return None`

### The `From` Conversion

That conversion step is crucial. `?` calls `From::from(e)` before returning. This means you can use `?` across different error types, as long as you've implemented `From<SourceError>` for your error type:

```rust
#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> AppError {
        AppError::Io(e)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> AppError {
        AppError::Parse(e)
    }
}

fn load_port_from_file(path: &str) -> Result<u16, AppError> {
    let contents = std::fs::read_to_string(path)?;  // io::Error -> AppError::Io via From
    let port = contents.trim().parse::<u16>()?;     // ParseIntError -> AppError::Parse via From
    Ok(port)
}
```

Without `From`, `?` only works when the function's return error type matches exactly.

### `?` in `main`

`main` can return `Result<(), E>`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("config.toml")?;
    println!("{}", contents);
    Ok(())
}
```

`Box<dyn std::error::Error>` is a catch-all that works with any error type implementing the `Error` trait. More on this in the error strategies section.

### Your notes
<!-- -->


---

## Result Combinators

Instead of `match` everywhere, the standard library provides combinator methods on `Result` and `Option`. These allow chaining operations in a functional style.

```rust
// map: transform the Ok value, leave Err unchanged
let doubled: Result<i32, String> = Ok(5).map(|n| n * 2);        // Ok(10)
let still_err: Result<i32, String> = Err("bad".into()).map(|n: i32| n * 2); // Err("bad")

// map_err: transform the Err value, leave Ok unchanged
let mapped: Result<i32, String> = Err(42i32).map_err(|n| format!("error code {}", n));

// and_then: chain a fallible operation (flatMap in other languages)
let result: Result<u16, _> = std::fs::read_to_string("config")
    .and_then(|s| s.trim().parse::<u16>().map_err(|e| std::io::Error::other(e.to_string())));

// or_else: recover from an error
let recovered: Result<i32, String> = Err("temporary failure".to_string())
    .or_else(|_| Ok::<i32, String>(0));  // default to 0 on any error

// ok: convert Result<T, E> to Option<T> — discards the error
let maybe_value: Option<i32> = Ok::<i32, String>(42).ok();  // Some(42)
let nothing: Option<i32> = Err::<i32, String>("bad".into()).ok(); // None
```

### Chaining with `and_then`

`and_then` is the combinator equivalent of the `?` operator — it chains operations that can fail:

```rust
fn parse_and_double(s: &str) -> Result<i32, String> {
    s.parse::<i32>()
        .map_err(|e| format!("parse error: {}", e))
        .and_then(|n| {
            if n < 0 {
                Err(format!("negative value: {}", n))
            } else {
                Ok(n * 2)
            }
        })
}
```

This is more compact when chaining is natural, but `?` with explicit `if` checks is often more readable for complex logic. Prefer whichever is clearer at the call site.

### `Option` Combinators

`Option` has the same family: `map`, `and_then`, `or_else`, `unwrap_or`, `filter`:

```rust
let port: Option<u16> = Some("8080")
    .map(|s| s.trim())
    .and_then(|s| s.parse().ok());  // None if parse fails

let header: Option<&str> = None;
let default = header.unwrap_or("application/json");
```

### Your notes
<!-- -->


---

## Custom Error Types

When building anything beyond a script, `String` errors are not enough. You want:
- Structured variants (callers can match on the specific error kind)
- `impl std::error::Error` so your type works with the ecosystem
- `From` impls so `?` can convert lower-level errors

### The `std::error::Error` Trait

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

To implement it, your type needs:
1. `Debug` (usually derived)
2. `Display` (custom format for human-readable messages)
3. `Error` itself (often trivially, just `impl Error for MyError {}`)
4. Optionally: `source()` to chain errors (the cause of this error)

### Manual Implementation

```rust
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    MissingField(String),
    InvalidValue { field: String, value: String, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "I/O error: {}", e),
            ConfigError::MissingField(field) => write!(f, "missing required field: {}", field),
            ConfigError::InvalidValue { field, value, reason } => {
                write!(f, "invalid value for '{}': '{}' — {}", field, value, reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),  // chain the underlying io::Error
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}
```

This is verbose. For production code, the `thiserror` crate eliminates the boilerplate.

### Your notes
<!-- -->


---

## `thiserror`: Derive Macros for Error Types

`thiserror` is a procedural macro crate that generates the `Display`, `Error`, and `From` implementations for you. It is the standard choice for library error types.

```toml
[dependencies]
thiserror = "2"
```

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid value for '{field}': '{value}' — {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
}
```

The `#[error("...")]` attribute generates `Display`. The `#[from]` attribute generates `From<inner_type>` and makes the field the `source()`.

Named fields in the format string (like `{field}`) refer to struct fields by name. Positional `{0}` works for tuple variants.

**This replaces roughly 30 lines of boilerplate with 15 lines of intent-driven code.** Use `thiserror` in any non-trivial project.

### Wrapping Errors with Context

A common pattern is adding context to an error as it propagates up. With `thiserror`:

```rust
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("failed to load config from {path}: {source}")]
    Config {
        path: String,
        #[source]  // makes this the source() without generating From
        source: ConfigError,
    },
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),
}
```

The `#[source]` attribute marks the field as the underlying cause without generating `From` (because we want to control how the wrapping happens — passing the `path` context).

### Your notes
<!-- -->


---

## `anyhow`: Ergonomic Error Handling for Applications

`anyhow` is the counterpart to `thiserror`. Where `thiserror` is for *defining* structured error types (libraries), `anyhow` is for *using* errors ergonomically (applications).

```toml
[dependencies]
anyhow = "1"
```

```rust
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    // anyhow::Result<T> = Result<T, anyhow::Error>
    // anyhow::Error can hold any error that implements std::error::Error
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file at {}", path))?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("invalid TOML in {}", path))?;

    Ok(config)
}
```

`anyhow::Error` is a type-erased error: it holds any `Box<dyn Error + Send + Sync>`. The `.context()` and `.with_context()` methods add a message wrapping the original error.

**Key features:**
- `anyhow!("message")` creates an ad-hoc error from a format string
- `.context("msg")` wraps an error with a message (eager — always allocates the string)
- `.with_context(|| "msg")` wraps an error lazily (closure — only allocates on error path)
- `downcast_ref::<ConcreteError>()` recovers the original error type for programmatic handling

### When to Use Which

| Situation | Use |
|---|---|
| Library crate | `thiserror` — callers need structured errors to match on |
| Application (binary) | `anyhow` — you're logging or displaying errors, not matching on variants |
| Internal module in an app | Often `thiserror` for internal types, then convert to `anyhow` at boundaries |
| Prototyping | `anyhow` — ergonomic, worry about structure later |
| `Box<dyn Error>` | Avoid in new code — `anyhow` is strictly better for the same purpose |

**The rule of thumb:** If a caller needs to programmatically distinguish between error variants, use `thiserror`. If errors are ultimately being logged or displayed to a user, use `anyhow`.

### Your notes
<!-- -->


---

## `Box<dyn Error>` as a Catch-All

Before `anyhow` existed, `Box<dyn std::error::Error>` was the idiomatic catch-all:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = std::fs::read_to_string("file.txt")?;
    let n: i32 = s.trim().parse()?;
    println!("{}", n);
    Ok(())
}
```

This works because:
1. Any type implementing `Error` can be converted to `Box<dyn Error>` automatically
2. `?` performs this conversion via a blanket `From` impl in the standard library

The limitation: you can't easily match on the error type to recover. `downcast_ref()` works but is awkward. Prefer `anyhow` in new code — it adds the same ergonomics plus `.context()` and richer diagnostics.

### Downcasting

Both `Box<dyn Error>` and `anyhow::Error` support downcasting — recovering the concrete error type:

```rust
use anyhow::Error;

fn handle_error(e: Error) {
    if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => eprintln!("file not found"),
            std::io::ErrorKind::PermissionDenied => eprintln!("permission denied"),
            _ => eprintln!("I/O error: {}", io_err),
        }
    } else {
        eprintln!("unknown error: {}", e);
    }
}
```

This is analogous to Java's `instanceof` check or TypeScript's `error instanceof SomeErrorClass`.

### Your notes
<!-- -->


---

## `panic!` vs `Result`

Rust has two failure modes:

| Mechanism | Use when |
|---|---|
| `Result<T, E>` | The error is expected and recoverable. Callers should handle it. |
| `panic!` | A programming invariant has been violated. The program cannot continue safely. |

### When to Panic

- **Index out of bounds**: `vec[n]` — a bug, not a user error
- **Integer overflow in debug builds**: `255u8 + 1` — caught by default in debug
- **Failed assertions**: `assert!()`, `assert_eq!()` — invariants that must hold
- **Unreachable code**: `unreachable!("should not be here")` — logical bug
- **Unimplemented stubs**: `todo!("implement later")`
- **Required startup configuration missing**: when there is truly no sensible default

### When to Use Result

Everything that could reasonably happen in the normal course of operation: file not found, network timeout, invalid user input, malformed data, database constraint violation.

```rust
// This should be a panic — it's a programming error
fn get_required_config_key(config: &HashMap<String, String>, key: &str) -> &str {
    config.get(key).expect("BUG: required key not in config at startup")
}

// This should be Result — the file might not exist
fn read_user_preferences(user_id: u64) -> Result<Preferences, ConfigError> {
    let path = format!("prefs/{}.toml", user_id);
    let contents = std::fs::read_to_string(&path)?;
    toml::from_str(&contents).map_err(ConfigError::Parse)
}
```

**The question to ask:** "Could a correct, well-written caller find themselves in this error state?" If yes, it's `Result`. If the only way to reach this state is a bug in the caller's code, it's `panic`.

### Panics in Libraries vs Applications

Libraries should almost never panic on user-controlled input. A panic in a library brought down production for the caller. Libraries should return `Result` and let the application decide whether to panic.

Applications can panic more freely — a controlled panic with a good error message is often better than silently proceeding with bad state.

### Your notes
<!-- -->


---

## Error Propagation Patterns in Practice

### Adding Context as Errors Propagate

The most important habit is adding context as errors move up the stack:

```rust
use anyhow::{Context, Result};

fn load_user(id: u64) -> Result<User> {
    let path = format!("users/{}.json", id);
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read user file for id {}", id))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("invalid JSON in user file for id {}", id))
}

fn handle_request(id: u64) -> Result<Response> {
    let user = load_user(id)
        .context("could not load user for request")?;
    Ok(Response::ok(user))
}
```

Without context, an error like `No such file or directory (os error 2)` gives no clue about *which* file or *why* we were reading it. With context layering, the error chain tells a story.

### Collecting Multiple Errors

`?` short-circuits on the first error. Sometimes you want to collect *all* errors — for form validation, for example:

```rust
fn validate_config(fields: &[(&str, &str)]) -> Result<(), Vec<String>> {
    let errors: Vec<String> = fields
        .iter()
        .filter_map(|(key, value)| {
            if value.is_empty() {
                Some(format!("field '{}' is required", key))
            } else {
                None
            }
        })
        .collect();

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

Standard `?` propagation is not for this. This is a legitimate use case for manual accumulation.

### Iterators and Results

`Iterator::collect()` has a special implementation for `Result<Vec<T>, E>`:

```rust
// Try to parse all strings; stop at first failure
let results: Result<Vec<u16>, _> = vec!["80", "443", "not_a_port", "8080"]
    .iter()
    .map(|s| s.parse::<u16>())
    .collect();  // Err(ParseIntError) — stops at "not_a_port"

// Use filter_map to skip failures instead of aborting
let valid_ports: Vec<u16> = vec!["80", "443", "not_a_port", "8080"]
    .iter()
    .filter_map(|s| s.parse::<u16>().ok())
    .collect();  // [80, 443, 8080]
```

The `.collect::<Result<Vec<_>, _>>()` pattern is idiomatic: all succeed or the first failure wins.

### Your notes
<!-- -->


---

## Cross-Language Comparison

### Error Handling Approaches

| Concept | Rust | Go | TypeScript |
|---|---|---|---|
| Primary mechanism | `Result<T, E>` enum | `(T, error)` multiple return | `throw/catch` exceptions |
| Propagation | `?` operator | `if err != nil { return ..., err }` | implicit (bubble up) |
| Ignoring errors | Compiler warns | No warning — easy to ignore | No compile error |
| Custom errors | `impl std::error::Error` | `errors.New` / `fmt.Errorf` / custom type | `extends Error` class |
| Error wrapping | `thiserror`, `anyhow` | `fmt.Errorf("msg: %w", err)` | No standard pattern |
| Panic equivalent | `panic!` | `panic()` | `throw` unchecked, `never` type |
| Nullable values | `Option<T>` | pointer nil checks | `T \| null \| undefined` |

### The ? Operator vs Go's if err != nil

The mechanical difference is noise:

```rust
// Rust
fn process(path: &str) -> Result<Data, AppError> {
    let text = std::fs::read_to_string(path)?;
    let parsed = parse_text(&text)?;
    let validated = validate(&parsed)?;
    Ok(transform(validated))
}
```

```go
// Go
func process(path string) (Data, error) {
    text, err := os.ReadFile(path)
    if err != nil {
        return Data{}, fmt.Errorf("reading %s: %w", path, err)
    }
    parsed, err := parseText(text)
    if err != nil {
        return Data{}, fmt.Errorf("parsing: %w", err)
    }
    validated, err := validate(parsed)
    if err != nil {
        return Data{}, fmt.Errorf("validating: %w", err)
    }
    return transform(validated), nil
}
```

The Go version is more verbose, but notice something: the Go version adds context at each step (`"reading %s: %w"`, `"parsing: %w"`). The Rust version does not — `?` just propagates. If you want context in Rust, you need `.with_context()` from anyhow, or `map_err`. This is a real tradeoff: `?` is terse but strips context unless you add it explicitly.

### Error Wrapping Comparison

Go: `fmt.Errorf("loading config: %w", err)` — wraps with a string message, `errors.Is`/`errors.As` traverse the chain

Rust/thiserror: explicit `#[source]` field in variant — the chain is structural, traversable via `.source()`

Rust/anyhow: `.context("loading config")` — similar to Go's wrapping, but richer

### Your notes
<!-- -->
