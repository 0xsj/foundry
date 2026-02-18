// solution_tests.rs — Reference Test Suite
//
// This is the complete solution for the testing-fundamentals standard exercise.
// All 6 acceptance criteria are addressed with inline comments explaining each choice.
//
// Run: rustc --test solution_tests.rs && ./solution_tests

#[path = "solution_rate_limiter.rs"]
mod rate_limiter;
use rate_limiter::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // ==========================================================================
    // FakeClock — Acceptance Criterion #2
    //
    // Design note: Clock::now_millis takes &self, so our advance method
    // must also take &self. Cell<u64> gives us interior mutability for a
    // Copy type without the overhead of RefCell or Mutex.
    //
    // This pattern comes up constantly: any time a trait requires &self but
    // you need mutation inside an implementation. Cell for Copy types,
    // RefCell for non-Copy types, Mutex/RwLock for thread safety.
    // ==========================================================================

    struct FakeClock {
        now: Cell<u64>,
    }

    impl FakeClock {
        fn new(start: u64) -> Self {
            FakeClock { now: Cell::new(start) }
        }

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
    // Fixture helper — reduces repetition across tests
    // ------------------------------------------------------------------

    fn limiter(capacity: u32, window_millis: u64) -> RateLimiter<FakeClock> {
        RateLimiter::new(capacity, window_millis, FakeClock::new(0))
    }

    // ==========================================================================
    // Acceptance Criterion #1: Unit tests for core behavior
    // ==========================================================================

    #[test]
    fn new_limiter_starts_with_full_capacity() {
        let l = limiter(5, 60_000);
        assert_eq!(l.available_tokens(), 5);
    }

    #[test]
    fn check_allows_requests_within_capacity() {
        let mut l = limiter(3, 60_000);
        assert!(l.check(), "1st request should be allowed");
        assert!(l.check(), "2nd request should be allowed");
        assert!(l.check(), "3rd request should be allowed");
    }

    #[test]
    fn check_decrements_available_tokens() {
        let mut l = limiter(5, 60_000);
        l.check();
        assert_eq!(l.available_tokens(), 4, "one token consumed");
        l.check();
        assert_eq!(l.available_tokens(), 3, "two tokens consumed");
    }

    #[test]
    fn check_blocks_requests_beyond_capacity() {
        let mut l = limiter(2, 60_000);
        l.check(); // 1
        l.check(); // 2
        assert!(!l.check(), "3rd request should be blocked (capacity=2)");
        assert_eq!(l.available_tokens(), 0);
    }

    #[test]
    fn window_reset_restores_full_capacity() {
        let mut l = limiter(3, 1_000);

        // Exhaust the capacity
        for _ in 0..3 { l.check(); }
        assert!(!l.check(), "should be blocked before window reset");

        // Jump past the window
        l.clock.advance(1_001);

        // Capacity should be fully restored
        assert!(l.check(), "should be allowed after window reset");
        assert_eq!(
            l.available_tokens(), 2,
            "3 restored, 1 consumed = 2 remaining"
        );
    }

    #[test]
    fn partial_capacity_restored_when_some_entries_still_in_window() {
        // At t=0: 2 requests. At t=600: 1 more request.
        // At t=1001: entries at t=0 are evicted (1001-0 = 1001 >= 1000).
        // Entry at t=600 is 401ms old — still in window.
        // So: capacity=3, in-window=1, available=2.
        let mut l = limiter(3, 1_000);
        l.check(); // t=0
        l.check(); // t=0
        l.clock.advance(600);
        l.check(); // t=600

        l.clock.advance(401); // t=1001
        // Entries at t=0 should be evicted (1001ms >= 1000ms window)
        // Entry at t=600 still in window (401ms < 1000ms)
        assert_eq!(
            l.available_tokens(), 2,
            "2 old entries evicted, 1 remains in window"
        );
    }

    #[test]
    fn window_boundary_is_exclusive() {
        // entry at t=0. now = 1000. 1000 - 0 = 1000 >= window_millis(1000)
        // So it IS evicted at t=1000. The boundary is exclusive on the old side.
        let mut l = limiter(1, 1_000);
        l.check(); // t=0, fills capacity
        l.clock.advance(1_000); // t=1000
        assert!(
            l.check(),
            "entry at t=0 should be evicted at t=1000 (age 1000 >= window 1000)"
        );
    }

    // ==========================================================================
    // Acceptance Criterion #3: #[should_panic] with expected
    //
    // Using bare #[should_panic] without expected is dangerous:
    // any panic passes, including unrelated panics (index out of bounds, unwrap, etc.).
    // The expected = "..." substring match pins the test to the specific invariant.
    // ==========================================================================

    #[test]
    #[should_panic(expected = "capacity must be > 0")]
    fn zero_capacity_panics_with_message() {
        RateLimiter::new(0, 1_000, FakeClock::new(0));
    }

    #[test]
    #[should_panic(expected = "window_millis must be > 0")]
    fn zero_window_panics_with_message() {
        RateLimiter::new(5, 0, FakeClock::new(0));
    }

    // ==========================================================================
    // Acceptance Criterion #4: Result-returning test
    //
    // The ? operator propagates parse errors cleanly.
    // Box<dyn std::error::Error> accepts any error type — most flexible.
    // If this test fails, the error is printed (no unwrap panic message).
    // ==========================================================================

    #[test]
    fn build_limiter_from_config_string() -> Result<(), Box<dyn std::error::Error>> {
        let (capacity, window_ms) = parse_rate_config("10/30s")?;
        let mut l = RateLimiter::new(capacity, window_ms, FakeClock::new(0));

        assert_eq!(l.capacity(), 10);
        assert_eq!(l.window_millis(), 30_000);
        assert!(l.check()); // should be allowed

        Ok(())
    }

    #[test]
    fn invalid_config_string_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        // This tests parse_rate_config returns Err for invalid input.
        // We use is_err() instead of ? because we expect an error here.
        assert!(parse_rate_config("bad").is_err());
        assert!(parse_rate_config("0/60s").is_err());
        assert!(parse_rate_config("10/60").is_err()); // missing 's'
        Ok(())
    }

    // ==========================================================================
    // Acceptance Criterion #6: #[ignore] test
    //
    // This simulates 5 windows of requests and checks overall ratios.
    // It's slow to reason about in a unit test sense (many iterations) but
    // not actually slow in wall-clock time — the point is the principle.
    //
    // Run with: ./solution_tests --include-ignored
    // ==========================================================================

    #[test]
    #[ignore]
    fn sustained_load_across_many_windows() {
        // capacity=10 per 1000ms window. Send 20 requests per window over 5 windows.
        // Expected: 10 allowed + 10 blocked per window = 50 + 50 total.
        let mut l = limiter(10, 1_000);
        let mut total_allowed = 0u32;
        let mut total_blocked = 0u32;
        let requests_per_window = 20u64;
        let windows = 5u64;

        for w in 0..windows {
            let window_start = w * 1_000;
            l.clock.advance(window_start.saturating_sub(l.clock.now_millis()));

            // Reset: advance to new window cleanly
            if w > 0 {
                l.clock.advance(1_000);
            }

            for r in 0..requests_per_window {
                // Space requests 10ms apart within the window
                if r > 0 { l.clock.advance(10); }
                if l.check() { total_allowed += 1; } else { total_blocked += 1; }
            }
        }

        let total = total_allowed + total_blocked;
        assert_eq!(
            total as u64,
            requests_per_window * windows,
            "total requests should equal sent requests"
        );
        assert!(total_blocked > 0, "some requests should have been blocked");
        assert!(total_allowed > 0, "some requests should have been allowed");
    }
}

fn main() {}
