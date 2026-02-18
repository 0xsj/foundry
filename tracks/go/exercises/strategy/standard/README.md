# Rate Limiter with Pluggable Strategies

## Scenario

Your team is building an API gateway that sits in front of several internal microservices. Different API consumers have different rate-limiting needs: free-tier users get a simple fixed-window limit, paid users get a token bucket (smoother, allows bursts), and enterprise clients get a sliding window (the most accurate but most expensive to compute). The gateway needs to select the right rate-limiting algorithm per request based on the client's tier, and new algorithms may be added as the product evolves.

## Brief

Implement a rate limiter system using the Strategy pattern. Define a `RateLimiter` interface that all strategies satisfy, then implement three concrete strategies: **Token Bucket**, **Fixed Window**, and **Sliding Window**. Build a `Gateway` that selects the appropriate limiter per client tier.

## Acceptance Criteria

- [ ] `RateLimiter` interface with `Allow(key string) bool` and `Name() string` methods
- [ ] `TokenBucketLimiter`: allows bursts up to capacity, refills at a steady rate
- [ ] `FixedWindowLimiter`: counts requests in fixed time windows, resets at window boundaries
- [ ] `SlidingWindowLimiter`: tracks request timestamps, counts within a rolling time window
- [ ] `Gateway` struct that maps client tiers to rate-limiting strategies
- [ ] `Gateway.HandleRequest(clientID string, tier string) (allowed bool, limiter string)` method
- [ ] All strategies are safe for concurrent use (multiple goroutines calling `Allow`)
- [ ] Compile-time interface guards for all three implementations
- [ ] All tests pass: `go test ./starter/`

## Constraints

- Do not use external packages -- standard library only
- Token bucket must support configurable capacity and refill rate
- Fixed window must support configurable window duration and max requests
- Sliding window must support configurable window duration and max requests
- The `Gateway` must handle unknown tiers gracefully (use a default limiter)
- Thread safety is required -- the test suite includes concurrent access tests

## Files

- `starter/main.go` -- Scaffold code with types and TODOs
- `starter/main_test.go` -- Full test suite (run with `go test`)

## Getting Started

```bash
cd starter
go test  # Should fail initially — implement the TODOs to make tests pass
```

## Hints

<details>
<summary>Hint 1: Token Bucket internals</summary>

A token bucket has a capacity (max tokens) and a refill rate (tokens per second). Each `Allow()` call consumes one token. Tokens are refilled based on elapsed time since the last refill. Use `time.Now()` and track `lastRefill time.Time` and `tokens float64`.

The key insight: don't run a background goroutine to refill. Instead, calculate how many tokens to add at each `Allow()` call based on elapsed time. This is called "lazy refill."

</details>

<details>
<summary>Hint 2: Fixed Window key design</summary>

A fixed window groups requests by time window. The window key is `floor(now / windowDuration)`. Use a `map[string]map[int64]int` where the outer key is the client ID and the inner key is the window number. Clean up old windows to prevent memory leaks.

</details>

<details>
<summary>Hint 3: Sliding Window with timestamps</summary>

For a sliding window, store the timestamps of recent requests in a slice per client. On each `Allow()` call, remove timestamps older than `now - windowDuration`, then check if the remaining count is below the limit. A `map[string][]time.Time` works well.

Be careful with slice memory: when removing old timestamps, re-slice from the front rather than creating a new slice each time.

</details>

<details>
<summary>Hint 4: Concurrency</summary>

Use `sync.Mutex` on each limiter. Lock at the start of `Allow()`, defer unlock. The mutex should protect all mutable state (token count, request counts, timestamp slices).

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
