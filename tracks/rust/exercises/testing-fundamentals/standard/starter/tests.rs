// tests.rs — Your test suite (starter)
//
// Include the rate limiter implementation:
#[path = "rate_limiter.rs"]
mod rate_limiter;
use rate_limiter::*;

// Write your tests inside this module.
// You'll need to implement FakeClock here before writing tests.

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // ------------------------------------------------------------------
    // TODO 1: Implement FakeClock
    //
    // Requirements:
    //   - Holds a Cell<u64> for the current time in milliseconds
    //   - FakeClock::new(start: u64) -> FakeClock
    //   - FakeClock::advance(&self, millis: u64) — advance via &self
    //   - Implements Clock (now_millis returns the current fake time)
    // ------------------------------------------------------------------

    // struct FakeClock { ... }
    // impl FakeClock { ... }
    // impl Clock for FakeClock { ... }

    // ------------------------------------------------------------------
    // TODO 2: Unit tests for core behavior
    //
    // Write tests for:
    //   - new limiter starts with full token count
    //   - check() returns true when tokens are available
    //   - check() decrements available tokens
    //   - check() returns false when capacity is exhausted
    //   - tokens refill after the window elapses
    //   - full capacity is restored after a window reset
    // ------------------------------------------------------------------

    // #[test]
    // fn new_limiter_starts_with_full_capacity() { ... }

    // #[test]
    // fn check_allows_requests_within_capacity() { ... }

    // #[test]
    // fn check_decrements_available_tokens() { ... }

    // #[test]
    // fn check_blocks_requests_beyond_capacity() { ... }

    // #[test]
    // fn window_reset_restores_full_capacity() { ... }

    // #[test]
    // fn window_boundary_is_exclusive() { ... }

    // ------------------------------------------------------------------
    // TODO 3: #[should_panic(expected = "...")] tests
    //
    // Test that zero capacity and zero window both panic with the correct
    // message substring. Do NOT use bare #[should_panic] without expected.
    // ------------------------------------------------------------------

    // #[test]
    // #[should_panic(expected = "???")]
    // fn zero_capacity_panics() { ... }

    // #[test]
    // #[should_panic(expected = "???")]
    // fn zero_window_panics() { ... }

    // ------------------------------------------------------------------
    // TODO 4: Result-returning test
    //
    // Use parse_rate_config("10/30s") and build a limiter with the result.
    // Use ? to propagate errors. Return Result<(), Box<dyn std::error::Error>>.
    // ------------------------------------------------------------------

    // #[test]
    // fn build_limiter_from_config_string() -> Result<(), Box<dyn std::error::Error>> {
    //     ...
    //     Ok(())
    // }

    // ------------------------------------------------------------------
    // TODO 5: #[ignore] test
    //
    // Write a test that simulates sustained load over many windows.
    // Mark it #[ignore] — it should only run when explicitly requested.
    // Assert that allowed + blocked = total requests, and that blocked > 0.
    // ------------------------------------------------------------------

    // #[test]
    // #[ignore]
    // fn sustained_load_across_many_windows() { ... }
}

// ------------------------------------------------------------------
// TODO 6: Add a doc test to RateLimiter in rate_limiter.rs
//
// Go to rate_limiter.rs and add a doc comment to the RateLimiter struct.
// The doc test should:
//   - Use `# ` to hide boilerplate
//   - Show constructing a RateLimiter with FakeClock
//   - Assert that check() returns true initially
// ------------------------------------------------------------------

fn main() {}
