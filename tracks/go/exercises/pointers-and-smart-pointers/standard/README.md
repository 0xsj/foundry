# Exercise: Connection Pool Manager

## Scenario

Your team operates a service that maintains a pool of reusable database connections. Connections are expensive to create — each one opens a TCP socket, completes a TLS handshake, and authenticates. The pool keeps a fixed number of connections ready so that handlers can borrow one, use it, and return it without paying the creation cost on every request.

You've been asked to implement the pool manager. It tracks which connections are idle (available to borrow) and which are active (currently in use), enforces a maximum pool size, and handles nil-safe borrowing with an optional timeout field per connection.

## Brief

Implement a `ConnectionPool` that manages `*Connection` objects with the following behavior:

1. **`Connection`** struct — represents a single DB connection with fields: `ID string`, `Host string`, `Port int`, `CreatedAt time.Time`, `LastUsedAt *time.Time` (nil until first use), `IdleTimeoutSecs *int` (nil means no idle timeout). Constructor: `NewConnection(id, host string, port int) *Connection`.

2. **`ConnectionPool`** struct — maintains two lists of connections: idle and active. Fields: `maxSize int`, `idle []*Connection`, `active []*Connection`, `mu sync.Mutex`. Constructor: `NewConnectionPool(maxSize int) (*ConnectionPool, error)` — reject maxSize < 1.

3. **`Borrow() (*Connection, error)`** — removes a connection from idle, adds to active, updates `LastUsedAt`, returns it. Returns an error (not a panic) if no idle connections are available.

4. **`Return(conn *Connection) error`** — moves a connection from active back to idle. Returns an error if `conn` is nil or if the connection is not in the active list.

5. **`Seed(conn *Connection) error`** — adds a new connection to the idle pool. Returns an error if `conn` is nil or if the pool is already at `maxSize`.

6. **`Stats() PoolStats`** — returns a snapshot with `IdleCount int`, `ActiveCount int`, `MaxSize int`.

7. **`IsExpired(conn *Connection) bool`** — returns true if `conn.IdleTimeoutSecs` is non-nil and the connection has been idle longer than that many seconds since `LastUsedAt`. Returns false if either pointer is nil (no timeout configured, or never used).

### Acceptance Criteria

- [ ] `NewConnection` sets `CreatedAt` to `time.Now()` and leaves `LastUsedAt` and `IdleTimeoutSecs` as nil
- [ ] `NewConnectionPool` returns an error for `maxSize < 1`
- [ ] `Borrow` returns an error (not a panic) when the idle pool is empty
- [ ] `Borrow` updates `LastUsedAt` on the borrowed connection (setting it from nil to `&now`)
- [ ] `Return` returns an error if `conn` is nil
- [ ] `Return` returns an error if the connection is not currently in the active list
- [ ] `Seed` returns an error if the pool is at capacity (idle + active >= maxSize)
- [ ] `Seed` returns an error if `conn` is nil
- [ ] `IsExpired` returns false when `IdleTimeoutSecs` is nil (no timeout set)
- [ ] `IsExpired` returns false when `LastUsedAt` is nil (never borrowed)
- [ ] `IsExpired` returns true when idle time exceeds the configured timeout
- [ ] All tests in `starter/main_test.go` pass
- [ ] Standard library only — no third-party packages

## Constraints

- All state mutations (`Borrow`, `Return`, `Seed`) must hold the mutex for the duration of the mutation
- `IsExpired` is a pure read — no mutex needed (it reads only from the `Connection` struct, not the pool)
- Do not use global state — all state lives in structs
- `LastUsedAt` and `IdleTimeoutSecs` must remain `*time.Time` and `*int` respectively — this is intentional pointer-as-optional-value practice

## Concepts Exercised

- Constructor returns `*T` with validation
- `*T` fields as optional values (`LastUsedAt`, `IdleTimeoutSecs`)
- Nil checks before dereferencing
- Pointer receiver methods for all state-mutating operations
- Shared state via pointer (`*ConnectionPool` passed around)
- `sync.Mutex` protecting shared slice state
- `PoolStats` returned by value (snapshot, not live state)

## Hints

<details>
<summary>Hint 1: Removing from a slice</summary>

To remove a connection from the active slice when returning it:
```go
for i, c := range p.active {
    if c == conn {  // pointer equality — same *Connection
        p.active = append(p.active[:i], p.active[i+1:]...)
        break
    }
}
```
Pointer equality (`c == conn`) compares addresses — two `*Connection` are equal only if they point to the same `Connection`.
</details>

<details>
<summary>Hint 2: Setting LastUsedAt from nil</summary>

`LastUsedAt` is `*time.Time` — nil until first use. To set it:
```go
now := time.Now()
conn.LastUsedAt = &now
```
You can't write `conn.LastUsedAt = &time.Now()` — you can't take the address of a function return value. Assign to a variable first.
</details>

<details>
<summary>Hint 3: IsExpired logic</summary>

```go
func IsExpired(conn *Connection) bool {
    if conn.IdleTimeoutSecs == nil || conn.LastUsedAt == nil {
        return false
    }
    idleDuration := time.Since(*conn.LastUsedAt)
    timeout := time.Duration(*conn.IdleTimeoutSecs) * time.Second
    return idleDuration > timeout
}
```
</details>

<details>
<summary>Hint 4: Pool capacity check in Seed</summary>

Total connections = idle + active. Check before adding:
```go
if len(p.idle)+len(p.active) >= p.maxSize {
    return fmt.Errorf("pool is at capacity (%d)", p.maxSize)
}
```
</details>
