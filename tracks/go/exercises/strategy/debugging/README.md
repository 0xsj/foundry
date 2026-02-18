# Debugging Exercise -- Strategy Pattern Rate Limiter

## Symptoms

A rate limiter service with pluggable strategies has four bugs. The code compiles (with one commented-out line that causes a compile error), but the tests reveal the following problems:

1. **Compile error**: One of the strategy types fails to satisfy the `RateLimiter` interface. The compile-time interface guard is commented out. Uncomment it to see the error, then fix the type.

2. **Panic on startup**: When the gateway is created with default settings, calling `HandleRequest` panics with `nil pointer dereference`. The fallback limiter is never initialized.

3. **Wrong strategy selected**: Enterprise clients are getting the free-tier rate limiter instead of their premium limiter. Configuration parsing maps the wrong tier string to the wrong strategy.

4. **Incorrect rate limiting**: The sliding window limiter allows far more requests than configured. The timestamp cleanup logic has an off-by-one error that removes valid (non-expired) timestamps.

## Context

The code implements a rate-limiting gateway similar to the standard exercise. Three rate-limiting strategies (token bucket, fixed window, sliding window) are wired into a gateway that selects the strategy based on client tier.

## Your Task

1. Read through `buggy.go` and identify all four bugs
2. Fix each bug
3. Verify tests pass: `go test`
4. Document your debugging process below

## Files

- `buggy.go` -- The code with four bugs
- `buggy_test.go` -- Tests that expose the bugs
- `solution.md` -- Explanation (don't peek until you've tried!)

---

## Your Debugging Process

### Hypothesis 1
What I thought the problem was:

Result:

### Hypothesis 2
What I thought the problem was:

Result:

### Hypothesis 3
What I thought the problem was:

Result:

### Hypothesis 4
What I thought the problem was:

Result:

### Root Causes
What the actual problems were:

### The Fixes
What I changed:

### Lessons
What I learned from this:
