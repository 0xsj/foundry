# Dependency Injection -- Go

## The Problem DI Solves

Every non-trivial program is a graph of collaborating components. A `UserService` needs a database, a `PaymentHandler` needs a payment gateway, an `AlertService` needs an email sender. The question is: **who creates these dependencies?**

The naive approach is to create them inside the component that needs them:

```go
type UserService struct {
    db *sql.DB
}

func NewUserService() *UserService {
    db, err := sql.Open("postgres", "host=localhost dbname=users")
    if err != nil {
        log.Fatal(err)
    }
    return &UserService{db: db}
}
```

This works until you need to:
- **Test it** -- you can't swap the real database for an in-memory one
- **Reuse it** -- every instance hardcodes the same connection string
- **Change it** -- switching from Postgres to MySQL means editing `UserService`
- **Compose it** -- two services that share a connection pool can't coordinate

The core problem: `UserService` controls its own dependency graph. It decides *what* it depends on and *how* to create it. This is **inversion of control** in reverse -- the component controls everything, making it rigid, untestable, and tightly coupled.

Dependency injection inverts this: **the caller provides the dependencies, the component just uses them.** The component declares what it needs (through constructor parameters or interfaces), and something higher up in the call stack assembles the pieces.

If you've written TypeScript with frameworks like NestJS or Angular, you've seen DI containers that automate this wiring. Go takes a different path: **manual constructor injection, guided by interfaces.** There's no magic, no decorators, no reflection-based autowiring. You wire things up explicitly in `main()`, and the compiler verifies every connection at build time.

This simplicity is a feature, not a limitation. Go's approach makes the dependency graph visible in code rather than hidden in configuration or runtime registration.

### Your notes
<!-- User adds insights here during learning -->


---

## Constructor Injection: The Go Standard

### The Pattern

Constructor injection in Go means passing dependencies as parameters to `New` functions:

```go
// UserService depends on abstractions, not concrete types.
type UserService struct {
    repo   UserRepository
    mailer EmailSender
    logger *slog.Logger
}

// NewUserService is the constructor. Every dependency is explicit.
func NewUserService(repo UserRepository, mailer EmailSender, logger *slog.Logger) *UserService {
    return &UserService{
        repo:   repo,
        mailer: mailer,
        logger: logger,
    }
}
```

Compare this to TypeScript:

```typescript
// TypeScript with manual DI (no container)
class UserService {
    constructor(
        private readonly repo: UserRepository,
        private readonly mailer: EmailSender,
        private readonly logger: Logger,
    ) {}
}
```

The shape is identical. Go just uses a free function (`NewUserService`) instead of a `constructor` method, because Go doesn't have classes. The principle is the same: **declare dependencies in the constructor, let the caller provide them.**

### Why Constructor Injection Over Other Forms

There are three common DI approaches:

| Approach | How | Tradeoff |
|----------|-----|----------|
| **Constructor injection** | Pass deps to `New` function | Dependencies are explicit and immutable. Can't forget one. |
| **Method injection** | Pass deps to individual methods | Good for per-call dependencies (like `context.Context`). Not for persistent collaborators. |
| **Property/field injection** | Set fields after construction | Object can exist in invalid state. Dependencies are mutable. Common in Java with `@Inject`. |

Go strongly favors constructor injection because:
1. **All fields are set before use** -- no half-initialized structs
2. **Dependencies are immutable** -- no race conditions from someone swapping a dependency mid-flight
3. **The compiler catches missing dependencies** -- forget an argument, get a compile error

Property injection exists in Go (exported fields), but it signals "optional configuration," not "required collaborator." If your struct can't function without a dependency, it belongs in the constructor.

### Validation in Constructors

Production constructors validate their inputs:

```go
func NewUserService(repo UserRepository, mailer EmailSender, logger *slog.Logger) (*UserService, error) {
    if repo == nil {
        return nil, errors.New("user service: repository is required")
    }
    if mailer == nil {
        return nil, errors.New("user service: email sender is required")
    }
    if logger == nil {
        logger = slog.Default() // Fallback to default logger
    }
    return &UserService{
        repo:   repo,
        mailer: mailer,
        logger: logger,
    }, nil
}
```

Notice the distinction: `repo` and `mailer` are required (nil means a bug), but `logger` has a sensible default. This is a common pattern -- required dependencies cause errors, optional ones have defaults.

### Your notes
<!-- User adds insights here during learning -->


---

## Accept Interfaces, Return Structs

### The Go Proverb

This is one of Go's most important design principles and the backbone of idiomatic DI. It means:

- **Function parameters** should be interface types (what behavior do I need?)
- **Return values** should be concrete struct types (here's exactly what you get)

```go
// Accept interface: the consumer defines what it needs.
type UserRepository interface {
    FindByID(ctx context.Context, id string) (*User, error)
    Save(ctx context.Context, user *User) error
}

// Return struct: the implementation provides a concrete type.
type PostgresUserRepo struct {
    db *sql.DB
}

func NewPostgresUserRepo(db *sql.DB) *PostgresUserRepo {
    return &PostgresUserRepo{db: db}
}
```

Why not return the interface?

```go
// DON'T do this in Go.
func NewUserRepo(db *sql.DB) UserRepository {
    return &PostgresUserRepo{db: db}
}
```

Returning the interface hides the concrete type, preventing callers from accessing implementation-specific methods, asserting the type in tests, or embedding the struct. It also obscures documentation -- `godoc` shows the interface, not the actual struct with its fields and methods.

### Where the Interface Lives

This is where Go diverges from most languages. In Java or C#, the interface lives in the *implementor's* package or in a shared "contracts" package. In Go:

**The interface lives where it's consumed, not where it's implemented.**

```
myapp/
├── service/
│   └── user.go         // Defines UserRepository interface (consumer)
├── postgres/
│   └── repo.go         // Implements *PostgresRepo (no imports from service/)
├── redis/
│   └── cache.go        // Implements *RedisCache (no imports from service/)
└── main.go             // Wires it all together
```

The `postgres` package doesn't import `service`. It doesn't know the `UserRepository` interface exists. It just has a struct with methods. The Go compiler verifies at the point of use (in `main.go` or wherever you pass the concrete type to a function expecting the interface) that the struct satisfies the interface.

This is fundamentally different from TypeScript, where you'd write `class PostgresRepo implements UserRepository`. In Go, satisfaction is implicit -- there's no `implements` keyword. This means:
- Implementations are decoupled from their consumers
- You can define interfaces for third-party types you don't control
- Interfaces stay small because they only describe what the consumer actually needs

### The io.Writer Standard Library Example

The standard library demonstrates this perfectly:

```go
// In package io -- defines the interface
type Writer interface {
    Write(p []byte) (n int, err error)
}

// In package os -- returns a concrete type
func Create(name string) (*File, error) { ... }

// *os.File satisfies io.Writer, but os never imports io.
// The consumer (e.g., json.NewEncoder) accepts io.Writer.
func json.NewEncoder(w io.Writer) *Encoder { ... }
```

`json.NewEncoder` doesn't know about files, network connections, or buffers. It accepts anything that can `Write`. You can pass `os.Stdout`, an `http.ResponseWriter`, a `bytes.Buffer`, or your own custom type. That's DI through interface design, built into the language.

### Your notes
<!-- User adds insights here during learning -->


---

## The Composition Root

### What It Is

The composition root is the single place in your application where the entire dependency graph is assembled. In Go, this is `main()` (or a function called from `main()`).

```go
func main() {
    // 1. Create infrastructure (things with no dependencies, or external deps)
    db, err := sql.Open("postgres", os.Getenv("DATABASE_URL"))
    if err != nil {
        log.Fatal(err)
    }
    defer db.Close()

    logger := slog.New(slog.NewJSONHandler(os.Stdout, nil))
    smtpClient := smtp.NewClient(os.Getenv("SMTP_HOST"), os.Getenv("SMTP_PORT"))

    // 2. Create repositories (depend on infrastructure)
    userRepo := postgres.NewUserRepo(db)
    orderRepo := postgres.NewOrderRepo(db)

    // 3. Create services (depend on repositories and infrastructure)
    emailSender := email.NewSender(smtpClient)
    userService := service.NewUserService(userRepo, emailSender, logger)
    orderService := service.NewOrderService(orderRepo, userRepo, logger)

    // 4. Create handlers (depend on services)
    userHandler := handler.NewUserHandler(userService)
    orderHandler := handler.NewOrderHandler(orderService)

    // 5. Create router and start server
    mux := http.NewServeMux()
    mux.Handle("GET /users/{id}", userHandler.Get())
    mux.Handle("POST /orders", orderHandler.Create())

    log.Fatal(http.ListenAndServe(":8080", mux))
}
```

### Why One Place?

The composition root pattern prevents dependency creation from leaking throughout your codebase. If services create their own dependencies, you get:

1. **Hidden dependencies** -- you can't see what a service needs without reading its internals
2. **Duplicate infrastructure** -- two services might each open their own database connection
3. **Testing friction** -- you can't intercept and replace dependencies
4. **Configuration sprawl** -- connection strings and API keys scattered across packages

With a composition root:
- Every dependency is visible in one place
- Shared resources (database pool, logger) are created once and passed to all consumers
- Tests create their own composition root with fakes and mocks
- Configuration is loaded once and threaded through the graph

### The Test Composition Root

Tests are just a different composition root:

```go
func TestUserService_CreateUser(t *testing.T) {
    // Test composition root: wire up fakes instead of real implementations
    repo := &fakeUserRepo{users: make(map[string]*User)}
    mailer := &fakeMailer{}
    logger := slog.New(slog.NewTextHandler(io.Discard, nil))

    svc := service.NewUserService(repo, mailer, logger)

    // Now test with full control over dependencies
    err := svc.CreateUser(context.Background(), "alice", "alice@example.com")
    if err != nil {
        t.Fatalf("unexpected error: %v", err)
    }

    // Assert side effects through the fakes
    if len(mailer.sent) != 1 {
        t.Errorf("expected 1 email sent, got %d", len(mailer.sent))
    }
}
```

The service code is identical in production and tests. Only the dependencies change. This is the payoff of DI -- isolation and control without modifying the code under test.

### Your notes
<!-- User adds insights here during learning -->


---

## Testing with DI: Fakes, Mocks, and Stubs

### The Test Double Spectrum

| Double | What it does | When to use |
|--------|-------------|-------------|
| **Stub** | Returns canned responses | You need a dependency to return specific values |
| **Fake** | Working implementation with shortcuts | In-memory database, local file system, etc. |
| **Mock** | Records calls, asserts interactions | Verifying *how* something was called (use sparingly) |
| **Spy** | Real implementation that records calls | Need real behavior plus verification |

Go strongly favors **fakes** over mocks. Why? Fakes test *behavior* (what the system does), mocks test *interaction* (how the system does it). Behavior tests are less brittle -- they survive refactoring. Interaction tests break when you change the implementation, even if the behavior is correct.

### Writing Fakes in Go

A fake implements the same interface as the real dependency but uses simple in-memory data structures:

```go
// The interface (defined in the consumer package)
type UserRepository interface {
    FindByID(ctx context.Context, id string) (*User, error)
    Save(ctx context.Context, user *User) error
    Delete(ctx context.Context, id string) error
}

// The fake (defined in test files or a testutil package)
type FakeUserRepo struct {
    mu    sync.Mutex
    users map[string]*User
    // Control knobs for testing error paths
    SaveErr   error
    FindErr   error
    DeleteErr error
}

func NewFakeUserRepo() *FakeUserRepo {
    return &FakeUserRepo{users: make(map[string]*User)}
}

func (f *FakeUserRepo) FindByID(ctx context.Context, id string) (*User, error) {
    if f.FindErr != nil {
        return nil, f.FindErr
    }
    f.mu.Lock()
    defer f.mu.Unlock()
    u, ok := f.users[id]
    if !ok {
        return nil, ErrNotFound
    }
    return u, nil
}

func (f *FakeUserRepo) Save(ctx context.Context, user *User) error {
    if f.SaveErr != nil {
        return f.SaveErr
    }
    f.mu.Lock()
    defer f.mu.Unlock()
    f.users[user.ID] = user
    return nil
}

func (f *FakeUserRepo) Delete(ctx context.Context, id string) error {
    if f.DeleteErr != nil {
        return f.DeleteErr
    }
    f.mu.Lock()
    defer f.mu.Unlock()
    delete(f.users, id)
    return nil
}
```

The fake is a real, working in-memory repository. You can save users, find them, delete them. It behaves like a database without the database. The `SaveErr`/`FindErr` fields let you simulate failures in specific tests.

### When to Mock (Sparingly)

Sometimes you need to verify *that* a dependency was called, not just *what* the system produced. Example: verifying that a notification was actually sent:

```go
type SpyEmailSender struct {
    Sent []SentEmail
}

type SentEmail struct {
    To      string
    Subject string
    Body    string
}

func (s *SpyEmailSender) Send(ctx context.Context, to, subject, body string) error {
    s.Sent = append(s.Sent, SentEmail{To: to, Subject: subject, Body: body})
    return nil
}

// In test:
spy := &SpyEmailSender{}
svc := NewUserService(repo, spy, logger)
svc.CreateUser(ctx, "alice", "alice@example.com")

if len(spy.Sent) != 1 || spy.Sent[0].To != "alice@example.com" {
    t.Error("expected welcome email to alice")
}
```

This is a spy -- it records calls for later assertion. It's appropriate here because sending an email is a side effect you need to verify. But don't reach for this pattern for every dependency. If you're asserting "the repository's `Save` was called exactly once with these parameters," you're testing implementation details, not behavior.

### Comparison: Go vs TypeScript vs Rust

| Aspect | Go | TypeScript | Rust |
|--------|-----|-----------|------|
| Interface satisfaction | Implicit | Explicit (`implements`) | Explicit (`impl Trait for`) |
| DI mechanism | Constructor functions | Constructor or DI container | Generic type parameters or trait objects |
| Test doubles | Hand-written fakes | Jest mocks, manual fakes | Mock crates or manual fakes |
| Autowiring | Not idiomatic (Wire exists) | Common (InversifyJS, tsyringe, NestJS) | Not common |
| Compile-time safety | Yes (interface check at use site) | Yes (type system) | Yes (trait bounds) |

In Rust, DI often uses generics:

```rust
struct UserService<R: UserRepository, E: EmailSender> {
    repo: R,
    mailer: E,
}

impl<R: UserRepository, E: EmailSender> UserService<R, E> {
    fn new(repo: R, mailer: E) -> Self {
        Self { repo, mailer }
    }
}
```

This is monomorphized at compile time -- each combination of concrete types generates specialized code. Go's interfaces use dynamic dispatch (vtable-like), which is slightly slower but more flexible (you can swap implementations at runtime without recompilation).

### Your notes
<!-- User adds insights here during learning -->


---

## Functional DI: Injecting Functions

### When Interfaces Are Overkill

Not every dependency needs a full interface. If a dependency has a single behavior, inject a function instead:

```go
// Instead of this interface:
type TimeProvider interface {
    Now() time.Time
}

// Inject a function:
type TokenService struct {
    now func() time.Time  // Dependency: how to get the current time
}

func NewTokenService(now func() time.Time) *TokenService {
    if now == nil {
        now = time.Now // Default to real time
    }
    return &TokenService{now: now}
}

func (t *TokenService) Generate(userID string) Token {
    return Token{
        UserID:    userID,
        IssuedAt:  t.now(),
        ExpiresAt: t.now().Add(24 * time.Hour),
    }
}
```

In production, pass `time.Now`. In tests, pass a function that returns a fixed time:

```go
func TestTokenExpiry(t *testing.T) {
    fixedTime := time.Date(2026, 1, 15, 10, 0, 0, 0, time.UTC)
    svc := NewTokenService(func() time.Time { return fixedTime })

    token := svc.Generate("user-123")

    expected := fixedTime.Add(24 * time.Hour)
    if token.ExpiresAt != expected {
        t.Errorf("expected expiry %v, got %v", expected, token.ExpiresAt)
    }
}
```

This is the same DI principle -- the component doesn't control how time works. But there's no interface, no struct implementing it. Just a function. Go's first-class functions make this natural.

### The HTTP Handler Pattern

The standard library uses functional DI everywhere. `http.HandleFunc` accepts a function, not an interface:

```go
// http.HandleFunc is functional DI.
// You're injecting behavior into the router.
http.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
    w.WriteHeader(http.StatusOK)
})
```

But when you need state (like a database reference), you use closures:

```go
func makeUserHandler(repo UserRepository) http.HandlerFunc {
    return func(w http.ResponseWriter, r *http.Request) {
        id := r.PathValue("id")
        user, err := repo.FindByID(r.Context(), id)
        if err != nil {
            http.Error(w, "not found", http.StatusNotFound)
            return
        }
        json.NewEncoder(w).Encode(user)
    }
}
```

The closure captures `repo`, making it available to the handler without global state. This is functional DI -- the dependency is injected through the closure's environment.

### Functions vs Interfaces: When to Use Which

| Use a function when... | Use an interface when... |
|------------------------|--------------------------|
| Dependency has a single method | Dependency has multiple related methods |
| Behavior is simple and stateless | Implementation needs internal state |
| You want maximum flexibility | You want a named contract |
| Standard library uses `func` type (e.g., `http.HandlerFunc`) | You need to mock multiple behaviors together |
| It's a factory, validator, or transformer | It's a repository, service, or client |

If you find yourself defining a single-method interface, consider whether a function type would be simpler. Go's `sort.Interface` has three methods, so an interface makes sense. A `Validator func(string) error` is just a function.

### Your notes
<!-- User adds insights here during learning -->


---

## Manual DI vs DI Containers

### Manual Wiring (The Go Way)

Most Go projects wire dependencies manually in `main()`. This is explicit, readable, and requires zero dependencies:

```go
func main() {
    db := mustOpenDB()
    logger := slog.Default()

    userRepo := postgres.NewUserRepo(db)
    mailer := sendgrid.NewClient(os.Getenv("SENDGRID_KEY"))
    userSvc := service.NewUserService(userRepo, mailer, logger)
    handler := api.NewUserHandler(userSvc)

    http.Handle("/users", handler)
    log.Fatal(http.ListenAndServe(":8080", nil))
}
```

**Pros:** No magic. You can click through every `New` function to see what's happening. The compiler checks everything. Easy to understand for new team members.

**Cons:** Can get verbose for large applications with dozens of services. Adding a new dependency to a service means updating the wiring code in `main()`.

### Google Wire (Code Generation)

Wire is Google's compile-time DI tool for Go. It generates the wiring code for you:

```go
// wire.go (you write this)
//go:build wireinject

package main

import "github.com/google/wire"

func InitializeApp() (*App, error) {
    wire.Build(
        postgres.NewUserRepo,      // Provides UserRepository
        sendgrid.NewClient,        // Provides EmailSender
        service.NewUserService,    // Needs UserRepository, EmailSender
        api.NewUserHandler,        // Needs UserService
        NewApp,                    // Needs UserHandler
    )
    return nil, nil  // Wire generates the real implementation
}
```

Running `wire` generates `wire_gen.go` with explicit wiring code -- no reflection, no runtime cost. It's essentially automated manual wiring.

### Uber dig/fx (Runtime DI)

Uber's `fx` framework uses reflection-based DI:

```go
func main() {
    fx.New(
        fx.Provide(
            postgres.NewUserRepo,
            sendgrid.NewClient,
            service.NewUserService,
            api.NewUserHandler,
        ),
        fx.Invoke(startServer),
    ).Run()
}
```

This looks clean, but dependencies are resolved at runtime. If you misconfigure something, you get a runtime error, not a compile error. The tradeoff: less boilerplate, less safety.

### Recommendation

| Project Size | Recommendation |
|-------------|----------------|
| Small (< 10 services) | Manual wiring. It's 20-50 lines of code. |
| Medium (10-50 services) | Manual wiring, possibly with helper functions to group related services. |
| Large (50+ services) | Consider Wire for code generation. Still compile-time safe. |
| Already using fx | fx is fine. Just be aware of runtime errors. |

Most Go projects do fine with manual wiring forever. The dependency graph is rarely so complex that you need a tool. If your `main()` is getting unwieldy, that's often a sign of an architectural problem (too many cross-cutting concerns, not enough layering), not a wiring problem.

### Your notes
<!-- User adds insights here during learning -->


---

## Standard Library DI in Action

### http.Handler and ResponseWriter

The `net/http` package is built on DI. Every handler receives its dependencies through the function signature:

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}
```

`ResponseWriter` is injected by the server -- you don't create it. The server creates the connection, wraps it in a `ResponseWriter`, and hands it to your handler. This is method injection for per-request dependencies.

### log.Logger and io.Writer

```go
// log.New accepts an io.Writer -- you inject where logs go.
logger := log.New(os.Stderr, "app: ", log.LstdFlags)      // Production: stderr
logger := log.New(&buf, "test: ", log.LstdFlags)           // Test: buffer
logger := log.New(io.Discard, "", 0)                        // Benchmarks: nowhere
```

The logger doesn't know where it writes. It accepts an `io.Writer` and writes to it. You control the destination. This is DI through the `io.Writer` interface.

### database/sql and driver.Driver

```go
// sql.Open uses a registered driver name.
db, err := sql.Open("postgres", connStr)
```

The `database/sql` package accepts a driver through registration (`sql.Register`). The `sql.DB` object doesn't know whether it's talking to Postgres, MySQL, or SQLite -- it works through the `driver.Driver` interface. This is a form of DI through the service locator pattern (global registration), which is one of the few places Go uses it because drivers are typically application-global.

### Your notes
<!-- User adds insights here during learning -->


---

## Anti-Patterns

### Service Locator

A service locator is a global registry where components look up their dependencies:

```go
// DON'T do this
var registry = make(map[string]interface{})

func Register(name string, impl interface{}) {
    registry[name] = impl
}

func Get(name string) interface{} {
    return registry[name]
}

// Service looks up its own dependencies
type UserService struct{}

func (s *UserService) CreateUser(name string) error {
    repo := Get("userRepo").(UserRepository)  // Runtime lookup, no compile safety
    mailer := Get("mailer").(EmailSender)      // Could panic if not registered
    // ...
}
```

Problems:
- Dependencies are hidden -- you can't see what `UserService` needs without reading its code
- No compile-time safety -- missing registrations crash at runtime
- Testing requires setting up the global registry
- Unclear initialization order

### God Struct

A struct with too many dependencies signals it's doing too much:

```go
// This struct has too many responsibilities
type AppService struct {
    userRepo    UserRepository
    orderRepo   OrderRepository
    productRepo ProductRepository
    mailer      EmailSender
    sms         SMSSender
    push        PushNotifier
    cache       CacheStore
    logger      *slog.Logger
    metrics     MetricsRecorder
    config      *Config
}
```

If your constructor takes more than 5-6 parameters, the struct probably has multiple responsibilities that should be split:

```go
// Better: separate notification concerns into their own service
type NotificationService struct {
    mailer EmailSender
    sms    SMSSender
    push   PushNotifier
    logger *slog.Logger
}

type UserService struct {
    repo     UserRepository
    notifier *NotificationService  // Composed, not flattened
    logger   *slog.Logger
}
```

### Global Singletons

```go
// DON'T do this
var db *sql.DB

func init() {
    var err error
    db, err = sql.Open("postgres", os.Getenv("DB_URL"))
    if err != nil {
        log.Fatal(err)
    }
}

type UserService struct{}

func (s *UserService) FindUser(id string) (*User, error) {
    return queryUser(db, id)  // Uses global -- impossible to test independently
}
```

Global singletons are the enemy of testability. They create hidden dependencies, prevent parallel test execution, and make initialization order implicit.

### Your notes
<!-- User adds insights here during learning -->


---

## Connecting to Other Patterns

DI doesn't exist in isolation. It's the glue that makes other patterns work:

- **[[strategy]]** -- DI is how you inject the chosen strategy into the consumer. Without DI, strategy selection is hardcoded.
- **[[factory]]** -- Factories create objects. DI wires them together. A factory might be a dependency itself (inject a `UserFactory` when the service needs to create users on demand).
- **[[repository]]** -- The repository interface is the canonical DI example in Go. The service accepts a repository interface; the composition root provides the concrete implementation.
- **[[decorator-middleware]]** -- Decorators wrap dependencies with additional behavior (logging, metrics, retries). They're injected in place of the original dependency using the same interface.
- **[[observer]]** -- Event listeners are injected into the event bus or emitter.

DI is also the foundation for architecture patterns like **[[hexagonal]]** (ports and adapters) and **[[clean-architecture]]** (the dependency rule). Without DI, you can't invert the dependency direction -- infrastructure would depend on domain, instead of the other way around.

### Your notes
<!-- User adds insights here during learning -->


---

## Key Takeaways

1. **DI in Go is just passing arguments.** No framework, no decorators, no magic. Constructor injection is the primary mechanism.
2. **Accept interfaces, return structs.** Define interfaces where they're consumed, implement them where it's convenient, wire them together in `main()`.
3. **The composition root is `main()`.** One place to see the entire dependency graph. Tests are an alternative composition root.
4. **Prefer fakes over mocks.** Fakes test behavior (what the system does). Mocks test interaction (how it does it). Behavior tests survive refactoring.
5. **Functional DI for single behaviors.** If a dependency is a single function, inject a function. Don't create an interface just for the sake of an interface.
6. **Manual wiring is fine.** Most Go projects never need Wire, dig, or fx. The explicit `main()` wiring is the recommended approach for most cases.
7. **Watch for the God struct.** If your constructor has 8 parameters, you probably need to decompose the struct into smaller collaborators.

### Your notes
<!-- User adds insights here during learning -->
