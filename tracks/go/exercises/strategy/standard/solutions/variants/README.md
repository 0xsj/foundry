# Solution Variants: Rate Limiter Strategies

## Overview

The reference solution implements three rate-limiting strategies. Each has different characteristics, making them suitable for different production contexts.

## Strategy Comparison

| Aspect | Token Bucket | Fixed Window | Sliding Window |
|--------|-------------|--------------|----------------|
| **Accuracy** | Good (smooths over time) | Fair (boundary problem) | Excellent (true rolling window) |
| **Memory per key** | O(1) (tokens + timestamp) | O(1) (counter + window ID) | O(n) (stores all timestamps in window) |
| **Time per Allow()** | O(1) | O(1) | O(n) (cleanup of expired entries) |
| **Burst handling** | Allows bursts up to capacity | Allows full limit at boundary edges (2x burst) | No burst beyond limit |
| **Complexity** | Medium | Low | Medium-High |
| **Best for** | General-purpose APIs | Simple rate limiting, non-critical | Strict rate limiting, billing-sensitive |

## The Fixed Window Boundary Problem

The main weakness of fixed window is the boundary problem. If the window is 1 second with a limit of 100:

```
Window 1: [0.0s --- 0.9s: 0 requests] [0.9s-1.0s: 100 requests]
Window 2: [1.0s-1.1s: 100 requests]   [1.1s --- 2.0s: 0 requests]
```

The client sent 200 requests in 0.2 seconds (from 0.9s to 1.1s), which is 2x the intended rate. The sliding window prevents this because it considers the full rolling window at any point.

## Token Bucket vs Leaky Bucket

The token bucket is often confused with the **leaky bucket** algorithm. The difference:

- **Token bucket**: Tokens accumulate. Requests consume tokens. Allows bursts.
- **Leaky bucket**: Requests enter a queue. They "leak" out at a fixed rate. Smooths traffic.

Token bucket is more common in API gateways because it allows legitimate bursts (e.g., a client that was idle for 10 seconds should be able to make several quick requests).

## Production Considerations Not Covered

The exercise focuses on the Strategy pattern. In production, you'd also need:

1. **Distributed rate limiting**: These implementations are in-memory, single-process. Production systems use Redis (with Lua scripts for atomicity) or a distributed rate-limiting service.

2. **Clock monotonicity**: `time.Now()` can jump backward (NTP adjustments). Production code should use `time.Since()` with a monotonic clock or handle clock drift.

3. **Memory management**: The per-key maps grow indefinitely. Production systems need eviction (LRU, TTL) or periodic cleanup goroutines.

4. **Response headers**: API gateways return rate-limit headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`). The interface could be extended to return this metadata.

5. **Weighted requests**: Some endpoints cost more than others. Token bucket naturally supports this (consume N tokens per request).

## Alternative Approaches

### Function-Based Strategy

Instead of an interface, you could use a function type:

```go
type AllowFunc func(key string) bool

func NewTokenBucket(capacity, refillRate float64) AllowFunc {
    // ... closure over state
    return func(key string) bool { ... }
}
```

This is simpler but loses the `Name()` method, making observability harder. For a rate limiter that needs to report which strategy is in use, the interface approach is better.

### Generic Strategy with Type Parameters

```go
type RateLimiter[C any] interface {
    Allow(key string) bool
    Config() C
    Name() string
}
```

This lets each strategy expose its own config type. Useful if you need to inspect or adjust strategy configuration at runtime, but adds complexity for minimal benefit in most cases.

### Strategy Registry with Dynamic Loading

```go
type Registry struct {
    factories map[string]func(config json.RawMessage) (RateLimiter, error)
}
```

Strategies are registered by name and created from configuration. This is the approach used by API gateway products (Kong, Envoy) where plugins define rate-limiting algorithms.
