// solution_rate_limiter.rs — Rate Limiter with Doc Test
//
// Identical to the starter's rate_limiter.rs, but with the doc test added.
// The doc test is on the RateLimiter struct (Acceptance Criterion #5).

use std::collections::VecDeque;

pub trait Clock {
    fn now_millis(&self) -> u64;
}

/// A token bucket rate limiter with an injectable clock.
///
/// Uses a sliding window algorithm: each call to [`check`] records the current
/// timestamp, and old entries outside the window are evicted before each check.
///
/// # Examples
///
/// ```
/// # use std::cell::Cell;
/// # struct FakeClock(Cell<u64>);
/// # impl FakeClock {
/// #     fn new(t: u64) -> Self { FakeClock(Cell::new(t)) }
/// #     fn advance(&self, ms: u64) { self.0.set(self.0.get() + ms); }
/// # }
/// # trait Clock { fn now_millis(&self) -> u64; }
/// # impl Clock for FakeClock { fn now_millis(&self) -> u64 { self.0.get() } }
/// # use std::collections::VecDeque;
/// # struct RateLimiter<C: Clock> { capacity: u32, window_millis: u64, entries: VecDeque<u64>, pub clock: C }
/// # impl<C: Clock> RateLimiter<C> {
/// #     fn new(capacity: u32, window_millis: u64, clock: C) -> Self {
/// #         RateLimiter { capacity, window_millis, entries: VecDeque::new(), clock }
/// #     }
/// #     fn check(&mut self) -> bool {
/// #         let now = self.clock.now_millis();
/// #         while let Some(&f) = self.entries.front() { if now.saturating_sub(f) >= self.window_millis { self.entries.pop_front(); } else { break; } }
/// #         if self.entries.len() < self.capacity as usize { self.entries.push_back(now); true } else { false }
/// #     }
/// # }
/// let clock = FakeClock::new(0);
/// let mut limiter = RateLimiter::new(3, 60_000, clock);
///
/// assert!(limiter.check());  // 1st request — allowed (2 tokens remain)
/// assert!(limiter.check());  // 2nd request — allowed (1 token remains)
/// assert!(limiter.check());  // 3rd request — allowed (0 tokens remain)
/// assert!(!limiter.check()); // 4th request — blocked (capacity exhausted)
/// ```
///
/// [`check`]: RateLimiter::check
pub struct RateLimiter<C: Clock> {
    capacity: u32,
    window_millis: u64,
    entries: VecDeque<u64>,
    pub clock: C,
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
}

#[derive(Debug)]
pub struct ConfigError(String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigError: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

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
