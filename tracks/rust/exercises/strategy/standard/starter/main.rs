// Rate Limiter with Pluggable Strategies
//
// Implement the three rate limiting strategies and the RateLimiter context.
// Run: rustc main.rs && ./main

use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Clock abstraction (for testability)
// ---------------------------------------------------------------------------

/// Abstraction over time so we can mock it in tests.
trait Clock {
    fn now(&self) -> Instant;
}

/// Real clock — uses Instant::now().
struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Mock clock for testing — you control what "now" returns.
struct MockClock {
    current: std::cell::Cell<Instant>,
}

impl MockClock {
    fn new() -> Self {
        Self {
            current: std::cell::Cell::new(Instant::now()),
        }
    }

    fn advance(&self, duration: Duration) {
        let now = self.current.get();
        self.current.set(now + duration);
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        self.current.get()
    }
}

// ---------------------------------------------------------------------------
// Rate limit result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum RateLimitResult {
    /// Request is allowed. Contains remaining quota information.
    Allowed { remaining: u64 },
    /// Request is rejected. Contains when the client can retry.
    Rejected { retry_after: Duration },
}

impl RateLimitResult {
    fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }
}

// ---------------------------------------------------------------------------
// Strategy trait
// ---------------------------------------------------------------------------

/// A rate limiting strategy that decides whether to allow or reject a request.
///
/// Requirements:
/// - Must be object safe (no generic methods, no Self return)
/// - Takes &mut self because strategies track internal state
/// - Accepts &dyn Clock for testability
trait RateLimitStrategy {
    /// Check if a request from the given client_id should be allowed.
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult;

    /// Human-readable name for logging.
    fn name(&self) -> &str;

    /// Reset state for a specific client (e.g., when admin clears limits).
    fn reset(&mut self, client_id: &str);
}

// ---------------------------------------------------------------------------
// TODO: Implement TokenBucketStrategy
// ---------------------------------------------------------------------------

/// Token Bucket: Tokens refill at a steady rate. Each request consumes one token.
/// Allows bursts up to the bucket capacity.
///
/// Fields you'll need:
/// - capacity: u64 — maximum tokens in the bucket
/// - refill_rate: f64 — tokens added per second
/// - Per-client state: current token count and last refill time
struct TokenBucketStrategy {
    capacity: u64,
    refill_rate: f64,
    // TODO: Add a HashMap to track per-client state
    // Each client needs: current tokens (f64) and last_refill (Instant)
}

impl TokenBucketStrategy {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        // TODO: Initialize the strategy
        todo!("Initialize TokenBucketStrategy")
    }
}

impl RateLimitStrategy for TokenBucketStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        // TODO: Implement token bucket logic
        //
        // Steps:
        // 1. Get or create client state (start with full bucket)
        // 2. Calculate elapsed time since last refill
        // 3. Add tokens: elapsed_secs * refill_rate, capped at capacity
        // 4. If tokens >= 1.0, consume one and return Allowed
        // 5. If tokens < 1.0, calculate retry_after and return Rejected
        //
        // retry_after = Duration from ((1.0 - tokens) / refill_rate) seconds
        todo!("Implement token bucket check")
    }

    fn name(&self) -> &str {
        "token_bucket"
    }

    fn reset(&mut self, client_id: &str) {
        // TODO: Remove client state so next request starts fresh
        todo!("Implement reset")
    }
}

// ---------------------------------------------------------------------------
// TODO: Implement SlidingWindowStrategy
// ---------------------------------------------------------------------------

/// Sliding Window: Counts requests in a rolling time window.
/// Rejects when count exceeds the limit.
///
/// Fields you'll need:
/// - max_requests: u64 — maximum requests allowed in the window
/// - window_size: Duration — the rolling window duration
/// - Per-client state: VecDeque<Instant> of request timestamps
struct SlidingWindowStrategy {
    max_requests: u64,
    window_size: Duration,
    // TODO: Add a HashMap to track per-client request timestamps
}

impl SlidingWindowStrategy {
    fn new(max_requests: u64, window_size: Duration) -> Self {
        // TODO: Initialize the strategy
        todo!("Initialize SlidingWindowStrategy")
    }
}

impl RateLimitStrategy for SlidingWindowStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        // TODO: Implement sliding window logic
        //
        // Steps:
        // 1. Get or create client's timestamp queue
        // 2. Remove all timestamps older than (now - window_size) from front
        // 3. If queue length < max_requests, add current timestamp, return Allowed
        // 4. If queue length >= max_requests, calculate retry_after from oldest timestamp
        //
        // retry_after = oldest_timestamp + window_size - now
        todo!("Implement sliding window check")
    }

    fn name(&self) -> &str {
        "sliding_window"
    }

    fn reset(&mut self, client_id: &str) {
        // TODO: Remove client's timestamp history
        todo!("Implement reset")
    }
}

// ---------------------------------------------------------------------------
// TODO: Implement FixedWindowStrategy
// ---------------------------------------------------------------------------

/// Fixed Window: Counts requests in discrete time windows.
/// Counter resets at window boundaries.
///
/// Fields you'll need:
/// - max_requests: u64 — maximum requests per window
/// - window_size: Duration — window duration
/// - Per-client state: window start time and current count
struct FixedWindowStrategy {
    max_requests: u64,
    window_size: Duration,
    // TODO: Add a HashMap to track per-client window state
    // Each client needs: window_start (Instant) and count (u64)
}

impl FixedWindowStrategy {
    fn new(max_requests: u64, window_size: Duration) -> Self {
        // TODO: Initialize the strategy
        todo!("Initialize FixedWindowStrategy")
    }
}

impl RateLimitStrategy for FixedWindowStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        // TODO: Implement fixed window logic
        //
        // Steps:
        // 1. Get or create client's window state
        // 2. If (now - window_start) >= window_size, reset: window_start = now, count = 0
        // 3. If count < max_requests, increment count, return Allowed
        // 4. If count >= max_requests, calculate retry_after from window end
        //
        // retry_after = window_start + window_size - now
        todo!("Implement fixed window check")
    }

    fn name(&self) -> &str {
        "fixed_window"
    }

    fn reset(&mut self, client_id: &str) {
        // TODO: Remove client's window state
        todo!("Implement reset")
    }
}

// ---------------------------------------------------------------------------
// TODO: Implement RateLimiter (context)
// ---------------------------------------------------------------------------

/// The RateLimiter holds a strategy as a trait object and delegates to it.
/// This allows the gateway to swap strategies at configuration time without
/// changing any calling code.
struct RateLimiter {
    // TODO: Add field for the strategy (Box<dyn RateLimitStrategy>)
    // TODO: Add field for the clock (Box<dyn Clock>)
    strategy_name: String, // for logging
}

impl RateLimiter {
    fn new(strategy: Box<dyn RateLimitStrategy>, clock: Box<dyn Clock>) -> Self {
        // TODO: Initialize with the given strategy and clock
        todo!("Initialize RateLimiter")
    }

    fn check(&mut self, client_id: &str) -> RateLimitResult {
        // TODO: Delegate to the strategy
        todo!("Delegate to strategy")
    }

    fn reset_client(&mut self, client_id: &str) {
        // TODO: Delegate reset to the strategy
        todo!("Delegate reset to strategy")
    }

    fn strategy_name(&self) -> &str {
        &self.strategy_name
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Token Bucket Tests ---

    #[test]
    fn token_bucket_allows_burst_up_to_capacity() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(5, 1.0);

        // Should allow 5 requests immediately (full bucket)
        for i in 0..5 {
            let result = strategy.check("client-1", &clock);
            assert!(
                result.is_allowed(),
                "request {} should be allowed",
                i + 1
            );
        }

        // 6th request should be rejected
        let result = strategy.check("client-1", &clock);
        assert!(
            !result.is_allowed(),
            "6th request should be rejected"
        );
    }

    #[test]
    fn token_bucket_refills_over_time() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(3, 1.0); // 1 token/sec

        // Drain all tokens
        for _ in 0..3 {
            strategy.check("client-1", &clock);
        }
        assert!(!strategy.check("client-1", &clock).is_allowed());

        // Advance 2 seconds — should have 2 tokens
        clock.advance(Duration::from_secs(2));
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn token_bucket_tracks_clients_independently() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(2, 1.0);

        // Drain client-1
        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        // client-2 should still have tokens
        assert!(strategy.check("client-2", &clock).is_allowed());
    }

    #[test]
    fn token_bucket_returns_remaining() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(3, 1.0);

        if let RateLimitResult::Allowed { remaining } = strategy.check("client-1", &clock) {
            assert_eq!(remaining, 2);
        } else {
            panic!("should be allowed");
        }
    }

    #[test]
    fn token_bucket_reset_restores_tokens() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(2, 1.0);

        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        strategy.reset("client-1");
        assert!(strategy.check("client-1", &clock).is_allowed());
    }

    // --- Sliding Window Tests ---

    #[test]
    fn sliding_window_allows_up_to_limit() {
        let clock = MockClock::new();
        let mut strategy = SlidingWindowStrategy::new(3, Duration::from_secs(60));

        for i in 0..3 {
            let result = strategy.check("client-1", &clock);
            assert!(result.is_allowed(), "request {} should be allowed", i + 1);
        }

        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn sliding_window_expires_old_requests() {
        let clock = MockClock::new();
        let mut strategy = SlidingWindowStrategy::new(2, Duration::from_secs(10));

        // Two requests at t=0
        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        // Advance past the window
        clock.advance(Duration::from_secs(11));

        // Old requests expired, should be allowed again
        assert!(strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn sliding_window_tracks_clients_independently() {
        let clock = MockClock::new();
        let mut strategy = SlidingWindowStrategy::new(1, Duration::from_secs(60));

        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        // Different client is independent
        assert!(strategy.check("client-2", &clock).is_allowed());
    }

    // --- Fixed Window Tests ---

    #[test]
    fn fixed_window_allows_up_to_limit() {
        let clock = MockClock::new();
        let mut strategy = FixedWindowStrategy::new(3, Duration::from_secs(60));

        for i in 0..3 {
            let result = strategy.check("client-1", &clock);
            assert!(result.is_allowed(), "request {} should be allowed", i + 1);
        }

        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn fixed_window_resets_at_boundary() {
        let clock = MockClock::new();
        let mut strategy = FixedWindowStrategy::new(2, Duration::from_secs(10));

        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        // Advance past window boundary
        clock.advance(Duration::from_secs(11));

        // Window reset — full quota again
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn fixed_window_returns_retry_after() {
        let clock = MockClock::new();
        let mut strategy = FixedWindowStrategy::new(1, Duration::from_secs(60));

        strategy.check("client-1", &clock);

        // Advance 20 seconds into the window
        clock.advance(Duration::from_secs(20));

        if let RateLimitResult::Rejected { retry_after } = strategy.check("client-1", &clock) {
            // Should wait ~40 seconds until window resets
            assert!(
                retry_after.as_secs() >= 39 && retry_after.as_secs() <= 41,
                "retry_after should be ~40s, got {:?}",
                retry_after
            );
        } else {
            panic!("should be rejected");
        }
    }

    // --- RateLimiter (Context) Tests ---

    #[test]
    fn rate_limiter_delegates_to_strategy() {
        let mut limiter = RateLimiter::new(
            Box::new(TokenBucketStrategy::new(2, 1.0)),
            Box::new(MockClock::new()),
        );

        assert!(limiter.check("client-1").is_allowed());
        assert!(limiter.check("client-1").is_allowed());
        assert!(!limiter.check("client-1").is_allowed());
    }

    #[test]
    fn rate_limiter_reports_strategy_name() {
        let limiter = RateLimiter::new(
            Box::new(SlidingWindowStrategy::new(10, Duration::from_secs(60))),
            Box::new(MockClock::new()),
        );
        assert_eq!(limiter.strategy_name(), "sliding_window");
    }

    #[test]
    fn rate_limiter_reset_delegates() {
        let mut limiter = RateLimiter::new(
            Box::new(FixedWindowStrategy::new(1, Duration::from_secs(60))),
            Box::new(MockClock::new()),
        );

        limiter.check("client-1");
        assert!(!limiter.check("client-1").is_allowed());

        limiter.reset_client("client-1");
        assert!(limiter.check("client-1").is_allowed());
    }
}

// ---------------------------------------------------------------------------
// main — demo usage
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Rate Limiter Exercise ===\n");
    println!("Run tests with: rustc --test main.rs && ./main");
    println!("Or implement the TODOs and run: rustc main.rs && ./main\n");

    // Demo: create a rate limiter with token bucket strategy
    let mut limiter = RateLimiter::new(
        Box::new(TokenBucketStrategy::new(5, 2.0)),
        Box::new(SystemClock),
    );

    println!("Strategy: {}", limiter.strategy_name());
    println!("Sending 7 requests for client-1:\n");

    for i in 1..=7 {
        let result = limiter.check("client-1");
        println!("  Request {}: {:?}", i, result);
    }

    println!("\nResetting client-1...");
    limiter.reset_client("client-1");

    println!("After reset:");
    let result = limiter.check("client-1");
    println!("  Request 1: {:?}", result);
}
