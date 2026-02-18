# Circuit Breaker Pattern — Rust

## The Problem: Cascading Failures in Distributed Systems

Every service that talks to another service over a network will eventually face the same problem: the remote service goes down. Maybe it's a database, maybe it's a third-party API, maybe it's another microservice in your mesh. When that happens, your service keeps sending requests that will never succeed. Each request ties up a thread (or task), burns through timeouts, and backs up your own callers. One failing dependency takes down your entire service graph.

The Circuit Breaker pattern solves this by wrapping remote calls in a state machine that monitors failures and **stops making doomed requests** once a threshold is reached. It's the software equivalent of an electrical circuit breaker — when too much current flows (too many failures), the breaker trips and cuts the circuit, protecting the system.

The pattern has three states:

```
     success                 timeout expires
  +----------+            +---------------+
  |          |            |               |
  v          |            v               |
CLOSED ----> OPEN ----> HALF-OPEN --------+
  (normal)   (tripped)   (probing)        |
  failures   all calls   one probe        |
  counted    rejected    allowed          |
             |                    success |
             |                    ------->+---> CLOSED
             |                            |
             +<------ probe fails --------+
```

- **Closed**: Normal operation. Requests pass through. Failures are counted. When failures hit a threshold, transition to Open.
- **Open**: Fail fast. All requests are immediately rejected without attempting the call. After a configurable timeout, transition to Half-Open.
- **Half-Open**: Recovery probe. Allow exactly one request through. If it succeeds, transition back to Closed. If it fails, transition back to Open.

This is a natural fit for Rust: the state machine maps directly to an enum, the failure tracking integrates with `Result`, and ownership semantics make sharing the breaker across threads explicit and safe.

### Your notes
<!-- -->


---

## Rust Enum as State Machine

The circuit breaker state is a textbook use case for Rust enums with data. Each variant carries the state it needs:

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
enum BreakerState {
    Closed {
        failure_count: u32,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen,
}
```

Compare this to how you'd model the same thing in Go and TypeScript:

| Language | State representation | Data per state | Exhaustiveness check |
|----------|---------------------|----------------|----------------------|
| **Rust** | `enum` with variants carrying data | Each variant has its own fields | Compiler-enforced via `match` |
| **Go** | `int` constants + separate fields on struct | All fields on one struct, some unused per state | No enforcement — must discipline yourself |
| **TypeScript** | Discriminated union with `type` field | Each variant is a separate type in a union | TypeScript checks with `switch` + `never` |

The Rust approach is the strictest: you literally cannot access `failure_count` when the state is `Open`, because the `Open` variant doesn't have that field. In Go, you'd have `failureCount` sitting on the struct even when the breaker is open — nothing stops you from reading it accidentally.

```rust
fn describe_state(state: &BreakerState) -> String {
    match state {
        BreakerState::Closed { failure_count } => {
            format!("Closed ({} failures)", failure_count)
        }
        BreakerState::Open { opened_at } => {
            let elapsed = opened_at.elapsed();
            format!("Open (tripped {:.1}s ago)", elapsed.as_secs_f64())
        }
        BreakerState::HalfOpen => {
            "Half-Open (probing)".to_string()
        }
    }
}
```

If you add a new state variant later (say, `Disabled` for maintenance mode), the compiler flags every `match` that doesn't handle it. This is why Rust enums are considered one of the language's strongest features for modeling state machines.

### Your notes
<!-- -->


---

## Building a Basic Circuit Breaker

Here's the core structure. The breaker wraps any operation that returns `Result<T, E>` and tracks success/failure:

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
enum State {
    Closed { failure_count: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, Clone)]
struct CircuitBreakerConfig {
    failure_threshold: u32,
    recovery_timeout: Duration,
}

struct CircuitBreaker {
    state: State,
    config: CircuitBreakerConfig,
}

#[derive(Debug)]
enum BreakerError<E> {
    Open,             // breaker is open, call not attempted
    Inner(E),         // the wrapped operation failed
}
```

The `BreakerError<E>` type is important. It distinguishes between "the breaker rejected your call" and "the breaker let your call through but it failed." Callers can pattern match on this to decide what to do:

```rust
match breaker.call(|| fetch_user(user_id)) {
    Ok(user) => handle_user(user),
    Err(BreakerError::Open) => serve_cached_response(),
    Err(BreakerError::Inner(e)) => log_and_retry(e),
}
```

This is where Rust's `Result` type is a natural fit. In Go, you'd return `(T, error)` and check `errors.Is(err, ErrCircuitOpen)`. In TypeScript, you'd throw different exception types or use a discriminated union. Rust's nested `Result` with a generic error enum gives you compile-time exhaustiveness checking on the failure modes.

### State Transitions

The transition logic lives in a single method:

```rust
impl CircuitBreaker {
    fn call<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match &self.state {
            State::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    // Timeout expired — transition to half-open and probe
                    self.state = State::HalfOpen;
                    self.execute_probe(operation)
                } else {
                    // Still in cooldown — reject immediately
                    Err(BreakerError::Open)
                }
            }
            State::HalfOpen => {
                // Already probing — reject additional requests
                Err(BreakerError::Open)
            }
            State::Closed { failure_count: _ } => {
                self.execute_closed(operation)
            }
        }
    }
}
```

Notice that `call` takes `FnOnce` — the operation is consumed when called. This is the right choice because we call the operation at most once per `call()` invocation. If we used `Fn`, we'd unnecessarily require the operation to be callable multiple times.

### Your notes
<!-- -->


---

## Thread Safety: Sharing the Breaker

In a real service, multiple request handlers need to share one circuit breaker per downstream dependency. This means thread-safe shared mutable state — Rust's ownership system forces you to be explicit about this.

### The Ownership Question

Who owns the circuit breaker? Consider a service with multiple endpoint handlers:

```rust
use std::sync::{Arc, RwLock};

// Each downstream service gets its own breaker
struct ServiceClient {
    breaker: Arc<RwLock<CircuitBreaker>>,
    base_url: String,
}

impl Clone for ServiceClient {
    fn clone(&self) -> Self {
        ServiceClient {
            breaker: Arc::clone(&self.breaker),
            base_url: self.base_url.clone(),
        }
    }
}
```

`Arc<RwLock<CircuitBreaker>>` is the standard pattern:
- `Arc` — shared ownership across threads (reference-counted pointer)
- `RwLock` — allows multiple readers OR one writer (not both)

Why `RwLock` over `Mutex`? Most calls only need to **read** the state (to check if the breaker is open). Only state transitions need write access. `RwLock` allows concurrent reads, which matters under high request volume.

| Wrapper | Concurrent reads | Write access | Overhead | Use when |
|---------|-----------------|--------------|----------|----------|
| `Mutex` | No (exclusive lock always) | Exclusive | Lower | Writes are frequent, reads are rare |
| `RwLock` | Yes (shared read lock) | Exclusive | Higher (two lock kinds) | Reads dominate (circuit breaker) |

> **Coming from Go?** This is like `sync.RWMutex` wrapping a struct. The difference is Rust's type system enforces that you can't access the breaker without acquiring the lock. In Go, nothing stops you from reading `breaker.state` without locking — it's a discipline issue, not a type-system issue.

> **Coming from TypeScript?** If you're in a single-threaded Node.js context, you don't need any of this — JavaScript's event loop gives you cooperative concurrency. But in a multi-threaded Rust service (or Tokio with `spawn`), you need explicit synchronization.

### Using the Thread-Safe Breaker

```rust
impl ServiceClient {
    fn fetch(&self, path: &str) -> Result<String, String> {
        let mut breaker = self.breaker.write().unwrap();
        breaker.call(|| {
            // Simulate HTTP request
            if path.contains("fail") {
                Err(format!("connection refused: {}", path))
            } else {
                Ok(format!("response from {}{}", self.base_url, path))
            }
        })
        .map_err(|e| match e {
            BreakerError::Open => "circuit breaker open: service unavailable".to_string(),
            BreakerError::Inner(e) => e,
        })
    }
}
```

There's a subtlety here: we hold the write lock for the entire duration of the operation call. This is simple but means only one request at a time can go through the breaker. For a more sophisticated approach, you'd:

1. Acquire a read lock to check the state
2. Drop the read lock
3. Execute the operation without holding any lock
4. Acquire a write lock to record the result

The advanced code example (`advanced.rs`) demonstrates this optimization.

### Your notes
<!-- -->


---

## Generic Circuit Breaker: Wrapping Any Operation

The power of Rust generics lets you build a breaker that wraps any fallible operation:

```rust
impl CircuitBreaker {
    fn call<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // ... state machine logic
    }
}
```

This signature means:
- `T` — any success type (HTTP response, database row, parsed JSON)
- `E` — any error type (IO error, timeout, parse error)
- `F` — any closure/function that returns `Result<T, E>`

The breaker doesn't care what the operation does. It only cares whether it succeeds or fails. This is the same principle as Go's `gobreaker` package, but with compile-time type safety:

```go
// Go (gobreaker) — type-erased with interface{}
cb := gobreaker.NewCircuitBreaker(settings)
result, err := cb.Execute(func() (interface{}, error) {
    return http.Get(url)
})
// Must type-assert: result.(*http.Response)
```

```typescript
// TypeScript (opossum) — generic but no ownership semantics
const breaker = new CircuitBreaker(fetchUser, { timeout: 3000 });
const user = await breaker.fire(userId);
// Type: Promise<User> if fetchUser returns Promise<User>
```

```rust
// Rust — fully generic, zero-cost, type-safe
let result: Result<User, BreakerError<HttpError>> = breaker.call(|| {
    fetch_user(user_id)
});
// No type assertion needed — the compiler knows the exact types
```

### Your notes
<!-- -->


---

## Builder Pattern for Configuration

Production circuit breakers need many configuration knobs. The builder pattern (covered in `tracks/rust/patterns/builder/`) is the idiomatic way to handle this:

```rust
struct CircuitBreakerBuilder {
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_calls: u32,
    on_state_change: Option<Box<dyn Fn(&State, &State) + Send + Sync>>,
}

impl CircuitBreakerBuilder {
    fn new() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_calls: 1,
            on_state_change: None,
        }
    }

    fn failure_threshold(mut self, n: u32) -> Self {
        self.failure_threshold = n;
        self
    }

    fn recovery_timeout(mut self, d: Duration) -> Self {
        self.recovery_timeout = d;
        self
    }

    fn on_state_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&State, &State) + Send + Sync + 'static,
    {
        self.on_state_change = Some(Box::new(f));
        self
    }

    fn build(self) -> CircuitBreaker {
        CircuitBreaker {
            state: State::Closed { failure_count: 0 },
            config: CircuitBreakerConfig {
                failure_threshold: self.failure_threshold,
                recovery_timeout: self.recovery_timeout,
            },
            // ...
        }
    }
}
```

Usage:

```rust
let breaker = CircuitBreakerBuilder::new()
    .failure_threshold(3)
    .recovery_timeout(Duration::from_secs(10))
    .on_state_change(|from, to| {
        eprintln!("Circuit breaker: {:?} -> {:?}", from, to);
    })
    .build();
```

The `on_state_change` callback demonstrates an important Rust pattern: storing closures. The callback must be `Send + Sync + 'static` because the breaker may be shared across threads. This is one of those Rust moments where the type system forces you to think about thread safety up front — in Go or TypeScript, you'd discover the race condition in production.

### Your notes
<!-- -->


---

## Integration with Result: Why Circuit Breaker Feels Native in Rust

The circuit breaker pattern maps naturally onto Rust's error handling because `Result<T, E>` is already the standard way to represent fallible operations. Every function that could fail returns `Result`, and the breaker just wraps that:

```rust
// Without circuit breaker
fn fetch_user(id: u64) -> Result<User, HttpError> { ... }

// With circuit breaker — same shape, enriched error type
fn fetch_user_protected(id: u64) -> Result<User, BreakerError<HttpError>> { ... }
```

You can use the `?` operator seamlessly:

```rust
fn get_user_profile(breaker: &mut CircuitBreaker, id: u64) -> Result<Profile, AppError> {
    let user = breaker.call(|| fetch_user(id))
        .map_err(|e| match e {
            BreakerError::Open => AppError::ServiceUnavailable,
            BreakerError::Inner(e) => AppError::Http(e),
        })?;

    let prefs = breaker.call(|| fetch_preferences(user.id))
        .map_err(|e| match e {
            BreakerError::Open => AppError::ServiceUnavailable,
            BreakerError::Inner(e) => AppError::Http(e),
        })?;

    Ok(Profile { user, prefs })
}
```

In contrast, Go requires manual error wrapping:

```go
result, err := cb.Execute(func() (interface{}, error) {
    return fetchUser(id)
})
if err != nil {
    if errors.Is(err, gobreaker.ErrOpenState) {
        return nil, ErrServiceUnavailable
    }
    return nil, fmt.Errorf("fetch user: %w", err)
}
user := result.(*User) // type assertion — can panic
```

The Rust version is more verbose in some ways (the `map_err` closures) but safer: no type assertions, no possibility of accessing the wrong error type, and the compiler ensures you handle both `Open` and `Inner` cases.

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go (`gobreaker`) | TypeScript (`opossum`) |
|--------|------|-------------------|------------------------|
| **State representation** | `enum` with data | `int` constants + struct fields | Class with string state |
| **Thread safety** | `Arc<RwLock<>>` explicit | `sync.Mutex` inside library | Single-threaded (event loop) |
| **Generic over operation** | `F: FnOnce() -> Result<T, E>` | `func() (interface{}, error)` | `(...args) => Promise<T>` |
| **Error discrimination** | `BreakerError<E>` enum | `errors.Is(err, ErrOpenState)` | `CircuitBreakerOpenError` class |
| **Configuration** | Builder pattern | `Settings` struct | Options object |
| **Async support** | `async fn` with `tokio::sync::RwLock` | Goroutines (built-in) | Native `Promise`/`async` |
| **Callback hooks** | `Box<dyn Fn() + Send + Sync>` | `func()` callbacks | EventEmitter |
| **Zero-cost abstraction** | Yes (monomorphized generics) | No (interface boxing) | No (dynamic dispatch always) |

### Key Takeaway

Rust's circuit breaker implementation is more verbose to set up than Go's or TypeScript's. But it gives you:
1. **Compile-time guarantees** that the breaker is used correctly across threads
2. **Exhaustive error handling** — you must handle the "breaker open" case
3. **Zero-cost generics** — no type erasure or boxing of the wrapped operation
4. **Explicit ownership** — clear who owns the breaker and who borrows it

The verbosity is front-loaded: you pay it once when writing the breaker, and every caller gets safety guarantees for free.

### Your notes
<!-- -->


---

## Preview: Related Patterns

The Circuit Breaker connects to several patterns you'll encounter:

- **Retry with backoff** — Circuit Breaker and retry are complementary. Retry handles transient failures (try again). Circuit Breaker handles sustained failures (stop trying). A common architecture: retry a few times, and if the breaker trips, stop retrying entirely.
- **Bulkhead** — Isolates failures to a partition. Circuit Breaker stops calls to a failing service; Bulkhead limits how many concurrent calls can go to any service, preventing resource exhaustion.
- **Timeout** — Every operation inside a circuit breaker should have a timeout. Without one, a slow service can hang the breaker in the "calling" state, holding the lock.
- **Strategy pattern** — The health probe in Half-Open state is a strategy: you might probe with a lightweight health check endpoint vs. a real request.
- **Observer pattern** — State change callbacks (`on_state_change`) are an observer/event system. Production breakers emit metrics on state transitions.

### Your notes
<!-- -->
