# Debugging Exercise — Nil Map Panic

## Symptoms

The program panics when trying to store cache entries:

```
panic: assignment to entry in nil map

goroutine 1 [running]:
main.Set(...)
```

## Context

You're building a simple in-memory cache for storing computed results. The cache should store string keys to string values and support `Set` and `Get` operations.

The code compiles successfully but crashes at runtime when you try to add the first entry.

## Your Task

1. Read through `buggy.go` and identify the root cause
2. Fix the bug
3. Verify tests pass: `go test`
4. Document your debugging process below

## Files

- `buggy.go` — The code with the bug
- `buggy_test.go` — Failing test(s)
- `solution.md` — Explanation (don't peek until you've tried!)

---

## Your Debugging Process

### Hypothesis 1
What I thought the problem was:


Result:


### Hypothesis 2 (if needed)


### Root Cause
What the actual problem was:


### The Fix
What I changed:


### Lessons
What I learned from this:
