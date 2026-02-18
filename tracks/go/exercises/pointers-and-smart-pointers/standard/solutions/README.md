# Solution: Connection Pool Manager

## Approach

The solution uses `*Connection` as the identity token for pool operations. When you `Borrow`, you get back the exact pointer that was `Seed`ed. When you `Return`, the pool uses pointer equality (`c == conn`) to find it in the active list. This means two `Connection` structs with identical fields but different addresses are different connections — correct for a pool.

## Key Decisions

**Why `*time.Time` for `LastUsedAt`?**
The field needs to model three states: "never borrowed" (nil), "borrowed at time T" (non-nil). A zero `time.Time` wouldn't work — you couldn't distinguish "never set" from "borrowed at January 1, year 1". Pointer-to-T is the Go idiom for optional values.

**Why `*int` for `IdleTimeoutSecs`?**
Same logic: nil = "no timeout configured", `&n` = "timeout is n seconds". Without the pointer, 0 would be ambiguous (no timeout, or zero-second timeout?).

**Why FIFO (taking from front of idle list)?**
Prevents some connections from sitting idle forever. A LIFO approach would always reuse the most recently returned connection, leaving older connections cold. FIFO is simple and fair.

**Why return `PoolStats` by value?**
It's a point-in-time snapshot. If it returned a pointer into the pool's internal state, callers would observe live mutations — which would require locking to read safely. Returning by value copies the data once (under the lock) and gives the caller a stable snapshot.

**Why does `Seed` count active connections toward capacity?**
Because active connections are still owned by the pool. The pool promises a total of at most `maxSize` connections. Borrowed connections count.

## Variants

See `variants/` for:
- **`with_channels.go`** — alternate implementation using a `chan *Connection` as the idle queue (idiomatic for high-concurrency scenarios; the channel acts as a bounded buffer)

## Performance Notes

| Operation | Time | Notes |
|-----------|------|-------|
| Borrow | O(1) | Slice pop from front (O(n) copy, but pool size is bounded and small) |
| Return | O(n) | Linear search through active list (n = active connections, small) |
| Seed | O(1) | Append to idle slice |
| Stats | O(1) | Reads len() on two slices |

For large pools (thousands of connections), consider a `map[*Connection]struct{}` for the active set to make `Return` O(1). For typical DB connection pools (5–100 connections), the linear scan is negligible.

## Running Tests

```
cd solutions/
go test -v .
```

```
cd starter/
go test -v .   # all tests fail until you implement the functions
```
