// Middleware Pipeline -- Reference Solution
//
// Implements a composable middleware stack using generic decoration (static dispatch).
// Each middleware wraps an inner Handler, adding one cross-cutting concern.
//
// Run tests: rustc --test solution.rs && ./solution
// Run program: rustc solution.rs && ./solution

use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn new(method: &str, path: &str) -> Self {
        Request {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    #[allow(dead_code)]
    fn with_body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
}

#[derive(Debug, Clone)]
struct Response {
    status: u16,
    body: String,
    headers: HashMap<String, String>,
}

impl Response {
    fn ok(body: &str) -> Self {
        Response {
            status: 200,
            body: body.to_string(),
            headers: HashMap::new(),
        }
    }

    fn error(status: u16, message: &str) -> Self {
        Response {
            status,
            body: message.to_string(),
            headers: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handler trait
// ---------------------------------------------------------------------------

trait Handler {
    fn handle(&self, req: &Request) -> Response;
}

// ---------------------------------------------------------------------------
// EchoHandler -- returns "{method} {path}" as the response body
// ---------------------------------------------------------------------------

struct EchoHandler;

impl Handler for EchoHandler {
    fn handle(&self, req: &Request) -> Response {
        Response::ok(&format!("{} {}", req.method, req.path))
    }
}

// ---------------------------------------------------------------------------
// LoggingMiddleware -- prints request/response info
// ---------------------------------------------------------------------------

struct LoggingMiddleware<H: Handler> {
    inner: H,
}

impl<H: Handler> LoggingMiddleware<H> {
    fn new(inner: H) -> Self {
        LoggingMiddleware { inner }
    }
}

impl<H: Handler> Handler for LoggingMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        println!("[log] {} {}", req.method, req.path);
        let response = self.inner.handle(req);
        println!("[log] {}", response.status);
        response
    }
}

// ---------------------------------------------------------------------------
// AuthMiddleware -- validates Authorization header
// ---------------------------------------------------------------------------

struct AuthMiddleware<H: Handler> {
    inner: H,
    valid_tokens: Vec<String>,
}

impl<H: Handler> AuthMiddleware<H> {
    fn new(inner: H, tokens: Vec<&str>) -> Self {
        AuthMiddleware {
            inner,
            valid_tokens: tokens.into_iter().map(String::from).collect(),
        }
    }
}

impl<H: Handler> Handler for AuthMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        // Skip auth for health endpoint
        if req.path == "/health" {
            return self.inner.handle(req);
        }

        match req.headers.get("Authorization") {
            None => Response::error(401, "unauthorized"),
            Some(token) if self.valid_tokens.contains(token) => {
                self.inner.handle(req)
            }
            Some(_) => Response::error(403, "forbidden"),
        }
    }
}

// ---------------------------------------------------------------------------
// RateLimitMiddleware -- tracks per-path request counts
//
// Uses RefCell for interior mutability: handle() takes &self but we need
// to mutate the counts HashMap. RefCell moves the borrow check to runtime.
// This is safe in single-threaded code. For multi-threaded, use Mutex.
// ---------------------------------------------------------------------------

struct RateLimitMiddleware<H: Handler> {
    inner: H,
    max_requests: u32,
    counts: RefCell<HashMap<String, u32>>,
}

impl<H: Handler> RateLimitMiddleware<H> {
    fn new(inner: H, max_requests: u32) -> Self {
        RateLimitMiddleware {
            inner,
            max_requests,
            counts: RefCell::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    fn reset(&self) {
        self.counts.borrow_mut().clear();
    }
}

impl<H: Handler> Handler for RateLimitMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        let mut counts = self.counts.borrow_mut();
        let count = counts.entry(req.path.clone()).or_insert(0);
        *count += 1;

        if *count > self.max_requests {
            return Response::error(429, "rate limit exceeded");
        }

        // Drop the borrow before calling inner (inner might also borrow something)
        drop(counts);

        self.inner.handle(req)
    }
}

// ---------------------------------------------------------------------------
// TimeoutMiddleware -- cooperative timeout (checks elapsed after completion)
//
// This is NOT a preemptive timeout. It calls the inner handler, then checks
// if the handler took too long. For true preemptive timeout, you need async
// with tokio::time::timeout or a separate thread.
// ---------------------------------------------------------------------------

struct TimeoutMiddleware<H: Handler> {
    inner: H,
    timeout: Duration,
}

impl<H: Handler> TimeoutMiddleware<H> {
    fn new(inner: H, timeout: Duration) -> Self {
        TimeoutMiddleware { inner, timeout }
    }
}

impl<H: Handler> Handler for TimeoutMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        let start = Instant::now();
        let response = self.inner.handle(req);
        let elapsed = start.elapsed();

        if elapsed > self.timeout {
            return Response::error(504, "gateway timeout");
        }

        response
    }
}

// ---------------------------------------------------------------------------
// MetricsMiddleware -- tracks request count and error count
//
// Uses Cell for interior mutability with simple numeric values.
// Cell<u64> is lighter than RefCell -- no runtime borrow checking needed
// because Cell copies the value in and out (no references to interior).
// ---------------------------------------------------------------------------

struct MetricsMiddleware<H: Handler> {
    inner: H,
    request_count: Cell<u64>,
    error_count: Cell<u64>,
}

impl<H: Handler> MetricsMiddleware<H> {
    fn new(inner: H) -> Self {
        MetricsMiddleware {
            inner,
            request_count: Cell::new(0),
            error_count: Cell::new(0),
        }
    }

    fn request_count(&self) -> u64 {
        self.request_count.get()
    }

    fn error_count(&self) -> u64 {
        self.error_count.get()
    }
}

impl<H: Handler> Handler for MetricsMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        self.request_count.set(self.request_count.get() + 1);

        let response = self.inner.handle(req);

        if response.status >= 400 {
            self.error_count.set(self.error_count.get() + 1);
        }

        response
    }
}

// ---------------------------------------------------------------------------
// Main: build the full pipeline and demonstrate it
// ---------------------------------------------------------------------------

fn main() {
    // Build the middleware stack (inside-out):
    //   MetricsMiddleware
    //     -> LoggingMiddleware
    //       -> TimeoutMiddleware
    //         -> AuthMiddleware
    //           -> RateLimitMiddleware
    //             -> EchoHandler
    //
    // Execution order (outside-in):
    //   1. Metrics: increment counter
    //   2. Logging: print request info
    //   3. Timeout: start timer
    //   4. Auth: check token
    //   5. RateLimit: check count
    //   6. Echo: produce response
    //   Then unwinds: timeout checks, logging prints status, metrics checks error

    let handler = MetricsMiddleware::new(
        LoggingMiddleware::new(
            TimeoutMiddleware::new(
                AuthMiddleware::new(
                    RateLimitMiddleware::new(EchoHandler, 5),
                    vec!["Bearer tok123", "Bearer admin"],
                ),
                Duration::from_millis(100),
            ),
        ),
    );

    println!("=== Middleware Pipeline Demo ===\n");

    // Request 1: Valid authenticated request
    println!("--- Request 1: GET /users (valid token) ---");
    let req = Request::new("GET", "/users")
        .with_header("Authorization", "Bearer tok123");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Request 2: Missing auth
    println!("--- Request 2: GET /users (no token) ---");
    let req = Request::new("GET", "/users");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Request 3: Health check (skips auth)
    println!("--- Request 3: GET /health (no auth needed) ---");
    let req = Request::new("GET", "/health");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Request 4: Wrong token
    println!("--- Request 4: POST /data (wrong token) ---");
    let req = Request::new("POST", "/data")
        .with_header("Authorization", "Bearer invalid");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Print metrics
    println!("=== Metrics ===");
    println!("Total requests: {}", handler.request_count());
    println!("Total errors: {}", handler.error_count());
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo_handler() {
        let handler = EchoHandler;
        let req = Request::new("GET", "/users");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "GET /users");
    }

    #[test]
    fn test_logging_passes_through() {
        let handler = LoggingMiddleware::new(EchoHandler);
        let req = Request::new("POST", "/data");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "POST /data");
    }

    #[test]
    fn test_auth_missing_token() {
        let handler = AuthMiddleware::new(EchoHandler, vec!["Bearer secret"]);
        let req = Request::new("GET", "/users");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 401);
    }

    #[test]
    fn test_auth_invalid_token() {
        let handler = AuthMiddleware::new(EchoHandler, vec!["Bearer secret"]);
        let req = Request::new("GET", "/users")
            .with_header("Authorization", "Bearer wrong");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn test_auth_valid_token() {
        let handler = AuthMiddleware::new(EchoHandler, vec!["Bearer secret"]);
        let req = Request::new("GET", "/users")
            .with_header("Authorization", "Bearer secret");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "GET /users");
    }

    #[test]
    fn test_auth_skips_health() {
        let handler = AuthMiddleware::new(EchoHandler, vec!["Bearer secret"]);
        let req = Request::new("GET", "/health");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_auth_multiple_tokens() {
        let handler = AuthMiddleware::new(EchoHandler, vec!["Bearer a", "Bearer b"]);

        let req_a = Request::new("GET", "/api")
            .with_header("Authorization", "Bearer a");
        let req_b = Request::new("GET", "/api")
            .with_header("Authorization", "Bearer b");

        assert_eq!(handler.handle(&req_a).status, 200);
        assert_eq!(handler.handle(&req_b).status, 200);
    }

    #[test]
    fn test_rate_limit_allows_under_limit() {
        let handler = RateLimitMiddleware::new(EchoHandler, 3);
        let req = Request::new("GET", "/api");

        for _ in 0..3 {
            let resp = handler.handle(&req);
            assert_eq!(resp.status, 200);
        }
    }

    #[test]
    fn test_rate_limit_blocks_over_limit() {
        let handler = RateLimitMiddleware::new(EchoHandler, 2);
        let req = Request::new("GET", "/api");

        handler.handle(&req); // 1
        handler.handle(&req); // 2
        let resp = handler.handle(&req); // 3 -- blocked
        assert_eq!(resp.status, 429);
    }

    #[test]
    fn test_rate_limit_per_path() {
        let handler = RateLimitMiddleware::new(EchoHandler, 1);

        let req_a = Request::new("GET", "/a");
        let req_b = Request::new("GET", "/b");

        assert_eq!(handler.handle(&req_a).status, 200);
        assert_eq!(handler.handle(&req_b).status, 200);

        // Second request to /a is blocked, /b still has one left
        assert_eq!(handler.handle(&req_a).status, 429);
    }

    #[test]
    fn test_rate_limit_reset() {
        let handler = RateLimitMiddleware::new(EchoHandler, 1);
        let req = Request::new("GET", "/api");

        handler.handle(&req);
        assert_eq!(handler.handle(&req).status, 429);

        handler.reset();
        assert_eq!(handler.handle(&req).status, 200);
    }

    #[test]
    fn test_metrics_counts_requests() {
        let handler = MetricsMiddleware::new(EchoHandler);
        let req = Request::new("GET", "/api");

        handler.handle(&req);
        handler.handle(&req);
        handler.handle(&req);

        assert_eq!(handler.request_count(), 3);
        assert_eq!(handler.error_count(), 0);
    }

    #[test]
    fn test_metrics_counts_errors() {
        let handler = MetricsMiddleware::new(
            AuthMiddleware::new(EchoHandler, vec!["Bearer valid"]),
        );

        let good_req = Request::new("GET", "/api")
            .with_header("Authorization", "Bearer valid");
        let bad_req = Request::new("GET", "/api");

        handler.handle(&good_req);
        handler.handle(&bad_req);
        handler.handle(&bad_req);

        assert_eq!(handler.request_count(), 3);
        assert_eq!(handler.error_count(), 2);
    }

    #[test]
    fn test_timeout_passes_fast_request() {
        let handler = TimeoutMiddleware::new(EchoHandler, Duration::from_secs(1));
        let req = Request::new("GET", "/fast");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_composed_stack() {
        let handler = MetricsMiddleware::new(
            LoggingMiddleware::new(
                AuthMiddleware::new(
                    RateLimitMiddleware::new(EchoHandler, 10),
                    vec!["Bearer tok123"],
                ),
            ),
        );

        // Valid request
        let req = Request::new("GET", "/users")
            .with_header("Authorization", "Bearer tok123");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "GET /users");

        // Unauthorized request
        let req = Request::new("GET", "/users");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 401);

        assert_eq!(handler.request_count(), 2);
        assert_eq!(handler.error_count(), 1);
    }

    #[test]
    fn test_full_pipeline_health_bypass() {
        let handler = MetricsMiddleware::new(
            LoggingMiddleware::new(
                TimeoutMiddleware::new(
                    AuthMiddleware::new(
                        RateLimitMiddleware::new(EchoHandler, 100),
                        vec!["Bearer tok"],
                    ),
                    Duration::from_secs(1),
                ),
            ),
        );

        // Health check with no auth should succeed
        let req = Request::new("GET", "/health");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, "GET /health");
    }

    #[test]
    fn test_echo_various_methods() {
        let handler = EchoHandler;
        for method in &["GET", "POST", "PUT", "DELETE", "PATCH"] {
            let req = Request::new(method, "/resource");
            let resp = handler.handle(&req);
            assert_eq!(resp.body, format!("{} /resource", method));
        }
    }
}
