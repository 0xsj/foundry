// proposed.rs — PR: "Add test coverage for RateLimiter"
//
// This file contains the production rate limiter (unchanged) and the
// proposed test suite. Review the test code for issues.
//
// Run: rustc --test proposed.rs && ./proposed

use std::cell::Cell;
use std::collections::VecDeque;

// --------------------------------------------------------------------------
// Production code (correct, do not review this section)
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

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn window_millis(&self) -> u64 {
        self.window_millis
    }

    // Private method — not part of public API
    fn evict_expired(&mut self, now: u64) {
        while let Some(&front) = self.entries.front() {
            if now.saturating_sub(front) >= self.window_millis {
                self.entries.pop_front();
            } else {
                break;
            }
        }
    }

    // Private field accessor — not part of public API
    fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

// --------------------------------------------------------------------------
// Proposed test suite — THIS IS WHAT YOU ARE REVIEWING
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Issue: The mock is tightly coupled to the implementation's internal
    // VecDeque structure. The test mock was designed around white-box knowledge
    // of how entries are stored, not around the Clock trait's contract.
    struct HardcodedEntryCountClock {
        calls: Cell<usize>,
        // Returns different timestamps based on the exact call sequence
        // that the current implementation happens to make internally.
        timestamps: Vec<u64>,
    }

    impl HardcodedEntryCountClock {
        fn new(timestamps: Vec<u64>) -> Self {
            HardcodedEntryCountClock { calls: Cell::new(0), timestamps }
        }
    }

    impl Clock for HardcodedEntryCountClock {
        fn now_millis(&self) -> u64 {
            let i = self.calls.get();
            self.calls.set(i + 1);
            // Returns timestamps in sequence based on call count.
            // Breaks if the implementation calls now_millis() a different
            // number of times (e.g., after refactoring).
            *self.timestamps.get(i).unwrap_or(&999_999)
        }
    }

    // Issue: Tests private implementation details
    // `evict_expired` and `entry_count` are private methods.
    // Testing them couples the test to the implementation, not the contract.
    #[test]
    fn evict_expired_removes_old_entries() {
        let clock = HardcodedEntryCountClock::new(vec![0, 0, 0, 1001]);
        let mut l = RateLimiter::new(3, 1000, clock);
        l.check(); // adds entry at t=0 (clock call 1)
        l.check(); // adds entry at t=0 (clock call 2)
        // Directly call the private method
        l.evict_expired(1001); // should evict both entries
        assert_eq!(l.entry_count(), 0, "both entries should be evicted");
    }

    // Issue: Hardcoded magic numbers without explanation
    #[test]
    fn check_allows_five_requests() {
        let mut l = RateLimiter::new(5, 60000, HardcodedEntryCountClock::new(
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0] // 10 zeros — why 10?
        ));
        assert!(l.check());
        assert!(l.check());
        assert!(l.check());
        assert!(l.check());
        assert!(l.check());
    }

    // Issue: Mock breaks if implementation is refactored to call now_millis()
    // at a different point in the call sequence. The window reset test
    // depends on the exact number of internal clock calls.
    #[test]
    fn window_resets_after_elapsed_time() {
        // The HardcodedEntryCountClock returns these timestamps in order:
        // call 0: t=0 (from check #1)
        // call 1: t=0 (from check #2)
        // call 2: t=1001 (from check #3, after "advancing time")
        // This only works if the implementation calls now_millis() exactly once per check().
        let clock = HardcodedEntryCountClock::new(vec![0, 0, 1001]);
        let mut l = RateLimiter::new(1, 1000, clock);
        l.check(); // fills capacity
        assert!(!l.check(), "should be blocked in same window"); // still t=0
        assert!(l.check(), "should be allowed after window reset"); // now t=1001
    }

    // Issue: No doc test on the public struct
    // The RateLimiter struct has no documentation showing basic usage.
    // A downstream user looking at the docs has no example to start from.

    // Issue: No test for the should_panic cases
    // There are no tests verifying that invalid inputs (zero capacity,
    // zero window) cause panics. The panic contract is documented in
    // the production code's comments but never verified.

    // Issue: Missing edge case — window boundary
    // No test for the boundary condition: what happens when the entry age
    // equals exactly window_millis? Is it evicted or kept?
    // (Answer: evicted — age >= window_millis. But this isn't tested.)
}

fn main() {}
