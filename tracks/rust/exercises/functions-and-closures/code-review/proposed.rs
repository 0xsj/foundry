// Retry/Backoff Utility — Proposed Code for Review
//
// A generic retry utility with configurable backoff strategy.
// The caller provides the fallible operation and an optional
// backoff function that computes the delay between attempts.
//
// Compiles and tests pass — but there are design issues to find.

use std::time::Duration;

// ---- Configuration ----

/// Backoff strategy configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
}

impl RetryConfig {
    pub fn new(max_attempts: u32, base_delay_ms: u64) -> RetryConfig {
        RetryConfig {
            max_attempts,
            base_delay_ms,
        }
    }
}

// ---- Core retry function ----

/// Retries `operation` up to `config.max_attempts` times.
/// On each failure, waits for the duration returned by `backoff(attempt)`.
///
/// Returns Ok(T) on first success, or Err(String) with the last error message.
pub fn retry_with_backoff<T>(
    config: &RetryConfig,
    // REVIEWER QUESTION: Is Box<dyn Fn> the right choice here?
    // What does requiring Box instead of impl Fn force the caller to do?
    operation: Box<dyn Fn() -> Result<T, String>>,
    backoff: Box<dyn Fn(u32) -> Duration>,
) -> Result<T, String> {
    let mut last_error = String::new();

    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = e;
                if attempt < config.max_attempts {
                    let delay = backoff(attempt);
                    // In production: std::thread::sleep(delay);
                    // Skipped here to keep tests fast.
                    let _ = delay;
                }
            }
        }
    }

    Err(format!("all {} attempts failed: {}", config.max_attempts, last_error))
}

// ---- Backoff strategy constructors ----

/// Returns a backoff function that always waits the same amount.
/// REVIEWER QUESTION: Should this return Box<dyn Fn> or impl Fn?
pub fn constant_backoff(delay_ms: u64) -> Box<dyn Fn(u32) -> Duration> {
    Box::new(move |_attempt| Duration::from_millis(delay_ms))
}

/// Returns a backoff function with exponential delay: base * 2^attempt.
pub fn exponential_backoff(base_ms: u64) -> Box<dyn Fn(u32) -> Duration> {
    Box::new(move |attempt| {
        let multiplier = 2u64.pow(attempt);
        Duration::from_millis(base_ms.saturating_mul(multiplier))
    })
}

/// Returns a backoff that caps the delay at a maximum value.
/// Wraps another backoff strategy.
/// REVIEWER QUESTION: The inner backoff is taken as Box<dyn Fn>. What does this mean
/// for the caller? What alternative signature would be more flexible?
pub fn capped_backoff(
    inner: Box<dyn Fn(u32) -> Duration>,
    max_delay_ms: u64,
) -> Box<dyn Fn(u32) -> Duration> {
    let max = Duration::from_millis(max_delay_ms);
    Box::new(move |attempt| inner(attempt).min(max))
}

// ---- Higher-level helpers ----

/// Retries `operation` with constant backoff, using default config.
pub fn retry_simple<T>(
    operation: Box<dyn Fn() -> Result<T, String>>,
    max_attempts: u32,
) -> Result<T, String> {
    let config = RetryConfig::new(max_attempts, 100);
    let backoff = constant_backoff(config.base_delay_ms);
    retry_with_backoff(&config, operation, backoff)
}

/// Runs `operation` and, if it fails, logs the error using the provided logger.
/// Then retries with the given config.
/// REVIEWER QUESTION: The operation is FnMut here but Fn in retry_with_backoff.
/// Is this distinction intentional? Is FnMut really needed here?
pub fn retry_with_logging<T>(
    config: &RetryConfig,
    operation: Box<dyn FnMut() -> Result<T, String>>,
    logger: Box<dyn Fn(u32, &str)>,
) -> Result<T, String> {
    let mut last_error = String::new();
    let mut op = operation;

    for attempt in 1..=config.max_attempts {
        match op() {
            Ok(value) => return Ok(value),
            Err(e) => {
                logger(attempt, &e);
                last_error = e;
            }
        }
    }

    Err(format!("all {} attempts failed: {}", config.max_attempts, last_error))
}

// ---- Thread-spawning retry ----

/// Spawns a thread to run a retry loop. The operation must be Send + 'static.
/// REVIEWER QUESTION: Does this signature force more allocation than necessary?
/// What would allow the same functionality with less indirection?
pub fn retry_in_background<T: Send + 'static>(
    config: RetryConfig,
    operation: Box<dyn Fn() -> Result<T, String> + Send>,
) -> std::thread::JoinHandle<Result<T, String>> {
    std::thread::spawn(move || {
        let backoff = constant_backoff(config.base_delay_ms);
        retry_with_backoff(&config, operation, backoff)
    })
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_succeeds_immediately() {
        let config = RetryConfig::new(3, 10);
        let result = retry_with_backoff(
            &config,
            Box::new(|| Ok::<i32, String>(42)),
            Box::new(|_| Duration::from_millis(10)),
        );
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_succeeds_after_failures() {
        let config = RetryConfig::new(5, 10);
        // Use Arc<AtomicU32> so the closure can own it ('static) while we read after
        let attempt_count = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&attempt_count);

        let result = retry_with_backoff(
            &config,
            Box::new(move || {
                let n = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err(format!("attempt {} failed", n))
                } else {
                    Ok(n)
                }
            }),
            Box::new(|_| Duration::ZERO),
        );

        assert_eq!(result, Ok(3));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_exhausts_attempts() {
        let config = RetryConfig::new(3, 10);
        let result = retry_with_backoff(
            &config,
            Box::new(|| Err::<i32, String>(String::from("always fails"))),
            Box::new(|_| Duration::ZERO),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("3"));
    }

    #[test]
    fn test_exponential_backoff_values() {
        let backoff = exponential_backoff(100);
        assert_eq!(backoff(1), Duration::from_millis(200));
        assert_eq!(backoff(2), Duration::from_millis(400));
        assert_eq!(backoff(3), Duration::from_millis(800));
    }

    #[test]
    fn test_capped_backoff() {
        let inner = exponential_backoff(100);
        let capped = capped_backoff(inner, 300);
        assert_eq!(capped(1), Duration::from_millis(200)); // 200 < 300, not capped
        assert_eq!(capped(2), Duration::from_millis(300)); // 400 > 300, capped
        assert_eq!(capped(5), Duration::from_millis(300)); // 3200 > 300, capped
    }

    #[test]
    fn test_retry_simple() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&attempt_count);

        let result = retry_simple(
            Box::new(move || {
                let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 2 { Err(String::from("not yet")) } else { Ok(n) }
            }),
            3,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_retry_with_logging() {
        let config = RetryConfig::new(3, 10);
        // Use Arc<Mutex<Vec>> so the logger closure can own it ('static)
        let log = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let log_for_closure = Arc::clone(&log);

        let result = retry_with_logging(
            &config,
            Box::new(|| Err::<i32, String>(String::from("error"))),
            Box::new(move |attempt, msg| {
                log_for_closure
                    .lock()
                    .unwrap()
                    .push(format!("attempt {}: {}", attempt, msg));
            }),
        );

        assert!(result.is_err());
        assert_eq!(log.lock().unwrap().len(), 3);
    }
}

fn main() {
    let config = RetryConfig::new(4, 50);
    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = std::sync::Arc::clone(&attempt_count);

    let result = retry_with_backoff(
        &config,
        Box::new(move || {
            let attempt = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            println!("Attempt {}", attempt);
            if attempt < 3 {
                Err(format!("not ready (attempt {})", attempt))
            } else {
                Ok(format!("success on attempt {}", attempt))
            }
        }),
        Box::new(|n| {
            let delay = Duration::from_millis(50 * 2u64.pow(n));
            println!("Backing off for {:?}", delay);
            delay
        }),
    );

    match result {
        Ok(msg) => println!("Result: {}", msg),
        Err(e) => println!("Failed: {}", e),
    }
}
