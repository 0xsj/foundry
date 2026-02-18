# Solution -- Debugging: Strategy Pattern Rate Limiter

## Bug 1: TokenBucketLimiter Does Not Satisfy RateLimiter

### The Problem

The `RateLimiter` interface requires:
```go
Allow(key string) bool
```

But `TokenBucketLimiter.Allow` has an extra parameter:
```go
func (t *TokenBucketLimiter) Allow(key string, weight int) bool
```

This means `*TokenBucketLimiter` does not satisfy `RateLimiter`. The compile-time interface guard was commented out, so the error was hidden until runtime.

### The Fix

Remove the `weight` parameter and hardcode the cost to 1 (matching the interface contract):

```go
func (t *TokenBucketLimiter) Allow(key string) bool {
    t.mu.Lock()
    defer t.mu.Unlock()

    now := time.Now()
    b, ok := t.buckets[key]
    if !ok {
        b = &tbucket{tokens: t.capacity, lastRefill: now}
        t.buckets[key] = b
    }

    elapsed := now.Sub(b.lastRefill).Seconds()
    b.tokens += elapsed * t.refillRate
    if b.tokens > t.capacity {
        b.tokens = t.capacity
    }
    b.lastRefill = now

    if b.tokens >= 1 {
        b.tokens--
        return true
    }
    return false
}
```

Also uncomment the interface guard:
```go
var _ RateLimiter = (*TokenBucketLimiter)(nil)
```

### Lesson

Always use compile-time interface guards (`var _ Interface = (*Type)(nil)`) immediately after defining a type that's meant to satisfy an interface. This catches signature mismatches at compile time instead of runtime. If you need weighted tokens, add a separate method like `AllowN(key string, n int) bool` alongside the `Allow` method.

### Related Pitfalls

- [[go-interface-method-set]] -- Method set rules determine interface satisfaction

---

## Bug 2: Gateway Nil Fallback Panic

### The Problem

`NewGateway` accepts a `fallback` parameter but never assigns it to the struct:

```go
return &Gateway{
    limiters: m,
    // fallback is not assigned!
}
```

When `HandleRequest` is called with an unknown tier, `g.fallback` is `nil`, and calling `limiter.Allow(clientID)` panics with a nil pointer dereference.

### The Fix

Assign the fallback in the struct literal:

```go
return &Gateway{
    limiters: m,
    fallback: fallback,
}
```

### Lesson

This is a classic "forgot to wire" bug. In Go, struct fields default to their zero value, and the zero value of an interface is `nil`. The code compiles fine because `nil` is a valid interface value -- the bug only appears at runtime when the nil interface is used. Strategies for prevention:

1. **Constructor validation**: Check that `fallback != nil` and panic or return an error if it is.
2. **Integration tests**: Always test the fallback/default path, not just the happy path.
3. **IDE inspection**: Many editors flag unused parameters. The `fallback` parameter was received but not stored.

### Related Pitfalls

- [[go-nil-interface-trap]] -- Nil interfaces and nil pointer dereferences

---

## Bug 3: Wrong Tier Mapping (Tier Swap)

### The Problem

After building the limiter map, `NewGateway` contains a "validation" loop that accidentally swaps the first and last tier's limiters:

```go
keys := make([]string, 0, len(m))
for k := range m {
    keys = append(keys, k)
}
if len(keys) >= 2 {
    m[keys[0]], m[keys[len(keys)-1]] = m[keys[len(keys)-1]], m[keys[0]]
}
```

This swaps whichever tiers happen to be first and last in the map iteration order (which is random in Go). Enterprise clients might get the free-tier limiter and vice versa.

### The Fix

Remove the entire swap block. It serves no purpose:

```go
func NewGateway(fallback RateLimiter, tiers map[string]RateLimiter) *Gateway {
    m := make(map[string]RateLimiter, len(tiers))
    for k, v := range tiers {
        m[k] = v
    }
    return &Gateway{
        limiters: m,
        fallback: fallback,
    }
}
```

### Lesson

This bug is insidious because:

1. **Map iteration order is random in Go.** The swap doesn't always affect the same tiers. Tests might pass in some runs and fail in others depending on map ordering. This is a flaky test waiting to happen.
2. **The bug looks intentional.** It has the shape of "sorting" or "normalization" code, which makes it easy to skip during code review.
3. **It corrupts configuration silently.** The gateway starts, accepts requests, and rate-limits -- but with the wrong limits for the wrong clients.

Prevention: Test the exact mapping, not just "it doesn't crash." The test `TestGateway_CorrectTierMapping` verifies that each tier maps to the expected limiter name.

---

## Bug 4: Sliding Window Off-by-One

### The Problem

The timestamp cleanup loop starts at index 1 instead of 0:

```go
validFrom := 1 // BUG: should be 0
for validFrom < len(ts) && !ts[validFrom].After(cutoff) {
    validFrom++
}
```

Starting at 1 means the first timestamp is always skipped during cleanup. On each `Allow()` call, one valid timestamp is dropped from the front of the slice. This makes the limiter "forget" a request on each call, allowing more requests through than the configured maximum.

For example, with `maxReqs=3`:
- Request 1: timestamps = [t1], len=1 < 3, allowed
- Request 2: cleanup drops t1 (starts at index 1, so t1 survives... wait, validFrom=1 means ts[1:] which drops t1). timestamps = [t2], len=1 < 3, allowed
- Request 3: cleanup drops t2. timestamps = [t3], len=1 < 3, allowed
- Request 4: cleanup drops t3. timestamps = [t4], len=1 < 3, allowed -- should be denied!

The limiter never reaches the limit because it forgets one request per call.

### The Fix

Start the loop at index 0:

```go
validFrom := 0
for validFrom < len(ts) && !ts[validFrom].After(cutoff) {
    validFrom++
}
```

Also, the condition should use `Before(cutoff)` instead of `!After(cutoff)`. Using `!After` means timestamps exactly equal to the cutoff are also removed, which could incorrectly remove a timestamp that's exactly at the window boundary. However, in practice with nanosecond precision, exact equality is rare. The more critical fix is the index.

### Lesson

Off-by-one errors in sliding window implementations are extremely common. The cleanup loop defines the boundary between "expired" and "valid" entries. Starting at the wrong index shifts that boundary. Prevention:

1. **Test exact counts.** Don't just test "allows some requests" -- test that exactly N are allowed for a limit of N.
2. **Test the denial path.** After allowing the maximum, verify the next request is denied immediately (no time passes).
3. **Trace through the algorithm by hand** with a small example (maxReqs=2, 3 requests).

### Related Pitfalls

- [[go-slice-reslice-gotcha]] -- Reslicing semantics and off-by-one errors

---

## Summary

| Bug | Category | Severity | Detection Method |
|-----|----------|----------|-----------------|
| Wrong method signature | Interface satisfaction | Compile error (if guard is enabled) | Compile-time interface guard |
| Nil fallback | Nil strategy panic | Runtime panic | Test the fallback/default path |
| Tier swap | Configuration corruption | Silent wrong behavior | Test exact tier-to-strategy mapping |
| Off-by-one cleanup | Logic error | Over-permissive rate limiting | Test exact request counts at the limit |

All four bugs relate to Strategy pattern implementation concerns:
- **Bug 1**: The strategy doesn't satisfy the interface contract
- **Bug 2**: A nil strategy causes a panic (always provide defaults)
- **Bug 3**: The wrong strategy is selected (test your wiring)
- **Bug 4**: A strategy's core logic is flawed (test exact behavior, not just happy paths)
