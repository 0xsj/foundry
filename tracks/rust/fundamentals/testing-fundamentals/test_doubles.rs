// test_doubles.rs — Trait-Based Test Doubles (Fakes)
//
// Covers: trait extraction for dependency injection, fake implementations,
//         Cell<T> for interior mutability in fakes, multiple trait implementations,
//         testing time-dependent behavior without real clocks
//
// Run: rustc --test test_doubles.rs && ./test_doubles

use std::cell::Cell;
use std::collections::VecDeque;

// --------------------------------------------------------------------------
// Clock abstraction — the key dependency we want to control in tests
// --------------------------------------------------------------------------

/// Abstraction over a time source. Production code uses SystemClock;
/// tests use FakeClock to control time precisely.
pub trait Clock {
    fn now_millis(&self) -> u64;
}

// --------------------------------------------------------------------------
// Production implementation
// --------------------------------------------------------------------------

pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        // In a real program: SystemTime::now()...as_millis() as u64
        // Here we return 0 so this file compiles standalone.
        0
    }
}

// --------------------------------------------------------------------------
// Audit log abstraction — a second dependency for the rate limiter
// --------------------------------------------------------------------------

/// Records rate limit decisions for auditing. Production: writes to a log file.
/// In tests: records to an in-memory list for assertion.
pub trait AuditLog {
    fn record_allowed(&mut self, key: &str, timestamp_millis: u64);
    fn record_blocked(&mut self, key: &str, timestamp_millis: u64);
}

/// Production implementation (minimal stub for this example)
pub struct NoopAuditLog;

impl AuditLog for NoopAuditLog {
    fn record_allowed(&mut self, _key: &str, _ts: u64) {}
    fn record_blocked(&mut self, _key: &str, _ts: u64) {}
}

// --------------------------------------------------------------------------
// Rate limiter — takes both dependencies via generics
// --------------------------------------------------------------------------

pub struct RateLimiter<C: Clock, A: AuditLog> {
    capacity: u32,
    window_millis: u64,
    entries: VecDeque<u64>, // timestamps of recent requests
    clock: C,
    audit: A,
}

impl<C: Clock, A: AuditLog> RateLimiter<C, A> {
    pub fn new(capacity: u32, window_millis: u64, clock: C, audit: A) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        assert!(window_millis > 0, "window_millis must be > 0");
        RateLimiter {
            capacity,
            window_millis,
            entries: VecDeque::new(),
            clock,
            audit,
        }
    }

    pub fn check(&mut self, key: &str) -> bool {
        let now = self.clock.now_millis();

        // Evict entries older than the window
        while let Some(&front) = self.entries.front() {
            if now - front >= self.window_millis {
                self.entries.pop_front();
            } else {
                break;
            }
        }

        if self.entries.len() < self.capacity as usize {
            self.entries.push_back(now);
            self.audit.record_allowed(key, now);
            true
        } else {
            self.audit.record_blocked(key, now);
            false
        }
    }

    pub fn available_tokens(&self) -> usize {
        self.capacity as usize - self.entries.len()
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // FakeClock — controllable time source
    //
    // The Clock trait takes &self (shared reference) for now_millis().
    // We need to advance time via &self too (because we pass the clock into
    // the limiter and want to keep a reference to advance it from outside).
    // Cell<T> provides interior mutability for Copy types like u64.
    // ------------------------------------------------------------------

    struct FakeClock {
        now: Cell<u64>,
    }

    impl FakeClock {
        fn new(start: u64) -> Self {
            FakeClock { now: Cell::new(start) }
        }

        /// Advance the fake clock by `millis` milliseconds.
        /// Takes &self — works through Cell<T>'s interior mutability.
        fn advance(&self, millis: u64) {
            self.now.set(self.now.get() + millis);
        }
    }

    impl Clock for FakeClock {
        fn now_millis(&self) -> u64 {
            self.now.get()
        }
    }

    // ------------------------------------------------------------------
    // SpyAuditLog — records decisions for assertion
    //
    // A "spy" is a test double that records what happened for later verification.
    // Unlike a mock (which has expectations set before the call), a spy
    // records calls and you assert on them afterwards.
    // ------------------------------------------------------------------

    #[derive(Default)]
    struct SpyAuditLog {
        allowed: Vec<(String, u64)>,
        blocked: Vec<(String, u64)>,
    }

    impl AuditLog for SpyAuditLog {
        fn record_allowed(&mut self, key: &str, timestamp_millis: u64) {
            self.allowed.push((key.to_string(), timestamp_millis));
        }

        fn record_blocked(&mut self, key: &str, timestamp_millis: u64) {
            self.blocked.push((key.to_string(), timestamp_millis));
        }
    }

    impl SpyAuditLog {
        fn allowed_count(&self) -> usize { self.allowed.len() }
        fn blocked_count(&self) -> usize { self.blocked.len() }

        fn was_allowed(&self, key: &str) -> bool {
            self.allowed.iter().any(|(k, _)| k == key)
        }

        fn was_blocked(&self, key: &str) -> bool {
            self.blocked.iter().any(|(k, _)| k == key)
        }
    }

    // ------------------------------------------------------------------
    // Helper: create a test limiter with both fakes
    // ------------------------------------------------------------------

    fn test_limiter(
        capacity: u32,
        window_millis: u64,
        start_time: u64,
    ) -> RateLimiter<FakeClock, SpyAuditLog> {
        RateLimiter::new(
            capacity,
            window_millis,
            FakeClock::new(start_time),
            SpyAuditLog::default(),
        )
    }

    // ------------------------------------------------------------------
    // Tests using FakeClock
    // ------------------------------------------------------------------

    #[test]
    fn allows_requests_within_capacity() {
        let mut limiter = test_limiter(3, 60_000, 0);
        assert!(limiter.check("user:alice"));
        assert!(limiter.check("user:alice"));
        assert!(limiter.check("user:alice"));
        assert_eq!(limiter.available_tokens(), 0);
    }

    #[test]
    fn blocks_requests_when_capacity_exhausted() {
        let mut limiter = test_limiter(2, 60_000, 0);
        limiter.check("user:bob");
        limiter.check("user:bob");
        assert!(!limiter.check("user:bob"), "3rd request should be blocked");
    }

    #[test]
    fn window_resets_frees_capacity() {
        let mut limiter = test_limiter(2, 1_000, 0);
        limiter.check("user:carol"); // t=0
        limiter.check("user:carol"); // t=0 — capacity exhausted
        assert!(!limiter.check("user:carol")); // still at t=0, blocked

        // Advance past window
        limiter.clock.advance(1_001);
        assert!(limiter.check("user:carol"), "should be allowed after window reset");
    }

    #[test]
    fn sliding_window_evicts_only_old_entries() {
        let mut limiter = test_limiter(3, 1_000, 0);
        limiter.check("key"); // t=0
        limiter.clock.advance(500);
        limiter.check("key"); // t=500ms
        limiter.clock.advance(600);
        // Now at t=1100ms. Entry at t=0 is 1100ms old → evicted.
        // Entry at t=500ms is 600ms old → still in window.
        // Capacity: 3, entries in window: 1, available: 2
        assert_eq!(limiter.available_tokens(), 2);
    }

    // ------------------------------------------------------------------
    // Tests using SpyAuditLog
    // ------------------------------------------------------------------

    #[test]
    fn audit_log_records_allowed_requests() {
        let mut limiter = test_limiter(5, 60_000, 0);
        limiter.check("user:alice");
        limiter.check("user:alice");

        assert_eq!(limiter.audit.allowed_count(), 2);
        assert_eq!(limiter.audit.blocked_count(), 0);
        assert!(limiter.audit.was_allowed("user:alice"));
    }

    #[test]
    fn audit_log_records_blocked_requests() {
        let mut limiter = test_limiter(1, 60_000, 0);
        limiter.check("user:dave"); // allowed
        limiter.check("user:dave"); // blocked

        assert_eq!(limiter.audit.allowed_count(), 1);
        assert_eq!(limiter.audit.blocked_count(), 1);
        assert!(limiter.audit.was_blocked("user:dave"));
    }

    #[test]
    fn audit_log_captures_timestamps() {
        let mut limiter = test_limiter(5, 60_000, 1_000_000);
        limiter.check("svc:api");
        limiter.clock.advance(500);
        limiter.check("svc:api");

        let timestamps: Vec<u64> = limiter.audit.allowed.iter().map(|(_, t)| *t).collect();
        assert_eq!(timestamps, vec![1_000_000, 1_000_500]);
    }

    // ------------------------------------------------------------------
    // Testing the fake itself (optional but demonstrates good discipline)
    // ------------------------------------------------------------------

    #[test]
    fn fake_clock_advances_correctly() {
        let clock = FakeClock::new(5_000);
        assert_eq!(clock.now_millis(), 5_000);
        clock.advance(1_000);
        assert_eq!(clock.now_millis(), 6_000);
        clock.advance(500);
        assert_eq!(clock.now_millis(), 6_500);
    }

    #[test]
    fn fake_clock_can_be_shared_and_advanced_from_outside_limiter() {
        // This test illustrates WHY we used Cell<u64> instead of a plain u64 field.
        // We keep a reference to the clock, pass it to the limiter, then advance it.
        // Without interior mutability, we couldn't do this after moving the clock.
        //
        // The FakeClock is moved into the limiter. We can still call advance()
        // via limiter.clock because it takes &self (not &mut self).
        let limiter = test_limiter(3, 1_000, 0);
        limiter.clock.advance(500);
        assert_eq!(limiter.clock.now_millis(), 500);
    }
}

fn main() {
    println!("Run with: rustc --test test_doubles.rs && ./test_doubles");
    println!("Key ideas:");
    println!("  - Extract dependencies into traits (Clock, AuditLog)");
    println!("  - Inject via generics: RateLimiter<C: Clock, A: AuditLog>");
    println!("  - FakeClock: Cell<T> for interior mutability through &self");
    println!("  - SpyAuditLog: record calls, assert afterwards");
}
