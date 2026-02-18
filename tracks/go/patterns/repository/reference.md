# Go Reference -- Repository Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), the [`database/sql`](https://pkg.go.dev/database/sql)
> package documentation, and established Go community patterns.
> Covers: interface design best practices, `database/sql` patterns, testing with
> interfaces, and context propagation in repository methods.

---

## Interface Design Best Practices

### Implicit Interface Satisfaction

Source: [Go Specification -- Interface types](https://go.dev/ref/spec#Interface_types)

A type `T` implements an interface `I` if `T` is an element of the type set of `I`. There is no explicit `implements` keyword. Satisfaction is checked structurally at compile time when a concrete type is assigned to an interface variable.

```go
// The interface is defined in the consumer package
type UserFinder interface {
    FindByEmail(ctx context.Context, email string) (*User, error)
}

// The implementation is in a separate package -- it does not import the consumer
type PgUserRepo struct{ db *sql.DB }

func (r *PgUserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
    // ...
}

// Satisfaction verified here, at the call site
var finder UserFinder = &PgUserRepo{db: db}
```

### Compile-Time Interface Guards

Use a blank variable declaration to verify at compile time that a type satisfies an interface:

```go
var _ UserFinder = (*PgUserRepo)(nil)
var _ UserFinder = (*MemoryUserStore)(nil)
```

This fails to compile if the type's method set does not include all methods required by the interface. Place these guards in the file that defines the implementation, near the type declaration.

### Interface Size Guidance

Source: [Go Proverbs](https://go-proverbs.github.io/) -- "The bigger the interface, the weaker the abstraction."

Source: [Effective Go -- Interfaces](https://go.dev/doc/effective_go#interfaces)

> Interfaces in Go provide a way to specify the behavior of an object: if something can do *this*, then it can be used *here*.

Standard library interfaces are famously small:

| Interface | Methods | Package |
|-----------|---------|---------|
| `io.Reader` | 1 | `io` |
| `io.Writer` | 1 | `io` |
| `io.Closer` | 1 | `io` |
| `fmt.Stringer` | 1 | `fmt` |
| `error` | 1 | builtin |
| `sort.Interface` | 3 | `sort` |
| `http.Handler` | 1 | `net/http` |
| `http.ResponseWriter` | 3 | `net/http` |

Repository interfaces should follow this philosophy. One to four methods per interface is typical for consumer-facing abstractions.

### Accept Interfaces, Return Structs

Source: Community convention, popularized by [Jack Lindamood](https://medium.com/@cep21/what-accept-interfaces-return-structs-means-in-go-2fe879e25ee8)

Functions should accept interface parameters (for flexibility) and return concrete struct types (for clarity). Applied to repositories:

```go
// Constructor accepts nothing about storage -- it's injected
func NewAuthService(users UserFinder, tokens TokenStore) *AuthService {
    return &AuthService{users: users, tokens: tokens}
}

// Factory returns a concrete type, not an interface
func NewPgUserRepo(db *sql.DB) *PgUserRepo {
    return &PgUserRepo{db: db}
}
```

### Interface Embedding and Composition

Source: [Go Specification -- Interface types](https://go.dev/ref/spec#Interface_types)

Interfaces can embed other interfaces to compose larger contracts from smaller ones:

```go
type Reader interface {
    FindByID(ctx context.Context, id string) (*Entity, error)
    List(ctx context.Context, opts ListOptions) ([]*Entity, error)
}

type Writer interface {
    Save(ctx context.Context, entity *Entity) error
    Delete(ctx context.Context, id string) error
}

// ReadWriter composes both -- only used by consumers that need full access
type ReadWriter interface {
    Reader
    Writer
}
```

This allows consumers to depend on the smallest interface they need.

---

## `database/sql` Package Patterns

Source: [Package sql](https://pkg.go.dev/database/sql)

### Opening a Connection Pool

```go
db, err := sql.Open("postgres", connStr)
if err != nil {
    log.Fatal(err)
}
defer db.Close()

// Configure the pool
db.SetMaxOpenConns(25)
db.SetMaxIdleConns(5)
db.SetConnMaxLifetime(5 * time.Minute)

// Verify connectivity
if err := db.PingContext(ctx); err != nil {
    log.Fatal(err)
}
```

`sql.Open` does not establish a connection. It validates the driver and DSN. Actual connections are created lazily on first use. Always call `PingContext` to verify.

### QueryRowContext -- Single Row

```go
var user User
err := db.QueryRowContext(ctx,
    `SELECT id, email, name FROM users WHERE id = $1`, id,
).Scan(&user.ID, &user.Email, &user.Name)

switch {
case errors.Is(err, sql.ErrNoRows):
    return nil, ErrNotFound
case err != nil:
    return nil, fmt.Errorf("query user by id: %w", err)
}
```

`QueryRowContext` returns exactly one row. If no rows match, `Scan` returns `sql.ErrNoRows`. This is the primary mechanism for "find by ID" or "find by unique key" repository methods.

### QueryContext -- Multiple Rows

```go
rows, err := db.QueryContext(ctx,
    `SELECT id, email, name FROM users WHERE role = $1 ORDER BY name LIMIT $2`,
    role, limit,
)
if err != nil {
    return nil, fmt.Errorf("query users by role: %w", err)
}
defer rows.Close()

var users []*User
for rows.Next() {
    var u User
    if err := rows.Scan(&u.ID, &u.Email, &u.Name); err != nil {
        return nil, fmt.Errorf("scanning user row: %w", err)
    }
    users = append(users, &u)
}
if err := rows.Err(); err != nil {
    return nil, fmt.Errorf("iterating user rows: %w", err)
}
return users, nil
```

**Important:** Always check `rows.Err()` after the loop. An error during iteration (network failure, context cancellation) will cause `rows.Next()` to return `false`, but the error is only available via `rows.Err()`.

**Important:** Always `defer rows.Close()`. Failing to close rows leaks the underlying database connection back to the pool.

### ExecContext -- Insert, Update, Delete

```go
result, err := db.ExecContext(ctx,
    `UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2`,
    name, id,
)
if err != nil {
    return fmt.Errorf("updating user: %w", err)
}

rowsAffected, err := result.RowsAffected()
if err != nil {
    return fmt.Errorf("checking rows affected: %w", err)
}
if rowsAffected == 0 {
    return ErrNotFound
}
```

`ExecContext` is used for statements that don't return rows. Check `RowsAffected()` for update/delete operations to determine if the target entity existed.

### Transactions

```go
tx, err := db.BeginTx(ctx, nil)
if err != nil {
    return fmt.Errorf("begin transaction: %w", err)
}
defer tx.Rollback() // safe to call after Commit -- returns sql.ErrTxDone

// Use tx instead of db for all operations within the transaction
_, err = tx.ExecContext(ctx, `UPDATE accounts SET balance = balance - $1 WHERE id = $2`, amount, fromID)
if err != nil {
    return fmt.Errorf("debit: %w", err)
}

_, err = tx.ExecContext(ctx, `UPDATE accounts SET balance = balance + $1 WHERE id = $2`, amount, toID)
if err != nil {
    return fmt.Errorf("credit: %w", err)
}

return tx.Commit()
```

**Pattern:** `defer tx.Rollback()` is safe even after a successful commit. After `Commit()`, `Rollback()` returns `sql.ErrTxDone` (which is ignored). This ensures the transaction is always cleaned up, even on panic.

### Prepared Statements

```go
stmt, err := db.PrepareContext(ctx,
    `SELECT id, email FROM users WHERE email = $1`)
if err != nil {
    return err
}
defer stmt.Close()

// Reuse for multiple queries
for _, email := range emails {
    var u User
    err := stmt.QueryRowContext(ctx, email).Scan(&u.ID, &u.Email)
    // ...
}
```

Prepared statements are useful when executing the same query many times with different parameters. For single-execution queries, `QueryRowContext`/`QueryContext` with inline SQL is simpler and has negligible performance difference due to server-side statement caching in modern databases.

### Placeholder Syntax by Driver

| Driver | Placeholder | Example |
|--------|-------------|---------|
| PostgreSQL (`lib/pq`, `pgx`) | `$1, $2, $3` | `WHERE id = $1 AND status = $2` |
| MySQL (`go-sql-driver/mysql`) | `?` | `WHERE id = ? AND status = ?` |
| SQLite (`mattn/go-sqlite3`) | `?` or `$1` | Both are supported |

Always use parameterized queries. Never concatenate user input into SQL strings.

---

## Sentinel Errors for Repositories

Define domain-specific errors that repository implementations translate into:

```go
package domain

import "errors"

var (
    // ErrNotFound indicates the requested entity does not exist.
    ErrNotFound = errors.New("not found")

    // ErrConflict indicates a unique constraint violation (duplicate entity).
    ErrConflict = errors.New("conflict: entity already exists")

    // ErrStaleData indicates the entity was modified by another process
    // since it was last read (optimistic locking failure).
    ErrStaleData = errors.New("stale data: entity was modified concurrently")
)
```

Usage in implementations:

```go
func (r *PgUserRepo) Save(ctx context.Context, user *User) error {
    _, err := r.db.ExecContext(ctx, insertSQL, user.Email, user.Name)
    if err != nil {
        var pgErr *pgconn.PgError
        if errors.As(err, &pgErr) && pgErr.Code == "23505" {
            return ErrConflict  // unique_violation
        }
        return fmt.Errorf("saving user: %w", err)
    }
    return nil
}
```

Usage in business logic:

```go
func (s *Service) Register(ctx context.Context, email, name string) error {
    user := &User{Email: email, Name: name}
    err := s.store.Save(ctx, user)
    if errors.Is(err, ErrConflict) {
        return fmt.Errorf("email %s is already registered", email)
    }
    return err
}
```

---

## Context Propagation

Source: [Package context](https://pkg.go.dev/context)

### Why Every Repository Method Needs `context.Context`

1. **Timeouts**: `context.WithTimeout` limits how long a database query can run.
2. **Cancellation**: If the upstream HTTP request is cancelled, the query should stop.
3. **Distributed tracing**: OpenTelemetry span context propagates via `context.Context`.
4. **Request-scoped values**: User identity, request ID, and other metadata.

```go
// Setting a timeout at the service layer
func (s *Service) GetUser(ctx context.Context, id string) (*User, error) {
    ctx, cancel := context.WithTimeout(ctx, 3*time.Second)
    defer cancel()
    return s.store.FindByID(ctx, id)
}

// The repository implementation passes ctx to the database driver
func (r *PgUserRepo) FindByID(ctx context.Context, id string) (*User, error) {
    // If the 3-second timeout expires, this query is cancelled
    return r.db.QueryRowContext(ctx, `SELECT ... WHERE id = $1`, id)
}
```

### Context-Aware Variants

All `database/sql` methods have `Context` variants. Always use them in repository implementations:

| Without Context | With Context (Use This) |
|----------------|------------------------|
| `db.Query(...)` | `db.QueryContext(ctx, ...)` |
| `db.QueryRow(...)` | `db.QueryRowContext(ctx, ...)` |
| `db.Exec(...)` | `db.ExecContext(ctx, ...)` |
| `db.Prepare(...)` | `db.PrepareContext(ctx, ...)` |
| `db.Begin()` | `db.BeginTx(ctx, nil)` |
| `tx.Commit()` | `tx.Commit()` (no context variant) |
| `tx.Rollback()` | `tx.Rollback()` (no context variant) |

---

## Testing with Interfaces

### In-Memory Fakes vs Generated Mocks

**Fakes** implement the interface with real (simplified) behavior:

```go
type MemoryUserStore struct {
    mu    sync.RWMutex
    users map[string]*User
}

func (m *MemoryUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    m.mu.RLock()
    defer m.mu.RUnlock()
    for _, u := range m.users {
        if u.Email == email {
            copy := *u
            return &copy, nil
        }
    }
    return nil, ErrNotFound
}
```

**Mocks** record calls and return predetermined values:

```go
type MockUserStore struct {
    FindByEmailFunc func(ctx context.Context, email string) (*User, error)
}

func (m *MockUserStore) FindByEmail(ctx context.Context, email string) (*User, error) {
    return m.FindByEmailFunc(ctx, email)
}
```

### When to Use Each

| Approach | Strengths | Weaknesses |
|----------|-----------|------------|
| **Fake** | Tests real behavior; resilient to refactoring; reusable across tests | More code to write; must maintain consistency |
| **Mock** | Quick to set up; can test error paths easily | Brittle; tests implementation, not behavior; breaks on refactor |

**Recommendation:** Use fakes for repository interfaces. Use mocks sparingly for external services where you need to simulate specific error conditions.

### Table-Driven Tests with Repository Fakes

```go
func TestAuthService_Login(t *testing.T) {
    tests := []struct {
        name      string
        email     string
        password  string
        setup     func(*MemoryUserStore)
        wantErr   error
    }{
        {
            name:     "valid credentials",
            email:    "alice@example.com",
            password: "correct-password",
            setup: func(store *MemoryUserStore) {
                store.Save(ctx, &User{
                    Email:        "alice@example.com",
                    PasswordHash: hashPassword("correct-password"),
                })
            },
            wantErr: nil,
        },
        {
            name:     "user not found",
            email:    "nobody@example.com",
            password: "any",
            setup:    func(store *MemoryUserStore) {},
            wantErr:  ErrInvalidCredentials,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            store := NewMemoryUserStore()
            tt.setup(store)

            svc := NewAuthService(store)
            _, err := svc.Login(ctx, tt.email, tt.password)

            if !errors.Is(err, tt.wantErr) {
                t.Errorf("Login() error = %v, want %v", err, tt.wantErr)
            }
        })
    }
}
```

---

## References

- [Go Specification -- Interface types](https://go.dev/ref/spec#Interface_types)
- [Effective Go -- Interfaces](https://go.dev/doc/effective_go#interfaces)
- [Go Proverbs](https://go-proverbs.github.io/)
- [Package sql](https://pkg.go.dev/database/sql)
- [Package context](https://pkg.go.dev/context)
- [Go wiki: SQL Database](https://go.dev/wiki/SQLInterface)
- [Go wiki: Table Driven Tests](https://go.dev/wiki/TableDrivenTests)
- [Accept Interfaces, Return Structs (Jack Lindamood)](https://medium.com/@cep21/what-accept-interfaces-return-structs-means-in-go-2fe879e25ee8)
