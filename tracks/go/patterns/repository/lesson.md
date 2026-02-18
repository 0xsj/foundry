# Repository Pattern -- Go

## The Problem Repository Solves

Your business logic needs to read and write data. Maybe it is users in a Postgres database today. Maybe it is events in DynamoDB tomorrow. Maybe it is a flat JSON file during development and testing. The naive approach is calling `db.Query` directly from your service layer. This works until it doesn't -- your service functions are littered with SQL strings, you can't test business logic without a running database, migrating to a different storage backend requires rewriting every function, and you can't add a caching layer without touching dozens of call sites.

The Repository pattern puts a wall between your business logic and your storage implementation. You define an interface that describes *what* your business logic needs (find a user by email, list active jobs, save an event), and then provide implementations that satisfy that interface for each storage backend. The business logic talks to the interface. It never knows -- and never cares -- whether data comes from Postgres, Redis, an in-memory map, or a mock in a test.

This is not an academic exercise. Every non-trivial Go codebase that talks to a database uses some form of this pattern. The standard library itself uses it -- `io.Reader` is a repository for bytes. `http.Handler` is a repository for request handling logic. The idea is the same: define a contract, let implementations vary.

### Coming from TypeScript

If you've used TypeORM, Prisma, or Drizzle, you've encountered repository-like abstractions:

```typescript
// TypeORM: the repository is built into the ORM
const userRepo = dataSource.getRepository(User);
const user = await userRepo.findOneBy({ email: "alice@example.com" });

// Prisma: the client itself acts as a repository
const user = await prisma.user.findUnique({ where: { email: "alice@example.com" } });
```

The key difference in Go: there is no ORM handing you a repository. You define the interface yourself, in the package that *uses* it, and you write the implementations. This gives you more control and clarity at the cost of more boilerplate. Go developers generally consider this a good trade -- you see exactly what SQL runs, you control error handling explicitly, and your interfaces are exactly the size your business logic needs (not a giant generic CRUD surface).

### Real-world situations where Repository appears

- **User management service**: CRUD operations on users, lookup by email, list with filters
- **Job queue**: Create jobs, claim next available, mark complete/failed, retry with backoff
- **Audit log store**: Append-only log of events, query by time range and entity
- **Feature flag service**: Read flags by key, toggle flags, list all flags for an environment
- **Configuration store**: Read config values, watch for changes, batch updates
- **Notification preferences**: Get user preferences, update channels, list unsubscribed
- **Session store**: Create session, look up by token, expire old sessions

Every one of these follows the same shape: a set of data operations abstracted behind an interface, with implementations that can be swapped for testing, migration, or different deployment environments.

### Your notes
<!-- User adds insights here during learning -->


---

## Interface Design: The Go Way

### Accept Interfaces, Return Structs

The single most important Go idiom for Repository is: **define the interface where it is consumed, not where it is implemented.**

In Java or C#, you define `IUserRepository` in a shared package, then have `PostgresUserRepository implements IUserRepository` in the data layer and the service imports both. In Go, the consumer owns the interface:

```go
// In package userservice -- defines what it needs
type UserStore interface {
    FindByEmail(ctx context.Context, email string) (*User, error)
    Save(ctx context.Context, user *User) error
}

type Service struct {
    store UserStore  // depends on interface, not concrete type
}

// In package postgres -- knows nothing about userservice.UserStore
type UserRepo struct {
    db *sql.DB
}

func (r *UserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
    // SQL query here
}

func (r *UserRepo) Save(ctx context.Context, user *User) error {
    // SQL insert/update here
}
```

The `postgres.UserRepo` never imports `userservice`. It doesn't know the `UserStore` interface exists. The Go compiler verifies the contract when you wire things together in `main`:

```go
func main() {
    db := connectDB()
    repo := &postgres.UserRepo{DB: db}
    svc := userservice.NewService(repo)  // compiler checks *UserRepo satisfies UserStore
}
```

This is fundamentally different from how Java/C#/TypeScript do it. The consequence: your interfaces are **small and specific**. A service that only reads users doesn't need a `CreateUser` method on its interface. A service that only creates users doesn't need `FindByEmail`. Each consumer defines exactly the surface it needs.

### Keep Interfaces Small

The Go proverb says: "The bigger the interface, the weaker the abstraction." A 10-method repository interface is a sign that either:

1. Your service is doing too much (break it up)
2. You're designing the interface from the implementation side (flip it)
3. You're copying a Java pattern (stop)

Compare:

```go
// Bad: God interface -- designed from the implementation side
type UserRepository interface {
    Create(ctx context.Context, user *User) error
    GetByID(ctx context.Context, id string) (*User, error)
    GetByEmail(ctx context.Context, email string) (*User, error)
    Update(ctx context.Context, user *User) error
    Delete(ctx context.Context, id string) error
    List(ctx context.Context, opts ListOpts) ([]*User, error)
    Count(ctx context.Context, filter Filter) (int, error)
    Search(ctx context.Context, query string) ([]*User, error)
    UpdatePassword(ctx context.Context, id string, hash []byte) error
    SetRole(ctx context.Context, id string, role Role) error
}
```

```go
// Good: Small interfaces -- designed from the consumer side
type UserFinder interface {
    FindByEmail(ctx context.Context, email string) (*User, error)
}

type UserWriter interface {
    Save(ctx context.Context, user *User) error
}

// Your auth service only needs to find users
type AuthService struct {
    users UserFinder
}

// Your admin service needs both
type AdminService struct {
    finder UserFinder
    writer UserWriter
}
```

A single concrete type (`postgres.UserRepo`) can satisfy all of these interfaces simultaneously. Each consumer only sees what it needs.

### Always Accept `context.Context`

Every repository method should take `context.Context` as its first parameter. This is non-negotiable in production Go:

```go
type JobStore interface {
    Create(ctx context.Context, job *Job) error
    Claim(ctx context.Context) (*Job, error)
    Complete(ctx context.Context, id string, result []byte) error
}
```

Why:
- **Timeouts**: Database queries need deadlines. `ctx, cancel := context.WithTimeout(ctx, 5*time.Second)` propagates through the repository to the actual query.
- **Cancellation**: If the HTTP request is cancelled, the database query should stop too.
- **Tracing**: Distributed tracing (OpenTelemetry) propagates span context through `context.Context`.
- **Values**: Request-scoped values (user ID, trace ID) are available inside the implementation.

If you skip `context.Context` in the interface, you'll add it later and break every implementation and every call site. Put it in from the start.

### Your notes
<!-- User adds insights here during learning -->


---

## In-Memory Implementation: The Testing Workhorse

The most important repository implementation is not Postgres or MySQL. It is the **in-memory implementation** that runs in your tests.

An in-memory repository stores data in Go maps and slices. It has no external dependencies, starts instantly, needs no cleanup, and runs fast enough to execute thousands of tests per second. This is the whole point of the repository pattern -- your business logic tests never touch a database.

```go
// MemoryUserStore is an in-memory implementation for testing.
type MemoryUserStore struct {
    mu    sync.RWMutex
    users map[string]*User  // indexed by ID
    nextID int
}

func NewMemoryUserStore() *MemoryUserStore {
    return &MemoryUserStore{
        users: make(map[string]*User),
    }
}

func (m *MemoryUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    m.mu.RLock()
    defer m.mu.RUnlock()

    for _, u := range m.users {
        if u.Email == email {
            // Return a copy -- caller should not mutate our internal state
            copy := *u
            return &copy, nil
        }
    }
    return nil, ErrNotFound
}

func (m *MemoryUserStore) Save(ctx context.Context, user *User) error {
    m.mu.Lock()
    defer m.mu.Unlock()

    if user.ID == "" {
        m.nextID++
        user.ID = fmt.Sprintf("user-%d", m.nextID)
    }

    // Store a copy -- we don't want the caller's later mutations to affect our state
    copy := *user
    m.users[copy.ID] = &copy
    return nil
}
```

### Critical detail: always copy

Notice the copies in both `FindByEmail` and `Save`. This is essential. If you return a pointer to your internal map value, the caller can mutate your repository's state without going through `Save`. If you store the caller's pointer directly, the caller can change the object after saving it. Both break the repository's integrity.

This mirrors what a real database does naturally -- when you read from Postgres, you get a *copy* of the data, not a pointer into the database's memory. Your in-memory implementation should behave the same way.

### Error conventions

Define sentinel errors for your repository:

```go
var (
    ErrNotFound    = errors.New("entity not found")
    ErrConflict    = errors.New("entity already exists")
    ErrStaleData   = errors.New("data has been modified by another process")
)
```

These are **domain errors**, not database errors. The repository translates database-specific errors (`sql.ErrNoRows`, unique constraint violations) into these domain errors. The business logic never sees `pq.Error` or `sqlite3.Error`.

### Your notes
<!-- User adds insights here during learning -->


---

## SQL-Backed Implementation

The production implementation typically wraps `database/sql`. Here's the same `UserStore` interface backed by Postgres:

```go
type PostgresUserRepo struct {
    db *sql.DB
}

func NewPostgresUserRepo(db *sql.DB) *PostgresUserRepo {
    return &PostgresUserRepo{db: db}
}

func (r *PostgresUserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
    var u User
    err := r.db.QueryRowContext(ctx,
        `SELECT id, email, name, role, created_at, updated_at
         FROM users
         WHERE email = $1`,
        email,
    ).Scan(&u.ID, &u.Email, &u.Name, &u.Role, &u.CreatedAt, &u.UpdatedAt)

    if errors.Is(err, sql.ErrNoRows) {
        return nil, ErrNotFound  // translate to domain error
    }
    if err != nil {
        return nil, fmt.Errorf("finding user by email: %w", err)
    }
    return &u, nil
}

func (r *PostgresUserRepo) Save(ctx context.Context, user *User) error {
    if user.ID == "" {
        // Insert
        err := r.db.QueryRowContext(ctx,
            `INSERT INTO users (email, name, role, created_at, updated_at)
             VALUES ($1, $2, $3, NOW(), NOW())
             RETURNING id`,
            user.Email, user.Name, user.Role,
        ).Scan(&user.ID)
        if err != nil {
            if isUniqueViolation(err) {
                return ErrConflict
            }
            return fmt.Errorf("inserting user: %w", err)
        }
        return nil
    }

    // Update
    result, err := r.db.ExecContext(ctx,
        `UPDATE users SET name = $1, role = $2, updated_at = NOW()
         WHERE id = $3`,
        user.Name, user.Role, user.ID,
    )
    if err != nil {
        return fmt.Errorf("updating user: %w", err)
    }

    rows, _ := result.RowsAffected()
    if rows == 0 {
        return ErrNotFound
    }
    return nil
}
```

### Key observations

**Error translation**: The Postgres implementation converts `sql.ErrNoRows` to `ErrNotFound` and unique constraint violations to `ErrConflict`. The business logic checks `errors.Is(err, ErrNotFound)` and never needs to import any database driver package.

**Context propagation**: Every database call uses the `Context` variants (`QueryRowContext`, `ExecContext`). This ensures timeouts and cancellation propagate to the actual query.

**SQL is explicit**: Unlike an ORM, every SQL query is visible. You see exactly what columns are fetched, what indexes are used, and what joins happen. This is considered a feature in Go culture, not a limitation.

**Returning IDs**: On insert, the Postgres implementation uses `RETURNING id` to populate the user's ID. The caller passes a user without an ID, and the repository fills it in. This is a common pattern.

### Your notes
<!-- User adds insights here during learning -->


---

## The Decorator Pattern: Repository Wrappers

Because the repository is an interface, you can wrap it with cross-cutting concerns that are transparent to the business logic:

### Caching Decorator

```go
type CachingUserStore struct {
    delegate UserStore
    cache    map[string]*User  // email -> user
    mu       sync.RWMutex
    ttl      time.Duration
}

func (c *CachingUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    c.mu.RLock()
    if u, ok := c.cache[email]; ok {
        c.mu.RUnlock()
        copy := *u
        return &copy, nil
    }
    c.mu.RUnlock()

    // Cache miss -- delegate to real store
    u, err := c.delegate.FindByEmail(ctx, email)
    if err != nil {
        return nil, err
    }

    c.mu.Lock()
    copy := *u
    c.cache[email] = &copy
    c.mu.Unlock()

    return u, nil
}
```

### Logging Decorator

```go
type LoggingUserStore struct {
    delegate UserStore
    logger   *slog.Logger
}

func (l *LoggingUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    start := time.Now()
    u, err := l.delegate.FindByEmail(ctx, email)
    l.logger.Info("FindByEmail",
        "email", email,
        "found", u != nil,
        "error", err,
        "duration", time.Since(start),
    )
    return u, err
}
```

### Metrics Decorator

```go
type MetricsUserStore struct {
    delegate  UserStore
    histogram *prometheus.HistogramVec
}

func (m *MetricsUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    start := time.Now()
    u, err := m.delegate.FindByEmail(ctx, email)
    status := "success"
    if err != nil {
        status = "error"
    }
    m.histogram.WithLabelValues("FindByEmail", status).Observe(time.Since(start).Seconds())
    return u, err
}
```

These decorators compose. In production, you wire them like this:

```go
func main() {
    db := connectDB()
    base := postgres.NewUserRepo(db)
    cached := cache.NewCachingUserStore(base, 5*time.Minute)
    logged := logging.NewLoggingUserStore(cached, logger)
    measured := metrics.NewMetricsUserStore(logged, histogram)

    svc := userservice.NewService(measured)
}
```

The service doesn't know it is talking through four layers of wrappers. Each layer does one thing. This is the Open/Closed Principle in action -- you extend behavior without modifying existing code.

### Your notes
<!-- User adds insights here during learning -->


---

## Unit of Work: Transactional Boundaries

The Repository pattern handles single-entity operations well. But what about operations that span multiple entities and must succeed or fail together? That is the **Unit of Work** pattern.

In Go, this typically manifests as a function that receives a transaction:

```go
// TxFunc defines a function that executes within a transaction
type TxFunc func(ctx context.Context, tx *sql.Tx) error

// WithTransaction executes fn within a database transaction
func WithTransaction(ctx context.Context, db *sql.DB, fn TxFunc) error {
    tx, err := db.BeginTx(ctx, nil)
    if err != nil {
        return fmt.Errorf("begin transaction: %w", err)
    }
    defer tx.Rollback()  // no-op if already committed

    if err := fn(ctx, tx); err != nil {
        return err  // rollback happens via defer
    }

    return tx.Commit()
}
```

Usage:

```go
func (s *TransferService) Transfer(ctx context.Context, fromID, toID string, amount int) error {
    return WithTransaction(ctx, s.db, func(ctx context.Context, tx *sql.Tx) error {
        // Both operations use the same transaction
        if err := s.accounts.WithTx(tx).Debit(ctx, fromID, amount); err != nil {
            return err
        }
        if err := s.accounts.WithTx(tx).Credit(ctx, toID, amount); err != nil {
            return err
        }
        return nil
    })
}
```

The repository needs a `WithTx` method that returns a copy of itself bound to the transaction. This keeps the repository interface clean while supporting transactional boundaries.

> **Common Pitfall:** See [[go-deferred-rollback]] for why `defer tx.Rollback()` is safe even after a successful commit.

### Your notes
<!-- User adds insights here during learning -->


---

## Repository vs DAO vs Active Record

These terms get confused constantly. Here is the actual difference:

| Pattern | Who owns the data logic? | Interface shape | Example |
|---------|--------------------------|-----------------|---------|
| **Repository** | Collection-like abstraction. Business logic thinks it is working with a collection of domain objects. | `Find(id) -> Entity`, `Save(entity)`, `FindByX(criteria)` | Go: interfaces in the consumer package |
| **DAO** | Data access abstraction. Closer to the database -- methods map to tables and queries. | `SelectByID(table, id)`, `Insert(table, row)`, `RunQuery(sql)` | Java JDBC wrappers |
| **Active Record** | Entity owns its own persistence. Each model instance can save itself. | `user.Save()`, `User.FindByEmail(email)` | Rails ActiveRecord, Django ORM |

In Go, the Repository pattern dominates because:
- Active Record requires code generation or reflection to attach persistence methods to every model (Go avoids this)
- DAO leaks the database abstraction (Go prefers domain-oriented interfaces)
- Repository maps naturally to Go interfaces (small, consumer-defined, implicit satisfaction)

### Standard library parallel: `io.Reader`

`io.Reader` is arguably the most successful repository interface ever designed:

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

It is a "repository for bytes." The consumer (`json.Decoder`, `bufio.Scanner`, `io.Copy`) doesn't know whether bytes come from a file, a network connection, a string, or a mock. The pattern is identical to what we do with data repositories -- just applied to byte streams.

### Your notes
<!-- User adds insights here during learning -->


---

## Pagination, Filtering, and Sorting

For list operations, the repository interface needs to support queries beyond "give me everything." There are two clean approaches:

### Option struct approach

```go
type ListOptions struct {
    Cursor    string        // opaque cursor for pagination (preferred over offset)
    Limit     int           // max items to return
    SortBy    string        // field name to sort by
    SortOrder string        // "asc" or "desc"
    Filters   []Filter      // field-level filters
}

type Filter struct {
    Field    string
    Operator string  // "eq", "gt", "lt", "contains", "in"
    Value    interface{}
}

type ListResult[T any] struct {
    Items      []T
    NextCursor string  // empty if no more pages
    Total      int     // total matching items (optional, expensive)
}

type UserStore interface {
    List(ctx context.Context, opts ListOptions) (*ListResult[User], error)
}
```

### Functional options approach

```go
type QueryOption func(*queryConfig)

func WithLimit(n int) QueryOption {
    return func(c *queryConfig) { c.limit = n }
}

func WithCursor(cursor string) QueryOption {
    return func(c *queryConfig) { c.cursor = cursor }
}

func WithFilter(field, op string, value interface{}) QueryOption {
    return func(c *queryConfig) {
        c.filters = append(c.filters, filter{field, op, value})
    }
}

type UserStore interface {
    List(ctx context.Context, opts ...QueryOption) (*ListResult[User], error)
}

// Usage
users, err := store.List(ctx, WithLimit(25), WithCursor(lastCursor), WithFilter("role", "eq", "admin"))
```

### Cursor-based vs offset-based pagination

- **Offset (LIMIT/OFFSET)**: Simple but breaks on large datasets. Inserting/deleting rows between pages causes items to be skipped or duplicated. Performance degrades linearly with offset size because the database must scan and discard all offset rows.
- **Cursor-based (keyset pagination)**: Uses the last item's sort key as a cursor. Consistent results regardless of mutations. Constant performance regardless of page depth. The trade-off: you can't jump to page 47 directly; you must traverse sequentially.

For production systems, cursor-based pagination is almost always the right choice.

### Your notes
<!-- User adds insights here during learning -->


---

## Anti-Patterns

### Leaking SQL into the interface

```go
// Bad: The interface exposes SQL concepts
type UserRepo interface {
    Query(ctx context.Context, sql string, args ...interface{}) ([]*User, error)
    ExecSQL(ctx context.Context, sql string, args ...interface{}) error
}
```

This defeats the entire purpose. The interface should describe *what* the business logic needs, not *how* the data is stored. If your interface has "SQL", "query", or "table" in the method names, you're designing from the wrong end.

### Returning ORM/database models

```go
// Bad: The interface returns sqlx-specific types
type UserRepo interface {
    FindByEmail(ctx context.Context, email string) (*sqlx.Row, error)
}
```

The repository should return domain types (`*User`), not database types. The translation from database rows to domain objects happens *inside* the implementation.

### Single interface for all consumers

```go
// Bad: God interface that every consumer must depend on
type UserRepository interface {
    Create, GetByID, GetByEmail, Update, Delete, List, Count, Search,
    UpdatePassword, SetRole, BulkCreate, Archive, Restore, GetLoginHistory,
    GetAuditTrail, GetActiveSessionCount, ...
}
```

No consumer needs all of these. The auth service needs `FindByEmail`. The admin dashboard needs `List` and `Count`. The password reset flow needs `FindByEmail` and `UpdatePassword`. Define interfaces per consumer.

### Testing with mocks instead of fakes

```go
// Fragile: Mock that asserts exact call sequence
mock.ExpectFindByEmail("alice@example.com").Return(&User{...}, nil)
mock.ExpectSave(expectedUser).Return(nil)

// Better: Fake with real behavior
store := NewMemoryUserStore()
store.Save(ctx, &User{Email: "alice@example.com", Name: "Alice"})
// Now test business logic against a real (in-memory) store
```

Mocks verify *how* the code interacts with the repository. Fakes verify *what* the code produces. Fakes are more resilient to refactoring because they don't break when you reorder method calls or add an extra read.

> **Go proverb:** "Don't mock what you don't own." For repositories, prefer in-memory fakes over generated mocks.

### Your notes
<!-- User adds insights here during learning -->


---

## Cross-Language Comparison

### TypeScript (TypeORM / Prisma)

```typescript
// TypeORM: generic repository pattern
const userRepo = dataSource.getRepository(User);
const user = await userRepo.findOneBy({ email });
await userRepo.save(user);

// Custom repository (class-based)
class UserRepository extends Repository<User> {
    findByEmail(email: string): Promise<User | null> {
        return this.findOneBy({ email });
    }
}
```

TypeScript ORMs typically give you a generic repository for free. The trade-off: you get a huge surface area (find, findOne, save, remove, count, createQueryBuilder...) whether you need it or not. Go's approach of defining only what you need results in smaller, more testable surfaces.

### Rust (diesel / sqlx)

```rust
// Rust: trait-based repository (similar to Go interfaces)
trait UserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;
    async fn save(&self, user: &User) -> Result<(), RepoError>;
}

// Implementation
struct PgUserRepo {
    pool: PgPool,
}

impl UserRepository for PgUserRepo {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepoError::from)
    }
}
```

Rust's trait system is explicit (like Java -- you declare `impl Trait for Type`), while Go's is implicit. Rust's `Result<Option<T>, E>` makes the "not found" case explicit in the type system, while Go uses a sentinel error. Both are valid -- Rust pushes more into the type system, Go pushes more into runtime checks with convention.

### Python (SQLAlchemy)

```python
# Python: typically uses the Active Record pattern
class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    email = Column(String, unique=True)

# Usage
user = session.query(User).filter_by(email=email).first()
session.add(user)
session.commit()
```

Python's SQLAlchemy blends Active Record and Data Mapper patterns. The session object acts as a Unit of Work. There is no explicit repository interface -- the ORM's query API is used directly. This is more concise but couples business logic to SQLAlchemy.

### Your notes
<!-- User adds insights here during learning -->


---

## Testing with Repository Interfaces

The repository pattern's payoff is clearest in testing. Compare:

### Without Repository (tightly coupled)

```go
func TestCreateOrder_InsufficientStock(t *testing.T) {
    // Need a real database running
    db := setupTestDB(t)       // slow, flaky, requires Docker
    defer teardownTestDB(t, db)
    seedProducts(t, db)        // complex setup

    svc := NewOrderService(db)
    err := svc.CreateOrder(ctx, "user-1", "product-1", 1000)

    if !errors.Is(err, ErrInsufficientStock) {
        t.Errorf("expected ErrInsufficientStock, got %v", err)
    }
}
```

### With Repository (decoupled)

```go
func TestCreateOrder_InsufficientStock(t *testing.T) {
    // Pure in-memory -- instant, deterministic, no infrastructure
    products := NewMemoryProductStore()
    products.Save(ctx, &Product{ID: "product-1", Stock: 5})

    orders := NewMemoryOrderStore()
    svc := NewOrderService(products, orders)

    err := svc.CreateOrder(ctx, "user-1", "product-1", 1000)

    if !errors.Is(err, ErrInsufficientStock) {
        t.Errorf("expected ErrInsufficientStock, got %v", err)
    }
}
```

The second test runs in microseconds, needs no Docker, has no flaky network calls, and clearly shows exactly what state is being tested. This is the reason people adopt the Repository pattern.

### Integration tests still matter

In-memory fakes test your business logic. You still need integration tests that verify your SQL implementation actually works against a real database. The difference: you need *fewer* integration tests (only the repository implementation itself), and your business logic tests are fast and reliable.

```
Unit tests (fast, many):       BusinessLogic <-> MemoryRepo
Integration tests (slow, few): PostgresRepo <-> Real Database
```

### Your notes
<!-- User adds insights here during learning -->


---

## Summary: When to Use Repository

**Use Repository when:**
- Business logic is complex enough to warrant separation from data access
- You need to test business logic without a database
- Multiple storage backends are possible (now or in the future)
- You want to add cross-cutting concerns (caching, logging, metrics) transparently
- Different services need different views of the same data (small interfaces per consumer)

**Skip Repository when:**
- The application is a thin CRUD wrapper with no business logic
- There is exactly one storage backend and no plans to change
- The added indirection isn't justified by the complexity

**Key Go idioms to remember:**
1. Define interfaces in the consumer package, not the implementation package
2. Keep interfaces small -- one to three methods is ideal
3. Always accept `context.Context` as the first parameter
4. Return domain types, not database types
5. Use sentinel errors (`ErrNotFound`, `ErrConflict`) instead of database-specific errors
6. Copy data in and out of in-memory implementations
7. Prefer fakes over mocks for testing
8. Use decorators for cross-cutting concerns

### Your notes
<!-- User adds insights here during learning -->


---

## Related Concepts

- [[strategy]] -- Repository is essentially strategy pattern applied to data access
- [[dependency-injection]] -- Repositories are injected into services via constructors
- [[decorator]] -- Caching, logging, metrics layers wrap repositories
- [[unit-of-work]] -- Transactional boundaries across multiple repositories
- [[io-reader]] -- `io.Reader` as the prototypical Go "data repository" interface
- [[testing-fundamentals]] -- In-memory fakes enable fast, isolated unit tests
