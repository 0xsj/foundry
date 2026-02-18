# Strategy Pattern -- Go

## The Problem Strategy Solves

You have a piece of behavior that varies depending on context, configuration, or runtime conditions. The naive approach is a chain of `if/else` or `switch` statements inside the function that needs the behavior. This works until it doesn't -- the function grows, new variants require editing the same function, tests become unwieldy, and you can't add new variants without touching the core logic.

The Strategy pattern extracts each variant into its own self-contained unit, then lets the caller (or configuration, or runtime decision) choose which one to plug in. The core logic doesn't know or care which variant it got -- it just calls the interface.

This is not abstract. You've done this in JavaScript/TypeScript every time you passed a callback:

```typescript
// TypeScript: strategy via callback
const items = [3, 1, 4, 1, 5];
items.sort((a, b) => a - b);  // ascending strategy
items.sort((a, b) => b - a);  // descending strategy
```

The `sort` function doesn't care how comparison works. It delegates that decision to the caller. That's the Strategy pattern. In Go, we have two idiomatic ways to express it: **interfaces** and **function types**. Both are first-class, both are used extensively in the standard library, and the choice between them depends on complexity.

### Real-world situations where Strategy appears

- **Notification dispatching**: Send via email, SMS, Slack, or webhook based on user preferences
- **Payment processing**: Charge via Stripe, PayPal, or bank transfer depending on region
- **Compression**: Apply gzip, zstd, or no compression based on payload size or client capabilities
- **Retry policies**: Use fixed delay, exponential backoff, or jittered backoff depending on the service being called
- **Rate limiting**: Token bucket, sliding window, or fixed window based on the API tier
- **Authentication**: API key, JWT, OAuth, or mTLS depending on the client type
- **Serialization**: JSON, protobuf, or msgpack depending on the transport

Every one of these follows the same shape: a behavior varies, the core code shouldn't know the details, and new variants should be addable without touching existing code.

### Your notes
<!-- User adds insights here during learning -->


---

## Interface-Based Strategy: The Go Way

### How Go Interfaces Enable Strategy at Zero Cost

In Java or C#, the Strategy pattern requires an abstract class or interface, concrete implementations that explicitly declare `implements`, and often a factory to wire them together. In Go, the pattern is lighter because of **implicit interface satisfaction** -- any type that has the right methods satisfies the interface, with no declaration or registration.

This means you can define a strategy interface in the package that *uses* it (the consumer), not in the packages that implement it. The implementations don't import the consumer. They don't know the interface exists. The compiler verifies the contract when the pieces are assembled.

```go
// In package notifier -- defines what it needs
type DeliveryStrategy interface {
    Deliver(ctx context.Context, recipient string, message Message) error
}

// In package email -- knows nothing about notifier.DeliveryStrategy
type Sender struct {
    smtpHost string
    smtpPort int
    from     string
}

func (s *Sender) Deliver(ctx context.Context, recipient string, msg Message) error {
    // SMTP delivery logic
    return nil
}

// In package slack -- also knows nothing about notifier.DeliveryStrategy
type Client struct {
    webhookURL string
    httpClient *http.Client
}

func (c *Client) Deliver(ctx context.Context, channel string, msg Message) error {
    // Slack webhook delivery
    return nil
}
```

At the assembly point (usually `main` or a wire function), you connect the pieces:

```go
func main() {
    var strategy notifier.DeliveryStrategy

    switch cfg.DeliveryMethod {
    case "email":
        strategy = &email.Sender{smtpHost: cfg.SMTPHost, smtpPort: 587, from: cfg.From}
    case "slack":
        strategy = &slack.Client{webhookURL: cfg.SlackWebhook, httpClient: http.DefaultClient}
    }

    svc := notifier.New(strategy)
    // ...
}
```

The `notifier` package never imports `email` or `slack`. The `email` and `slack` packages never import `notifier`. Dependencies flow inward, not outward. This is **dependency inversion** -- the high-level module defines the abstraction, the low-level modules implement it, and neither depends on the other directly.

### Under the Hood: Interface Dispatch

When you call `strategy.Deliver(...)`, Go performs dynamic dispatch through the interface's internal vtable. The interface value is a two-word pair:

```
strategy (interface value):
+-----------+-----------+
|   *itab   |   data    |
+-----------+-----------+
     |            |
     v            v
  itab:       *email.Sender{...}
  - type: *email.Sender
  - methods: [Deliver: 0x401234]
```

The `itab` (interface table) is a cached structure that maps the interface's method set to the concrete type's method addresses. The first time you assign a `*email.Sender` to a `DeliveryStrategy`, Go builds this mapping and caches it. Subsequent calls go through a single pointer indirection -- roughly equivalent to a C++ virtual function call.

This is cheaper than you'd think. The itab lookup is cached per (interface, concrete type) pair. The data pointer is either a direct pointer to the concrete value or, for small values (one word or less), the value itself stored inline. There's no heap allocation for the interface value itself -- it lives on the stack.

Compared to Java/C# abstract classes: Go has no class hierarchy to traverse, no vtable inherited from parent classes, and no virtual method resolution chain. It's a flat, single-level dispatch. The cost is one pointer dereference plus one indirect function call.

### When to Use Interface-Based Strategy

Use interfaces when:

- The strategy has **multiple methods** (like a full storage backend with Get, Set, Delete)
- The strategy has **internal state** that needs initialization and cleanup (connection pools, file handles)
- You need **compile-time verification** that implementations are complete
- The strategy is a **major architectural boundary** (swapping entire subsystems)
- You want to **mock it in tests** using a test double

```go
// Multi-method strategy -- interface is the right choice
type CacheBackend interface {
    Get(ctx context.Context, key string) ([]byte, bool, error)
    Set(ctx context.Context, key string, value []byte, ttl time.Duration) error
    Delete(ctx context.Context, key string) error
    Close() error
}
```

### Your notes
<!-- -->


---

## Function-Based Strategy: Go's Lightweight Alternative

### First-Class Functions as Strategies

When a strategy is a single behavior with no state, Go's function types are cleaner than a one-method interface. You define a function type, and any function with the matching signature satisfies it.

```go
// A backoff strategy determines how long to wait before the next retry
type BackoffFunc func(attempt int) time.Duration

// Fixed delay -- always wait the same amount
func FixedBackoff(delay time.Duration) BackoffFunc {
    return func(attempt int) time.Duration {
        return delay
    }
}

// Exponential backoff -- double the wait each attempt
func ExponentialBackoff(base time.Duration, maxDelay time.Duration) BackoffFunc {
    return func(attempt int) time.Duration {
        delay := base * time.Duration(1<<uint(attempt))
        if delay > maxDelay {
            return maxDelay
        }
        return delay
    }
}

// Jittered backoff -- exponential with randomness to prevent thundering herd
func JitteredBackoff(base time.Duration, maxDelay time.Duration) BackoffFunc {
    return func(attempt int) time.Duration {
        delay := base * time.Duration(1<<uint(attempt))
        if delay > maxDelay {
            delay = maxDelay
        }
        jitter := time.Duration(rand.Int63n(int64(delay) / 2))
        return delay/2 + jitter
    }
}
```

Usage is natural -- you pass a function where a function is expected:

```go
func retryWithBackoff(ctx context.Context, maxAttempts int, backoff BackoffFunc, operation func() error) error {
    var lastErr error
    for attempt := 0; attempt < maxAttempts; attempt++ {
        if err := operation(); err == nil {
            return nil
        } else {
            lastErr = err
        }
        if attempt < maxAttempts-1 {
            select {
            case <-time.After(backoff(attempt)):
            case <-ctx.Done():
                return ctx.Err()
            }
        }
    }
    return fmt.Errorf("all %d attempts failed: %w", maxAttempts, lastErr)
}
```

### The http.HandlerFunc Pattern

The Go standard library uses a powerful bridge pattern: a function type that implements an interface. This lets callers provide either a full struct or a simple function.

```go
// The interface
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}

// The function type that also satisfies the interface
type HandlerFunc func(ResponseWriter, *Request)

func (f HandlerFunc) ServeHTTP(w ResponseWriter, r *Request) {
    f(w, r)
}
```

Now callers can pass either:

```go
// A struct with state -- implements Handler via method
type authHandler struct {
    tokenStore TokenStore
    next       http.Handler
}
func (h *authHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) { ... }

// A simple function -- wrapped via HandlerFunc
http.Handle("/health", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
    w.WriteHeader(http.StatusOK)
}))
```

This is the **adapter pattern meets strategy pattern** -- `HandlerFunc` adapts a bare function into the `Handler` interface. You can use the same trick for your own strategies:

```go
type Compressor interface {
    Compress(data []byte) ([]byte, error)
}

type CompressorFunc func(data []byte) ([]byte, error)

func (f CompressorFunc) Compress(data []byte) ([]byte, error) {
    return f(data)
}
```

Now consumers accept `Compressor`, and callers provide either a struct implementation or a `CompressorFunc` wrapper around a simple function. Best of both worlds.

### Comparison: TypeScript and Rust

**TypeScript** uses functions as strategies naturally -- JavaScript's first-class functions and closures make this the default:

```typescript
type BackoffFn = (attempt: number) => number;

function exponentialBackoff(base: number): BackoffFn {
    return (attempt) => base * Math.pow(2, attempt);
}

async function retry<T>(
    maxAttempts: number,
    backoff: BackoffFn,
    operation: () => Promise<T>,
): Promise<T> {
    // ...
}
```

In TypeScript, there's no distinction between "function-based strategy" and "interface-based strategy" at the language level. You might define a `Backoff` interface with a `delay(attempt: number): number` method, but it's more idiomatic to just use a function type. Classes are used for strategies with complex state, but plain functions dominate for simple behaviors.

**Rust** takes a different approach. Function-based strategies use closures with trait bounds:

```rust
fn retry_with_backoff<F, B, T, E>(
    max_attempts: usize,
    backoff: B,
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Result<T, E>,
    B: Fn(usize) -> Duration,
{
    // ...
}
```

Rust's closures implement `Fn`, `FnMut`, or `FnOnce` traits depending on how they capture variables. For interface-based strategies, Rust uses trait objects (`Box<dyn Strategy>`) for dynamic dispatch or generics (`impl Strategy` / `T: Strategy`) for static dispatch. The generic approach is zero-cost -- the compiler generates a specialized version for each concrete type -- but produces larger binaries.

| Aspect | Go | TypeScript | Rust |
|--------|-----|------------|------|
| Function strategy | `type BackoffFunc func(int) time.Duration` | `type BackoffFn = (attempt: number) => number` | `Fn(usize) -> Duration` |
| Interface strategy | `type Backoff interface { Delay(int) time.Duration }` | `interface Backoff { delay(attempt: number): number }` | `trait Backoff { fn delay(&self, attempt: usize) -> Duration; }` |
| Bridge pattern | `func (f BackoffFunc) Delay(a int) time.Duration { return f(a) }` | Not needed (functions and interfaces interchangeable) | `impl Backoff for F where F: Fn(usize) -> Duration` (blanket impl) |
| Dispatch | Always dynamic | N/A (no runtime dispatch) | Static (generics) or dynamic (`dyn Trait`) |
| Adding new strategy | Just implement the methods | Just match the type signature | `impl Trait for NewType` |

### When to Use Function-Based Strategy

Use function types when:

- The strategy is a **single behavior** (one function signature)
- Implementations are **short and stateless** (or state is captured in a closure)
- You want callers to provide **inline anonymous functions**
- The strategy is a **callback** or **hook** in a pipeline

```go
// Good fit for function type -- single behavior, stateless
type HashFunc func(data []byte) []byte

// Callers provide inline:
cache := NewCache(sha256Hash)
cache := NewCache(md5Hash)
cache := NewCache(func(data []byte) []byte { return data }) // no-op for testing
```

### Your notes
<!-- -->


---

## Strategy Selection: Static vs Dynamic

### Compile-Time Selection (Configuration)

The simplest case: the strategy is chosen once at startup based on configuration and never changes. This is the most common pattern in production systems.

```go
func newCompressor(cfg Config) (Compressor, error) {
    switch cfg.Compression {
    case "gzip":
        return &GzipCompressor{level: cfg.CompressionLevel}, nil
    case "zstd":
        return &ZstdCompressor{level: cfg.CompressionLevel}, nil
    case "none":
        return NoopCompressor{}, nil
    default:
        return nil, fmt.Errorf("unknown compression: %q", cfg.Compression)
    }
}
```

The returned `Compressor` is stored once and used for the lifetime of the application. No per-request decisions, no overhead beyond the initial setup.

### Runtime Selection (Per-Request)

Sometimes the strategy changes per request, per user, or per data characteristics. This is where the pattern becomes more powerful -- and where you need to be careful about performance.

```go
type NotificationService struct {
    strategies map[string]DeliveryStrategy
    fallback   DeliveryStrategy
}

func (s *NotificationService) Notify(ctx context.Context, user User, msg Message) error {
    strategy, ok := s.strategies[user.PreferredChannel]
    if !ok {
        strategy = s.fallback
    }
    return strategy.Deliver(ctx, user.ContactInfo, msg)
}
```

The strategy map is populated at startup, but the selection happens at runtime. This is a registry pattern combined with strategy -- you pre-register all available strategies and look them up by key.

### Strategy Chains and Composition

Strategies can be composed. A common pattern is wrapping strategies with decorators:

```go
// A strategy that logs and delegates
type loggingDelivery struct {
    logger   *slog.Logger
    delegate DeliveryStrategy
}

func (l *loggingDelivery) Deliver(ctx context.Context, recipient string, msg Message) error {
    l.logger.Info("delivering notification",
        "recipient", recipient,
        "channel", fmt.Sprintf("%T", l.delegate),
    )
    start := time.Now()
    err := l.delegate.Deliver(ctx, recipient, msg)
    l.logger.Info("delivery complete",
        "duration", time.Since(start),
        "error", err,
    )
    return err
}
```

Now any strategy can be wrapped with logging:

```go
emailStrategy := &loggingDelivery{
    logger:   logger,
    delegate: &email.Sender{...},
}
```

This is the **decorator pattern applied to strategies** -- each decorator satisfies the same interface and wraps another implementation. You can stack them: logging, metrics, circuit breaking, rate limiting -- all wrapping the core strategy.

### Your notes
<!-- -->


---

## Strategy in the Go Standard Library

The standard library uses the Strategy pattern extensively, though it doesn't call it that. Recognizing these patterns helps you design your own APIs.

### `sort.Interface`

```go
type Interface interface {
    Len() int
    Less(i, j int) bool
    Swap(i, j int)
}
```

The `Less` method is the strategy -- it determines ordering. The sort algorithm doesn't care whether you're sorting by name, date, priority, or a custom comparator. This is interface-based strategy.

After generics (Go 1.18+), the function-based approach became idiomatic:

```go
slices.SortFunc(people, func(a, b Person) int {
    return cmp.Compare(a.Age, b.Age)
})
```

Same pattern, lighter syntax. The strategy is a comparison function.

### `io.Reader` / `io.Writer`

Every function that accepts `io.Reader` is using the Strategy pattern. The reading strategy varies -- files, network sockets, in-memory buffers, HTTP response bodies, compressed streams -- but the consumer doesn't care.

```go
func processData(r io.Reader) error {
    // Works with *os.File, *http.Response.Body, *bytes.Buffer,
    // *gzip.Reader, *tls.Conn, or any other Reader
}
```

### `http.Handler`

The entire Go HTTP routing ecosystem is built on the Strategy pattern. Each handler is a strategy for processing a specific request pattern:

```go
mux.Handle("/api/users", usersHandler)    // strategy for /api/users
mux.Handle("/api/orders", ordersHandler)  // strategy for /api/orders
```

Middleware wraps handlers -- decorating strategies with cross-cutting concerns:

```go
mux.Handle("/api/users", authMiddleware(loggingMiddleware(usersHandler)))
```

### `crypto.Hash`

```go
h := sha256.New()   // strategy: SHA-256
h := md5.New()      // strategy: MD5
h.Write(data)
sum := h.Sum(nil)
```

All hash functions satisfy `hash.Hash`, which embeds `io.Writer`. You can hash data from any `io.Reader` using `io.Copy` -- two strategy patterns composed.

### Your notes
<!-- -->


---

## Anti-Patterns and Pitfalls

### The God Interface

When a strategy interface has too many methods, it becomes a burden to implement and defeats the purpose of swappability:

```go
// BAD: God interface -- too many methods
type StorageStrategy interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
    List(prefix string) ([]string, error)
    Watch(prefix string) <-chan Event
    Transaction(fn func(tx Tx) error) error
    Backup(w io.Writer) error
    Stats() Stats
    Close() error
}
```

Every implementation must provide all 9 methods. Most callers only need 1-2 of them. Break it into small, composable interfaces:

```go
type Reader interface { Get(key string) ([]byte, error) }
type Writer interface { Set(key string, value []byte) error }
type Watcher interface { Watch(prefix string) <-chan Event }
```

### Nil Strategy Panic

If you store a strategy in a struct field and forget to initialize it, calling the strategy method panics:

```go
type Pipeline struct {
    compressor Compressor  // nil if not set
}

func (p *Pipeline) Process(data []byte) ([]byte, error) {
    return p.compressor.Compress(data)  // PANIC if compressor is nil
}
```

Always validate at construction time or provide a default:

```go
func NewPipeline(opts ...Option) *Pipeline {
    p := &Pipeline{
        compressor: NoopCompressor{},  // safe default
    }
    for _, opt := range opts {
        opt(p)
    }
    return p
}
```

### Strategy That Mutates Shared State

If a strategy holds mutable state and is shared across goroutines, you have a data race:

```go
// DANGEROUS: shared mutable state
type CountingCompressor struct {
    delegate Compressor
    count    int  // not thread-safe
}

func (c *CountingCompressor) Compress(data []byte) ([]byte, error) {
    c.count++  // race condition if called from multiple goroutines
    return c.delegate.Compress(data)
}
```

Use `sync.Mutex` or `atomic` operations if strategies must track state:

```go
type CountingCompressor struct {
    delegate Compressor
    count    atomic.Int64
}

func (c *CountingCompressor) Compress(data []byte) ([]byte, error) {
    c.count.Add(1)
    return c.delegate.Compress(data)
}
```

### Your notes
<!-- -->


---

## Putting It Together: Designing a Strategy-Based System

Here's the decision framework for using the Strategy pattern in Go:

### Step 1: Identify the Varying Behavior

Ask: "What behavior changes based on context, configuration, or user preference?" If the answer is "nothing varies," you don't need the pattern. Don't pre-abstract.

### Step 2: Choose Interface vs Function Type

| If the strategy... | Use |
|---|---|
| Has one method, no state | Function type |
| Has one method, with state | Either (function with closure captures state; interface works too) |
| Has multiple related methods | Interface |
| Needs initialization or cleanup | Interface (with Close/Shutdown method) |
| Is a callback or hook | Function type |
| Is a major architectural boundary | Interface |

### Step 3: Define the Contract

Write the interface or function type in the **consuming** package, not the implementing package. Keep it minimal -- only the methods the consumer actually calls.

### Step 4: Implement and Wire

Create concrete implementations. Wire them in `main` or a configuration layer. Use factory functions for complex construction.

### Step 5: Add a Default

Every strategy field should either be required (validated at construction) or have a sensible default. Never leave a nil strategy in a struct.

### Step 6: Test with Doubles

The whole point of strategies is testability. In tests, inject a simple implementation:

```go
func TestPipeline(t *testing.T) {
    // Strategy that returns input unchanged
    noop := CompressorFunc(func(data []byte) ([]byte, error) {
        return data, nil
    })

    p := NewPipeline(WithCompressor(noop))
    result, err := p.Process([]byte("hello"))
    // ...
}
```

### Your notes
<!-- -->


---

## Preview: Related Patterns

The Strategy pattern rarely lives alone. These patterns often work alongside it:

- **Factory** (covered in factory module): Creates the right strategy based on configuration. The `newCompressor(cfg)` function above is a factory.
- **Decorator** (covered in decorator module): Wraps a strategy with additional behavior (logging, metrics, retries). Same interface in, same interface out.
- **Template Method** (covered in template-method module): Defines a skeleton algorithm where specific steps are strategies. In Go, this often appears as a struct with function fields.
- **Dependency Injection** (covered in architecture module): The mechanism for providing strategies to consumers. In Go, typically constructor injection via function parameters.

### Your notes
<!-- -->
