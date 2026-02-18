# Rust Reference — Circuit Breaker Pattern

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [std library docs](https://doc.rust-lang.org/std/)
> for the `circuit-breaker` module. Covers: enum-based state machines, `Instant` and `Duration` for timing, `Arc<RwLock<>>` for shared mutable state, `Result` for error handling in breakers.

---

## Enum-Based State Machines

Source: [Rust Reference - Enumerations](https://doc.rust-lang.org/reference/items/enumerations.html)

An enumeration (enum) declares a type with multiple variants. Each variant can carry associated data of different types.

```rust
enum State {
    Closed { failure_count: u32 },          // struct-like variant
    Open { opened_at: std::time::Instant }, // struct-like variant
    HalfOpen,                               // unit variant
}
```

### Variant Data Rules

| Variant Kind | Syntax | Access | Example |
|-------------|--------|--------|---------|
| Unit | `Variant` | No data | `HalfOpen` |
| Struct-like | `Variant { field: Type }` | Named fields | `Closed { failure_count: 0 }` |
| Tuple-like | `Variant(Type)` | Positional | `Error(String)` |

### Pattern Matching on Enums

Source: [Rust Reference - Match Expressions](https://doc.rust-lang.org/reference/expressions/match-expr.html)

```rust
match state {
    State::Closed { failure_count } => { /* failure_count is bound */ }
    State::Open { opened_at } => { /* opened_at is bound */ }
    State::HalfOpen => { /* no fields to bind */ }
}
```

Exhaustiveness is enforced: the compiler will error if any variant is unhandled. Using `_` as a catch-all disables this — avoid it for state machines where you want to be forced to handle new variants.

### Deriving Traits on Enums

```rust
#[derive(Debug, Clone, PartialEq)]
enum State {
    Closed { failure_count: u32 },
    Open { opened_at: Instant },  // Instant implements Debug + Clone
    HalfOpen,
}
```

| Trait | What it provides | Requirement |
|-------|-----------------|-------------|
| `Debug` | `{:?}` formatting | All fields must implement `Debug` |
| `Clone` | `.clone()` method | All fields must implement `Clone` |
| `PartialEq` | `==` comparison | All fields must implement `PartialEq` |
| `Copy` | Implicit copy on assignment | All fields must be `Copy` (not possible with `Instant`) |

Note: `Instant` implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`.

---

## `std::time::Instant` and `std::time::Duration`

Source: [std::time::Instant](https://doc.rust-lang.org/std/time/struct.Instant.html), [std::time::Duration](https://doc.rust-lang.org/std/time/struct.Duration.html)

### Instant

A measurement of a monotonically nondecreasing clock. Opaque and useful only with `Duration`.

```rust
use std::time::Instant;

let now = Instant::now();
// ... some operation ...
let elapsed: Duration = now.elapsed();  // time since `now` was created
```

| Method | Returns | Description |
|--------|---------|-------------|
| `Instant::now()` | `Instant` | Current time from monotonic clock |
| `instant.elapsed()` | `Duration` | Time elapsed since this instant |
| `instant.duration_since(earlier)` | `Duration` | Time between two instants |
| `instant + duration` | `Instant` | Instant in the future |
| `instant - duration` | `Instant` | Instant in the past |
| `instant1 - instant2` | `Duration` | Difference (panics if negative) |

**Important for circuit breakers:** `Instant` uses a monotonic clock, meaning it is not affected by system clock changes (NTP adjustments, daylight saving, etc.). This is why you should use `Instant` instead of `SystemTime` for timeouts and elapsed time measurements.

### Duration

A span of time.

```rust
use std::time::Duration;

let d = Duration::from_secs(30);
let d = Duration::from_millis(500);
let d = Duration::from_micros(100);
let d = Duration::new(5, 500_000_000);  // 5.5 seconds
```

| Method | Returns | Description |
|--------|---------|-------------|
| `Duration::from_secs(n)` | `Duration` | n seconds |
| `Duration::from_millis(n)` | `Duration` | n milliseconds |
| `Duration::from_secs_f64(n)` | `Duration` | n seconds (float) |
| `duration.as_secs()` | `u64` | Whole seconds |
| `duration.as_millis()` | `u128` | Whole milliseconds |
| `duration.as_secs_f64()` | `f64` | Total seconds as float |

### Comparison

`Duration` and `Instant` both implement `PartialOrd` and `Ord`, enabling direct comparison:

```rust
if instant.elapsed() >= Duration::from_secs(30) {
    // timeout has passed
}
```

---

## `Arc<RwLock<T>>` for Shared Mutable State

Source: [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html), [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)

### Arc (Atomically Reference Counted)

Thread-safe shared ownership. Multiple `Arc` pointers to the same allocation; the value is dropped when the last `Arc` is dropped.

```rust
use std::sync::Arc;

let breaker = Arc::new(CircuitBreaker::new());
let breaker2 = Arc::clone(&breaker);  // same allocation, ref count incremented
// breaker and breaker2 point to the same CircuitBreaker
```

| Property | Value |
|----------|-------|
| Thread safe | Yes (`Send + Sync` when `T: Send + Sync`) |
| Overhead | Atomic reference count (two `AtomicUsize`: strong + weak) |
| Interior mutability | No — `Arc<T>` gives `&T` only. Pair with `Mutex` or `RwLock` for mutation. |
| Cloning | `Arc::clone()` increments ref count (cheap, no deep copy) |
| Cycle safety | No cycle detection. Use `Weak` to break cycles. |

### RwLock (Read-Write Lock)

Allows concurrent read access OR exclusive write access.

```rust
use std::sync::RwLock;

let lock = RwLock::new(breaker);

// Multiple readers simultaneously
{
    let state = lock.read().unwrap();  // RwLockReadGuard
    println!("{:?}", state);
}  // read lock released when guard is dropped

// Exclusive writer
{
    let mut state = lock.write().unwrap();  // RwLockWriteGuard
    state.record_failure();
}  // write lock released when guard is dropped
```

| Method | Returns | Blocks when |
|--------|---------|-------------|
| `lock.read()` | `LockResult<RwLockReadGuard<T>>` | A writer holds the lock |
| `lock.write()` | `LockResult<RwLockWriteGuard<T>>` | Any reader or writer holds the lock |
| `lock.try_read()` | `TryLockResult<RwLockReadGuard<T>>` | Never blocks (returns `Err` if locked) |
| `lock.try_write()` | `TryLockResult<RwLockWriteGuard<T>>` | Never blocks (returns `Err` if locked) |

### Poisoning

If a thread panics while holding a lock, the lock becomes "poisoned." Subsequent `lock()` calls return `Err(PoisonError)`. The `.unwrap()` in examples will panic on poison — production code should handle this:

```rust
let guard = lock.read().unwrap_or_else(|poisoned| {
    // The data may be in an inconsistent state, but we can still access it
    poisoned.into_inner()
});
```

### Combining Arc and RwLock

```rust
use std::sync::{Arc, RwLock};

struct SharedBreaker {
    inner: Arc<RwLock<CircuitBreaker>>,
}

impl SharedBreaker {
    fn new(breaker: CircuitBreaker) -> Self {
        Self {
            inner: Arc::new(RwLock::new(breaker)),
        }
    }
}

// SharedBreaker is Clone (Arc::clone is cheap)
impl Clone for SharedBreaker {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
```

### Mutex vs RwLock

| Property | `Mutex<T>` | `RwLock<T>` |
|----------|-----------|-------------|
| Read access | Exclusive | Shared (multiple concurrent readers) |
| Write access | Exclusive | Exclusive |
| Overhead | Lower (one lock kind) | Higher (two lock kinds, reader count tracking) |
| Starvation risk | No | Writers can be starved by continuous readers (platform-dependent) |
| Best when | Write-heavy, or critical sections are short | Read-heavy (circuit breaker state checks) |

---

## Result Type for Error Handling in Breakers

Source: [std::result::Result](https://doc.rust-lang.org/std/result/enum.Result.html)

### Nested Result Pattern

Circuit breakers add a layer of error handling on top of the wrapped operation's errors:

```rust
#[derive(Debug)]
enum BreakerError<E> {
    Open,       // breaker rejected the call
    Inner(E),   // wrapped operation failed
}

// The call method returns Result<T, BreakerError<E>>
fn call<T, E, F>(&mut self, op: F) -> Result<T, BreakerError<E>>
where
    F: FnOnce() -> Result<T, E>,
```

### Mapping Errors

```rust
// Convert BreakerError<E> to a unified error type
breaker.call(|| operation())
    .map_err(|e| match e {
        BreakerError::Open => AppError::ServiceUnavailable,
        BreakerError::Inner(e) => AppError::from(e),
    })?;
```

### Display and Error Trait Implementation

For production use, implement `Display` and `std::error::Error`:

```rust
use std::fmt;

impl<E: fmt::Display> fmt::Display for BreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakerError::Open => write!(f, "circuit breaker is open"),
            BreakerError::Inner(e) => write!(f, "operation failed: {}", e),
        }
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for BreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    where
        E: std::error::Error + 'static,
    {
        match self {
            BreakerError::Open => None,
            BreakerError::Inner(e) => Some(e),
        }
    }
}
```

---

## FnOnce for Single-Use Operations

Source: [Rust Reference - Closure Types](https://doc.rust-lang.org/reference/types/closure.html)

The circuit breaker's `call` method accepts `F: FnOnce() -> Result<T, E>`:

```rust
fn call<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
where
    F: FnOnce() -> Result<T, E>,
```

| Closure Trait | Can call | `self` parameter | Use in breaker |
|--------------|----------|------------------|---------------|
| `FnOnce` | Once | `self` (by value) | Best fit — operation runs at most once per call |
| `FnMut` | Multiple times | `&mut self` | Unnecessarily restrictive on callers |
| `Fn` | Multiple times | `&self` | Most restrictive — prevents closures that move values |

Using `FnOnce` is the most permissive choice: it accepts closures that capture by value (`move` closures), by mutable reference, or by shared reference. Since the breaker calls the operation at most once, there's no reason to require `FnMut` or `Fn`.

---

## Send and Sync Marker Traits

Source: [Rust Reference - Send and Sync](https://doc.rust-lang.org/reference/special-types-and-traits.html#send-and-sync)

For a circuit breaker to be shared across threads:

```rust
// The breaker type must be Send + Sync
// Arc<RwLock<T>> is Send + Sync when T: Send + Sync
// CircuitBreaker is Send + Sync when all its fields are Send + Sync
```

| Trait | Meaning | Required for |
|-------|---------|-------------|
| `Send` | Safe to transfer to another thread | Moving breaker to a spawned thread |
| `Sync` | Safe to share references between threads | `&CircuitBreaker` shared via `Arc` |

Auto-trait rules:
- `Arc<T>` is `Send + Sync` when `T: Send + Sync`
- `RwLock<T>` is `Send + Sync` when `T: Send`
- `Box<dyn Fn()>` is NOT `Send` unless declared `Box<dyn Fn() + Send>`
- Raw pointers (`*const T`, `*mut T`) are neither `Send` nor `Sync`

If your circuit breaker stores callbacks (`Box<dyn Fn()>`), they must be `Box<dyn Fn() + Send + Sync>` for the breaker to be shareable across threads.

---

## References

- [The Rust Programming Language, Ch. 6 - Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [The Rust Programming Language, Ch. 16 - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [std::time Module](https://doc.rust-lang.org/std/time/index.html)
- [std::sync Module](https://doc.rust-lang.org/std/sync/index.html)
- [Rust API Guidelines - Type Safety](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [Microsoft Azure Architecture - Circuit Breaker Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)
- [Release It! by Michael Nygard](https://pragprog.com/titles/mnee2/release-it-second-edition/) — Origin of the circuit breaker pattern in software
