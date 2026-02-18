# Chain of Responsibility Pattern — Rust

## The Problem Chain of Responsibility Solves

Request processing is rarely a single step. An incoming HTTP request needs authentication, rate limiting, input validation, logging, and finally the actual business logic. An expense approval needs junior manager review, then senior manager, then finance director, depending on the amount. A log record needs filtering by severity, formatting, then routing to the right output.

The naive approach is a monolithic function:

```rust
fn handle_request(req: &Request) -> Response {
    // 20 lines of auth checking
    if !check_auth(req) { return Response::unauthorized(); }
    // 15 lines of rate limiting
    if is_rate_limited(req) { return Response::too_many_requests(); }
    // 10 lines of input validation
    if let Err(e) = validate(req) { return Response::bad_request(e); }
    // 30 lines of logging setup
    log_request(req);
    // Finally, the actual logic buried at line 75
    process(req)
}
```

Every new processing step requires modifying this function. Testing authentication means dragging along rate limiting. Reordering steps requires surgery on a single fragile function. Adding a step for only certain request types means nested conditionals.

The Chain of Responsibility pattern fixes this by **decomposing processing into independent, composable handlers**, each responsible for one concern. Each handler either processes the request and stops, or delegates to the next handler in the chain.

In Rust, this pattern has unique challenges and opportunities that do not exist in garbage-collected languages. Ownership semantics force you to think carefully about who owns the chain, who owns the request, and how handlers communicate. The reward is chains that are both safe and zero-cost when composed with generics.

### Your notes
<!-- -->


---

## Approach 1: Trait-Based Handler Chain

This is the classic OOP-style chain. Define a `Handler` trait, implement it for each processing step, and link handlers together via `Option<Box<dyn Handler>>`.

### The Handler Trait

```rust
trait Handler {
    fn handle(&self, request: &mut Request) -> Result<Response, HandlerError>;
}
```

Each handler needs a reference to the "next" handler. The classic approach stores it inside each handler:

```rust
struct AuthHandler {
    token_store: TokenStore,
    next: Option<Box<dyn Handler>>,
}

impl Handler for AuthHandler {
    fn handle(&self, request: &mut Request) -> Result<Response, HandlerError> {
        let token = request.headers.get("Authorization")
            .ok_or(HandlerError::Unauthorized)?;

        if !self.token_store.validate(token) {
            return Err(HandlerError::Unauthorized);
        }

        // Delegate to next handler
        match &self.next {
            Some(next) => next.handle(request),
            None => Ok(Response::ok()), // End of chain
        }
    }
}
```

### Ownership Challenge: Who Owns the Next Handler?

This is where Rust diverges from Go or TypeScript. In Go, you would write:

```go
type Handler interface {
    Handle(r *Request) Response
}

type AuthHandler struct {
    next Handler // Interface value — implicitly a pointer, GC manages lifetime
}
```

In TypeScript:

```typescript
interface Handler {
    handle(req: Request): Response;
    setNext(handler: Handler): Handler;
}
```

In both languages, the garbage collector handles the lifetime of the next handler. In Rust, you must choose an ownership strategy:

| Strategy | Type | When to Use |
|----------|------|-------------|
| **Owned** | `Option<Box<dyn Handler>>` | Chain built once, used many times. Each handler owns the next. |
| **Borrowed** | `Option<&dyn Handler>` | Chain assembled from existing handlers. Needs lifetime annotations. |
| **Shared** | `Option<Arc<dyn Handler>>` | Chain shared across threads. Cloning the chain is cheap. |

The owned approach (`Box`) is the most common because it avoids lifetime headaches and works well for chains that are constructed at startup and live for the program's duration.

### Building the Chain

Building a linked-list chain in Rust reads "inside out" — you construct from the tail:

```rust
let chain = AuthHandler::new(
    token_store,
    Some(Box::new(RateLimitHandler::new(
        limiter,
        Some(Box::new(ValidationHandler::new(
            schema,
            None, // End of chain
        ))),
    ))),
);
```

This is ugly. A builder pattern cleans it up:

```rust
struct ChainBuilder {
    handlers: Vec<Box<dyn Handler>>,
}

impl ChainBuilder {
    fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    fn add(mut self, handler: Box<dyn Handler>) -> Self {
        self.handlers.push(handler);
        self
    }

    fn build(self) -> Box<dyn Handler> {
        // Link handlers in reverse order
        let mut chain: Option<Box<dyn Handler>> = None;
        for handler in self.handlers.into_iter().rev() {
            // Each handler wraps the previous chain as its "next"
            // (requires Handler to accept next in construction)
        }
        chain.expect("chain must have at least one handler")
    }
}
```

We will see a clean builder implementation in the `approval.rs` example.

### Your notes
<!-- -->


---

## Approach 2: Enum-Based Chain

When the set of handlers is known at compile time (closed set), an enum can replace trait objects entirely. This avoids heap allocation and dynamic dispatch.

```rust
enum ValidationStep {
    FormatCheck { next: Option<Box<ValidationStep>> },
    SizeLimit { max_bytes: usize, next: Option<Box<ValidationStep>> },
    AuthToken { valid_tokens: Vec<String>, next: Option<Box<ValidationStep>> },
}

impl ValidationStep {
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        match self {
            ValidationStep::FormatCheck { next } => {
                if !request.body.starts_with('{') {
                    return Err(ValidationError::InvalidFormat);
                }
                Self::delegate(next, request)
            }
            ValidationStep::SizeLimit { max_bytes, next } => {
                if request.body.len() > *max_bytes {
                    return Err(ValidationError::TooLarge);
                }
                Self::delegate(next, request)
            }
            ValidationStep::AuthToken { valid_tokens, next } => {
                let token = request.headers.get("auth")
                    .ok_or(ValidationError::MissingToken)?;
                if !valid_tokens.contains(token) {
                    return Err(ValidationError::InvalidToken);
                }
                Self::delegate(next, request)
            }
        }
    }

    fn delegate(next: &Option<Box<ValidationStep>>, request: &Request) -> Result<(), ValidationError> {
        match next {
            Some(handler) => handler.validate(request),
            None => Ok(()),
        }
    }
}
```

**Trade-offs vs trait objects:**

| | Enum | Trait Object |
|---|---|---|
| New handler | Must modify enum (violates open-closed) | Just implement trait (open for extension) |
| Dispatch | Static match (compiler optimizable) | Dynamic vtable lookup |
| Allocation | Still needs `Box` for recursion | Needs `Box` for trait object |
| Exhaustiveness | Compiler checks all variants handled | No compile-time guarantee |
| Best for | Known, stable handler set | Plugin systems, configurable chains |

In practice, the enum approach is good for internal validation chains where the steps are well-known and stable. For extensible middleware systems, trait objects win.

### Your notes
<!-- -->


---

## Approach 3: Functional Chain with Closures

The lightest approach: store handlers as closures in a `Vec`. No traits, no enums — just functions.

```rust
type HandlerFn = Box<dyn Fn(&mut Request) -> Result<(), ProcessingError>>;

struct Pipeline {
    handlers: Vec<HandlerFn>,
}

impl Pipeline {
    fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    fn add<F>(&mut self, handler: F)
    where
        F: Fn(&mut Request) -> Result<(), ProcessingError> + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    fn execute(&self, request: &mut Request) -> Result<(), ProcessingError> {
        for handler in &self.handlers {
            handler(request)?; // Short-circuit on first error
        }
        Ok(())
    }
}
```

Usage:

```rust
let mut pipeline = Pipeline::new();

pipeline.add(|req| {
    if req.body.len() > 1_000_000 {
        return Err(ProcessingError::new("payload too large"));
    }
    Ok(())
});

pipeline.add(|req| {
    req.body = req.body.trim().to_string();
    Ok(())
});

pipeline.add(|req| {
    if !req.headers.contains_key("content-type") {
        req.headers.insert("content-type".to_string(), "application/json".to_string());
    }
    Ok(())
});
```

This is essentially how middleware stacks work in many Rust web frameworks. The `?` operator provides automatic short-circuiting — if any handler returns `Err`, the chain stops.

**Trade-offs:**

| Aspect | Functional Chain | Trait Chain |
|--------|------------------|-------------|
| Flexibility | Closures capture arbitrary state | Each handler is a dedicated struct |
| Testability | Harder to test individual closures in isolation | Each handler is independently testable |
| State inspection | Cannot inspect handler configuration after construction | Can query handler properties |
| Composition | Trivial to compose (just push another closure) | Requires rebuilding the linked list |
| Debugging | Anonymous closures in stack traces | Named types in stack traces |

**When to choose closures:** Simple processing pipelines where handlers are short, the chain is linear (no branching), and you do not need to inspect or reconfigure individual handlers.

### Your notes
<!-- -->


---

## Tower-Style Service Chain (Advanced)

The Rust ecosystem has converged on the Tower pattern for composable middleware. This is not the classic Chain of Responsibility, but it is the idiomatic Rust equivalent in async service architectures. Understanding the connection is valuable.

Tower defines a `Service` trait:

```rust
// Simplified from the actual tower::Service trait
trait Service<Request> {
    type Response;
    type Error;

    fn call(&mut self, req: Request) -> Result<Self::Response, Self::Error>;
}
```

Middleware wraps an inner service:

```rust
struct LoggingMiddleware<S> {
    inner: S,
    prefix: String,
}

impl<S, Req> Service<Req> for LoggingMiddleware<S>
where
    S: Service<Req>,
    Req: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(&mut self, req: Req) -> Result<Self::Response, Self::Error> {
        println!("{}: processing {:?}", self.prefix, req);
        self.inner.call(req)
    }
}
```

This is Chain of Responsibility via **generic composition** — each layer wraps the next at the type level, creating a statically-dispatched chain. The compiler monomorphizes the entire chain into a single type, eliminating all dynamic dispatch.

The trade-off: the chain type becomes complex (`LoggingMiddleware<RateLimitMiddleware<AuthMiddleware<AppService>>>`), though type aliases and `impl Service` return types hide this in practice.

**Connection to GoF Chain of Responsibility:**
- Same intent: decouple sender from receiver, allow multiple objects to handle a request
- Same shape: handlers linked together, each deciding to handle or pass along
- Different mechanism: generic nesting vs linked list of trait objects

### Your notes
<!-- -->


---

## Rust-Specific Concerns

### Lifetimes in Handler Chains

If handlers borrow data rather than owning it, lifetimes appear:

```rust
struct AuthHandler<'a> {
    valid_tokens: &'a [String],
    next: Option<Box<dyn Handler + 'a>>,
}
```

The `'a` on the trait object (`dyn Handler + 'a`) tells the compiler that the trait object may contain references with lifetime `'a`. Without this annotation, `dyn Handler` defaults to `dyn Handler + 'static`, meaning the handler cannot borrow non-static data.

**Recommendation:** For most handler chains, owned data (`String`, `Vec`, etc.) is simpler. Use borrowed data only when you have a clear performance reason (large lookup tables shared across handlers).

### Send + Sync for Threaded Chains

If the chain will be shared across threads (common in web servers), handlers must be `Send + Sync`:

```rust
type ThreadSafeHandler = Box<dyn Handler + Send + Sync>;

struct ChainBuilder {
    handlers: Vec<Box<dyn Handler + Send + Sync>>,
}
```

This constrains what handlers can contain:
- `Rc<T>` disallowed (not `Send`) — use `Arc<T>` instead
- `RefCell<T>` disallowed (not `Sync`) — use `Mutex<T>` or `RwLock<T>` instead
- Raw pointers require `unsafe impl Send/Sync`

### Recursive Types Need Indirection

A handler that references itself (via "next") is a recursive type. Rust requires indirection:

```rust
// Compile error: recursive type has infinite size
struct Handler {
    next: Option<Handler>,
}

// Fix: use Box for indirection
struct Handler {
    next: Option<Box<Handler>>,
}
```

`Box` places the next handler on the heap, giving the struct a fixed size (a pointer). This is not unique to Chain of Responsibility — it applies to any recursive data structure in Rust (linked lists, trees, etc.).

### Your notes
<!-- -->


---

## Cross-Language Comparison

| Aspect | Rust | Go | TypeScript |
|--------|------|-----|------------|
| **Handler interface** | `trait Handler` | `type Handler interface` | `interface Handler` |
| **Next handler** | `Option<Box<dyn Handler>>` (owned) | `Handler` field (GC-managed) | `Handler \| null` property |
| **Chain building** | Inside-out or builder pattern | Simple struct assignment | `setNext()` method chain |
| **Middleware idiom** | Tower `Service<Request>` wrapping | `http.Handler` / `func(http.Handler) http.Handler` | Express `(req, res, next) => {}` |
| **Short-circuit** | `Result` + `?` operator | Early return | `throw` or skip `next()` |
| **Thread safety** | `Send + Sync` bounds required | goroutine-safe by convention | Single-threaded (or Worker boundary) |
| **Memory** | Explicit ownership, no GC overhead | GC handles chain lifecycle | GC handles chain lifecycle |
| **Dynamic dispatch cost** | ~2ns vtable lookup per handler | ~5ns interface dispatch | Engine-optimized (monomorphic inline caches) |

### Go's http.Handler Middleware Pattern

Go's most common Chain of Responsibility is the HTTP middleware stack:

```go
func LoggingMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        log.Printf("%s %s", r.Method, r.URL.Path)
        next.ServeHTTP(w, r)
    })
}
```

This is the functional composition approach — each middleware is a function that wraps the next handler. Rust's Tower pattern is directly analogous, but with static dispatch via generics.

### TypeScript Express Middleware

Express uses a callback-based chain where `next()` delegates to the next middleware:

```typescript
app.use((req, res, next) => {
    if (!req.headers.authorization) {
        return res.status(401).json({ error: 'unauthorized' });
    }
    next(); // Delegate to next middleware
});
```

The key difference: Express middleware is responsible for calling `next()` (push model). Rust trait-based chains typically call `self.next.handle()` (also push), but the functional pipeline approach iterates externally (pull model via the `for` loop).

### Your notes
<!-- -->


---

## When to Use Each Approach

| Scenario | Recommended Approach | Why |
|----------|----------------------|-----|
| Web server middleware stack | Tower-style generic wrapping | Static dispatch, composable, ecosystem standard |
| Validation pipeline (known steps) | Functional chain (`Vec<Box<dyn Fn>>`) | Simple, flexible ordering, easy to test |
| Approval workflow (domain logic) | Trait-based chain with `Box<dyn Handler>` | Each approver has distinct state and logic |
| Config-driven processing | Trait objects with runtime chain assembly | Handlers determined by config at startup |
| Performance-critical hot path | Enum chain or generic wrapping | Zero dynamic dispatch overhead |
| Plugin system | Trait objects loaded via dynamic libraries | Must use dynamic dispatch |

### When NOT to Use Chain of Responsibility

- **Single handler is sufficient.** If there is always exactly one handler, the pattern is overhead.
- **Order does not matter.** If handlers are independent and order-agnostic, a simple `Vec` of validators with `iter().all()` is clearer.
- **Complex branching.** If different request types need fundamentally different handler chains, consider the Strategy pattern for selecting chains rather than one chain that branches internally.

### Your notes
<!-- -->
