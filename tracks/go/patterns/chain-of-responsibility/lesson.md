# Chain of Responsibility Pattern -- Go

## The Problem Chain of Responsibility Solves

You have a request that needs to pass through a series of processing steps. Each step might handle the request, modify it, reject it, or pass it along to the next step. The naive approach is a single function with a growing pile of `if/else` blocks -- check authentication, then check authorization, then check rate limits, then validate the body, then actually handle the request. Every new concern means editing that function, re-testing all the branches, and hoping the ordering stays correct.

The Chain of Responsibility pattern decouples the sender of a request from the logic that processes it by giving multiple handlers a chance to handle the request. Each handler either processes the request and stops, processes it and passes it along, or simply passes it along untouched. The chain is ordered, each handler is independent, and adding or removing a handler doesn't require changing any other handler.

This is not abstract. If you've written Express middleware in TypeScript, you already know this pattern:

```typescript
// TypeScript/Express: each middleware is a link in the chain
app.use(cors());              // handler 1: CORS headers
app.use(authenticate());      // handler 2: verify JWT, might short-circuit with 401
app.use(authorize('admin'));   // handler 3: check role, might short-circuit with 403
app.use(rateLimit());         // handler 4: throttle, might short-circuit with 429
app.post('/users', handler);  // final handler: actual business logic
```

Each middleware calls `next()` to pass control forward, or sends a response to short-circuit the chain. Express's middleware stack *is* Chain of Responsibility. Go's `net/http` middleware follows the exact same pattern, and it's the single most important design pattern in Go web development.

### Where Chain of Responsibility appears in the real world

- **HTTP middleware stacks**: Authentication, authorization, rate limiting, logging, CORS, compression, request ID injection
- **Validation pipelines**: Schema validation, business rule validation, authorization checks -- first failure stops the chain
- **Approval workflows**: Expense approvals escalate from team lead to manager to director based on amount
- **Event processing**: Log entries routed through handlers based on severity -- console (all), file (warn+), alerting (error only)
- **Plugin systems**: Plugins get a chance to handle or modify requests in priority order
- **Error recovery**: Each handler tries to recover from specific error types before escalating
- **Command dispatching**: CLI commands routed to the first handler that recognizes the command

### How Chain of Responsibility differs from Decorator

Chain of Responsibility and Decorator look almost identical in Go -- both wrap handlers, both use the `func(http.Handler) http.Handler` signature. The difference is intent:

| Aspect | Decorator | Chain of Responsibility |
|--------|-----------|------------------------|
| **Primary purpose** | Add behavior to existing functionality | Route requests to the right handler |
| **Short-circuiting** | Rare -- decorators usually always call the inner handler | Common -- handlers may stop the chain |
| **Handler awareness** | Each decorator adds *around* the core | Each handler decides *whether* to continue |
| **Composition** | Nesting (wrapping layers) | Sequence (ordered pipeline) |
| **Example** | Logging decorator always logs, then calls inner | Auth handler rejects bad tokens, never reaching inner |

In practice, Go HTTP middleware is *both* patterns at once. A logging middleware is a decorator (always wraps). An auth middleware is chain of responsibility (conditionally stops). The implementation is identical -- the mental model differs.

### Your notes
<!-- User adds insights here during learning -->


---

## Implementation Shape 1: Linked-List Chain

The classic GoF (Gang of Four) Chain of Responsibility uses a linked list: each handler holds a reference to the next handler. When a handler can't process a request, it delegates to its successor.

### The Interface

```go
// Handler defines what each link in the chain can do
type Handler interface {
    Handle(request Request) (Response, error)
    SetNext(handler Handler) Handler
}
```

### The Base Handler

In Go, you embed a base struct to avoid repeating the linked-list plumbing in every handler:

```go
// BaseHandler provides default chain behavior
type BaseHandler struct {
    next Handler
}

func (b *BaseHandler) SetNext(handler Handler) Handler {
    b.next = handler
    return handler  // return handler for fluent chaining: a.SetNext(b).SetNext(c)
}

func (b *BaseHandler) Handle(req Request) (Response, error) {
    if b.next != nil {
        return b.next.Handle(req)
    }
    return Response{}, fmt.Errorf("no handler could process the request")
}
```

Each concrete handler embeds `BaseHandler` and overrides `Handle`. If it can process the request, it does. If not, it calls `b.BaseHandler.Handle(req)` to pass to the next link.

### The Approval Chain Example

```go
type ExpenseApprover struct {
    BaseHandler
    name  string
    limit float64
}

func (a *ExpenseApprover) Handle(req Request) (Response, error) {
    if req.Amount <= a.limit {
        return Response{
            Approved: true,
            Approver: a.name,
        }, nil
    }
    // Can't approve -- pass to next
    fmt.Printf("%s cannot approve $%.2f (limit: $%.2f), escalating...\n",
        a.name, req.Amount, a.limit)
    return a.BaseHandler.Handle(req)
}
```

Wire it up:

```go
teamLead := &ExpenseApprover{name: "Team Lead", limit: 100}
manager := &ExpenseApprover{name: "Manager", limit: 1000}
director := &ExpenseApprover{name: "Director", limit: 10000}
vp := &ExpenseApprover{name: "VP", limit: math.MaxFloat64}

teamLead.SetNext(manager).SetNext(director).SetNext(vp)

resp, err := teamLead.Handle(Request{Amount: 5000})
// Team Lead cannot approve $5000 (limit: $100), escalating...
// Manager cannot approve $5000 (limit: $1000), escalating...
// Director approves
```

### When to use linked-list chains

- When the chain order is fixed at construction time
- When handlers need to be composed fluently: `a.SetNext(b).SetNext(c)`
- When you're porting from a language where this is the standard approach (Java, C#)

### Downsides

- More boilerplate than the slice approach
- Harder to inspect or modify the chain after construction
- The `SetNext` return trick is idiomatic in Java but feels foreign in Go

### Your notes
<!-- User adds insights here during learning -->


---

## Implementation Shape 2: Slice-Based Chain (The Go Way)

The more idiomatic Go approach stores handlers in a slice and iterates over them. This is simpler, more flexible, and closer to how Go developers actually think about middleware.

### The Signature

```go
// HandlerFunc is a single step in the chain
type HandlerFunc func(ctx context.Context, req *Request) (*Response, error)

// Chain runs handlers in order until one handles the request or all pass
type Chain struct {
    handlers []HandlerFunc
}

func NewChain(handlers ...HandlerFunc) *Chain {
    return &Chain{handlers: handlers}
}

func (c *Chain) Execute(ctx context.Context, req *Request) (*Response, error) {
    for _, handler := range c.handlers {
        resp, err := handler(ctx, req)
        if err != nil {
            return nil, err  // handler rejected the request
        }
        if resp != nil {
            return resp, nil  // handler produced a response
        }
        // resp == nil && err == nil means "I didn't handle this, try next"
    }
    return nil, fmt.Errorf("no handler could process the request")
}
```

### Convention: Three Return States

The slice-based chain relies on a convention for handler return values:

| Return | Meaning |
|--------|---------|
| `(nil, error)` | Handler rejected the request. Stop the chain. |
| `(response, nil)` | Handler processed the request. Stop the chain. |
| `(nil, nil)` | Handler passed. Continue to next handler. |

This is clean but requires discipline. An alternative is an explicit enum:

```go
type Decision int

const (
    Pass    Decision = iota  // I didn't handle this
    Handled                   // I handled it, here's the response
    Reject                    // I rejected it, here's the error
)
```

### Dynamic Modification

The slice approach makes runtime modification trivial:

```go
func (c *Chain) Add(handler HandlerFunc) {
    c.handlers = append(c.handlers, handler)
}

func (c *Chain) Prepend(handler HandlerFunc) {
    c.handlers = append([]HandlerFunc{handler}, c.handlers...)
}

func (c *Chain) Remove(index int) {
    c.handlers = append(c.handlers[:index], c.handlers[index+1:]...)
}
```

You can add handlers at runtime based on configuration, feature flags, or request properties. This is much harder with linked lists.

### Your notes
<!-- User adds insights here during learning -->


---

## Implementation Shape 3: Functional Middleware (The `net/http` Way)

Go's HTTP ecosystem uses a functional approach that merges Chain of Responsibility with Decorator. Each middleware is a function that takes and returns `http.Handler`:

```go
type Middleware func(http.Handler) http.Handler
```

The chain is built by composing these functions. The key insight: each middleware receives the *rest of the chain* as the `next` parameter and decides whether to call it.

### Building the Chain

```go
func Chain(handler http.Handler, middlewares ...Middleware) http.Handler {
    // Apply in reverse order so the first middleware listed runs first
    for i := len(middlewares) - 1; i >= 0; i-- {
        handler = middlewares[i](handler)
    }
    return handler
}
```

Usage:

```go
mux := http.NewServeMux()
mux.HandleFunc("/api/users", handleUsers)

// Read left to right: requests hit logging first, then auth, then rate limit
handler := Chain(mux,
    withLogging,
    withAuth,
    withRateLimit,
)

http.ListenAndServe(":8080", handler)
```

### Short-Circuiting

This is where Chain of Responsibility diverges from pure Decorator. A middleware can choose not to call `next`:

```go
func withAuth(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        token := r.Header.Get("Authorization")
        if token == "" {
            // SHORT-CIRCUIT: don't call next, respond immediately
            http.Error(w, "unauthorized", http.StatusUnauthorized)
            return
        }

        claims, err := validateToken(token)
        if err != nil {
            http.Error(w, "invalid token", http.StatusUnauthorized)
            return
        }

        // Enrich context and continue the chain
        ctx := context.WithValue(r.Context(), claimsKey, claims)
        next.ServeHTTP(w, r.WithContext(ctx))
    })
}
```

When `withAuth` rejects a request, `withRateLimit` and the actual handler never run. The request stops at authentication. This is Chain of Responsibility -- the auth handler decided the request's fate.

### The `http.Handler` Chain Is Go's Most Important Pattern

Nearly every Go web application uses this pattern. Here's a realistic production middleware stack:

```go
handler := Chain(router,
    withRequestID,       // inject X-Request-ID header
    withLogging,         // log request start/end with timing
    withRecover,         // recover from panics, return 500
    withCORS,            // set CORS headers
    withAuth,            // validate JWT, set user in context
    withRBAC("admin"),   // check role-based permissions
    withRateLimit(100),  // 100 requests per minute
    withTimeout(30*time.Second),  // cancel after 30s
)
```

Each middleware is independent. You can reorder them (though order matters for correctness). You can add or remove them without touching any other middleware. You can test each one in isolation. This is the power of Chain of Responsibility.

### Your notes
<!-- User adds insights here during learning -->


---

## Validation Chains: First Failure Stops

A common use of Chain of Responsibility is validation, where each validator checks one aspect of a request and the first failure stops processing:

```go
type Validator func(req *Request) error

func ValidateAll(req *Request, validators ...Validator) error {
    for _, validate := range validators {
        if err := validate(req); err != nil {
            return err  // first failure stops the chain
        }
    }
    return nil  // all validators passed
}
```

Usage:

```go
err := ValidateAll(req,
    validateSchema,       // is the JSON well-formed?
    validateRequiredFields, // are required fields present?
    validateFieldTypes,    // are field types correct?
    validateBusinessRules, // does the data make business sense?
)
```

This is chain of responsibility in its simplest form: an ordered sequence of checks where the first failure is authoritative.

### Collect-All vs First-Failure

Sometimes you want *all* errors, not just the first. That's a different pattern (closer to a pipeline or map):

```go
func ValidateCollectAll(req *Request, validators ...Validator) []error {
    var errs []error
    for _, validate := range validators {
        if err := validate(req); err != nil {
            errs = append(errs, err)
        }
    }
    return errs
}
```

This is *not* Chain of Responsibility -- it's a pipeline. The distinction matters: Chain of Responsibility implies short-circuiting. If every handler always runs, you have a pipeline.

### Your notes
<!-- User adds insights here during learning -->


---

## Cross-Language Comparison

### TypeScript: Express Middleware

Express middleware is Chain of Responsibility. Each middleware receives `(req, res, next)` and decides whether to call `next()`:

```typescript
// TypeScript/Express
app.use((req, res, next) => {
  const token = req.headers.authorization;
  if (!token) {
    res.status(401).json({ error: 'unauthorized' });
    return;  // short-circuit -- next() never called
  }
  req.user = validateToken(token);
  next();  // continue the chain
});
```

The key difference from Go: Express middleware receives `next` as a *callback*, while Go middleware receives `next` as a *wrapped handler*. Express uses runtime callbacks; Go uses compile-time composition.

| Aspect | Go `http.Handler` chain | Express middleware | Koa middleware |
|--------|------------------------|-------------------|----------------|
| **Chain mechanism** | Function composition | Callback (`next()`) | Async generator (`await next()`) |
| **Short-circuit** | Don't call `next.ServeHTTP` | Don't call `next()` | Don't call `await next()` |
| **Error propagation** | Write to `ResponseWriter` | Call `next(err)` | `throw` or `ctx.throw()` |
| **Request enrichment** | `context.WithValue` | Mutate `req` object | Set on `ctx.state` |
| **Type safety** | Compile-time (interface) | Runtime | Runtime |

### Rust: Tower Service Layers

Rust's tower crate uses a `Service` trait that's essentially Chain of Responsibility with strong types:

```rust
// Rust/Tower: Service trait
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

Tower layers wrap services the same way Go middleware wraps handlers. The difference: Rust enforces backpressure via `poll_ready()`, and the type system ensures each layer's input/output types align at compile time. Go's approach is simpler (just `http.Handler`) but loses that type-level guarantee.

### Your notes
<!-- User adds insights here during learning -->


---

## When to Use Chain of Responsibility

### Use it when:

- Requests need to pass through a series of independent processing steps
- Each step might handle the request, transform it, or reject it
- You need to add/remove/reorder steps without editing other steps
- Short-circuiting is important (auth failure = stop processing)
- The same chain structure is used across different request types

### Don't use it when:

- Every handler always runs (that's a pipeline, use a simple loop)
- There's only one handler (just call it directly)
- The chain order never changes and there are only 2-3 steps (simple if/else is clearer)
- You need guaranteed execution of cleanup steps (use `defer` instead)

### Anti-Patterns to Avoid

**1. Invisible chains**
If you can't easily see what handlers are in the chain and in what order, you have a debugging nightmare. Always make the chain composition visible at the call site.

**2. Swallowed errors**
A handler in the middle of the chain encounters an error, logs it, and continues as if nothing happened. The final handler processes an invalid request because the error was silently consumed.

```go
// BAD: error swallowed mid-chain
func badMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        _, err := validateSomething(r)
        if err != nil {
            log.Printf("validation failed: %v", err)  // logged but ignored!
        }
        next.ServeHTTP(w, r)  // continues regardless
    })
}
```

**3. Shared mutable state between handlers**
Handlers that write to shared state (maps, slices) without coordination create race conditions. Use `context.WithValue` for request-scoped data instead of shared maps.

**4. Chain built per request**
Building the chain fresh for every request is wasteful. Build the chain once at startup, then reuse it. Each request gets its own context, but the chain structure is shared.

**5. Circular chains**
Handler A delegates to Handler B, which delegates back to Handler A. This creates an infinite loop. In linked-list chains, validate that `SetNext` doesn't create a cycle.

### Your notes
<!-- User adds insights here during learning -->


---

## How It Works Under the Hood

### Memory Layout

In the `func(http.Handler) http.Handler` approach, each middleware creates a closure that captures the `next` handler. The final composed handler is a nested set of closures:

```
withLogging(withAuth(withRateLimit(actualHandler)))
```

In memory, this is a chain of function values, each holding a reference to the next. When a request arrives, execution flows inward through the closures:

```
withLogging closure
  -> calls withAuth closure
       -> calls withRateLimit closure
            -> calls actualHandler
            <- returns
       <- returns (withRateLimit after-logic runs)
  <- returns (withAuth after-logic runs)
<- returns (withLogging after-logic runs)
```

This is a call stack, not a data structure. Each middleware can run logic *before* and *after* the inner handler, which is why logging middleware can measure request duration -- it records `time.Now()` before calling `next`, and `time.Since(start)` after `next` returns.

### Performance Characteristics

| Approach | Chain setup | Per-request overhead | Memory |
|----------|-------------|---------------------|---------|
| Linked list | O(n) construction | O(n) traversal | One allocation per handler |
| Slice | O(n) construction | O(n) iteration | Single slice allocation |
| Functional composition | O(n) construction | O(n) call stack depth | One closure per middleware |

All three approaches are O(n) per request where n is the number of handlers. For HTTP middleware stacks (typically 5-15 handlers), the overhead is negligible compared to actual I/O. Do not optimize the chain mechanism -- optimize the handlers themselves.

### Your notes
<!-- User adds insights here during learning -->


---

## Pattern Connections

- **[[decorator]]** -- Decorator always wraps; Chain of Responsibility conditionally stops. In Go HTTP, they share the same `func(http.Handler) http.Handler` signature.
- **[[strategy]]** -- Strategy picks *one* behavior. Chain of Responsibility tries *multiple* behaviors in order.
- **[[observer]]** -- Observer broadcasts to *all* subscribers. Chain of Responsibility sends to handlers *in order until one handles it*.
- **[[pipeline]]** -- Pipeline processes *every* step. Chain of Responsibility *may short-circuit*.
- **[[command]]** -- Command encapsulates a request as an object. Chain of Responsibility routes that object to the right handler.

### Your notes
<!-- User adds insights here during learning -->
