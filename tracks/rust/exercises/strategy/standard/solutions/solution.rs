// Rate Limiter with Pluggable Strategies — Reference Solution
//
// Demonstrates trait objects (Box<dyn RateLimitStrategy>) for runtime
// strategy selection. Each strategy tracks per-client state independently.
//
// Run: rustc solution.rs && ./solution
// Test: rustc --test solution.rs && ./solution

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Clock abstraction (for testability)
// ---------------------------------------------------------------------------

trait Clock {
    fn now(&self) -> Instant;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

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
    Allowed { remaining: u64 },
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

trait RateLimitStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult;
    fn name(&self) -> &str;
    fn reset(&mut self, client_id: &str);
}

// ---------------------------------------------------------------------------
// Token Bucket Strategy
// ---------------------------------------------------------------------------

struct TokenBucketState {
    tokens: f64,
    last_refill: Instant,
}

struct TokenBucketStrategy {
    capacity: u64,
    refill_rate: f64, // tokens per second
    clients: HashMap<String, TokenBucketState>,
}

impl TokenBucketStrategy {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            clients: HashMap::new(),
        }
    }
}

impl RateLimitStrategy for TokenBucketStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        let now = clock.now();
        let capacity = self.capacity;
        let refill_rate = self.refill_rate;

        let state = self.clients.entry(client_id.to_string()).or_insert_with(|| {
            TokenBucketState {
                tokens: capacity as f64,
                last_refill: now,
            }
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(state.last_refill);
        let new_tokens = elapsed.as_secs_f64() * refill_rate;
        state.tokens = (state.tokens + new_tokens).min(capacity as f64);
        state.last_refill = now;

        // Try to consume one token
        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            RateLimitResult::Allowed {
                remaining: state.tokens as u64,
            }
        } else {
            // Calculate how long until one token is available
            let deficit = 1.0 - state.tokens;
            let wait_secs = deficit / refill_rate;
            RateLimitResult::Rejected {
                retry_after: Duration::from_secs_f64(wait_secs),
            }
        }
    }

    fn name(&self) -> &str {
        "token_bucket"
    }

    fn reset(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }
}

// ---------------------------------------------------------------------------
// Sliding Window Strategy
// ---------------------------------------------------------------------------

struct SlidingWindowStrategy {
    max_requests: u64,
    window_size: Duration,
    clients: HashMap<String, VecDeque<Instant>>,
}

impl SlidingWindowStrategy {
    fn new(max_requests: u64, window_size: Duration) -> Self {
        Self {
            max_requests,
            window_size,
            clients: HashMap::new(),
        }
    }
}

impl RateLimitStrategy for SlidingWindowStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        let now = clock.now();
        let window_start = now.checked_sub(self.window_size).unwrap_or(now);

        let timestamps = self
            .clients
            .entry(client_id.to_string())
            .or_insert_with(VecDeque::new);

        // Evict expired timestamps from the front
        while let Some(&front) = timestamps.front() {
            if front < window_start {
                timestamps.pop_front();
            } else {
                break;
            }
        }

        if (timestamps.len() as u64) < self.max_requests {
            timestamps.push_back(now);
            let remaining = self.max_requests - timestamps.len() as u64;
            RateLimitResult::Allowed { remaining }
        } else {
            // The oldest request in the window determines when a slot opens
            let oldest = timestamps.front().copied().unwrap_or(now);
            let retry_after = (oldest + self.window_size).duration_since(now);
            RateLimitResult::Rejected { retry_after }
        }
    }

    fn name(&self) -> &str {
        "sliding_window"
    }

    fn reset(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }
}

// ---------------------------------------------------------------------------
// Fixed Window Strategy
// ---------------------------------------------------------------------------

struct FixedWindowState {
    window_start: Instant,
    count: u64,
}

struct FixedWindowStrategy {
    max_requests: u64,
    window_size: Duration,
    clients: HashMap<String, FixedWindowState>,
}

impl FixedWindowStrategy {
    fn new(max_requests: u64, window_size: Duration) -> Self {
        Self {
            max_requests,
            window_size,
            clients: HashMap::new(),
        }
    }
}

impl RateLimitStrategy for FixedWindowStrategy {
    fn check(&mut self, client_id: &str, clock: &dyn Clock) -> RateLimitResult {
        let now = clock.now();
        let window_size = self.window_size;

        let state = self.clients.entry(client_id.to_string()).or_insert_with(|| {
            FixedWindowState {
                window_start: now,
                count: 0,
            }
        });

        // Check if current window has expired
        let elapsed = now.duration_since(state.window_start);
        if elapsed >= window_size {
            state.window_start = now;
            state.count = 0;
        }

        if state.count < self.max_requests {
            state.count += 1;
            let remaining = self.max_requests - state.count;
            RateLimitResult::Allowed { remaining }
        } else {
            let window_end = state.window_start + window_size;
            let retry_after = window_end.duration_since(now);
            RateLimitResult::Rejected { retry_after }
        }
    }

    fn name(&self) -> &str {
        "fixed_window"
    }

    fn reset(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }
}

// ---------------------------------------------------------------------------
// RateLimiter (Context)
// ---------------------------------------------------------------------------

struct RateLimiter {
    strategy: Box<dyn RateLimitStrategy>,
    clock: Box<dyn Clock>,
    strategy_name: String,
}

impl RateLimiter {
    fn new(strategy: Box<dyn RateLimitStrategy>, clock: Box<dyn Clock>) -> Self {
        let strategy_name = strategy.name().to_string();
        Self {
            strategy,
            clock,
            strategy_name,
        }
    }

    fn check(&mut self, client_id: &str) -> RateLimitResult {
        self.strategy.check(client_id, self.clock.as_ref())
    }

    fn reset_client(&mut self, client_id: &str) {
        self.strategy.reset(client_id);
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

        for i in 0..5 {
            let result = strategy.check("client-1", &clock);
            assert!(result.is_allowed(), "request {} should be allowed", i + 1);
        }

        let result = strategy.check("client-1", &clock);
        assert!(!result.is_allowed(), "6th request should be rejected");
    }

    #[test]
    fn token_bucket_refills_over_time() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(3, 1.0);

        for _ in 0..3 {
            strategy.check("client-1", &clock);
        }
        assert!(!strategy.check("client-1", &clock).is_allowed());

        clock.advance(Duration::from_secs(2));
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn token_bucket_tracks_clients_independently() {
        let clock = MockClock::new();
        let mut strategy = TokenBucketStrategy::new(2, 1.0);

        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

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

        strategy.check("client-1", &clock);
        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

        clock.advance(Duration::from_secs(11));

        assert!(strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn sliding_window_tracks_clients_independently() {
        let clock = MockClock::new();
        let mut strategy = SlidingWindowStrategy::new(1, Duration::from_secs(60));

        strategy.check("client-1", &clock);
        assert!(!strategy.check("client-1", &clock).is_allowed());

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

        clock.advance(Duration::from_secs(11));

        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(strategy.check("client-1", &clock).is_allowed());
        assert!(!strategy.check("client-1", &clock).is_allowed());
    }

    #[test]
    fn fixed_window_returns_retry_after() {
        let clock = MockClock::new();
        let mut strategy = FixedWindowStrategy::new(1, Duration::from_secs(60));

        strategy.check("client-1", &clock);

        clock.advance(Duration::from_secs(20));

        if let RateLimitResult::Rejected { retry_after } = strategy.check("client-1", &clock) {
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
    println!("=== Rate Limiter — Reference Solution ===\n");

    // Demo 1: Token Bucket
    println!("--- Token Bucket (capacity=5, refill=2/sec) ---");
    let mut limiter = RateLimiter::new(
        Box::new(TokenBucketStrategy::new(5, 2.0)),
        Box::new(SystemClock),
    );

    for i in 1..=7 {
        let result = limiter.check("demo-client");
        println!("  Request {}: {:?}", i, result);
    }

    // Demo 2: Sliding Window
    println!("\n--- Sliding Window (max=3, window=60s) ---");
    let mut limiter = RateLimiter::new(
        Box::new(SlidingWindowStrategy::new(3, Duration::from_secs(60))),
        Box::new(SystemClock),
    );

    for i in 1..=5 {
        let result = limiter.check("demo-client");
        println!("  Request {}: {:?}", i, result);
    }

    // Demo 3: Fixed Window
    println!("\n--- Fixed Window (max=2, window=30s) ---");
    let mut limiter = RateLimiter::new(
        Box::new(FixedWindowStrategy::new(2, Duration::from_secs(30))),
        Box::new(SystemClock),
    );

    for i in 1..=4 {
        let result = limiter.check("demo-client");
        println!("  Request {}: {:?}", i, result);
    }

    // Demo 4: Strategy swap at runtime
    println!("\n--- Strategy Swap (shows dynamic dispatch) ---");
    let strategies: Vec<Box<dyn RateLimitStrategy>> = vec![
        Box::new(TokenBucketStrategy::new(2, 1.0)),
        Box::new(SlidingWindowStrategy::new(2, Duration::from_secs(60))),
        Box::new(FixedWindowStrategy::new(2, Duration::from_secs(60))),
    ];

    for strategy in strategies {
        let mut limiter = RateLimiter::new(strategy, Box::new(SystemClock));
        let r1 = limiter.check("client-x");
        let r2 = limiter.check("client-x");
        let r3 = limiter.check("client-x");
        println!(
            "  {}: allowed={}, allowed={}, allowed={}",
            limiter.strategy_name(),
            r1.is_allowed(),
            r2.is_allowed(),
            r3.is_allowed()
        );
    }
}
