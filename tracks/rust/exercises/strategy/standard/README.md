# Standard Exercise: API Rate Limiter with Pluggable Strategies

## Scenario

You're building middleware for an API gateway that enforces rate limits on incoming requests. Different endpoints need different rate limiting algorithms — the public search API uses a sliding window to smooth traffic, the webhook receiver uses a token bucket to allow short bursts, and internal health check endpoints use a simple fixed window. The rate limiter must be configured at startup and swapped without changing the gateway code.

## Brief

Implement a `RateLimiter` that accepts pluggable rate limiting strategies via trait objects. Each strategy decides whether a request should be allowed or rejected based on the client's request history.

Implement three strategies:
1. **Token Bucket** — Tokens refill at a fixed rate. Each request consumes one token. Allows bursts up to bucket capacity.
2. **Sliding Window** — Counts requests in a rolling time window. Rejects when count exceeds the limit.
3. **Fixed Window** — Counts requests in discrete time windows (e.g., per-minute). Resets at window boundaries.

## Acceptance Criteria

- [ ] Define a `RateLimitStrategy` trait with a method to check if a request is allowed
- [ ] Implement `TokenBucketStrategy` that allows bursts up to capacity and refills tokens over time
- [ ] Implement `SlidingWindowStrategy` that tracks request timestamps in a rolling window
- [ ] Implement `FixedWindowStrategy` that resets counts at window boundaries
- [ ] Create a `RateLimiter` struct that holds a `Box<dyn RateLimitStrategy>` and delegates to it
- [ ] The `RateLimiter` must work with any strategy interchangeably
- [ ] All tests in the starter pass when implementations are complete
- [ ] Handle edge cases: first request, window boundary transitions, bucket empty/refill

## Constraints

- Strategies must use `&mut self` (they track internal state like token count and timestamps)
- Use `std::time::Instant` for time tracking (the starter provides a `Clock` trait for testability)
- No external crates — standard library only
- The `RateLimitStrategy` trait must be object safe (no generic methods)

## Files

- `starter/main.rs` — Scaffold with trait definitions and TODOs
- `solutions/solution.rs` — Complete reference implementation
- `my-solution/` — Your implementation (copy starter here to begin)

## Getting Started

```bash
# Copy starter to your workspace
cp starter/main.rs my-solution/main.rs

# Run (will fail until you implement the TODOs)
cd my-solution
rustc main.rs && ./main
```

## Hints

<details>
<summary>Hint 1: Token bucket refill logic</summary>

When checking a request, calculate how much time has elapsed since the last refill. Add `elapsed * refill_rate` tokens to the bucket (capped at capacity). Then check if there's at least one token to consume.

```rust
let elapsed = now.duration_since(self.last_refill);
let new_tokens = elapsed.as_secs_f64() * self.refill_rate;
self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
```

</details>

<details>
<summary>Hint 2: Sliding window cleanup</summary>

Keep a `VecDeque<Instant>` of request timestamps. Before checking the limit, remove all timestamps older than `window_size` from the front. Then check if the remaining count is under the limit.

</details>

<details>
<summary>Hint 3: Fixed window boundaries</summary>

A fixed window resets at regular intervals. You can track the "current window start" as an `Instant`. When a new request arrives, check if `now - window_start >= window_size`. If so, reset the counter and update `window_start`.

</details>

<details>
<summary>Hint 4: Making it testable</summary>

The starter includes a `Clock` trait so you can inject a mock clock in tests. Your strategies should accept a `&dyn Clock` or use a generic `C: Clock` parameter to get the current time, rather than calling `Instant::now()` directly.

</details>

## Solution

After completing your implementation, compare with `solutions/solution.rs`. See `solutions/README.md` for discussion of the approach and tradeoffs.
