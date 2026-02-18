# Testing Fundamentals — Rust

## How Rust's Test System Works Under the Hood

### Tests Are Just Functions

In Rust, tests are ordinary functions annotated with `#[test]`. The compiler — specifically
`rustc --test` or `cargo test` — links in a built-in test harness (from `libtest`) when
the `--test` flag is present. That harness discovers every `#[test]`-annotated function,
runs each one in its own thread, and reports pass/fail/panic.

```rust
#[test]
fn config_rejects_zero_timeout() {
    let result = parse_timeout("0");
    assert!(result.is_err());
}
```

There is no magic framework to install. No `import { expect }` at the top. No test runner
config file. `cargo test` compiles your crate in test mode and runs the built-in harness.

The harness catches panics (each test runs in a thread). A test passes if the function
returns normally. A test fails if the function panics — including via any of the assertion
macros, which all panic on failure.

**Comparison to Go:** Go has `func TestFoo(t *testing.T)` and uses `t.Error()` /
`t.Fatal()` to signal failure. Rust's assertions panic directly, which is simpler but
means failure is less controlled — a panicking test always stops that test immediately
(no equivalent of `t.Error` continuing execution). Go's table-driven tests are idiomatic;
Rust doesn't have a built-in equivalent but achieves the same thing with loops or macros.

**Comparison to TypeScript/Jest:** Jest tests call `expect(x).toBe(y)` which throws on
failure. Rust's `assert_eq!(x, y)` does the same thing — panics on mismatch. The mental
model is identical, just different syntax.

### Your notes
<!-- -->


---

## The Assertion Macros

### `assert!` — Boolean Check

Panics if the expression is false.

```rust
#[test]
fn rate_limit_allows_first_request() {
    let mut limiter = RateLimiter::new(10, Duration::from_secs(1));
    assert!(limiter.check());
}
```

With a custom failure message (the format string gets evaluated only on failure):

```rust
#[test]
fn rate_limit_blocks_after_capacity() {
    let mut limiter = RateLimiter::new(2, Duration::from_secs(1));
    limiter.check(); // 1
    limiter.check(); // 2 — at capacity
    let allowed = limiter.check(); // 3 — should be blocked
    assert!(
        !allowed,
        "expected request 3 to be blocked, limiter capacity is 2"
    );
}
```

The message is a format string, not a plain `&str`:

```rust
assert!(
    result.is_ok(),
    "parse failed with input {:?}: {:?}",
    input,
    result.err()
);
```

### `assert_eq!` and `assert_ne!` — Equality Checks

`assert_eq!(left, right)` panics if left != right, printing both values.
`assert_ne!(left, right)` panics if left == right.

```rust
#[test]
fn bucket_refills_after_window() {
    let mut limiter = RateLimiter::new(5, Duration::from_secs(60));
    for _ in 0..5 { limiter.check(); }

    // Simulate time passing
    limiter.advance_time(Duration::from_secs(60));

    let tokens = limiter.available_tokens();
    assert_eq!(tokens, 5, "bucket should fully refill after one window");
}
```

**The argument order convention:** `assert_eq!(actual, expected)` is one school;
`assert_eq!(expected, actual)` is another. Rust's `libtest` labels them `left` and `right`
in failure output — it does not enforce a semantic convention. The failure message prints:

```
thread 'bucket_refills_after_window' panicked at 'assertion `left == right` failed
  left: 3
 right: 5'
```

Pick a convention and stick to it. Many Rustaceans use `assert_eq!(actual, expected)`
because it reads like "assert that actual equals expected." Go's `t.Errorf` typically
puts expected first; Jest puts expected second (`expect(actual).toBe(expected)`).

**What's required:** The type must implement `PartialEq` for `assert_eq!` and `Debug`
for the failure message. Most standard types do. For your own types, derive both:

```rust
#[derive(Debug, PartialEq)]
struct TokenState {
    available: u32,
    window_start: u64,
}
```

### Custom Messages

All three macros accept an optional format string after the required arguments:

```rust
assert_eq!(
    limiter.available_tokens(), 5,
    "after refill, expected 5 tokens but got {} (window: {:?})",
    limiter.available_tokens(),
    limiter.current_window()
);
```

The message is evaluated lazily — only computed when the assertion fails. This means
you can call functions in the message without paying cost on success.

### Your notes
<!-- -->


---

## `#[should_panic]` — Testing Expected Panics

When your API is designed to panic on invalid inputs, you test that the panic happens:

```rust
#[test]
#[should_panic]
fn rate_limiter_rejects_zero_capacity() {
    RateLimiter::new(0, Duration::from_secs(1));  // should panic
}
```

This test passes if the function panics, and fails if it returns normally.

**The problem:** `#[should_panic]` is too broad. It passes for any panic — even an
unrelated `unwrap()` failure or an out-of-bounds access. Use the `expected` attribute
to match against the panic message:

```rust
#[test]
#[should_panic(expected = "capacity must be > 0")]
fn rate_limiter_rejects_zero_capacity() {
    RateLimiter::new(0, Duration::from_secs(1));
}
```

The test passes only if the panic message *contains* the expected string (substring match,
not exact match). This pins the test to the right panic, not just any panic.

**When to use `#[should_panic]`:**
- Constructors that enforce invariants (`RateLimiter::new(0, ...)`)
- Index operations you expect to be out of bounds
- Explicit `panic!()` in unreachable branches

**When to use `Result<_, E>` instead:** If the caller might reasonably handle the error,
return `Result`. Reserve panics for programmer errors — inputs that should never reach
production code. Testing via `should_panic` means you've decided this is a hard contract.

### Your notes
<!-- -->


---

## Result-Returning Tests

Tests can return `Result<(), E>` instead of `()`. If the test returns `Err`, it fails
with the error printed. This lets you use the `?` operator directly in test bodies —
no `unwrap()` required.

```rust
#[test]
fn config_file_parses_correctly() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r#"{"capacity": 100, "window_secs": 60}"#;
    let config: LimiterConfig = serde_json::from_str(raw)?;  // ? operator works!

    assert_eq!(config.capacity, 100);
    assert_eq!(config.window_secs, 60);

    Ok(())
}
```

**When to use this:**
- Tests that call fallible functions and would otherwise need many `.unwrap()` calls
- Tests involving I/O (reading files, network — rare in unit tests)
- Anywhere you want the propagation convenience of `?`

**Comparison to Go:** Go doesn't have `?`. You write `if err != nil { t.Fatal(err) }`.
Rust's result-returning tests are more ergonomic for error-heavy code paths.

**The `E` type:** `Box<dyn std::error::Error>` is the most flexible — it accepts any
error type. For tests where all errors come from a single type, use that type directly.

### Your notes
<!-- -->


---

## `#[cfg(test)]` — The Test Module Convention

Unit tests in Rust live in a module annotated with `#[cfg(test)]`, usually at the bottom
of the same file as the code being tested. The `cfg(test)` attribute means this module
is only compiled when running tests — it does not appear in your release binary.

```rust
// src/rate_limiter.rs

pub struct RateLimiter {
    capacity: u32,
    available: u32,
    window: Duration,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(capacity: u32, window: Duration) -> Self {
        if capacity == 0 {
            panic!("capacity must be > 0");
        }
        // ...
    }

    pub fn check(&mut self) -> bool {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;  // bring the module's items into scope

    #[test]
    fn new_limiter_has_full_capacity() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        assert_eq!(limiter.available_tokens(), 10);
    }
}
```

`use super::*` is standard here. The test module is a child module, so it needs to
explicitly import from the parent. Because it's in the same *file*, it has access to
`pub(crate)` and even private items — this is intentional. Unit tests in Rust are
allowed to test private implementation.

**Comparison to Go:** Go test files are in the same package (same directory, `_test.go`
suffix). They can test unexported identifiers. Same philosophy — unit tests have white-box
access. External test packages (e.g., `package mylib_test`) are used for black-box testing.
Rust achieves the same distinction via `mod tests { use super::* }` (white-box) vs
integration tests in `tests/` (black-box).

### Why `cfg(test)` Matters

Without `cfg(test)`, test dependencies and test helper code would be compiled into your
production binary. `cfg(test)` ensures:
- Test-only code (fixtures, helpers, mocks) is excluded from release builds
- Test-only dependencies (from `[dev-dependencies]` in Cargo.toml) are not linked
- Build times for production builds aren't affected by test code

### Your notes
<!-- -->


---

## Test Organization: Unit vs Integration Tests

### Unit Tests — Same File, Private Access

Located inside `#[cfg(test)] mod tests` in the source file. Can access private fields
and functions. These are your fast, surgical tests.

```
src/
├── lib.rs
├── rate_limiter.rs    ← #[cfg(test)] mod tests { ... } at the bottom
└── token_bucket.rs    ← same
```

### Integration Tests — The `tests/` Directory

Integration tests live in a `tests/` directory at the crate root (next to `src/`). Each
file is compiled as a separate crate that links against your library. This means they can
only access your public API — exactly what a downstream user would see.

```
my-crate/
├── src/
│   └── lib.rs
└── tests/
    ├── rate_limiter_integration.rs    ← tests public API only
    └── config_loading.rs
```

```rust
// tests/rate_limiter_integration.rs
use my_crate::RateLimiter;  // only public items

#[test]
fn rate_limiter_enforces_capacity_across_calls() {
    let mut limiter = RateLimiter::new(3, std::time::Duration::from_secs(60));
    assert!(limiter.check()); // 1
    assert!(limiter.check()); // 2
    assert!(limiter.check()); // 3
    assert!(!limiter.check()); // 4 — blocked
}
```

Run only integration tests:
```
cargo test --test rate_limiter_integration
```

Run only unit tests (no integration tests, no doc tests):
```
cargo test --lib
```

### Your notes
<!-- -->


---

## Doc Tests — Tests in Documentation

Any ```` ```rust ```` block in a doc comment is compiled and run as a test:

```rust
/// A token bucket rate limiter.
///
/// ```
/// use my_crate::{RateLimiter, Duration};
///
/// let mut limiter = RateLimiter::new(5, Duration::from_secs(60));
/// assert!(limiter.check()); // first request allowed
/// ```
pub struct RateLimiter { /* ... */ }
```

Run doc tests only:
```
cargo test --doc
```

**Why doc tests are powerful:**
- Documentation that shows usage — and the compiler guarantees it compiles and produces the described output
- Documentation that drifts from the actual API causes the test to fail
- Users see working examples; developers can't accidentally break them

**What to doc test:** Public API entry points, constructor examples, common usage patterns.
Don't doc test complex internal logic — that belongs in `#[cfg(test)]` unit tests.

**The `# ` prefix hides lines from rendered docs:**

```rust
/// ```
/// # use my_crate::RateLimiter;
/// # use std::time::Duration;
/// # let mut limiter = RateLimiter::new(5, Duration::from_secs(1));
/// // Check returns true when tokens are available
/// assert!(limiter.check());
/// ```
```

Lines starting with `# ` are compiled (needed for the example to work) but not shown in
the rendered documentation. Use them for boilerplate setup the reader doesn't need to see.

### Your notes
<!-- -->


---

## Test Fixtures and Setup Patterns

Rust has no `BeforeEach` / `AfterEach` hooks. The idiomatic approach is to extract
setup into a helper function:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Shared fixture — returns a pre-configured limiter
    fn standard_limiter() -> RateLimiter {
        RateLimiter::new(10, Duration::from_secs(60))
    }

    fn exhausted_limiter() -> RateLimiter {
        let mut limiter = standard_limiter();
        for _ in 0..10 { limiter.check(); }
        limiter
    }

    #[test]
    fn fresh_limiter_allows_requests() {
        let mut limiter = standard_limiter();
        assert!(limiter.check());
    }

    #[test]
    fn exhausted_limiter_blocks_requests() {
        let mut limiter = exhausted_limiter();
        assert!(!limiter.check());
    }
}
```

**No shared mutable state between tests.** Each test function should create its own
fixtures. Rust's test runner runs tests in parallel by default — shared mutable state
causes races and flaky tests. If you genuinely need serialized access (e.g., tests that
touch a database or file), use `#[serial_test]` from a crate, or design tests to use
isolated state.

**Comparison to Go:** Go uses `TestMain(m *testing.M)` for suite-level setup/teardown and
helper functions for per-test fixtures. Same philosophy: favor explicit helpers over magic hooks.

### Your notes
<!-- -->


---

## Test Doubles Using Traits

Rust has no mocking framework built in. The idiomatic approach: define a trait for the
dependency, implement it with your real type and with a fake type for testing.

The real scenario: a rate limiter needs a time source. In production, use `Instant::now()`.
In tests, control time precisely.

```rust
// Define a trait for the clock dependency
pub trait Clock {
    fn now_millis(&self) -> u64;
}

// Production implementation
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

// Rate limiter takes any Clock
pub struct RateLimiter<C: Clock> {
    capacity: u32,
    available: u32,
    window_millis: u64,
    window_start: u64,
    clock: C,
}

impl<C: Clock> RateLimiter<C> {
    pub fn new(capacity: u32, window_millis: u64, clock: C) -> Self {
        RateLimiter {
            capacity,
            available: capacity,
            window_millis,
            window_start: clock.now_millis(),
            clock,
        }
    }

    pub fn check(&mut self) -> bool {
        let now = self.clock.now_millis();
        if now - self.window_start >= self.window_millis {
            self.available = self.capacity;
            self.window_start = now;
        }
        if self.available > 0 {
            self.available -= 1;
            true
        } else {
            false
        }
    }
}

// Test fake — controls time explicitly
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct FakeClock {
        now: Cell<u64>,  // Cell for interior mutability — allows &self mutation
    }

    impl FakeClock {
        fn new(start: u64) -> Self {
            FakeClock { now: Cell::new(start) }
        }

        fn advance(&self, millis: u64) {
            self.now.set(self.now.get() + millis);
        }
    }

    impl Clock for FakeClock {
        fn now_millis(&self) -> u64 {
            self.now.get()
        }
    }

    #[test]
    fn window_resets_after_elapsed_time() {
        let clock = FakeClock::new(1_000_000);
        let mut limiter = RateLimiter::new(2, 1000, clock);

        assert!(limiter.check()); // 1
        assert!(limiter.check()); // 2
        assert!(!limiter.check()); // blocked

        limiter.clock.advance(1001); // advance past the window
        assert!(limiter.check()); // should be allowed — window reset
    }
}
```

**Why `Cell<u64>` and not `mut u64`?** The `Clock` trait's `now_millis` takes `&self`
(shared reference). A test fake needs to advance time via `&self` too (so we can call
`advance` after passing the clock to the limiter). `Cell<T>` provides interior mutability —
it allows mutation through a shared reference. For types that don't implement `Copy`,
you'd use `RefCell<T>` instead.

**The trait-based fake vs a mocking framework:**
- No runtime overhead — the fake is a concrete type, calls are monomorphized
- Compiler checks the fake implements the trait correctly
- You must write the fake by hand — but it's usually 10-20 lines
- Real mocking frameworks (like `mockall`) generate this boilerplate via macros

### Your notes
<!-- -->


---

## `#[ignore]` — Slow or External Tests

Mark tests that are slow (integration tests hitting a real database) or require external
resources not always available:

```rust
#[test]
#[ignore]
fn rate_limiter_sustained_load_test() {
    // Runs 100k requests over 10 seconds — too slow for default test run
    let mut limiter = RateLimiter::new(1000, Duration::from_secs(1));
    for i in 0..100_000 {
        let _ = limiter.check();
        if i % 1000 == 0 { std::thread::sleep(Duration::from_millis(1)); }
    }
}
```

By default, `cargo test` skips `#[ignore]` tests.

Run ignored tests explicitly:
```
cargo test -- --ignored
```

Run all tests including ignored:
```
cargo test -- --include-ignored
```

### Your notes
<!-- -->


---

## `cargo test` Flags

```
cargo test                          # run all tests (unit + integration + doc)
cargo test --lib                    # only unit tests (src/)
cargo test --doc                    # only doc tests
cargo test --test my_integration    # only tests/my_integration.rs
cargo test rate_limit               # only tests whose name contains "rate_limit"
cargo test -- --nocapture           # show println! output (default: captured)
cargo test -- --test-threads=1      # run tests serially (no parallelism)
cargo test -- --ignored             # run only #[ignore] tests
cargo test -- -q                    # quiet output (dots, not full test names)
```

**The `--` separator:** Arguments before `--` go to `cargo test`. Arguments after `--`
go to the test harness (`libtest`). So `cargo test --lib -- --nocapture` runs lib tests
with output printing enabled.

**Test name filtering:** `cargo test rate_limit` runs any test whose fully-qualified name
contains the string `rate_limit` — this includes the module path, so
`tests::rate_limit_blocks_after_capacity` would match.

### Your notes
<!-- -->


---

## Property-Based Testing with `proptest` (Awareness)

Property-based testing generates hundreds of random inputs and checks that a property
holds for all of them. Instead of "does `check()` return true for input 5?", you assert
"does `check()` always return true when tokens are available, regardless of capacity?".

The `proptest` crate is the Rust standard for this. Brief example:

```rust
// Cargo.toml [dev-dependencies]:
// proptest = "1"

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fresh_limiter_allows_capacity_requests(capacity in 1u32..=1000) {
            let mut limiter = RateLimiter::new(capacity, 60_000, SystemClock);
            for _ in 0..capacity {
                assert!(limiter.check());
            }
            assert!(!limiter.check()); // one beyond capacity is blocked
        }
    }
}
```

`proptest!` generates random values for each variable within the given range (a
"strategy") and shrinks failing inputs to the minimal reproducing case.

**When property-based testing shines:**
- Invariants that should hold for all inputs: "encode then decode = identity"
- Boundary conditions: parsers, arithmetic, data structure invariants
- Comparing two implementations: "my fast implementation equals the naive implementation"

This module won't go deep on proptest — it's covered in the patterns module.

### Your notes
<!-- -->


---

## Cross-Language Comparison

### Test Mechanisms

| Concept | Rust | Go | TypeScript (Jest) |
|---|---|---|---|
| Test function marker | `#[test]` attribute | `func TestFoo(t *testing.T)` | `test('name', () => {...})` |
| Test failure | Panic (`assert!` panics) | `t.Error()` / `t.Fatal()` | throw (Jest catches it) |
| Continue after failure | No (panic stops the test) | Yes (`t.Error` continues) | Configurable |
| Equality | `assert_eq!(a, b)` | `if a != b { t.Errorf(...) }` | `expect(a).toBe(b)` |
| Expected panic | `#[should_panic(expected="...")]` | `defer recover()` | `expect(() => ...).toThrow()` |
| Table-driven tests | Loop in test body | Idiomatic with `t.Run` | `test.each([[...]])` |
| Test isolation | Per-test (threads, no shared state) | Per-test (goroutines) | Per-test (isolated module) |

### Comparing the Table-Driven Pattern

Go has built-in idiom for table-driven tests. Rust achieves the same by looping:

```go
// Go — idiomatic table-driven
func TestCapacityEdgeCases(t *testing.T) {
    tests := []struct {
        name     string
        capacity uint32
        wantErr  bool
    }{
        {"zero capacity", 0, true},
        {"one token", 1, false},
        {"large capacity", 1000, false},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            _, err := NewRateLimiter(tt.capacity, time.Second)
            if (err != nil) != tt.wantErr {
                t.Errorf("got err=%v, wantErr=%v", err, tt.wantErr)
            }
        })
    }
}
```

```rust
// Rust — loop over cases
#[test]
fn capacity_edge_cases() {
    struct Case { name: &'static str, capacity: u32, should_panic: bool }

    let cases = vec![
        Case { name: "one token", capacity: 1, should_panic: false },
        Case { name: "large capacity", capacity: 1000, should_panic: false },
    ];

    for case in cases {
        let result = std::panic::catch_unwind(|| {
            RateLimiter::new(case.capacity, 60_000, SystemClock)
        });
        assert_eq!(
            result.is_err(),
            case.should_panic,
            "case '{}' failed",
            case.name
        );
    }
}
```

For constructor panics, `std::panic::catch_unwind` lets you test panics inline without
`#[should_panic]`. The downside: it can't catch `abort` panics or panics across FFI boundaries.

**The key difference:** Go's `t.Run` creates sub-tests visible in output as
`TestCapacityEdgeCases/zero_capacity`. Rust's loop produces a single test entry. For
equivalent visibility, write separate test functions or use a macro.

### Your notes
<!-- -->
