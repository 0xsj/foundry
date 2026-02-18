# Solution — Rate Limiter with Pluggable Strategies

## Approach

The solution uses **trait objects** (`Box<dyn RateLimitStrategy>`) for the strategy dispatch. This is the right choice here because:

1. The strategy is chosen from configuration at startup — it's a runtime decision
2. We want to store different strategies in a `Vec` for the swap demo
3. The rate limiter middleware doesn't need to be generic over the strategy type
4. The dispatch overhead (vtable lookup) is negligible compared to the actual I/O that rate limiting gates

Each strategy maintains per-client state using a `HashMap<String, ClientState>`. This is important — a single rate limiter instance serves many clients, and each client has independent limits.

## Key Decisions

1. **`HashMap` for per-client state**: Each strategy stores client state in a `HashMap<String, _>`. An alternative would be to make the strategy per-client and have the `RateLimiter` manage the map, but that would complicate the trait interface and violate single responsibility — the strategy should own its algorithm including state management.

2. **`Clock` trait for testability**: Time-dependent code is notoriously hard to test. The `Clock` trait lets us inject a `MockClock` that we control, making tests deterministic and fast (no `sleep()` calls). In production, `SystemClock` simply delegates to `Instant::now()`.

3. **`&dyn Clock` (not generic)**: The `check` method takes `&dyn Clock` rather than being generic over `C: Clock`. This keeps the `RateLimitStrategy` trait object safe. If `check` were `fn check<C: Clock>(&mut self, client_id: &str, clock: &C)`, the trait would not be object safe (generic methods prevent `dyn Trait`).

4. **`RateLimitResult` enum**: Returns structured data (remaining quota or retry-after duration) rather than a simple boolean. This enables the HTTP layer to set proper `X-RateLimit-Remaining` and `Retry-After` headers.

5. **`Cell<Instant>` in MockClock**: We use `Cell` so the mock clock can be advanced via `&self` (shared reference). This avoids needing `&mut` access in tests where the clock is shared between the strategy and test assertions.

## Complexity Analysis

| Strategy | Time per `check()` | Space per client |
|----------|-------------------|------------------|
| Token Bucket | O(1) | O(1) — just tokens + timestamp |
| Sliding Window | O(k) where k = expired entries | O(n) — stores all timestamps in window |
| Fixed Window | O(1) | O(1) — just count + window start |

**Sliding window tradeoff:** The VecDeque stores every request timestamp within the window. For high-traffic clients (thousands of requests per window), this uses more memory than the other strategies. A production implementation might use a probabilistic approach (sliding window counter/log hybrid) to reduce memory.

## Strategy Comparison

| Strategy | Burst Handling | Boundary Behavior | Memory | Best For |
|----------|---------------|-------------------|--------|----------|
| Token Bucket | Allows bursts up to capacity | Smooth refill | Lowest | APIs that allow short bursts |
| Sliding Window | No bursts allowed | Smooth, no boundary effects | Highest | Strict per-second/minute limits |
| Fixed Window | No bursts within window | Boundary spike possible* | Low | Simple per-minute/hour quotas |

*Fixed window boundary problem: A client can make `max` requests at the end of one window and `max` at the start of the next, effectively doubling their rate at the boundary. The sliding window avoids this.

## Implementation Notes

- **Token refill precision**: We use `f64` for tokens to handle fractional refill rates smoothly. A production implementation might use integer math with millisecond precision to avoid floating-point drift.
- **`checked_sub` for Instant**: `Instant::checked_sub` handles the edge case where the window size exceeds the time since program start. Without this, subtracting a large Duration from a small Instant would panic.
- **No locking**: These strategies use `&mut self`, so they're not thread-safe as-is. In production, you'd wrap the `RateLimiter` in a `Mutex` or use `Arc<Mutex<dyn RateLimitStrategy>>`. A more sophisticated approach uses per-client sharded locks.

## Further Reading

- [[strategy]] — Cross-language strategy pattern comparison
- [[patterns/rust/strategy]] — Rust-specific strategy deep dive
- Token bucket algorithm: https://en.wikipedia.org/wiki/Token_bucket
- Sliding window rate limiting: https://blog.cloudflare.com/counting-things-a-lot-of-different-things/
- Go implementation comparison: `tracks/go/exercises/strategy/`
