// Circuit Breaker — Basic Implementation
//
// A three-state circuit breaker generic over the wrapped operation.
// Demonstrates: enum state machine, pattern matching on transitions,
// failure counting, timeout-based recovery.
//
// Run: rustc basic.rs && ./basic
// Test: rustc --test basic.rs && ./basic

use std::time::{Duration, Instant};
use std::fmt;

// ---------------------------------------------------------------------------
// State Machine
// ---------------------------------------------------------------------------

/// Each variant carries only the data relevant to that state.
/// Closed tracks failures. Open tracks when it opened. HalfOpen has no data.
#[derive(Debug, Clone)]
enum State {
    Closed { failure_count: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Closed { failure_count } => {
                write!(f, "Closed(failures={})", failure_count)
            }
            State::Open { opened_at } => {
                write!(f, "Open(elapsed={:.1}s)", opened_at.elapsed().as_secs_f64())
            }
            State::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Config {
    /// Number of consecutive failures before the breaker opens.
    failure_threshold: u32,
    /// How long to wait in Open state before transitioning to HalfOpen.
    recovery_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
        }
    }
}

// ---------------------------------------------------------------------------
// Error Types
// ---------------------------------------------------------------------------

/// Wraps the inner operation's error type, adding a circuit-open variant.
#[derive(Debug)]
enum BreakerError<E> {
    /// The breaker is open — the operation was not attempted.
    Open,
    /// The operation was attempted and returned an error.
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for BreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakerError::Open => write!(f, "circuit breaker is open"),
            BreakerError::Inner(e) => write!(f, "operation failed: {}", e),
        }
    }
}

// ---------------------------------------------------------------------------
// Circuit Breaker
// ---------------------------------------------------------------------------

struct CircuitBreaker {
    state: State,
    config: Config,
}

impl CircuitBreaker {
    fn new(config: Config) -> Self {
        Self {
            state: State::Closed { failure_count: 0 },
            config,
        }
    }

    /// Execute an operation through the circuit breaker.
    ///
    /// - Closed: pass through, count failures
    /// - Open: reject immediately, unless recovery timeout has elapsed
    /// - HalfOpen: allow one probe, transition based on result
    fn call<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match &self.state {
            State::Closed { .. } => self.call_closed(operation),
            State::Open { opened_at } => {
                // Check if recovery timeout has elapsed
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    self.state = State::HalfOpen;
                    self.call_half_open(operation)
                } else {
                    Err(BreakerError::Open)
                }
            }
            State::HalfOpen => {
                // Only one probe allowed — reject additional calls
                Err(BreakerError::Open)
            }
        }
    }

    fn call_closed<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match operation() {
            Ok(value) => {
                // Success resets the failure count
                self.state = State::Closed { failure_count: 0 };
                Ok(value)
            }
            Err(e) => {
                let new_count = match &self.state {
                    State::Closed { failure_count } => failure_count + 1,
                    _ => 1,
                };

                if new_count >= self.config.failure_threshold {
                    println!(
                        "  [breaker] Threshold reached ({}/{}). Opening circuit.",
                        new_count, self.config.failure_threshold
                    );
                    self.state = State::Open {
                        opened_at: Instant::now(),
                    };
                } else {
                    self.state = State::Closed {
                        failure_count: new_count,
                    };
                }
                Err(BreakerError::Inner(e))
            }
        }
    }

    fn call_half_open<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match operation() {
            Ok(value) => {
                println!("  [breaker] Probe succeeded. Closing circuit.");
                self.state = State::Closed { failure_count: 0 };
                Ok(value)
            }
            Err(e) => {
                println!("  [breaker] Probe failed. Re-opening circuit.");
                self.state = State::Open {
                    opened_at: Instant::now(),
                };
                Err(BreakerError::Inner(e))
            }
        }
    }

    fn state(&self) -> &State {
        &self.state
    }

    fn is_closed(&self) -> bool {
        matches!(self.state, State::Closed { .. })
    }

    fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_breaker(threshold: u32, timeout_ms: u64) -> CircuitBreaker {
        CircuitBreaker::new(Config {
            failure_threshold: threshold,
            recovery_timeout: Duration::from_millis(timeout_ms),
        })
    }

    #[test]
    fn test_closed_passes_through_on_success() {
        let mut cb = make_breaker(3, 1000);
        let result = cb.call(|| Ok::<&str, &str>("hello"));
        assert_eq!(result.unwrap(), "hello");
        assert!(cb.is_closed());
    }

    #[test]
    fn test_closed_counts_failures() {
        let mut cb = make_breaker(3, 1000);
        let _ = cb.call(|| Err::<(), &str>("fail"));
        let _ = cb.call(|| Err::<(), &str>("fail"));
        // Two failures, threshold is 3 — still closed
        assert!(cb.is_closed());
        match &cb.state {
            State::Closed { failure_count } => assert_eq!(*failure_count, 2),
            _ => panic!("expected Closed"),
        }
    }

    #[test]
    fn test_opens_at_threshold() {
        let mut cb = make_breaker(3, 1000);
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"));
        }
        assert!(cb.is_open());
    }

    #[test]
    fn test_open_rejects_calls() {
        let mut cb = make_breaker(1, 60_000);
        let _ = cb.call(|| Err::<(), &str>("fail")); // trips the breaker
        assert!(cb.is_open());

        let result = cb.call(|| Ok::<&str, &str>("should not run"));
        assert!(matches!(result, Err(BreakerError::Open)));
    }

    #[test]
    fn test_success_resets_failure_count() {
        let mut cb = make_breaker(3, 1000);
        let _ = cb.call(|| Err::<(), &str>("fail"));
        let _ = cb.call(|| Err::<(), &str>("fail"));
        // Two failures, then a success should reset
        let _ = cb.call(|| Ok::<&str, &str>("ok"));
        match &cb.state {
            State::Closed { failure_count } => assert_eq!(*failure_count, 0),
            _ => panic!("expected Closed with 0 failures"),
        }
    }

    #[test]
    fn test_half_open_probe_success_closes() {
        let mut cb = make_breaker(1, 10); // 10ms timeout
        let _ = cb.call(|| Err::<(), &str>("fail")); // trips
        assert!(cb.is_open());

        std::thread::sleep(Duration::from_millis(15));

        // Next call should transition to HalfOpen and probe
        let result = cb.call(|| Ok::<&str, &str>("recovered"));
        assert_eq!(result.unwrap(), "recovered");
        assert!(cb.is_closed());
    }

    #[test]
    fn test_half_open_probe_failure_reopens() {
        let mut cb = make_breaker(1, 10);
        let _ = cb.call(|| Err::<(), &str>("fail")); // trips

        std::thread::sleep(Duration::from_millis(15));

        // Probe fails — should reopen
        let result = cb.call(|| Err::<(), &str>("still failing"));
        assert!(matches!(result, Err(BreakerError::Inner("still failing"))));
        assert!(cb.is_open());
    }

    #[test]
    fn test_breaker_error_display() {
        let open: BreakerError<String> = BreakerError::Open;
        assert_eq!(format!("{}", open), "circuit breaker is open");

        let inner: BreakerError<String> = BreakerError::Inner("timeout".to_string());
        assert_eq!(format!("{}", inner), "operation failed: timeout");
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn main() {
    let mut cb = CircuitBreaker::new(Config {
        failure_threshold: 3,
        recovery_timeout: Duration::from_millis(100),
    });

    // Simulated operation that fails for the first N calls
    let mut call_count = 0u32;
    let total_calls = 10;

    for i in 1..=total_calls {
        call_count += 1;
        let result = cb.call(|| -> Result<String, String> {
            if call_count <= 4 {
                Err(format!("connection refused (attempt {})", call_count))
            } else {
                Ok(format!("response #{}", call_count))
            }
        });

        match result {
            Ok(val) => println!("Call {}: OK — {}", i, val),
            Err(BreakerError::Open) => println!("Call {}: REJECTED — {}", i, cb.state()),
            Err(BreakerError::Inner(e)) => println!("Call {}: FAILED — {}", i, e),
        }

        // Small delay to let the recovery timeout pass
        if i == 5 {
            println!("\n--- Waiting for recovery timeout ---\n");
            std::thread::sleep(Duration::from_millis(150));
        }
    }

    println!("\nFinal state: {}", cb.state());
}
