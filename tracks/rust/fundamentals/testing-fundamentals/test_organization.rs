// test_organization.rs — Test Organization, Fixtures, and Helpers
//
// Covers: #[cfg(test)] module convention, shared fixture functions,
//         test helpers, setup/teardown patterns, organizing test modules
//
// Run: rustc --test test_organization.rs && ./test_organization

// --------------------------------------------------------------------------
// Production code
// --------------------------------------------------------------------------

/// A sliding-window request log that records timestamps of recent requests.
/// Used to enforce rate limits over a rolling time window (vs fixed window).
#[derive(Debug)]
pub struct RequestLog {
    entries: Vec<u64>,       // timestamps in milliseconds
    window_millis: u64,
    capacity: u32,
}

impl RequestLog {
    pub fn new(capacity: u32, window_millis: u64) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        assert!(window_millis > 0, "window_millis must be > 0");
        RequestLog {
            entries: Vec::new(),
            window_millis,
            capacity,
        }
    }

    /// Record a request at the given timestamp. Returns true if allowed,
    /// false if the limit has been reached within the current window.
    pub fn record(&mut self, timestamp_millis: u64) -> bool {
        // Evict entries older than the window
        let window = self.window_millis;
        self.entries.retain(|&t| timestamp_millis - t < window);

        if self.entries.len() < self.capacity as usize {
            self.entries.push(timestamp_millis);
            true
        } else {
            false
        }
    }

    pub fn count_in_window(&self, now_millis: u64) -> usize {
        self.entries
            .iter()
            .filter(|&&t| now_millis - t < self.window_millis)
            .count()
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn window_millis(&self) -> u64 {
        self.window_millis
    }
}

/// Parses a rate limit configuration string in the form "N/Xs"
/// e.g. "100/60s" = 100 requests per 60 seconds.
pub fn parse_rate_limit(s: &str) -> Result<(u32, u64), String> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(format!("invalid format '{}': expected N/Xs", s));
    }

    let capacity = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("invalid capacity: '{}'", parts[0]))?;

    let window_str = parts[1];
    if !window_str.ends_with('s') {
        return Err(format!("window must end with 's': got '{}'", window_str));
    }

    let secs = window_str[..window_str.len() - 1]
        .parse::<u64>()
        .map_err(|_| format!("invalid window seconds: '{}'", window_str))?;

    if capacity == 0 {
        return Err("capacity must be > 0".to_string());
    }
    if secs == 0 {
        return Err("window seconds must be > 0".to_string());
    }

    Ok((capacity, secs * 1000))
}

// --------------------------------------------------------------------------
// Tests — organized with cfg(test) and helper functions
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Fixture functions — setup helpers used across multiple tests.
    // Rust doesn't have BeforeEach; helpers are just regular functions.
    // ------------------------------------------------------------------

    /// A fresh log with capacity 5 and a 60-second window.
    fn standard_log() -> RequestLog {
        RequestLog::new(5, 60_000)
    }

    /// A log that's already had N requests recorded, starting at timestamp 0.
    fn log_with_requests(n: u32) -> RequestLog {
        let mut log = standard_log();
        for i in 0..n {
            log.record(i as u64 * 100); // space requests 100ms apart
        }
        log
    }

    /// An exhausted log — all capacity used up.
    fn exhausted_log() -> RequestLog {
        log_with_requests(5) // standard_log has capacity 5
    }

    // ------------------------------------------------------------------
    // Unit tests — RequestLog basics
    // ------------------------------------------------------------------

    #[test]
    fn new_log_starts_empty() {
        let log = standard_log();
        assert_eq!(log.count_in_window(0), 0);
    }

    #[test]
    fn record_allows_requests_up_to_capacity() {
        let mut log = standard_log();
        for i in 0..5 {
            let allowed = log.record(i * 1000);
            assert!(allowed, "request {} should be allowed (capacity 5)", i + 1);
        }
    }

    #[test]
    fn record_blocks_requests_beyond_capacity() {
        let mut log = exhausted_log();
        let allowed = log.record(10_000); // still within the window
        assert!(!allowed, "request beyond capacity should be blocked");
    }

    #[test]
    fn count_in_window_reflects_current_entries() {
        let log = log_with_requests(3);
        let count = log.count_in_window(300); // 300ms — all 3 entries are within 60s window
        assert_eq!(count, 3);
    }

    // ------------------------------------------------------------------
    // Sliding window behavior
    // ------------------------------------------------------------------

    #[test]
    fn old_requests_evicted_outside_window() {
        let mut log = RequestLog::new(2, 1000); // 1 second window
        log.record(0);    // at t=0
        log.record(500);  // at t=500ms

        // At t=1100, the entry at t=0 is now 1100ms old — outside the 1000ms window
        // The entry at t=500 is 600ms old — still inside
        let allowed = log.record(1100);
        assert!(
            allowed,
            "at t=1100, oldest entry (t=0) should have been evicted, freeing capacity"
        );
        assert_eq!(log.count_in_window(1100), 2);
    }

    #[test]
    fn window_boundary_is_exclusive() {
        let mut log = RequestLog::new(1, 1000);
        log.record(0);   // fills capacity

        // Exactly at the boundary: t=1000 - 0 = 1000 >= window_millis(1000)
        // The retain condition is `timestamp_millis - t < self.window_millis`
        // So t=0 is evicted when now=1000 because 1000 - 0 = 1000, NOT < 1000
        let allowed = log.record(1000);
        assert!(allowed, "entry at t=0 should be evicted at t=1000 (boundary exclusive)");
    }

    // ------------------------------------------------------------------
    // parse_rate_limit tests — organized in a sub-module
    // Sub-modules within cfg(test) are fine and keep things organized
    // ------------------------------------------------------------------

    mod parsing {
        use super::*;

        fn assert_parse_ok(input: &str, expected_capacity: u32, expected_window_ms: u64) {
            let result = parse_rate_limit(input);
            assert!(
                result.is_ok(),
                "expected '{}' to parse successfully, got: {:?}",
                input,
                result
            );
            let (cap, win) = result.unwrap();
            assert_eq!(cap, expected_capacity, "wrong capacity for input '{}'", input);
            assert_eq!(win, expected_window_ms, "wrong window for input '{}'", input);
        }

        fn assert_parse_err(input: &str, expected_fragment: &str) {
            let result = parse_rate_limit(input);
            assert!(
                result.is_err(),
                "expected '{}' to fail, but got: {:?}",
                input,
                result
            );
            let err = result.unwrap_err();
            assert!(
                err.contains(expected_fragment),
                "error '{}' did not contain '{}'",
                err,
                expected_fragment
            );
        }

        #[test]
        fn parses_standard_format() {
            assert_parse_ok("100/60s", 100, 60_000);
        }

        #[test]
        fn parses_per_second_limit() {
            assert_parse_ok("10/1s", 10, 1_000);
        }

        #[test]
        fn parses_per_hour_limit() {
            assert_parse_ok("1000/3600s", 1000, 3_600_000);
        }

        #[test]
        fn rejects_missing_separator() {
            assert_parse_err("100", "invalid format");
        }

        #[test]
        fn rejects_non_numeric_capacity() {
            assert_parse_err("abc/60s", "invalid capacity");
        }

        #[test]
        fn rejects_window_without_s_suffix() {
            assert_parse_err("100/60", "window must end with 's'");
        }

        #[test]
        fn rejects_zero_capacity() {
            assert_parse_err("0/60s", "capacity must be > 0");
        }

        #[test]
        fn rejects_zero_window() {
            assert_parse_err("10/0s", "window seconds must be > 0");
        }
    }

    // ------------------------------------------------------------------
    // Round-trip test: parse config, build log, record requests
    // Demonstrates integration between components in a unit test context
    // ------------------------------------------------------------------

    #[test]
    fn parsed_config_builds_functional_log() -> Result<(), String> {
        let (capacity, window_ms) = parse_rate_limit("3/10s")?;
        let mut log = RequestLog::new(capacity, window_ms);

        assert!(log.record(0));     // 1
        assert!(log.record(1000));  // 2
        assert!(log.record(2000));  // 3
        assert!(!log.record(3000)); // 4 — blocked

        Ok(())
    }
}

fn main() {
    println!("Run with: rustc --test test_organization.rs && ./test_organization");
}
