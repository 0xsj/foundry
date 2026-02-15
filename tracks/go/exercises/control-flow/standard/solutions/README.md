# Solution: Retry Mechanism

## Approach

The reference solution demonstrates idiomatic Go control flow patterns:

1. **Guard clauses** for input validation (nil check, bounds check)
2. **For loop** with clear iteration bounds (`attempt <= maxRetries`)
3. **Early returns** on success and non-retryable errors
4. **Type assertion** to distinguish retryable from non-retryable errors
5. **Exponential backoff** using bit shifting (`1 << attempt` = 2^attempt)

## Key Implementation Details

### Guard Clauses First

```go
if op == nil {
    return fmt.Errorf("operation cannot be nil")
}
if maxRetries < 0 {
    return fmt.Errorf("maxRetries must be >= 0")
}
```

Validates inputs before proceeding. Prevents panic and clarifies preconditions.

### Loop Structure

```go
for attempt := 0; attempt <= maxRetries; attempt++ {
    // ...
}
```

Iterates `maxRetries + 1` times (initial attempt + retries). Clear, predictable bounds.

### Early Returns

```go
if err == nil {
    return nil  // success - exit immediately
}

var retryableErr *RetryableError
if !errors.As(err, &retryableErr) {
    return fmt.Errorf("non-retryable error: %w", err)  // exit immediately
}
```

The happy path and non-retryable errors both exit early. No unnecessary retries.

### Exponential Backoff

```go
delay := time.Second * time.Duration(1<<attempt)
```

Bit shifting (`1 << n`) is equivalent to `2^n`:
- attempt 0: `1 << 0` = 1 second
- attempt 1: `1 << 1` = 2 seconds
- attempt 2: `1 << 2` = 4 seconds
- attempt 3: `1 << 3` = 8 seconds

### Error Wrapping

```go
return fmt.Errorf("all retries exhausted after %d attempts: %w", attempt+1, err)
```

Uses `%w` to wrap the original error, preserving the error chain for debugging.

## Alternative Approaches

### Variant 1: With Jitter

Add randomness to backoff to prevent thundering herd:

```go
import "math/rand"

baseDelay := time.Second * time.Duration(1<<attempt)
jitter := time.Duration(rand.Intn(1000)) * time.Millisecond
delay := baseDelay + jitter
```

**Trade-off:** More realistic for production, but harder to test due to randomness.

### Variant 2: With Context

Support cancellation via context:

```go
func RetryWithContext(ctx context.Context, op Operation, maxRetries int) error {
    for attempt := 0; attempt <= maxRetries; attempt++ {
        // Check if context is cancelled
        select {
        case <-ctx.Done():
            return ctx.Err()
        default:
        }

        err := op()
        // ... rest of logic

        // Sleep with context awareness
        select {
        case <-time.After(delay):
        case <-ctx.Done():
            return ctx.Err()
        }
    }
}
```

**Trade-off:** More flexible (can timeout/cancel), but more complex.

### Variant 3: Configurable Backoff Strategy

Make backoff pluggable:

```go
type BackoffStrategy func(attempt int) time.Duration

func RetryWithBackoff(op Operation, maxRetries int, backoff BackoffStrategy) error {
    // ...
    delay := backoff(attempt)
    // ...
}

// Usage
exponential := func(attempt int) time.Duration {
    return time.Second * time.Duration(1<<attempt)
}
linear := func(attempt int) time.Duration {
    return time.Second * time.Duration(attempt+1)
}
```

**Trade-off:** More flexible, but introduces complexity for a simple use case.

## Performance Notes

### Time Complexity
- **O(n)** where n = maxRetries + 1
- Each iteration does constant work (operation + sleep)

### Space Complexity
- **O(1)** - no additional allocations (error wrapping reuses existing error)

### Total Time
With exponential backoff and maxRetries = n:
- Total time = operation time × attempts + Σ(2^i) seconds for i=0 to n-1
- For maxRetries = 5: up to 6 attempts + (1 + 2 + 4 + 8 + 16) = 31 seconds of backoff

## Production Considerations

In real systems, you might also want:
- **Maximum backoff cap** (e.g., don't wait more than 30s)
- **Jitter** to prevent thundering herd
- **Context support** for cancellation/timeouts
- **Metrics/logging** for observability
- **Circuit breaker** integration to stop retrying unhealthy services
- **Configurable backoff strategies** (linear, exponential, fibonacci)

## Related Patterns

- **Circuit Breaker** (prevents cascading failures)
- **Bulkhead** (isolates failures)
- **Timeout** (bounds execution time)
- **Fallback** (graceful degradation)

These are part of the resilience patterns covered later in the curriculum.
