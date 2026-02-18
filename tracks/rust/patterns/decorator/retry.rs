// Decorator Pattern: Function Decorators (Closure-Based)
//
// Demonstrates: Wrapping Fn() -> Result<T, E> with retry, timeout,
// and logging behavior. Shows closure-based composition as an alternative
// to trait-based decoration for simple cases.
//
// Concepts: Fn trait bounds, closure capture, impl Fn return types,
// decorator composition, move semantics with closures.
//
// Run: rustc retry.rs && ./retry

use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicU32, Ordering};
use std::fmt;

// ---------------------------------------------------------------------------
// Error type for our operations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct OperationError {
    message: String,
    retryable: bool,
}

impl OperationError {
    fn retryable(message: &str) -> Self {
        OperationError {
            message: message.to_string(),
            retryable: true,
        }
    }

    fn fatal(message: &str) -> Self {
        OperationError {
            message: message.to_string(),
            retryable: false,
        }
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.message,
            if self.retryable { "retryable" } else { "fatal" }
        )
    }
}

// ---------------------------------------------------------------------------
// Decorator 1: with_retry
//
// Wraps a fallible operation with automatic retry logic.
// Only retries if the error is marked as retryable.
// ---------------------------------------------------------------------------

fn with_retry<F, T>(
    operation: F,
    max_retries: usize,
    backoff_ms: u64,
) -> impl Fn() -> Result<T, OperationError>
where
    F: Fn() -> Result<T, OperationError>,
{
    move || {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match operation() {
                Ok(val) => {
                    if attempt > 0 {
                        println!("  [retry] succeeded on attempt {}", attempt + 1);
                    }
                    return Ok(val);
                }
                Err(e) => {
                    if !e.retryable {
                        println!("  [retry] fatal error, not retrying: {}", e);
                        return Err(e);
                    }
                    if attempt < max_retries {
                        let delay = backoff_ms * (1 << attempt); // exponential backoff
                        println!(
                            "  [retry] attempt {} failed: {} -- retrying in {}ms",
                            attempt + 1,
                            e,
                            delay
                        );
                        thread::sleep(Duration::from_millis(delay));
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap())
    }
}

// ---------------------------------------------------------------------------
// Decorator 2: with_timeout
//
// Wraps an operation with a simple timeout check.
// NOTE: This is a cooperative timeout -- it checks elapsed time after the
// operation completes. For true preemptive timeout, you'd need async/threads.
// This is sufficient for demonstrating the decorator pattern.
// ---------------------------------------------------------------------------

fn with_timeout<F, T>(
    operation: F,
    timeout: Duration,
) -> impl Fn() -> Result<T, OperationError>
where
    F: Fn() -> Result<T, OperationError>,
{
    move || {
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        if elapsed > timeout {
            println!(
                "  [timeout] operation took {:?}, exceeds {:?}",
                elapsed, timeout
            );
            return Err(OperationError::retryable(&format!(
                "operation timed out after {:?}",
                elapsed
            )));
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Decorator 3: with_logging
//
// Wraps an operation with before/after logging and timing.
// ---------------------------------------------------------------------------

fn with_logging<F, T>(
    label: &str,
    operation: F,
) -> impl Fn() -> Result<T, OperationError>
where
    F: Fn() -> Result<T, OperationError>,
    T: fmt::Debug,
{
    let label = label.to_string(); // own the label for the closure
    move || {
        println!("[{}] starting operation", label);
        let start = Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        match &result {
            Ok(val) => println!(
                "[{}] success in {:?}: {:?}",
                label, elapsed, val
            ),
            Err(e) => println!(
                "[{}] failed in {:?}: {}",
                label, elapsed, e
            ),
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Decorator 4: with_counter
//
// Counts how many times the operation is invoked.
// Demonstrates using AtomicU32 for shared state in a Fn closure.
// ---------------------------------------------------------------------------

fn with_counter<F, T>(
    operation: F,
    counter: &'static AtomicU32,
) -> impl Fn() -> Result<T, OperationError>
where
    F: Fn() -> Result<T, OperationError>,
{
    move || {
        let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  [counter] invocation #{}", n);
        operation()
    }
}

// ---------------------------------------------------------------------------
// Simulated operations (these would be real I/O in production)
// ---------------------------------------------------------------------------

/// Simulates a flaky database query that fails the first N times.
fn make_flaky_db_query(fail_count: usize) -> impl Fn() -> Result<String, OperationError> {
    let attempts = AtomicU32::new(0);
    move || {
        let n = attempts.fetch_add(1, Ordering::Relaxed) as usize;
        if n < fail_count {
            Err(OperationError::retryable("connection reset by peer"))
        } else {
            Ok(format!("query result (attempt {})", n + 1))
        }
    }
}

/// Simulates an operation that always fails with a non-retryable error.
fn make_auth_failure() -> impl Fn() -> Result<String, OperationError> {
    || Err(OperationError::fatal("invalid credentials"))
}

/// Simulates a slow operation.
fn make_slow_operation(delay_ms: u64) -> impl Fn() -> Result<String, OperationError> {
    move || {
        thread::sleep(Duration::from_millis(delay_ms));
        Ok(format!("completed after {}ms", delay_ms))
    }
}

// ---------------------------------------------------------------------------
// Global counter for demonstration
// ---------------------------------------------------------------------------

static CALL_COUNTER: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== RETRY DECORATOR ===\n");

    // Operation that fails 2 times then succeeds
    let flaky_query = make_flaky_db_query(2);
    let reliable_query = with_retry(flaky_query, 3, 10);

    println!("Calling flaky query with retry(max=3, backoff=10ms):");
    match reliable_query() {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Final error: {}\n", e),
    }

    println!("=== RETRY WITH FATAL ERROR ===\n");

    // Fatal errors should NOT be retried
    let auth_op = make_auth_failure();
    let retried_auth = with_retry(auth_op, 3, 10);

    println!("Calling auth operation with retry (should not retry fatal errors):");
    match retried_auth() {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Final error: {}\n", e),
    }

    println!("=== TIMEOUT DECORATOR ===\n");

    let slow_op = make_slow_operation(50);
    let timed_op = with_timeout(slow_op, Duration::from_millis(30));

    println!("Calling slow operation (50ms) with 30ms timeout:");
    match timed_op() {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Timeout error: {}\n", e),
    }

    let fast_op = make_slow_operation(5);
    let timed_op = with_timeout(fast_op, Duration::from_millis(100));

    println!("Calling fast operation (5ms) with 100ms timeout:");
    match timed_op() {
        Ok(val) => println!("Result: {}\n", val),
        Err(e) => println!("Error: {}\n", e),
    }

    println!("=== LOGGING DECORATOR ===\n");

    let query = make_flaky_db_query(0); // succeeds first try
    let logged_query = with_logging("db-fetch", query);

    logged_query().ok();
    println!();

    println!("=== COMPOSED DECORATORS ===\n");

    // Compose: logging -> retry -> counter -> flaky operation
    // Reads inside-out: flaky_query is wrapped with counter, then retry, then logging
    //
    // Execution order (outside-in):
    //   1. logging prints "starting"
    //   2. retry calls the inner operation
    //   3. counter increments and calls the operation
    //   4. flaky_query executes (may fail)
    //   5. on failure, retry loops back to step 3
    //   6. logging prints result

    CALL_COUNTER.store(0, Ordering::Relaxed);

    let flaky_query = make_flaky_db_query(2); // fails twice
    let counted = with_counter(flaky_query, &CALL_COUNTER);
    let retried = with_retry(counted, 3, 5);
    let logged = with_logging("composed-pipeline", retried);

    println!("Composed: logging(retry(counter(flaky_query)))");
    println!("Expect: 3 counter increments, 2 retries, 1 success\n");
    match logged() {
        Ok(val) => println!("\nFinal result: {}", val),
        Err(e) => println!("\nFinal error: {}", e),
    }
    println!(
        "Total invocations recorded: {}\n",
        CALL_COUNTER.load(Ordering::Relaxed)
    );

    println!("=== RETRY + TIMEOUT COMPOSED ===\n");

    // Timeout wraps each individual attempt, retry wraps the whole thing
    // If an attempt times out, it returns a retryable error, so retry will try again

    // First attempt: slow (times out). Second attempt: fast (succeeds).
    let attempt_count = AtomicU32::new(0);
    let variable_speed_op = move || -> Result<String, OperationError> {
        let n = attempt_count.fetch_add(1, Ordering::Relaxed);
        if n == 0 {
            // First call: simulate slow response
            thread::sleep(Duration::from_millis(60));
            Ok("slow result".to_string())
        } else {
            // Subsequent calls: fast
            thread::sleep(Duration::from_millis(5));
            Ok("fast result".to_string())
        }
    };

    let with_to = with_timeout(variable_speed_op, Duration::from_millis(30));
    let reliable = with_retry(with_to, 2, 5);
    let logged = with_logging("timeout-retry", reliable);

    println!("Composed: logging(retry(timeout(variable_speed_op)))");
    println!("First attempt will timeout, second will succeed\n");
    match logged() {
        Ok(val) => println!("\nFinal result: {}", val),
        Err(e) => println!("\nFinal error: {}", e),
    }
}
