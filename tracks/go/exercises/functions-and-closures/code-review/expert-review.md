# Expert Review: Exponential Backoff with Closures

## Critical Issues

### 1. `Backoff`: Shared `attempt` counter across invocations

**Location:** `Backoff`, `attempt := 0` before the returned function

`attempt` is declared in `Backoff`'s scope and captured by the returned closure. On the **first** call to the returned function, `attempt` starts at 0 and increments correctly. But on a **second** call, `attempt` still holds its value from the previous run — so the loop condition `attempt < maxAttempts` is already false, and the function returns immediately without trying the operation at all.

```go
// Bug: attempt is shared
retry := Backoff(op, 3, time.Millisecond)
retry()  // works: attempt goes 0→1→2→3, op tried 3 times
retry()  // broken: attempt is 3, loop never executes, op never called
```

The docstring says "The returned function can be called multiple times to retry the full sequence" — which is directly contradicted by this bug.

**Fix:** Move `attempt` inside the returned function so each call starts fresh:

```go
func Backoff(op Op, maxAttempts int, baseDelay time.Duration) func() Result {
    return func() Result {
        attempt := 0  // reset on every call to the returned function
        start := time.Now()
        for attempt < maxAttempts {
            // ...
        }
        // ...
    }
}
```

**Concept:** Closures capture variables by reference. State captured outside the returned function persists across calls. State declared inside the returned function resets on each call. Choosing where to declare state is a key closure design decision.

---

### 2. `Backoff`: Error information is discarded

**Location:** `Backoff`, the failure `Result` return

```go
LastErr: fmt.Errorf("failed after %d attempts", maxAttempts),
```

This creates a new error that only contains the attempt count. The actual error from `op()` — the root cause — is thrown away. A caller trying to distinguish "database connection refused" from "certificate expired" can't.

This is an insidious bug because the code looks correct. It returns an error, the error has a message, but it contains no diagnostic information.

**Fix:** Wrap the original error:

```go
return Result{
    Attempts: attempt,
    LastErr:  fmt.Errorf("failed after %d attempts: %w", maxAttempts, lastErr),
    Duration: time.Since(start),
}
```

Where `lastErr` is tracked across loop iterations:

```go
var lastErr error
for attempt < maxAttempts {
    lastErr = op()
    attempt++
    if lastErr == nil { ... }
    // ...
}
return Result{..., LastErr: fmt.Errorf("...: %w", lastErr)}
```

The `%w` verb (Go 1.13+) wraps the error so callers can use `errors.Is()` and `errors.As()` to unwrap and inspect the underlying cause.

**Concept:** Error wrapping with `%w` is the Go standard for preserving error chains. Replacing the original error with a new one severs the chain and loses diagnostic information.

---

## Major Concerns

### 3. `WithHooks`: `defer after(attempt, err)` evaluates arguments at defer time

**Location:** `WithHooks`, the `defer after(attempt, err)` statement

`defer` evaluates its **arguments immediately** when the statement is encountered, but delays the **call** until the function returns. So:

```go
attempt++
defer after(attempt, err)  // attempt and err are evaluated HERE
return err
```

`attempt` is captured as its current value (e.g., 1), and `err` is captured as the return value of `op()`. The defer fires after `return err`, but the values were already frozen. In this specific case, since `after` is called with the already-computed values and the function returns immediately after, the behavior happens to be correct for `attempt` and `err` — but it's fragile and misleading.

More importantly: the call to `defer after(...)` happens after `attempt++` and after `err` is set, so the values are actually correct in this specific code path. But consider if code changed:

```go
attempt++
defer after(attempt, err)  // if you add code below that changes err, defer won't see it
err = someOtherOp()        // this change to err is NOT seen by the deferred after()
return err
```

The idiomatic pattern when the deferred call needs the final state is a closure:

```go
defer func() { after(attempt, err) }()
```

This reads `attempt` and `err` when the closure executes, not when `defer` runs.

Additionally, the `after` hook should arguably not use `defer` at all — it's not a cleanup function, it's an observability hook. An explicit call makes the intent clearer:

```go
return func() error {
    before()
    err := op()
    attempt++
    after(attempt, err)  // explicit — always runs, no defer confusion
    return err
}
```

**Concept:** Use `defer f(x)` when you want the value of `x` at defer time. Use `defer func() { f(x) }()` when you want the value of `x` at return time. Use explicit calls when the operation isn't cleanup/teardown.

---

### 4. `MakeAttempts`: Closure captures loop variable by reference

**Location:** `MakeAttempts`, `result[i] = func() (int, error) { return i + 1, op() }`

All returned functions close over the same `i` variable. After the loop, `i == maxAttempts`. Every function in `result` returns `maxAttempts + 1` regardless of its position.

```go
attempts := MakeAttempts(op, 3)
attempts[0]()  // returns (4, ...) — not (1, ...)
attempts[1]()  // returns (4, ...) — not (2, ...)
```

**Fix:**

```go
for i := 0; i < maxAttempts; i++ {
    i := i  // shadow with a new per-iteration variable
    result[i] = func() (int, error) {
        return i + 1, op()
    }
}
```

**Concept:** This is the same loop closure gotcha from the debugging exercise. It's common enough that the Go team fixed it in Go 1.22 (per-iteration loop variable scoping). Whenever you create closures in a loop, verify they don't capture the loop variable.

---

## Minor Suggestions

### 5. `Schedule`: Anonymous struct return type is non-idiomatic

**Location:** `Schedule` return type

```go
func Schedule(fns []Op) struct { Completed int; Err error }
```

Anonymous struct return types are rarely seen in Go except in table-driven tests. For a shared SDK, this forces callers to use the anonymous struct syntax, which is clunky and not assignable to a named type without a conversion.

**Fix:** Define a named type or use idiomatic multiple returns:

```go
// Option A: multiple returns (idiomatic for simple cases)
func Schedule(fns []Op) (int, error) {
    for i, fn := range fns {
        if err := fn(); err != nil {
            return i, err
        }
    }
    return len(fns), nil
}

// Option B: named struct (if the caller needs to store both fields together)
type ScheduleResult struct {
    Completed int
    Err       error
}

func Schedule(fns []Op) ScheduleResult { ... }
```

**Concept:** Go functions returning `(value, error)` is universally idiomatic. Anonymous struct returns are a red flag in production code — they make the API harder to use and document.

---

### 6. `Jitter`: Not actually random, exported-style name for unexported helper

**Location:** `Jitter` function

Two issues:
1. The function always returns `delay * 1.5` — it's deterministic and always returns the maximum of the intended range. Real jitter needs `math/rand` and should return a value in `[delay*0.5, delay*1.5]`. The comment acknowledges it's not truly random but calls it "for testability" — that's not a valid reason. Testability is better served by accepting a `rand.Rand` as a parameter or making jitter a function type.

2. `Jitter` is unexported-style logic but exported. If it's an internal helper, it should be unexported (`jitter`). If it's part of the public API, it should actually implement the contract it advertises.

**Fix:**

```go
import "math/rand"

// jitter returns a duration in [delay*0.5, delay*1.5]
func jitter(delay time.Duration, r *rand.Rand) time.Duration {
    factor := 0.5 + r.Float64()  // [0.5, 1.5)
    return time.Duration(float64(delay) * factor)
}
```

Or, if it needs to be exported and testable:

```go
type JitterFn func(time.Duration) time.Duration

func DefaultJitter(delay time.Duration) time.Duration {
    factor := 0.5 + rand.Float64()
    return time.Duration(float64(delay) * factor)
}
```

---

## Positive Feedback

- The `Result` struct is well-designed — returning structured data instead of multiple return values for a complex outcome is the right call here
- `WithHooks` is a useful and composable design — the before/after hook pattern is idiomatic for observability without coupling the retry logic to specific metrics systems
- The exponential backoff formula (`baseDelay * 2^(attempt-1)`) is mathematically correct
- Using named types (`Op`) for the function type makes the code readable

---

## Summary

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | Critical | `Backoff` | `attempt` captured outside returned func — state persists across calls |
| 2 | Critical | `Backoff` | Original error discarded — `%w` needed for error chain preservation |
| 3 | Major | `WithHooks` | `defer after(attempt, err)` — non-idiomatic, fragile defer argument evaluation |
| 4 | Major | `MakeAttempts` | Loop closure captures `i` by reference — all return `maxAttempts+1` |
| 5 | Minor | `Schedule` | Anonymous struct return type — non-idiomatic, use `(int, error)` |
| 6 | Minor | `Jitter` | Not actually random; export decision inconsistent with helper nature |
