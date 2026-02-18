// buggy.rs — Debugging Exercise: Broken Test Suite
//
// The rate limiter below is CORRECT. The bugs are all in the test code.
// Find and fix all four bugs. Read the test names — they describe what
// the test should verify. A passing test with a misleading name is also a bug.
//
// Run: rustc --test buggy.rs && ./buggy

use std::cell::Cell;
use std::collections::VecDeque;

// --------------------------------------------------------------------------
// Production code — DO NOT MODIFY
// --------------------------------------------------------------------------

pub trait Clock {
    fn now_millis(&self) -> u64;
}

pub struct RateLimiter<C: Clock> {
    capacity: u32,
    window_millis: u64,
    entries: VecDeque<u64>,
    pub clock: C,
}

impl<C: Clock> RateLimiter<C> {
    pub fn new(capacity: u32, window_millis: u64, clock: C) -> Self {
        if capacity == 0 {
            panic!("capacity must be > 0, got 0");
        }
        if window_millis == 0 {
            panic!("window_millis must be > 0, got 0");
        }
        RateLimiter {
            capacity,
            window_millis,
            entries: VecDeque::new(),
            clock,
        }
    }

    pub fn check(&mut self) -> bool {
        let now = self.clock.now_millis();
        while let Some(&front) = self.entries.front() {
            if now.saturating_sub(front) >= self.window_millis {
                self.entries.pop_front();
            } else {
                break;
            }
        }
        if self.entries.len() < self.capacity as usize {
            self.entries.push_back(now);
            true
        } else {
            false
        }
    }

    pub fn available_tokens(&self) -> usize {
        self.capacity as usize - self.entries.len()
    }
}

// --------------------------------------------------------------------------
// Test code — BUGS ARE IN HERE
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeClock {
        now: Cell<u64>,
    }

    impl FakeClock {
        fn new(start: u64) -> Self { FakeClock { now: Cell::new(start) } }
        fn advance(&self, millis: u64) { self.now.set(self.now.get() + millis); }
    }

    impl Clock for FakeClock {
        fn now_millis(&self) -> u64 { self.now.get() }
    }

    fn limiter(capacity: u32, window_millis: u64) -> RateLimiter<FakeClock> {
        RateLimiter::new(capacity, window_millis, FakeClock::new(0))
    }

    // ---- Bug 1: assert_eq! arguments are reversed ----
    //
    // This test is supposed to verify that after consuming 2 tokens from a
    // capacity-5 limiter, 3 tokens remain. But the assertion always passes,
    // even if the actual token count is wrong. Why?
    //
    // The failure message when a real bug exists would show the "wrong" values,
    // but because the arguments are swapped, it compares the wrong things and
    // may silently pass even when there's a regression.
    //
    // (Hint: assert_eq!(actual, expected) — which should be left, which right?
    // More importantly: what happens when both arguments are the same expression?)

    #[test]
    fn consuming_tokens_decrements_available_count() {
        let mut l = limiter(5, 60_000);
        l.check();
        l.check();
        // BUG: both arguments are the same expression
        assert_eq!(l.available_tokens(), l.available_tokens());
    }

    // ---- Bug 2: test body is empty — tests nothing ----
    //
    // This test always passes. It's supposed to verify that the rate limiter
    // blocks requests beyond its capacity. But the test body does nothing.
    // The test name is a lie.

    #[test]
    fn requests_beyond_capacity_are_blocked() {
        // BUG: this test body is completely empty — it always passes
        // regardless of whether blocking works or not
    }

    // ---- Bug 3: #[should_panic] without expected — too permissive ----
    //
    // This test is supposed to verify that creating a rate limiter with
    // zero capacity panics with the message "capacity must be > 0".
    // But the test also passes if, say, the panic is caused by an unrelated
    // array index out of bounds, or a different internal invariant violation.
    // Fix it so only the correct panic causes the test to pass.

    #[test]
    #[should_panic]
    fn zero_capacity_panics_with_correct_message() {
        RateLimiter::new(0, 1_000, FakeClock::new(0));
    }

    // ---- Bug 4: shared mutable state between tests (via static variable) ----
    //
    // This test uses a static mut counter to track how many times it has been
    // called. The test works in isolation but fails or produces unpredictable
    // results when the full test suite runs in parallel (the default).
    //
    // The fix: remove the shared state. Each test should be completely
    // independent. Capture what you need in a local variable.
    //
    // Note: rustc may warn about static_mut_refs; that warning points at the bug.

    static mut CALL_COUNT: u32 = 0;

    #[test]
    fn check_returns_true_for_first_request() {
        // BUG: mutating a static variable in a test that runs in parallel
        // with other tests causes a data race
        unsafe { CALL_COUNT += 1; }

        let mut l = limiter(5, 60_000);
        let result = l.check();
        // The actual assertion we care about:
        assert!(result, "first check() should return true");

        // This extra assertion is also wrong — CALL_COUNT can be any value
        // depending on which tests ran before this one, in which order
        unsafe { assert!(CALL_COUNT > 0, "call count should be positive"); }
    }
}

fn main() {}
