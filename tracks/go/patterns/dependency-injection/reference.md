# Go Reference -- Dependency Injection

> Extracted from [Effective Go](https://go.dev/doc/effective_go),
> [Go Code Review Comments](https://github.com/golang/go/wiki/CodeReviewComments),
> [Go Proverbs](https://go-proverbs.github.io/), and
> [Go Blog: Wire](https://go.dev/blog/wire)
> for the `dependency-injection` module. Covers: interface design for DI,
> constructor patterns, composition root, testing with interfaces, Wire code generation.

---

## Interfaces and Implicit Satisfaction

Source: [Go Spec -- Interface Types](https://go.dev/ref/spec#Interface_types)

An interface type defines a type set. A variable of interface type can store a value of any type that is in the type set of the interface. Such a type is said to *implement* the interface.

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

A type implements an interface by implementing its methods. There is no explicit declaration of intent (`implements` keyword). Implementation is structural (implicit), not nominal (explicit).

### Interface Embedding

Interfaces can embed other interfaces to compose larger contracts:

```go
type ReadWriter interface {
    Reader
    Writer
}

type ReadWriteCloser interface {
    Reader
    Writer
    Closer
}
```

### Empty Interface

The empty interface `interface{}` (or `any` since Go 1.18) is satisfied by every type. Avoid it in DI -- it provides no compile-time checking:

```go
// Bad: no type safety
func NewService(deps ...interface{}) *Service

// Good: explicit dependencies
func NewService(repo UserRepository, mailer EmailSender) *Service
```

---

## Interface Design Guidelines

Source: [Go Code Review Comments -- Interfaces](https://github.com/golang/go/wiki/CodeReviewComments#interfaces)

### Define Interfaces at Point of Use

> Go interfaces generally belong in the package that uses values of the interface type, not the package that implements those values. The implementing package should return concrete (usually pointer or struct) types.

```go
// package consumer -- defines what it needs
package consumer

type Store interface {
    Get(key string) (string, error)
    Set(key, value string) error
}

type Service struct {
    store Store
}
```

```go
// package redis -- returns concrete type, no dependency on consumer
package redis

type Client struct { /* ... */ }

func New(addr string) *Client { return &Client{/* ... */} }
func (c *Client) Get(key string) (string, error) { /* ... */ }
func (c *Client) Set(key, value string) error { /* ... */ }
```

### Keep Interfaces Small

Source: [Go Proverbs](https://go-proverbs.github.io/)

> The bigger the interface, the weaker the abstraction.

Prefer interfaces with 1-3 methods. Large interfaces are hard to implement, hard to mock, and usually indicate the consumer depends on too much behavior.

| Interface | Methods | Package |
|-----------|---------|---------|
| `io.Reader` | 1 | io |
| `io.Writer` | 1 | io |
| `io.Closer` | 1 | io |
| `io.ReadWriter` | 2 | io (embedded) |
| `fmt.Stringer` | 1 | fmt |
| `sort.Interface` | 3 | sort |
| `http.Handler` | 1 | net/http |
| `http.ResponseWriter` | 3 | net/http |
| `driver.Driver` | 1 | database/sql/driver |

### Compile-Time Interface Check

Use a blank identifier assignment to verify a type satisfies an interface at compile time:

```go
var _ UserRepository = (*PostgresRepo)(nil)
var _ io.Writer = (*MyWriter)(nil)
```

This creates no runtime cost. If the type doesn't satisfy the interface, compilation fails.

---

## Constructor Patterns

Source: [Effective Go -- Constructors](https://go.dev/doc/effective_go#composite_literals)

### Basic Constructor

```go
func NewUserService(repo UserRepository, logger *slog.Logger) *UserService {
    return &UserService{
        repo:   repo,
        logger: logger,
    }
}
```

Convention: `New<TypeName>` returns `*TypeName`. If the package has only one primary type, use `New()`:

```go
package cache

func New(maxSize int) *LRU { ... }
```

### Constructor with Error

When construction can fail (validation, connection, etc.):

```go
func NewUserService(repo UserRepository, logger *slog.Logger) (*UserService, error) {
    if repo == nil {
        return nil, fmt.Errorf("user service: repository is required")
    }
    return &UserService{repo: repo, logger: logger}, nil
}
```

### Functional Options Pattern

For optional dependencies and configuration:

```go
type Option func(*UserService)

func WithLogger(logger *slog.Logger) Option {
    return func(s *UserService) {
        s.logger = logger
    }
}

func WithTimeout(d time.Duration) Option {
    return func(s *UserService) {
        s.timeout = d
    }
}

func NewUserService(repo UserRepository, opts ...Option) *UserService {
    s := &UserService{
        repo:    repo,
        logger:  slog.Default(),
        timeout: 30 * time.Second,
    }
    for _, opt := range opts {
        opt(s)
    }
    return s
}
```

Use functional options when:
- A struct has many optional configurations
- You want sensible defaults with overrides
- The set of options may grow over time

Do not use functional options for required dependencies. Those belong as explicit parameters.

---

## Testing with Interfaces

Source: [Go Blog -- Using Subtests and Sub-benchmarks](https://go.dev/blog/subtests), [Testing Package](https://pkg.go.dev/testing)

### Table-Driven Tests with Injected Dependencies

```go
func TestUserService_FindUser(t *testing.T) {
    tests := []struct {
        name    string
        repo    UserRepository
        userID  string
        want    *User
        wantErr bool
    }{
        {
            name: "found",
            repo: &fakeRepo{users: map[string]*User{
                "u1": {ID: "u1", Name: "Alice"},
            }},
            userID: "u1",
            want:   &User{ID: "u1", Name: "Alice"},
        },
        {
            name:    "not found",
            repo:    &fakeRepo{users: map[string]*User{}},
            userID:  "u999",
            wantErr: true,
        },
        {
            name:    "repo error",
            repo:    &fakeRepo{err: errors.New("connection refused")},
            userID:  "u1",
            wantErr: true,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            svc := NewUserService(tt.repo, slog.Default())
            got, err := svc.FindUser(context.Background(), tt.userID)
            if (err != nil) != tt.wantErr {
                t.Errorf("error = %v, wantErr %v", err, tt.wantErr)
            }
            if !reflect.DeepEqual(got, tt.want) {
                t.Errorf("got %+v, want %+v", got, tt.want)
            }
        })
    }
}
```

### Test Helper for Common Setup

```go
func setupTestService(t *testing.T) (*UserService, *FakeRepo, *SpyMailer) {
    t.Helper()
    repo := NewFakeRepo()
    mailer := &SpyMailer{}
    logger := slog.New(slog.NewTextHandler(io.Discard, nil))
    svc := NewUserService(repo, mailer, logger)
    return svc, repo, mailer
}
```

---

## Google Wire

Source: [Go Blog -- Compile-time Dependency Injection With Go Cloud's Wire](https://go.dev/blog/wire)

Wire is a code generation tool that automates the composition root. It analyzes provider functions and generates explicit initialization code.

### Provider

A provider is any function that returns a value. Wire uses the function signature to determine what it provides and what it needs:

```go
func NewUserRepo(db *sql.DB) *UserRepo { ... }          // Provides *UserRepo, needs *sql.DB
func NewUserService(repo *UserRepo) *UserService { ... } // Provides *UserService, needs *UserRepo
```

### Injector

An injector is a function that Wire generates. You write the signature and a `wire.Build` call:

```go
//go:build wireinject

func InitializeUserService(db *sql.DB) *UserService {
    wire.Build(NewUserRepo, NewUserService)
    return nil // Wire generates the real body
}
```

Wire generates:

```go
func InitializeUserService(db *sql.DB) *UserService {
    userRepo := NewUserRepo(db)
    userService := NewUserService(userRepo)
    return userService
}
```

### Interface Bindings

When a provider returns a concrete type but a consumer needs an interface:

```go
var UserRepoSet = wire.NewSet(
    NewPostgresRepo,
    wire.Bind(new(UserRepository), new(*PostgresRepo)),
)
```

### When to Use Wire

| Criterion | Manual | Wire |
|-----------|--------|------|
| Dependency count | < 20 providers | 20+ providers |
| Team size | Small | Large (less context per person) |
| Safety | Compile-time | Compile-time (generated code) |
| Debugging | Read main() | Read wire_gen.go |
| Build dependency | None | `go install github.com/google/wire/cmd/wire` |

---

## Uber dig and fx

Source: [Uber dig](https://pkg.go.dev/go.uber.org/dig), [Uber fx](https://pkg.go.dev/go.uber.org/fx)

### dig (Container)

dig is a reflection-based DI container:

```go
container := dig.New()
container.Provide(NewPostgresRepo)
container.Provide(NewUserService)
container.Invoke(func(svc *UserService) {
    // Use svc
})
```

### fx (Application Framework)

fx builds on dig to manage application lifecycle:

```go
app := fx.New(
    fx.Provide(NewPostgresRepo),
    fx.Provide(NewUserService),
    fx.Invoke(RegisterHandlers),
)
app.Run()
```

### Tradeoffs vs Manual Wiring

| Aspect | Manual | dig/fx |
|--------|--------|--------|
| Error detection | Compile time | Runtime (app startup) |
| Debugging | Step through main() | Step through reflection |
| Learning curve | None | Moderate |
| Lifecycle management | Manual | Built-in (OnStart/OnStop hooks) |

---

## Key Standard Library DI Examples

| Package | Interface | Injected Via | What It Decouples |
|---------|-----------|-------------|-------------------|
| `io` | `Reader`, `Writer` | Function parameters | Data source/sink from processing |
| `net/http` | `Handler` | `Handle`/`HandleFunc` | Request handling from routing |
| `net/http` | `ResponseWriter` | Method parameter | Response writing from HTTP details |
| `log/slog` | `Handler` | `New(handler)` | Log formatting from log production |
| `database/sql` | `driver.Driver` | `Register` (service locator) | SQL API from database driver |
| `encoding/json` | `Marshaler`, `Unmarshaler` | Method dispatch | Serialization from type definition |
| `sort` | `Interface` | Function parameter | Sort algorithm from comparison logic |
| `context` | `Context` | First parameter (convention) | Request-scoped data from business logic |

---

## Common Patterns Quick Reference

### Required dependency (constructor parameter)
```go
func New(repo Repository) *Service { return &Service{repo: repo} }
```

### Optional dependency (functional option)
```go
func WithLogger(l *slog.Logger) Option { return func(s *Service) { s.log = l } }
```

### Default dependency (nil check in constructor)
```go
if logger == nil { logger = slog.Default() }
```

### Compile-time interface assertion
```go
var _ Repository = (*PostgresRepo)(nil)
```

### Test fake with error control
```go
type FakeRepo struct { Err error; data map[string]string }
```

### Closure-based functional DI
```go
type Fetcher func(ctx context.Context, url string) ([]byte, error)
```
