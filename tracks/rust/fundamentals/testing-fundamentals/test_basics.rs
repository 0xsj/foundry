// test_basics.rs — Rust Testing Fundamentals
//
// Covers: #[test], assert!, assert_eq!, assert_ne!, custom messages,
//         #[should_panic], Result-returning tests, #[ignore]
//
// Run: rustc --test test_basics.rs && ./test_basics

// --------------------------------------------------------------------------
// The domain: a minimal token bucket rate limiter
// This is what we're testing throughout these examples.
// --------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct RateLimiter {
    capacity: u32,
    available: u32,
    window_millis: u64,
    window_start: u64,  // milliseconds since epoch (faked via a counter here)
    clock: u64,         // simulated clock for standalone file usage
}

impl RateLimiter {
    pub fn new(capacity: u32, window_millis: u64) -> Self {
        if capacity == 0 {
            panic!("capacity must be > 0, got 0");
        }
        if window_millis == 0 {
            panic!("window_millis must be > 0, got 0");
        }
        RateLimiter {
            capacity,
            available: capacity,
            window_millis,
            window_start: 0,
            clock: 0,
        }
    }

    pub fn check(&mut self) -> bool {
        // Check if current window has expired
        if self.clock - self.window_start >= self.window_millis {
            self.available = self.capacity;
            self.window_start = self.clock;
        }

        if self.available > 0 {
            self.available -= 1;
            true
        } else {
            false
        }
    }

    pub fn advance_clock(&mut self, millis: u64) {
        self.clock += millis;
    }

    pub fn available_tokens(&self) -> u32 {
        self.available
    }
}

// --------------------------------------------------------------------------
// Basic assertions: assert!, assert_eq!, assert_ne!
// --------------------------------------------------------------------------

#[test]
fn new_limiter_has_full_capacity() {
    let limiter = RateLimiter::new(10, 1000);
    // assert! — boolean check
    assert!(limiter.available_tokens() > 0);
}

#[test]
fn new_limiter_available_tokens_equals_capacity() {
    let limiter = RateLimiter::new(5, 60_000);
    // assert_eq! — prints both values on failure
    assert_eq!(limiter.available_tokens(), 5);
}

#[test]
fn two_different_limiters_have_independent_state() {
    let limiter_a = RateLimiter::new(5, 1000);
    let limiter_b = RateLimiter::new(10, 1000);
    // assert_ne! — checks they are not equal
    assert_ne!(limiter_a.available_tokens(), limiter_b.available_tokens());
}

// --------------------------------------------------------------------------
// Custom failure messages
// --------------------------------------------------------------------------

#[test]
fn check_decrements_available_tokens() {
    let mut limiter = RateLimiter::new(3, 60_000);
    limiter.check(); // consume one token

    let tokens = limiter.available_tokens();
    // Custom message with formatting — only evaluated on failure
    assert_eq!(
        tokens, 2,
        "expected 2 tokens after 1 request, but got {} (capacity=3)",
        tokens
    );
}

#[test]
fn limiter_blocks_after_capacity_exhausted() {
    let mut limiter = RateLimiter::new(2, 60_000);
    limiter.check(); // 1
    limiter.check(); // 2 — at capacity
    let allowed = limiter.check(); // 3 — should be blocked

    assert!(
        !allowed,
        "request 3 should have been blocked (capacity=2, available={})",
        limiter.available_tokens()
    );
}

// --------------------------------------------------------------------------
// #[should_panic] — testing that invalid inputs panic
// --------------------------------------------------------------------------

#[test]
#[should_panic]
fn zero_capacity_panics() {
    RateLimiter::new(0, 1000); // must panic
}

#[test]
#[should_panic]
fn zero_window_panics() {
    RateLimiter::new(10, 0); // must panic
}

// Better: pin the panic to a specific message substring
#[test]
#[should_panic(expected = "capacity must be > 0")]
fn zero_capacity_panics_with_correct_message() {
    RateLimiter::new(0, 1000);
}

#[test]
#[should_panic(expected = "window_millis must be > 0")]
fn zero_window_panics_with_correct_message() {
    RateLimiter::new(10, 0);
}

// --------------------------------------------------------------------------
// Result<(), E>-returning tests — use ? instead of unwrap()
// --------------------------------------------------------------------------

#[derive(Debug)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

fn parse_capacity(s: &str) -> Result<u32, ParseError> {
    s.parse::<u32>()
        .map_err(|e| ParseError(format!("invalid capacity '{}': {}", s, e)))
}

fn parse_window_millis(s: &str) -> Result<u64, ParseError> {
    s.parse::<u64>()
        .map_err(|e| ParseError(format!("invalid window '{}': {}", s, e)))
}

#[test]
fn valid_capacity_string_parses() -> Result<(), ParseError> {
    let cap = parse_capacity("100")?;  // ? propagates failure — no unwrap needed
    assert_eq!(cap, 100);
    Ok(())
}

#[test]
fn valid_window_string_parses() -> Result<(), Box<dyn std::error::Error>> {
    let w = parse_window_millis("60000")?;
    assert_eq!(w, 60_000);
    Ok(())
}

#[test]
fn build_limiter_from_parsed_strings() -> Result<(), Box<dyn std::error::Error>> {
    let capacity = parse_capacity("50")?;
    let window = parse_window_millis("30000")?;
    let limiter = RateLimiter::new(capacity, window);
    assert_eq!(limiter.available_tokens(), 50);
    Ok(())
}

// --------------------------------------------------------------------------
// #[ignore] — slow or external tests
// --------------------------------------------------------------------------

#[test]
#[ignore]
fn sustained_throughput_over_multiple_windows() {
    // This test takes real time — marked ignore to skip in default runs.
    // Run with: ./test_basics --include-ignored
    let mut limiter = RateLimiter::new(10, 100);
    let mut allowed = 0u32;
    let mut blocked = 0u32;

    for i in 0..1000u64 {
        if limiter.check() { allowed += 1; } else { blocked += 1; }
        // Simulate time: advance 11ms every 10 iterations (one window tick)
        if i % 10 == 9 {
            limiter.advance_clock(110);
        }
    }

    assert!(allowed > 0, "should have allowed some requests");
    assert!(blocked > 0, "should have blocked some requests");
}

// --------------------------------------------------------------------------
// Window reset behavior
// --------------------------------------------------------------------------

#[test]
fn window_resets_after_elapsed_time() {
    let mut limiter = RateLimiter::new(3, 1000);
    limiter.check(); // 1
    limiter.check(); // 2
    limiter.check(); // 3 — capacity exhausted

    assert!(!limiter.check(), "should be blocked before window reset");

    limiter.advance_clock(1001); // past the 1000ms window
    assert!(limiter.check(), "should be allowed after window reset");
    assert_eq!(
        limiter.available_tokens(), 2,  // 3 restored, 1 consumed
        "should have 2 tokens remaining after first request in new window"
    );
}

fn main() {
    println!("Run with: rustc --test test_basics.rs && ./test_basics");
    println!("Or use: cargo test (from a Cargo project)");
}
