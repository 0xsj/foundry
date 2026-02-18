# Solution: Service Config Loader Debugging

## Bug 1: `%v` instead of `%w` breaks `errors.Is(err, ErrTimeout)`

**Location:** `loadConfig`, the timeout error return

**What:**

```go
// WRONG
return fmt.Errorf("connection to %s:%d timed out: %v", cfg.Host, cfg.Port, ErrTimeout)
```

`%v` formats the error as a string — it calls `ErrTimeout.Error()` and embeds the text. The result is a new error with no structural link to `ErrTimeout`. `errors.Is` traverses the chain using `Unwrap()`, but `%v` creates no chain — there's nothing to unwrap.

```go
// After the bug, the error is:
// "connection to db.prod.internal:5432 timed out: timeout"
// This is a plain string, not a wrapped error. ErrTimeout is lost.

errors.Is(err, ErrTimeout)  // false — chain is severed
```

**Fix:**

```go
return fmt.Errorf("connection to %s:%d timed out: %w", cfg.Host, cfg.Port, ErrTimeout)
```

`%w` wraps `ErrTimeout` in the new error. The resulting error has an `Unwrap()` method that returns `ErrTimeout`. `errors.Is` finds it.

```go
errors.Is(err, ErrTimeout)  // true — found via Unwrap()
```

**How to spot it:** If `errors.Is` is failing for a sentinel you know is in the error message, check whether the error was created with `%w` (wraps) or `%v` (doesn't wrap). The output looks identical, but only `%w` builds the chain.

**Prevention:** Any time you need `errors.Is` to work through the chain, use `%w`. Use `%v` only when you're creating a terminal error message where the original type doesn't matter.

**Related:** [[go-error-wrapping]]

---

## Bug 2: `%v` in `connect` severs the `ErrConnectionRefused` chain

**Location:** `connect`, the error return from `dial`

**What:**

```go
// WRONG
return fmt.Errorf("connect to %s: %v", address, err)
```

Same mechanism as Bug 1, different function. `dial` returns `ErrConnectionRefused` directly. `connect` wraps it with `%v`, creating a new string-only error. The sentinel is lost.

```go
// dial returns: ErrConnectionRefused (the actual sentinel)
// connect wraps with %v: "connect to down.db.internal:5432: connection refused"
// errors.Is(err, ErrConnectionRefused): false — ErrConnectionRefused is gone
```

**Fix:**

```go
return fmt.Errorf("connect to %s: %w", address, err)
```

**Key insight:** Every layer in the call stack that wraps an error needs `%w` to preserve the chain. One `%v` anywhere in the chain severs it from that point down. Bugs 1 and 2 are the same root cause — they're included together because this pattern is extremely common.

---

## Bug 3: `panic` for empty address in library code

**Location:** `ConnectWithRetry`, the empty address check

**What:**

```go
// WRONG
if address == "" {
    panic("address cannot be empty")
}
```

Library code **must never panic for bad user input**. A panic propagates up the goroutine's call stack, firing all deferred functions, and terminates the goroutine unless something calls `recover`. In an HTTP server, one panicking goroutine that isn't recovered will take down the entire server. Even if a `recover` is in place upstream, it converts the panic to an opaque error — the caller loses the ability to inspect and handle it programmatically.

The rule: **panic for programming errors** (things that should never happen in correctly-written code), **return errors for user/input errors** (things that can happen at runtime with bad inputs).

An empty address is a user error. The caller made a mistake. Return an error so they can handle it.

**Fix:**

```go
if address == "" {
    return fmt.Errorf("ConnectWithRetry: address cannot be empty")
}
```

**When IS panic acceptable?**
- `mustXxx` functions that wrap initialization with hardcoded values (e.g., `regexp.MustCompile`)
- Internal invariant violations that represent bugs in *your* code (not the caller's)
- `init()` functions where failure means the program cannot start

**Prevention:** Search your library code for `panic(`. Ask: "Is this reachable with normal (even bad) user inputs?" If yes, convert to an error return.

---

## Bug 4: Swallowed ping error (error checked but not returned)

**Location:** `openPool`, the `pool.ping()` call

**What:**

```go
// WRONG
if err := pool.ping(); err != nil {
    _ = err  // err is assigned to blank identifier — effectively discarded
}
return pool, nil  // always succeeds!
```

This is the most insidious bug in the set. The error is *not* ignored in the `err != nil` sense — there's a check. But the check body does nothing useful. The `_ = err` silences the "variable declared but not used" compiler error without actually handling the error. The function then returns `pool, nil` as if nothing went wrong.

This pattern is often introduced when a developer adds error handling to an existing function and forgets to actually propagate the error — or when they "silence a warning" without understanding the implications.

```go
// The ping failure is invisible to the caller:
pool, err := openPool("dead-host:5432", 5)
err == nil  // true — caller has no idea the ping failed
pool != nil // true — caller gets a pool that doesn't work
```

**Fix:**

```go
if err := pool.ping(); err != nil {
    return nil, fmt.Errorf("openPool: ping failed: %w", err)
}
```

**Prevention:**
- `staticcheck` and `errcheck` linters catch `_ = err` patterns
- Code review: look for `if err != nil { _ = err }` or similar no-op bodies
- The mental model: when you check an error, you must either return it, log it with context, or explicitly handle the case. Assigning to `_` inside an `if err != nil` block is almost always a bug.

The subtle variant: discarding the entire return without checking:

```go
pool.ping()  // return value completely ignored — different but equally bad
```

Both are caught by `errcheck`. Neither is acceptable in production code.

---

## Summary

| Bug | Concept | Root Cause | Fix |
|-----|---------|-----------|-----|
| `errors.Is(err, ErrTimeout)` always false | Error wrapping with `%w` | `%v` creates string error, not wrapped error — chain is severed | Change `%v` → `%w` |
| `errors.Is(err, ErrConnectionRefused)` always false | Error wrapping with `%w` | Same as above — intermediate `connect` function uses `%v` | Change `%v` → `%w` |
| `ConnectWithRetry` panics on empty address | panic vs error for user input | Library code panicked instead of returning an error | Return `fmt.Errorf(...)` instead of `panic(...)` |
| `openPool` succeeds when ping fails | Swallowed error | Error was checked but assigned to `_` — never returned | `return nil, fmt.Errorf("openPool: %w", err)` |

**Pattern:** Bugs 1 and 2 are about error *construction* (`%w` vs `%v`). Bug 3 is about error *philosophy* (when to panic). Bug 4 is about error *propagation* (always return errors you intend to surface).

## Related Pitfalls

- [[go-error-wrapping]] — `%w` vs `%v` in `fmt.Errorf`
- [[go-panic-in-library-code]] — when panic is and is not appropriate
- [[go-swallowed-errors]] — patterns that look like error handling but aren't
