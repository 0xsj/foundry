// Middleware Pipeline -- Starter
//
// Implement the Handler trait and five middleware types.
// Run tests: rustc --test main.rs && ./main
// Run program: rustc main.rs && ./main

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
// EchoHandler -- core handler
// ---------------------------------------------------------------------------

struct EchoHandler;

// TODO: Implement Handler for EchoHandler
// Should return Response::ok with body: "{method} {path}"

// ---------------------------------------------------------------------------
// LoggingMiddleware
// ---------------------------------------------------------------------------

struct LoggingMiddleware<H: Handler> {
    inner: H,
}

impl<H: Handler> LoggingMiddleware<H> {
    fn new(inner: H) -> Self {
        LoggingMiddleware { inner }
    }
}

// TODO: Implement Handler for LoggingMiddleware
// Before calling inner: println!("[log] {} {}", req.method, req.path)
// After calling inner: println!("[log] {}", response.status)
// Return the inner handler's response

// ---------------------------------------------------------------------------
// AuthMiddleware
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

// TODO: Implement Handler for AuthMiddleware
// - If path is "/health", skip auth and call inner directly
// - If "Authorization" header is missing, return Response::error(401, "unauthorized")
// - If "Authorization" header value is not in valid_tokens, return Response::error(403, "forbidden")
// - Otherwise, call inner handler

// ---------------------------------------------------------------------------
// RateLimitMiddleware
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

    fn reset(&self) {
        self.counts.borrow_mut().clear();
    }
}

// TODO: Implement Handler for RateLimitMiddleware
// - Increment count for req.path in self.counts
// - If count > self.max_requests, return Response::error(429, "rate limit exceeded")
// - Otherwise, call inner handler

// ---------------------------------------------------------------------------
// TimeoutMiddleware
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

// TODO: Implement Handler for TimeoutMiddleware
// - Record start time with Instant::now()
// - Call inner handler
// - If elapsed > self.timeout, return Response::error(504, "gateway timeout")
// - Otherwise, return the inner handler's response

// ---------------------------------------------------------------------------
// MetricsMiddleware
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

// TODO: Implement Handler for MetricsMiddleware
// - Increment request_count
// - Call inner handler
// - If response.status >= 400, increment error_count
// - Return the response

// ---------------------------------------------------------------------------
// Main: build the pipeline and demonstrate it
// ---------------------------------------------------------------------------

fn main() {
    // TODO: Build a composed middleware stack:
    //   MetricsMiddleware<LoggingMiddleware<TimeoutMiddleware<AuthMiddleware<RateLimitMiddleware<EchoHandler>>>>>
    //
    // Then send several test requests through it and print the results.

    println!("TODO: implement the middleware pipeline");
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
        // No auth header, but health endpoint should pass through
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
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
        let resp = handler.handle(&req); // 3 -- should be blocked
        assert_eq!(resp.status, 429);
    }

    #[test]
    fn test_rate_limit_per_path() {
        let handler = RateLimitMiddleware::new(EchoHandler, 1);

        let req_a = Request::new("GET", "/a");
        let req_b = Request::new("GET", "/b");

        let resp_a = handler.handle(&req_a);
        let resp_b = handler.handle(&req_b);
        assert_eq!(resp_a.status, 200);
        assert_eq!(resp_b.status, 200);

        // Second request to /a should be blocked
        let resp_a2 = handler.handle(&req_a);
        assert_eq!(resp_a2.status, 429);
    }

    #[test]
    fn test_rate_limit_reset() {
        let handler = RateLimitMiddleware::new(EchoHandler, 1);
        let req = Request::new("GET", "/api");

        handler.handle(&req); // 1 -- ok
        let resp = handler.handle(&req); // 2 -- blocked
        assert_eq!(resp.status, 429);

        handler.reset();
        let resp = handler.handle(&req); // 1 again after reset
        assert_eq!(resp.status, 200);
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
        // Auth middleware will produce 401 errors, which metrics should count
        let handler = MetricsMiddleware::new(
            AuthMiddleware::new(EchoHandler, vec!["Bearer valid"]),
        );

        let good_req = Request::new("GET", "/api")
            .with_header("Authorization", "Bearer valid");
        let bad_req = Request::new("GET", "/api"); // no auth

        handler.handle(&good_req);
        handler.handle(&bad_req);
        handler.handle(&bad_req);

        assert_eq!(handler.request_count(), 3);
        assert_eq!(handler.error_count(), 2);
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
    fn test_timeout_passes_fast_request() {
        let handler = TimeoutMiddleware::new(EchoHandler, Duration::from_secs(1));
        let req = Request::new("GET", "/fast");
        let resp = handler.handle(&req);
        assert_eq!(resp.status, 200);
    }
}
