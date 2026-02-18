# Rust Reference — Testing Fundamentals

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Programming Language (Book)](https://doc.rust-lang.org/book/ch11-00-testing.html),
> and [Rustdoc](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
> for the `testing-fundamentals` module. Covers: `#[test]` attribute, assertion macros,
> `#[should_panic]`, `Result`-returning tests, `#[cfg(test)]`, `#[ignore]`, doc tests,
> integration tests, and `cargo test` flags.

---

## The `#[test]` Attribute

Source: [reference/attributes/testing.html](https://doc.rust-lang.org/reference/attributes/testing.html)

The `#[test]` attribute marks a function as a test function. Test functions must have
the signature `fn() -> ()` or `fn() -> Result<(), E>` where `E: Error`.

```
#[test]
fn test_name() { ... }
```

**Constraints:**
- Must be a bare function (not a method in an `impl` block)
- May not be `async` without an async test runtime (e.g., `#[tokio::test]`)
- May not have parameters
- Compiled and discovered only when building with `--test` flag or via `cargo test`

The test harness is provided by `libtest`, part of the Rust standard distribution.

---

## Assertion Macros

Source: [std::macro.assert](https://doc.rust-lang.org/std/macro.assert.html),
[std::macro.assert_eq](https://doc.rust-lang.org/std/macro.assert_eq.html),
[std::macro.assert_ne](https://doc.rust-lang.org/std/macro.assert_ne.html)

### `assert!(expr)` / `assert!(expr, msg, ...)`

Panics if `expr` evaluates to `false`. Optional format string and arguments are passed to
`panic!` on failure.

```rust
assert!(1 + 1 == 2);
assert!(x.is_some(), "expected Some but got None for input {:?}", x);
```

### `assert_eq!(left, right)` / `assert_eq!(left, right, msg, ...)`

Panics if `left != right`. Requires `left` and `right` to implement `PartialEq` and `Debug`.
The failure message prints both values labeled `left` and `right`.

```rust
assert_eq!(2 + 2, 4);
assert_eq!(result, expected, "mismatch for input {:?}", input);
```

### `assert_ne!(left, right)` / `assert_ne!(left, right, msg, ...)`

Panics if `left == right`. Same type requirements as `assert_eq!`.

```rust
assert_ne!(first_id, second_id);
```

### Panic Behavior

All three macros call `panic!` with the format string on failure. Since each test runs
in its own OS thread, the panic is caught by the test harness — it does not terminate
the process. Other tests continue running.

---

## `#[should_panic]`

Source: [reference/attributes/testing.html#the-should_panic-attribute](https://doc.rust-lang.org/reference/attributes/testing.html#the-should_panic-attribute)

Marks a test that is expected to panic. The test passes if the body panics, fails if
it returns normally.

```rust
#[test]
#[should_panic]
fn panics_on_bad_input() {
    parse("invalid");
}
```

### `expected` Subattribute

```rust
#[test]
#[should_panic(expected = "substring")]
fn panics_with_specific_message() {
    panic!("this is the substring of the panic message");
}
```

The `expected` value is matched as a **substring** of the panic message. The test
fails if the panic message does not contain `expected`.

If both `#[should_panic]` and a return value are present, `#[should_panic]` takes
precedence — a return value of `Ok(())` is not reached when a panic is expected.

---

## Result-Returning Tests

Source: [book/ch11-01-writing-tests.html#using-resultt-e-in-tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html#using-resultt-e-in-tests)

Test functions may return `Result<(), E>` where `E` implements the `Error` trait or
can be displayed. Returning `Err(e)` causes the test to fail with `e`'s `Debug` output.

```rust
#[test]
fn it_works() -> Result<(), String> {
    if 2 + 2 == 4 {
        Ok(())
    } else {
        Err(String::from("two plus two does not equal four"))
    }
}
```

The `?` operator can be used inside result-returning tests:

```rust
#[test]
fn parses_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_str("key=value")?;
    assert_eq!(config.get("key"), Some("value"));
    Ok(())
}
```

**Note:** `#[should_panic]` cannot be combined with `Result<(), E>` return type.

---

## `#[cfg(test)]`

Source: [reference/conditional-compilation.html](https://doc.rust-lang.org/reference/conditional-compilation.html),
[book/ch11-03-test-organization.html](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

The `cfg(test)` configuration predicate is true only when compiling for test (`--test`
flag or `cargo test`). Code in `#[cfg(test)]` blocks is excluded from non-test builds.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() { /* ... */ }
}
```

Idiomatic placement: at the end of the source file containing the code under test.

The test module is a regular child module. `use super::*` imports items from the parent
module. Items declared `pub(crate)` or without visibility restriction in the parent
module are accessible to the test module, including private items (same file).

---

## `#[ignore]`

Source: [reference/attributes/testing.html#the-ignore-attribute](https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute)

Marks a test to be skipped during a normal `cargo test` run.

```rust
#[test]
#[ignore]
fn slow_integration_test() { /* ... */ }
```

**Invocation to run ignored tests:**
```
cargo test -- --ignored           # run only ignored tests
cargo test -- --include-ignored   # run all tests including ignored
```

May be used with `#[should_panic]`.

---

## Integration Tests

Source: [book/ch11-03-test-organization.html#integration-tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)

Integration test files live in the `tests/` directory at the crate root. Each `.rs` file
in `tests/` is compiled as a separate crate. These crates link against the library crate
and can only access its public API.

```
crate-root/
├── src/
│   └── lib.rs
└── tests/
    ├── integration_test_1.rs
    └── integration_test_2.rs
```

No `#[cfg(test)]` annotation is required — files in `tests/` are only compiled during test runs.

Subdirectories under `tests/` can contain shared helpers. A file at `tests/common/mod.rs`
is not treated as a test file but can be imported by test files.

### Selecting Integration Tests

```
cargo test --test integration_test_1    # run tests in tests/integration_test_1.rs only
```

---

## Doc Tests

Source: [rustdoc/write-documentation/documentation-tests.html](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

Code blocks in doc comments (preceded by `///` or `/** */`) that are tagged as `rust`
(or untagged, since `rust` is the default) are compiled and run as tests.

```rust
/// Returns the sum of two numbers.
///
/// ```
/// let result = my_crate::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

### Hidden Setup Lines

Lines beginning with `# ` (hash-space) are included in compilation but hidden in rendered
HTML documentation:

```rust
/// ```
/// # use my_crate::Foo;
/// # let mut foo = Foo::new();
/// foo.do_something();  // this line appears in docs
/// ```
```

### Doc Test Attributes

| Annotation in code block header | Effect |
|---|---|
| ` ```rust ` | Compiled and run as a test (default) |
| ` ```rust,no_run ` | Compiled but not run |
| ` ```rust,compile_fail ` | Expected to fail to compile |
| ` ```rust,ignore ` | Not compiled or run |
| ` ```text ` | Not treated as Rust code |

### Panics in Doc Tests

Doc tests may test for panics using `should_panic`:

```rust
/// ```should_panic
/// my_crate::divide(1, 0);  // panics on division by zero
/// ```
```

---

## `cargo test` Reference

Source: [doc.rust-lang.org/cargo/commands/cargo-test.html](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

### Cargo Flags (before `--`)

| Flag | Description |
|---|---|
| `--lib` | Run unit tests (`src/`) only |
| `--doc` | Run doc tests only |
| `--test <name>` | Run integration test file `tests/<name>.rs` only |
| `--bins` | Run tests from binary targets |
| `--all-targets` | Run tests from all targets |
| `--release` | Build tests with optimizations |
| `--features <feat>` | Enable specified features |
| `--no-fail-fast` | Continue running after a test failure |

### Test Harness Flags (after `--`)

| Flag | Description |
|---|---|
| `<filter>` | Only run tests whose names contain this string |
| `--nocapture` | Print output from tests (default: captured) |
| `--test-threads=<n>` | Number of threads for parallel test execution |
| `--ignored` | Run only `#[ignore]`-marked tests |
| `--include-ignored` | Run all tests including `#[ignore]` |
| `-q` | Quiet output |
| `--list` | Print test names without running them |

### Examples

```sh
cargo test                            # all tests
cargo test rate_limit                 # tests whose name contains "rate_limit"
cargo test --lib -- --nocapture       # unit tests, show stdout
cargo test -- --test-threads=1        # run tests serially
cargo test -- --ignored               # only skipped tests
```

---

## `std::panic::catch_unwind`

Source: [std::panic::catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)

Invokes a closure, catching any panic. Returns `Ok(value)` if the closure completed
normally, or `Err(payload)` if it panicked.

```rust
use std::panic;

let result = panic::catch_unwind(|| {
    RateLimiter::new(0, 1000, SystemClock)
});

assert!(result.is_err(), "expected panic for zero capacity");
```

**Constraints:**
- The closure must be `UnwindSafe` (or wrapped with `AssertUnwindSafe`)
- Does not catch panics that use `panic = "abort"` in the profile
- Does not catch foreign (C) exceptions
- Intended for test infrastructure and FFI boundary code, not general error handling

---

## Type Requirements for Assertion Macros

| Macro | Required traits on compared types |
|---|---|
| `assert!` | None (expression must be `bool`) |
| `assert_eq!` | `PartialEq + Debug` |
| `assert_ne!` | `PartialEq + Debug` |

Derive both for custom types used in assertions:

```rust
#[derive(Debug, PartialEq)]
struct Config {
    capacity: u32,
    window_millis: u64,
}
```

---

## Cargo.toml: Dev Dependencies

Test-only dependencies go under `[dev-dependencies]`. They are compiled only for test
and example targets, not included in library or binary release builds.

```toml
[dev-dependencies]
proptest = "1"
mockall = "0.12"
```

Referenced from test code as any other dependency:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    // ...
}
```
