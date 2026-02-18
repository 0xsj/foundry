// Circuit Breaker — HTTP-like Service Client
//
// Demonstrates a realistic usage pattern: per-endpoint circuit breakers
// protecting a service client. Simulates network failures to show how
// breakers isolate failing endpoints from healthy ones.
//
// Run: rustc service.rs && ./service
// Test: rustc --test service.rs && ./service

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Circuit Breaker (reusable core)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum State {
    Closed { failure_count: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

#[derive(Debug, Clone)]
struct BreakerConfig {
    failure_threshold: u32,
    recovery_timeout: Duration,
}

#[derive(Debug)]
enum BreakerError<E> {
    Open,
    Inner(E),
}

impl<E: fmt::Display> fmt::Display for BreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakerError::Open => write!(f, "circuit breaker is open"),
            BreakerError::Inner(e) => write!(f, "{}", e),
        }
    }
}

struct CircuitBreaker {
    name: String,
    state: State,
    config: BreakerConfig,
}

impl CircuitBreaker {
    fn new(name: &str, config: BreakerConfig) -> Self {
        Self {
            name: name.to_string(),
            state: State::Closed { failure_count: 0 },
            config,
        }
    }

    fn call<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match &self.state {
            State::Closed { .. } => self.call_closed(operation),
            State::Open { opened_at } => {
                if opened_at.elapsed() >= self.config.recovery_timeout {
                    self.transition(State::HalfOpen);
                    self.call_half_open(operation)
                } else {
                    Err(BreakerError::Open)
                }
            }
            State::HalfOpen => Err(BreakerError::Open),
        }
    }

    fn call_closed<T, E, F>(&mut self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match operation() {
            Ok(value) => {
                self.transition(State::Closed { failure_count: 0 });
                Ok(value)
            }
            Err(e) => {
                let new_count = match &self.state {
                    State::Closed { failure_count } => failure_count + 1,
                    _ => 1,
                };
                if new_count >= self.config.failure_threshold {
                    self.transition(State::Open {
                        opened_at: Instant::now(),
                    });
                } else {
                    self.transition(State::Closed {
                        failure_count: new_count,
                    });
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
                self.transition(State::Closed { failure_count: 0 });
                Ok(value)
            }
            Err(e) => {
                self.transition(State::Open {
                    opened_at: Instant::now(),
                });
                Err(BreakerError::Inner(e))
            }
        }
    }

    fn transition(&mut self, new_state: State) {
        let old_label = self.state_label();
        self.state = new_state;
        let new_label = self.state_label();
        if old_label != new_label {
            println!("  [{}] {} -> {}", self.name, old_label, new_label);
        }
    }

    fn state_label(&self) -> &str {
        match &self.state {
            State::Closed { .. } => "Closed",
            State::Open { .. } => "Open",
            State::HalfOpen => "HalfOpen",
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }
}

// ---------------------------------------------------------------------------
// Service Client with per-endpoint breakers
// ---------------------------------------------------------------------------

/// Simulated HTTP response
#[derive(Debug, Clone)]
struct Response {
    status: u16,
    body: String,
}

/// Simulated service error
#[derive(Debug, Clone)]
struct ServiceError {
    kind: ErrorKind,
    message: String,
}

#[derive(Debug, Clone)]
enum ErrorKind {
    ConnectionRefused,
    Timeout,
    ServerError,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

/// A service client that maintains a circuit breaker per endpoint.
///
/// In a real system, each downstream service (users-api, payments-api, etc.)
/// gets its own breaker. This prevents a failing payments service from
/// tripping the breaker for the users service.
struct ServiceClient {
    base_url: String,
    breakers: HashMap<String, CircuitBreaker>,
    default_config: BreakerConfig,
    // Simulation: tracks which endpoints are "down"
    failing_endpoints: Vec<String>,
}

impl ServiceClient {
    fn new(base_url: &str, config: BreakerConfig) -> Self {
        Self {
            base_url: base_url.to_string(),
            breakers: HashMap::new(),
            default_config: config,
            failing_endpoints: Vec::new(),
        }
    }

    /// Simulate an endpoint being down
    fn set_endpoint_failing(&mut self, endpoint: &str) {
        if !self.failing_endpoints.contains(&endpoint.to_string()) {
            self.failing_endpoints.push(endpoint.to_string());
        }
    }

    /// Simulate an endpoint recovering
    fn set_endpoint_healthy(&mut self, endpoint: &str) {
        self.failing_endpoints.retain(|e| e != endpoint);
    }

    /// Make a GET request through the circuit breaker for this endpoint.
    fn get(&mut self, endpoint: &str) -> Result<Response, BreakerError<ServiceError>> {
        // Lazily create a breaker for this endpoint
        if !self.breakers.contains_key(endpoint) {
            let breaker = CircuitBreaker::new(endpoint, self.default_config.clone());
            self.breakers.insert(endpoint.to_string(), breaker);
        }

        let is_failing = self.failing_endpoints.contains(&endpoint.to_string());
        let base = self.base_url.clone();

        let breaker = self.breakers.get_mut(endpoint).unwrap();
        breaker.call(move || {
            // Simulate network call
            if is_failing {
                Err(ServiceError {
                    kind: ErrorKind::ConnectionRefused,
                    message: format!("failed to connect to {}{}", base, endpoint),
                })
            } else {
                Ok(Response {
                    status: 200,
                    body: format!("{{\"endpoint\": \"{}\", \"status\": \"ok\"}}", endpoint),
                })
            }
        })
    }

    /// Check if a specific endpoint's breaker is open
    fn is_endpoint_open(&self, endpoint: &str) -> bool {
        self.breakers
            .get(endpoint)
            .map(|b| b.is_open())
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Thread-safe wrapper using Arc<RwLock<>>
// ---------------------------------------------------------------------------

/// Demonstrates sharing a service client across threads.
/// Each clone shares the same underlying client (and its breakers).
#[derive(Clone)]
struct SharedServiceClient {
    inner: Arc<RwLock<ServiceClient>>,
}

impl SharedServiceClient {
    fn new(client: ServiceClient) -> Self {
        Self {
            inner: Arc::new(RwLock::new(client)),
        }
    }

    fn get(&self, endpoint: &str) -> Result<Response, String> {
        let mut client = self.inner.write().unwrap();
        client.get(endpoint).map_err(|e| match e {
            BreakerError::Open => format!("[{}] circuit breaker open — skipping", endpoint),
            BreakerError::Inner(e) => format!("[{}] {}", endpoint, e),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> ServiceClient {
        ServiceClient::new(
            "http://api.internal",
            BreakerConfig {
                failure_threshold: 3,
                recovery_timeout: Duration::from_millis(50),
            },
        )
    }

    #[test]
    fn test_healthy_endpoint_succeeds() {
        let mut client = make_client();
        let result = client.get("/users");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, 200);
    }

    #[test]
    fn test_failing_endpoint_trips_breaker() {
        let mut client = make_client();
        client.set_endpoint_failing("/payments");

        // Three failures should trip the breaker
        for _ in 0..3 {
            let _ = client.get("/payments");
        }
        assert!(client.is_endpoint_open("/payments"));
    }

    #[test]
    fn test_failing_endpoint_does_not_affect_healthy() {
        let mut client = make_client();
        client.set_endpoint_failing("/payments");

        // Trip the payments breaker
        for _ in 0..3 {
            let _ = client.get("/payments");
        }

        // Users endpoint should still work
        let result = client.get("/users");
        assert!(result.is_ok());
        assert!(!client.is_endpoint_open("/users"));
    }

    #[test]
    fn test_open_breaker_rejects_fast() {
        let mut client = make_client();
        client.set_endpoint_failing("/search");

        for _ in 0..3 {
            let _ = client.get("/search");
        }

        // Subsequent calls should be rejected without attempting
        let start = Instant::now();
        for _ in 0..100 {
            let _ = client.get("/search");
        }
        // Should be nearly instant — no network calls made
        assert!(start.elapsed() < Duration::from_millis(10));
    }

    #[test]
    fn test_recovery_after_timeout() {
        let mut client = make_client();
        client.set_endpoint_failing("/inventory");

        for _ in 0..3 {
            let _ = client.get("/inventory");
        }
        assert!(client.is_endpoint_open("/inventory"));

        // Simulate recovery
        client.set_endpoint_healthy("/inventory");
        std::thread::sleep(Duration::from_millis(60));

        // Should probe and succeed
        let result = client.get("/inventory");
        assert!(result.is_ok());
        assert!(!client.is_endpoint_open("/inventory"));
    }

    #[test]
    fn test_shared_client_across_threads() {
        let client = SharedServiceClient::new(make_client());

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let c = client.clone();
                let endpoint = if i % 2 == 0 { "/users" } else { "/orders" };
                std::thread::spawn(move || c.get(endpoint))
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Service Client with Per-Endpoint Circuit Breakers ===\n");

    let mut client = ServiceClient::new(
        "http://api.internal",
        BreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(200),
        },
    );

    // Scenario: payments service goes down, but users service stays healthy
    println!("--- Phase 1: Payments service goes down ---\n");
    client.set_endpoint_failing("/payments");

    for i in 1..=5 {
        // Try both endpoints
        let users_result = client.get("/users");
        let payments_result = client.get("/payments");

        print!("Request {}: ", i);
        match users_result {
            Ok(r) => print!("users=OK({}) ", r.status),
            Err(BreakerError::Open) => print!("users=BREAKER_OPEN "),
            Err(BreakerError::Inner(e)) => print!("users=ERR({}) ", e),
        }
        match payments_result {
            Ok(r) => print!("payments=OK({}) ", r.status),
            Err(BreakerError::Open) => print!("payments=BREAKER_OPEN "),
            Err(BreakerError::Inner(e)) => print!("payments=ERR({}) ", e),
        }
        println!();
    }

    println!("\nPayments breaker open: {}", client.is_endpoint_open("/payments"));
    println!("Users breaker open: {}", client.is_endpoint_open("/users"));

    // Scenario: payments recovers
    println!("\n--- Phase 2: Waiting for recovery timeout ---\n");
    std::thread::sleep(Duration::from_millis(250));
    client.set_endpoint_healthy("/payments");

    for i in 1..=3 {
        let payments_result = client.get("/payments");
        print!("Recovery request {}: ", i);
        match payments_result {
            Ok(r) => println!("payments=OK({})", r.status),
            Err(BreakerError::Open) => println!("payments=BREAKER_OPEN"),
            Err(BreakerError::Inner(e)) => println!("payments=ERR({})", e),
        }
    }

    println!("\nFinal state:");
    println!("  Payments breaker open: {}", client.is_endpoint_open("/payments"));
    println!("  Users breaker open: {}", client.is_endpoint_open("/users"));

    // Demonstrate thread-safe sharing
    println!("\n--- Phase 3: Shared client across threads ---\n");
    let shared = SharedServiceClient::new(ServiceClient::new(
        "http://api.internal",
        BreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_millis(200),
        },
    ));

    let mut handles = Vec::new();
    for thread_id in 0..4 {
        let client = shared.clone();
        handles.push(std::thread::spawn(move || {
            let endpoint = match thread_id % 3 {
                0 => "/users",
                1 => "/orders",
                _ => "/inventory",
            };
            match client.get(endpoint) {
                Ok(r) => println!("  Thread {}: {} -> {}", thread_id, endpoint, r.body),
                Err(e) => println!("  Thread {}: {} -> {}", thread_id, endpoint, e),
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
