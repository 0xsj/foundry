# Solution: Retry Wrapper Debugging

## Bug 1: Closure Captures Loop Variable by Reference (`buildAttempts`)

**Location:** `buildAttempts`, line `attempts[i] = func() (int, error) { return i + 1, op() }`

**What:** All attempt functions close over the same `i` variable. By the time any of them is called, the loop has finished and `i` equals `maxAttempts`. Every attempt function returns `maxAttempts + 1` instead of its own index.

```go
// WRONG
for i := 0; i < maxAttempts; i++ {
    attempts[i] = func() (int, error) {
        return i + 1, op()  // i is the loop variable — same for all closures
    }
}
// After loop: i == maxAttempts
// All closures return maxAttempts + 1
```

**Fix: Shadow `i` with a new variable per iteration**

```go
for i := 0; i < maxAttempts; i++ {
    i := i  // new variable — each closure captures its own i
    attempts[i] = func() (int, error) {
        return i + 1, op()
    }
}
```

**Alternative fix: pass as argument**

```go
for i := 0; i < maxAttempts; i++ {
    attempts[i] = func(n int) attemptFunc {
        return func() (int, error) { return n + 1, op() }
    }(i)
}
```

**Why it happens:** Go closures capture variables by reference (via pointer), not by value. The loop variable `i` is a single variable that changes on every iteration. Every closure holds a pointer to the same `i`. When the closures finally execute, they all read the same final value.

**Go 1.22 note:** Starting in Go 1.22, `for` loop variables have per-iteration scope, fixing this automatically. Code compiled with `go 1.22` in `go.mod` would not have this bug. But you'll encounter it in older code and in every other language without this fix (JS `var` in loops, Python `for` variable, etc.).

**Prevention:** Whenever you create a closure inside a loop, ask: "does this closure capture the loop variable?" If yes, shadow it or pass it as an argument.

**Related:** [[go-closure-loop-gotcha]]

---

## Bug 2: Defer Argument Evaluated Too Early (`WithCleanup`)

**Location:** `defer cleanup(lastAttempt)` inside the retry loop

**What:** `defer cleanup(lastAttempt)` evaluates `lastAttempt` **immediately** when the defer statement executes — not when the deferred call runs. So each iteration registers a `defer cleanup(attempt)` with the value of `lastAttempt` at that moment. After 3 failing attempts:

- Iteration 1: `defer cleanup(1)` — captured as 1
- Iteration 2: `defer cleanup(2)` — captured as 2
- Iteration 3: `defer cleanup(3)` — captured as 3

Three cleanup calls are registered, with values 1, 2, and 3. They all fire on function return (LIFO: 3, 2, 1). The test expects cleanup called once with the last attempt number.

```go
// WRONG: defer inside loop — defers accumulate
for attempt := 1; attempt <= maxAttempts; attempt++ {
    lastAttempt = attempt
    defer cleanup(lastAttempt)  // argument evaluated NOW — 3 defers registered
    // ...
}
```

**Fix 1: Single defer outside the loop**

```go
// Register the defer once. It captures lastAttempt by address via the closure.
defer func() { cleanup(lastAttempt) }()

for attempt := 1; attempt <= maxAttempts; attempt++ {
    lastAttempt = attempt
    // ...
}
```

Now `lastAttempt` is read when the deferred closure runs (at function return), not when the defer statement executes. It will have the value of the final attempt.

**Fix 2: Use named return and defer outside loop**

```go
func WithCleanup(op Operation, maxAttempts int, cleanup func(int)) (result Result, err error) {
    var lastAttempt int
    defer func() { cleanup(lastAttempt) }()
    // loop...
}
```

**Why it happens:** Defer argument evaluation is a subtle but important rule: function values and arguments in a defer statement are evaluated immediately when the statement is encountered, but the function call is deferred. This means `defer f(x)` captures the current value of `x` for use later — but `defer func() { f(x) }()` captures a reference to `x` and reads it when the defer fires.

**Key distinction:**

| Form | When `x` is read |
|------|-----------------|
| `defer f(x)` | Immediately — current value of `x` |
| `defer func() { f(x) }()` | When defer fires — final value of `x` |

**Prevention:** When you want a deferred call to use the final value of a variable (not the value at defer time), wrap it in a closure: `defer func() { cleanup(lastAttempt) }()`.

---

## Bug 3: `defer` Inside Retry Loop Causes Resource Leak

**Location:** `defer t.Stop()` inside the `for` loop in `Retry`

**What:** `defer t.Stop()` registers a deferred call on every loop iteration. None of them fire until `Retry` returns. If `maxAttempts` is 100 and each timer holds a file descriptor, all 100 descriptors stay open until the function returns.

```go
// WRONG: defer inside loop — all defers accumulate
for i := 0; i < maxAttempts; i++ {
    t := newTimer(i)
    defer t.Stop()  // registers a new defer — doesn't fire until Retry returns
    err = op()
    // ...
}
// All timer.Stop() calls run here, at function return
```

**Fix 1: Explicit stop in the loop (no defer)**

```go
for i := 0; i < maxAttempts; i++ {
    t := newTimer(i)
    timers = append(timers, t)

    err = op()
    attempts++
    t.Stop()  // explicit call — runs at end of each iteration

    if err == nil {
        return
    }
}
```

**Fix 2: Extract to a helper (defer runs when helper returns)**

```go
func runWithTimer(op Operation, attempt int) (*fakeTimer, error) {
    t := newTimer(attempt)
    defer t.Stop()  // runs when runWithTimer returns — after each attempt
    return t, op()
}

func Retry(op Operation, maxAttempts int) (attempts int, timers []*fakeTimer, err error) {
    for i := 0; i < maxAttempts; i++ {
        t, opErr := runWithTimer(op, i)
        timers = append(timers, t)
        attempts++
        err = opErr
        if err == nil {
            return
        }
    }
    return
}
```

**Why it happens:** `defer` is scoped to the **function**, not the iteration. Every `defer` in a loop body adds a new entry to the function's defer chain. They all fire when the surrounding function returns. This is fundamentally different from RAII in C++ or `using`/`with` in C# and Python, which scope cleanup to the block.

**Prevention:** Never `defer` a resource close inside a loop unless you're intentionally cleaning up at function return. If you want per-iteration cleanup, use explicit calls or extract the per-iteration work to a function.

---

## Summary

| Bug | Concept | Root Cause | Fix |
|-----|---------|-----------|-----|
| Wrong attempt index | Closure variable capture | Loop variable captured by reference — all closures share the final value | Shadow `i` per iteration: `i := i` |
| Wrong cleanup argument | Defer argument evaluation | Defer args evaluated at defer time, not call time | Use a closure: `defer func() { cleanup(last) }()` |
| Timer leak | Defer scope | `defer` inside loop fires at function return, not iteration end | Explicit stop in loop, or extract to helper |

All three bugs stem from misunderstanding **when** things happen in Go: when a closure captures its variables, when defer arguments are evaluated, and when deferred calls fire. The functions-and-closures module covers all three. Re-read the defer section with these bugs in mind.
