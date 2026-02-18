// rate_limiter.rs — Token Bucket Rate Limiter (starter)
//
// This is the production code. Do not modify the public API.
// Add a doc comment with a doc test to the RateLimiter struct.
//
// Run tests with: rustc --test tests.rs (which includes this file)

use std::collections::VecDeque;

// --------------------------------------------------------------------------
// Clock trait — injectable time source
// --------------------------------------------------------------------------

pub trait Clock {
    fn now_millis(&self) -> u64;
}

// --------------------------------------------------------------------------
// RateLimiter
// --------------------------------------------------------------------------

// TODO: Add a doc comment here with a doc test showing basic usage.
// The doc test should:
//   - Construct a RateLimiter with FakeClock
//   - Call check() at least twice
//   - Assert the return values
//   - Use `# ` to hide the import/setup lines
pub struct RateLimiter<C: Clock> {
    capacity: u32,
    window_millis: u64,
    entries: VecDeque<u64>,
    pub clock: C,   // pub so tests can call clock.advance()
}

impl<C: Clock> RateLimiter<C> {
    /// Create a new rate limiter.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0 or `window_millis` is 0.
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

    /// Check whether a request is allowed under the rate limit.
    ///
    /// Returns `true` if the request is allowed and a token was consumed.
    /// Returns `false` if the limit has been reached for the current window.
    pub fn check(&mut self) -> bool {
        let now = self.clock.now_millis();

        // Evict entries older than the window
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

    /// Number of tokens currently available (requests that can be made right now).
    pub fn available_tokens(&self) -> usize {
        self.capacity as usize - self.entries.len()
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn window_millis(&self) -> u64 {
        self.window_millis
    }
}

// --------------------------------------------------------------------------
// Config parsing helper (used in the Result-returning test)
// --------------------------------------------------------------------------

#[derive(Debug)]
pub struct ConfigError(String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigError: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Parse a rate config string in the form "N/Xs" (e.g. "100/60s").
/// Returns (capacity, window_millis).
pub fn parse_rate_config(s: &str) -> Result<(u32, u64), ConfigError> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return Err(ConfigError(format!("expected N/Xs format, got '{}'", s)));
    }

    let capacity = parts[0]
        .parse::<u32>()
        .map_err(|_| ConfigError(format!("invalid capacity: '{}'", parts[0])))?;

    let window_str = parts[1];
    if !window_str.ends_with('s') {
        return Err(ConfigError(format!(
            "window must end with 's', got '{}'",
            window_str
        )));
    }

    let secs = window_str[..window_str.len() - 1]
        .parse::<u64>()
        .map_err(|_| ConfigError(format!("invalid window: '{}'", window_str)))?;

    if capacity == 0 {
        return Err(ConfigError("capacity must be > 0".to_string()));
    }
    if secs == 0 {
        return Err(ConfigError("window must be > 0 seconds".to_string()));
    }

    Ok((capacity, secs * 1000))
}
