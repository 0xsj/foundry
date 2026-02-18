// Strategy Pattern: Retry Policies via Closures
//
// Demonstrates: Functional/closure-based strategy pattern using Fn traits
//
// Scenario: An HTTP client that accepts pluggable retry policies. Each policy
// is a closure that takes the attempt number and returns either a Duration
// to wait or None to stop retrying. This is the ideal case for closure-based
// strategy — the behavior is a single function, and defining a struct for
// each policy would be overkill.
//
// Run: rustc retry.rs && ./retry

use std::fmt;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Strategy type: a closure
// ---------------------------------------------------------------------------

/// A retry policy decides whether to retry and how long to wait.
/// Takes the attempt number (0-based) and returns Some(delay) or None.
///
/// We use Box<dyn Fn> because:
/// 1. Closures that capture state have unique, anonymous types
/// 2. We need to store the policy in a struct
/// 3. Fn (not FnMut) because the policy should be callable multiple times
///    without mutating state — each call is independent
type RetryPolicy = Box<dyn Fn(u32) -> Option<Duration>>;

// ---------------------------------------------------------------------------
// Strategy factories: functions that return closures
// ---------------------------------------------------------------------------

/// Exponential backoff: wait base * 2^attempt, up to max_retries attempts.
/// With optional jitter to prevent thundering herd.
fn exponential_backoff(base_ms: u64, max_retries: u32, jitter: bool) -> RetryPolicy {
    Box::new(move |attempt: u32| {
        if attempt >= max_retries {
            return None;
        }
        let delay = base_ms.saturating_mul(2u64.pow(attempt));
        let delay = if jitter {
            // Deterministic "jitter" for reproducible output (in production: use rand)
            let pseudo_jitter = (attempt as u64 * 7 + 3) % (delay / 4 + 1);
            delay + pseudo_jitter
        } else {
            delay
        };
        Some(Duration::from_millis(delay))
    })
}

/// Constant delay: always wait the same amount.
fn constant_delay(ms: u64, max_retries: u32) -> RetryPolicy {
    Box::new(move |attempt: u32| {
        if attempt >= max_retries {
            return None;
        }
        Some(Duration::from_millis(ms))
    })
}

/// Linear backoff: wait base * (attempt + 1).
fn linear_backoff(base_ms: u64, max_retries: u32) -> RetryPolicy {
    Box::new(move |attempt: u32| {
        if attempt >= max_retries {
            return None;
        }
        let delay = base_ms.saturating_mul(attempt as u64 + 1);
        Some(Duration::from_millis(delay))
    })
}

/// No retry: fail immediately on first error.
fn no_retry() -> RetryPolicy {
    Box::new(|_| None)
}

/// Retry forever with a fixed delay (use carefully).
fn retry_forever(ms: u64) -> RetryPolicy {
    Box::new(move |_| Some(Duration::from_millis(ms)))
}

/// Composite: try policy A first, fall back to policy B.
/// Useful for "fast retries then slow retries" patterns.
fn composite(fast: RetryPolicy, fast_count: u32, slow: RetryPolicy) -> RetryPolicy {
    Box::new(move |attempt: u32| {
        if attempt < fast_count {
            fast(attempt)
        } else {
            slow(attempt - fast_count)
        }
    })
}

// ---------------------------------------------------------------------------
// Context: the HTTP client
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    body: String,
}

#[derive(Debug)]
struct HttpError {
    status: u16,
    message: String,
    attempts: u32,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP {} after {} attempt(s): {}",
            self.status, self.attempts, self.message
        )
    }
}

/// Simulates an HTTP endpoint that fails a certain number of times before succeeding.
struct MockEndpoint {
    url: String,
    fail_count: u32,
    calls: std::cell::Cell<u32>,
}

impl MockEndpoint {
    fn new(url: &str, fail_count: u32) -> Self {
        Self {
            url: url.to_string(),
            fail_count,
            calls: std::cell::Cell::new(0),
        }
    }

    fn call(&self) -> Result<HttpResponse, HttpError> {
        let n = self.calls.get();
        self.calls.set(n + 1);
        if n < self.fail_count {
            Err(HttpError {
                status: 503,
                message: format!("{} is temporarily unavailable", self.url),
                attempts: 1,
            })
        } else {
            Ok(HttpResponse {
                status: 200,
                body: format!("{{\"ok\": true, \"from\": \"{}\"}}", self.url),
            })
        }
    }

    fn reset(&self) {
        self.calls.set(0);
    }
}

struct HttpClient {
    retry_policy: RetryPolicy,
    name: String,
}

impl HttpClient {
    fn new(name: &str, retry_policy: RetryPolicy) -> Self {
        Self {
            retry_policy,
            name: name.to_string(),
        }
    }

    fn get(&self, endpoint: &MockEndpoint) -> Result<HttpResponse, HttpError> {
        let mut attempt: u32 = 0;
        loop {
            println!("  [{}] Attempt {} for {}", self.name, attempt + 1, endpoint.url);
            match endpoint.call() {
                Ok(response) => {
                    println!(
                        "  [{}] Success (status {}): {}",
                        self.name, response.status, response.body
                    );
                    return Ok(response);
                }
                Err(e) => {
                    println!("  [{}] Error: {}", self.name, e.message);
                    match (self.retry_policy)(attempt) {
                        Some(delay) => {
                            println!(
                                "  [{}] Waiting {:?} before retry...",
                                self.name, delay
                            );
                            // In production: std::thread::sleep(delay) or tokio::time::sleep
                            attempt += 1;
                        }
                        None => {
                            println!("  [{}] No more retries, giving up.", self.name);
                            return Err(HttpError {
                                status: e.status,
                                message: e.message,
                                attempts: attempt + 1,
                            });
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Retry Policies (Closure-Based Strategy) ===\n");

    // Endpoint that fails 3 times then succeeds
    let endpoint = MockEndpoint::new("https://api.example.com/data", 3);

    // --- Exponential backoff (should succeed on attempt 4) ---
    println!("--- Exponential Backoff (base=100ms, max=5) ---");
    let client = HttpClient::new("expo", exponential_backoff(100, 5, false));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Constant delay (should succeed on attempt 4) ---
    println!("--- Constant Delay (500ms, max=5) ---");
    let client = HttpClient::new("const", constant_delay(500, 5));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Linear backoff ---
    println!("--- Linear Backoff (200ms base, max=5) ---");
    let client = HttpClient::new("linear", linear_backoff(200, 5));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- No retry (should fail immediately) ---
    println!("--- No Retry ---");
    let client = HttpClient::new("no-retry", no_retry());
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Too few retries (should fail after 2 attempts) ---
    println!("--- Exponential Backoff (max=2, not enough) ---");
    let client = HttpClient::new("too-few", exponential_backoff(100, 2, false));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Composite: 2 fast retries then slow retries ---
    println!("--- Composite (2 fast at 50ms, then slow at 1000ms, max 3 slow) ---");
    let fast = constant_delay(50, 2);
    let slow = constant_delay(1000, 3);
    let client = HttpClient::new("composite", composite(fast, 2, slow));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Exponential with jitter ---
    println!("--- Exponential Backoff with Jitter (base=100ms, max=5) ---");
    let client = HttpClient::new("jitter", exponential_backoff(100, 5, true));
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
    endpoint.reset();

    // --- Inline closure (ad-hoc strategy) ---
    println!("--- Inline Closure (custom one-off policy) ---");
    let custom_policy: RetryPolicy = Box::new(|attempt| {
        // Retry up to 4 times, with delay = 100ms * fibonacci-ish sequence
        let delays = [100, 100, 200, 300];
        delays.get(attempt as usize).map(|&ms| Duration::from_millis(ms))
    });
    let client = HttpClient::new("custom", custom_policy);
    match client.get(&endpoint) {
        Ok(r) => println!("  Result: {} {}\n", r.status, r.body),
        Err(e) => println!("  Result: {}\n", e),
    }
}
