# Decorator / Middleware Pattern -- Rust

## The Problem Decorator Solves

You have a core behavior -- handling an HTTP request, reading bytes from a stream, executing a database query -- and you need to layer on cross-cutting concerns: logging, timing, authentication, retries, compression. The naive approach dumps all of this into the core function:

```rust
fn handle_request(req: &Request) -> Response {
    let start = Instant::now();
    log::info!("incoming: {} {}", req.method, req.path);

    if !check_auth(&req.headers) {
        return Response::unauthorized();
    }

    if rate_limiter.check(req.client_ip).is_err() {
        return Response::too_many_requests();
    }

    // ... actual business logic buried 20 lines deep ...

    let elapsed = start.elapsed();
    metrics::histogram!("request_duration", elapsed);
    response
}
```

Every concern is tangled together. Testing the business logic requires faking auth, rate limiting, and metrics. Adding a new concern means modifying this function. Removing one means careful surgery. The function violates single responsibility so thoroughly that it barely has a primary responsibility anymore.

The **Decorator pattern** (called **middleware** in web frameworks, **wrapper** in I/O stacks) fixes this by wrapping the core behavior in layers, each handling one concern. Each layer has the same interface as the core, so layers compose freely. In Rust, this manifests in several distinct idioms depending on whether you're decorating types, traits, or functions.

### Your notes
<!-- -->


---

## Approach 1: Trait-Based Decoration

This is the fundamental Rust pattern. You define a trait for the core behavior, implement it for a concrete type, then create wrapper types that also implement the trait while delegating to an inner value.

```rust
trait Handler {
    fn handle(&self, request: &str) -> String;
}

struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(&self, request: &str) -> String {
        format!("Echo: {}", request)
    }
}

// Decorator: wraps any Handler, adds logging
struct LoggingHandler<H: Handler> {
    inner: H,
    prefix: String,
}

impl<H: Handler> Handler for LoggingHandler<H> {
    fn handle(&self, request: &str) -> String {
        println!("[{}] >> {}", self.prefix, request);
        let response = self.inner.handle(request);
        println!("[{}] << {}", self.prefix, response);
        response
    }
}
```

The key insight: `LoggingHandler<H>` implements `Handler` for any inner `H: Handler`. The type system guarantees the decorator and the decorated have the same interface. You can stack decorators:

```rust
let handler = LoggingHandler {
    inner: TimingHandler {
        inner: AuthHandler {
            inner: EchoHandler,
            // ...
        },
    },
    prefix: "app".to_string(),
};
```

The resulting type is `LoggingHandler<TimingHandler<AuthHandler<EchoHandler>>>`. This is fully resolved at compile time -- zero dynamic dispatch overhead. But the type gets unwieldy, and you cannot change the middleware stack at runtime.

### Static vs Dynamic Dispatch

You have the same choice as with Strategy:

| Approach | Type | Dispatch | Stack changes at runtime? | Allocation |
|----------|------|----------|---------------------------|------------|
| `struct Wrapper<H: Handler>` | Generic | Static (monomorphized) | No | Stack |
| `struct Wrapper { inner: Box<dyn Handler> }` | Trait object | Dynamic (vtable) | Yes | Heap |

For middleware stacks configured at startup and never changed, generics win -- zero overhead. For plugin systems or stacks built from configuration files, trait objects are necessary.

```rust
// Dynamic dispatch version
struct LoggingHandler {
    inner: Box<dyn Handler>,
    prefix: String,
}

impl Handler for LoggingHandler {
    fn handle(&self, request: &str) -> String {
        println!("[{}] >> {}", self.prefix, request);
        let response = self.inner.handle(request);
        println!("[{}] << {}", self.prefix, response);
        response
    }
}

// Now you can build stacks dynamically
fn build_stack(config: &Config) -> Box<dyn Handler> {
    let mut handler: Box<dyn Handler> = Box::new(EchoHandler);
    if config.enable_auth {
        handler = Box::new(AuthHandler { inner: handler });
    }
    if config.enable_logging {
        handler = Box::new(LoggingHandler { inner: handler, prefix: "app".into() });
    }
    handler
}
```

**Comparison with Go:** Go's `http.Handler` interface and `func(http.Handler) http.Handler` pattern is the dynamic-dispatch version. Go has no generics-based zero-cost alternative. In TypeScript, the `@decorator` syntax is syntactic sugar over function wrapping, closer to the closure-based approach below.

### Your notes
<!-- -->


---

## Approach 2: The Newtype Wrapper Pattern

Sometimes you want to add behavior to an existing type without defining a new trait. The newtype pattern wraps a type in a single-field struct:

```rust
struct Meters(f64);
struct Celsius(f64);
```

For decoration, the newtype wraps a type and re-exposes its interface with additions. The standard library uses this extensively:

```rust
use std::io::{self, Read, BufRead, BufReader};

// BufReader<R> wraps any R: Read, adding buffered reading
let file = std::fs::File::open("data.txt").unwrap();
let reader = BufReader::new(file);  // BufReader<File>

// BufReader<File> implements Read (delegates to inner File)
// AND adds BufRead methods (read_line, lines, etc.)
for line in reader.lines() {
    println!("{}", line.unwrap());
}
```

`BufReader<R>` is a decorator: it implements `Read` by delegating to its inner `R`, and adds buffering behavior on top. The pattern:

1. Wrap the inner type: `struct Wrapper<T>(T)` or `struct Wrapper<T> { inner: T, ... }`
2. Implement the same trait(s) as the inner type, delegating where appropriate
3. Add new methods or trait implementations for the extended behavior

### The `Deref` Temptation

You might think: "I'll implement `Deref<Target = Inner>` so my wrapper transparently delegates all method calls." This works for smart pointer-like wrappers (that is literally what `Deref` is for), but abusing it for general decoration is an anti-pattern:

```rust
use std::ops::Deref;

// ANTI-PATTERN: Using Deref for non-pointer-like delegation
struct LoggingVec<T> {
    inner: Vec<T>,
    log: Vec<String>,
}

impl<T> Deref for LoggingVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.inner
    }
}
```

Problems with this approach:
- `Deref` coercion is implicit -- readers cannot tell when decoration is happening
- `DerefMut` allows bypassing your wrapper entirely (mutating the inner value directly)
- If the inner type has methods that conflict with your wrapper's methods, auto-deref resolution is confusing
- The Rust API guidelines explicitly say: **`Deref` should only be implemented for smart pointers**

The correct alternative is explicit delegation:

```rust
impl<T> LoggingVec<T> {
    fn push(&mut self, value: T) {
        self.log.push(format!("push at len={}", self.inner.len()));
        self.inner.push(value);
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    // Delegate explicitly for each method you want to expose
}
```

Yes, this is more boilerplate. That is the point -- each delegated method is a conscious decision, and you can intercept any of them.

### Your notes
<!-- -->


---

## Approach 3: The I/O Chain Pattern

Rust's `std::io` module is the best real-world example of the decorator pattern in the standard library. The `Read` and `Write` traits define I/O interfaces, and adapter types layer behavior:

```
File (raw bytes)
  -> BufReader (buffering)
    -> Take (limit bytes read)
      -> Chain (concatenate with another reader)
```

Each adapter implements `Read` and wraps an inner `Read`. The chain is built via method calls on the `Read` trait itself:

```rust
use std::io::{self, Read, Cursor};

fn main() {
    let data = Cursor::new(b"Hello, World! This is a long message.");

    // Chain decorators: take only 13 bytes, then buffer
    let mut limited = data.take(13);
    let mut buf = String::new();
    limited.read_to_string(&mut buf).unwrap();

    println!("{}", buf);  // "Hello, World!"
}
```

The key adapter types in `std::io`:

| Adapter | Wraps | Adds | Method |
|---------|-------|------|--------|
| `BufReader<R>` | `R: Read` | Buffered reading, `lines()` | `BufReader::new(r)` |
| `BufWriter<W>` | `W: Write` | Buffered writing | `BufWriter::new(w)` |
| `Take<R>` | `R: Read` | Limits bytes read | `r.take(n)` |
| `Chain<R1, R2>` | Two `R: Read` | Concatenates readers | `r1.chain(r2)` |
| `Bytes<R>` | `R: Read` | Byte-by-byte iterator | `r.bytes()` |

Writing your own I/O decorator follows the same pattern:

```rust
struct CountingReader<R: Read> {
    inner: R,
    bytes_read: u64,
}

impl<R: Read> CountingReader<R> {
    fn new(inner: R) -> Self {
        CountingReader { inner, bytes_read: 0 }
    }

    fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.bytes_read += n as u64;
        Ok(n)
    }
}
```

Now `CountingReader` composes with any `Read` source and any other `Read` decorator. You can count bytes read from a file through a buffer through a decompressor -- the chain just works.

### Your notes
<!-- -->


---

## Approach 4: Closure-Based Decoration

For simple function decoration, closures are often sufficient. This mirrors Go's `func(http.Handler) http.Handler` pattern and TypeScript's function wrappers.

```rust
fn with_retry<F, T, E>(f: F, max_retries: usize) -> impl Fn() -> Result<T, E>
where
    F: Fn() -> Result<T, E>,
    E: std::fmt::Display,
{
    move || {
        for attempt in 0..max_retries {
            match f() {
                Ok(val) => return Ok(val),
                Err(e) => {
                    if attempt < max_retries - 1 {
                        eprintln!("attempt {} failed: {}, retrying...", attempt + 1, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        f()  // unreachable, but satisfies compiler
    }
}
```

Usage:

```rust
let fetch_data = || -> Result<String, String> {
    // simulate flaky operation
    Err("connection timeout".to_string())
};

let reliable_fetch = with_retry(fetch_data, 3);
let result = reliable_fetch();
```

Closure decorators compose by nesting:

```rust
let operation = with_logging("fetch", with_retry(with_timeout(fetch_data, 5), 3));
```

The limitation: closures are opaque. You cannot inspect a decorated closure to see what middleware is applied. For introspectable stacks, use the trait-based approach.

### Closures vs Trait Objects for Middleware

| Feature | Closures | Trait objects |
|---------|----------|---------------|
| Overhead | Zero (monomorphized) or one vtable call | One vtable call per layer |
| Introspection | None -- opaque | Can add `name()`, `Debug`, etc. |
| State | Captured in closure env | Struct fields |
| Composability | Nesting gets ugly | Clean generic stacking |
| Use case | Simple one-off wrappers | Full middleware systems |

### Your notes
<!-- -->


---

## Approach 5: Tower-Style Service/Layer Architecture

The `tower` ecosystem (used by Axum, Tonic, and Hyper) defines a production-grade middleware pattern. While `tower` itself is an external crate, the pattern is worth understanding because it represents the Rust community's answer to composable middleware.

The core idea has two traits:

```rust
// Simplified version of tower::Service
trait Service {
    type Request;
    type Response;
    type Error;

    fn call(&mut self, req: Self::Request) -> Result<Self::Response, Self::Error>;
}

// A Layer wraps a Service to produce a new Service
trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

A `Layer` is a factory that takes a service and returns a decorated service. This separates the *configuration* of middleware (the Layer) from its *execution* (the resulting Service):

```rust
struct TimeoutLayer {
    duration_ms: u64,
}

// The Layer produces a TimeoutService<S> for any inner service S
impl<S: Service> Layer<S> for TimeoutLayer {
    type Service = TimeoutService<S>;

    fn layer(&self, inner: S) -> TimeoutService<S> {
        TimeoutService {
            inner,
            timeout_ms: self.duration_ms,
        }
    }
}

struct TimeoutService<S> {
    inner: S,
    timeout_ms: u64,
}

impl<S: Service> Service for TimeoutService<S> {
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;  // In real code, this would be a broader error type

    fn call(&mut self, req: Self::Request) -> Result<Self::Response, Self::Error> {
        // In real tower, this would be async with a timeout future
        self.inner.call(req)
    }
}
```

Stacking layers:

```rust
let service = EchoService;
let service = LoggingLayer.layer(service);
let service = TimeoutLayer { duration_ms: 5000 }.layer(service);
let service = AuthLayer { token: "secret".into() }.layer(service);
```

This reads bottom-to-top: `EchoService` is wrapped by logging, then timeout, then auth. The outermost layer runs first on the request.

**Why separate Layer from Service?** Because layers are reusable configuration objects. You can define a layer stack once and apply it to multiple services. The layer holds config (timeout duration, auth tokens); the service holds runtime state (connection pools, counters).

### Your notes
<!-- -->


---

## Ownership Challenges in Decoration

Rust's ownership model creates unique challenges for the decorator pattern that don't exist in garbage-collected languages.

### Challenge 1: Owned vs Borrowed Inner Values

```rust
// This works: decorator owns the inner handler
struct Logger<H: Handler> {
    inner: H,  // owned
}

// This is trickier: decorator borrows the inner handler
struct Logger<'a, H: Handler> {
    inner: &'a H,  // borrowed -- lifetime needed
}
```

Owning is simpler -- no lifetime annotations, the decorator controls the inner value's lifetime. Borrowing is useful when multiple decorators need to share the same inner handler, but introduces lifetimes everywhere.

In practice, most decorators own their inner value. If sharing is needed, use `Arc`:

```rust
use std::sync::Arc;

struct SharedLogger {
    inner: Arc<dyn Handler>,  // shared ownership, thread-safe
}
```

### Challenge 2: Mutable Decoration

If your decorator needs mutable access (e.g., counting requests):

```rust
struct CountingHandler<H: Handler> {
    inner: H,
    count: std::cell::Cell<u64>,  // interior mutability for &self methods
}

impl<H: Handler> Handler for CountingHandler<H> {
    fn handle(&self, request: &str) -> String {
        self.count.set(self.count.get() + 1);
        self.inner.handle(request)
    }
}
```

For thread-safe mutable state, use `AtomicU64` or `Mutex`:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct CountingHandler<H: Handler> {
    inner: H,
    count: AtomicU64,
}
```

### Challenge 3: Send + Sync Bounds

If your decorated service will be used across threads (virtually all async web servers), every layer must be `Send + Sync`. This propagates through the entire stack:

```rust
// Thread-safe handler trait
trait Handler: Send + Sync {
    fn handle(&self, request: &str) -> String;
}

// Decorator must also be Send + Sync
// This is automatic IF all fields are Send + Sync
struct Logger<H: Handler> {
    inner: H,          // Send + Sync if H is
    prefix: String,    // String is Send + Sync
    // Rc<String>      // would break: Rc is NOT Send
}
```

**Rule of thumb:** If your handler trait requires `Send + Sync`, use `Arc` instead of `Rc`, `AtomicU64` instead of `Cell<u64>`, and `Mutex` instead of `RefCell`.

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go | TypeScript |
|--------|------|-----|------------|
| **Primary idiom** | Trait impl + generic wrapper | `func(Handler) Handler` | Class extends / `@decorator` |
| **Zero-cost option** | Yes (generics, monomorphization) | No (always interface dispatch) | No (always runtime wrapping) |
| **I/O decorators** | `BufReader<R>`, `Take<R>` | `bufio.Reader`, `io.LimitReader` | Node streams with `.pipe()` |
| **Framework pattern** | Tower `Service` + `Layer` | `http.Handler` middleware | Express `app.use()` |
| **Composition** | Type-level nesting or `Box<dyn>` | Function chaining | Function chaining or class wrapping |
| **Ownership concern** | Must decide owned vs borrowed | GC handles it | GC handles it |
| **Thread safety** | `Send + Sync` bounds | Goroutine-safe by convention | Single-threaded event loop |

### Go middleware pattern (for comparison):

```go
func withLogging(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        log.Printf(">> %s %s", r.Method, r.URL.Path)
        next.ServeHTTP(w, r)
        log.Printf("<< %s %s", r.Method, r.URL.Path)
    })
}

// Stacking: reads inside-out
handler := withLogging(withAuth(withMetrics(appHandler)))
```

This is the dynamic-dispatch, closure-based approach. Rust can do the same, but also offers the generic, zero-cost alternative that Go cannot.

### Your notes
<!-- -->


---

## When to Use Which Approach

| Situation | Approach | Reason |
|-----------|----------|--------|
| Fixed middleware stack, performance-critical | Generic trait decoration | Zero overhead, resolved at compile time |
| Runtime-configurable middleware | `Box<dyn Trait>` decoration | Stack built from config or feature flags |
| Standard library I/O | I/O chain pattern | Compose `Read`/`Write` adapters |
| Simple function wrapping | Closure decoration | Minimal boilerplate for one-off wrappers |
| Large framework middleware | Tower Service/Layer | Separates config from execution, battle-tested |
| Adding behavior to foreign type | Newtype wrapper | Orphan rule compliance, explicit delegation |

### Your notes
<!-- -->


---

## Key Takeaways

1. **Decoration is composition, not inheritance.** Rust has no inheritance. Decoration via trait implementation is the idiomatic replacement.

2. **Generic wrappers give you zero-cost abstraction.** `LoggingHandler<AuthHandler<EchoHandler>>` has zero dynamic dispatch -- the entire call chain is monomorphized.

3. **`Box<dyn Trait>` enables runtime flexibility.** When the middleware stack is determined by configuration, trait objects are the right tool.

4. **Do not abuse `Deref` for delegation.** It is for smart pointers. Use explicit method forwarding for decorators.

5. **The I/O chain pattern is the standard library's decorator.** `BufReader`, `Take`, `Chain` -- these are decorators. Study them.

6. **Ownership flows through the decorator stack.** Each layer typically owns the next. For shared layers, use `Arc`. For mutable state in `&self` methods, use `Cell`/`Atomic`/`Mutex`.

7. **Tower's Service/Layer separation is the community standard** for production middleware in async Rust.

### Your notes
<!-- -->
