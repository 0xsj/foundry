// Decorator Pattern: Request/Response Middleware Stack
//
// Demonstrates: Trait-based decoration with both generic (static dispatch)
// and trait object (dynamic dispatch) approaches. Builds a middleware stack
// for an HTTP-like request/response pipeline.
//
// Concepts: trait impl delegation, execution order, Box<dyn Trait>,
// Send + Sync considerations, Debug for stack introspection.
//
// Run: rustc middleware.rs && ./middleware

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

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

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

// ---------------------------------------------------------------------------
// Handler trait (the interface every decorator implements)
// ---------------------------------------------------------------------------

trait Handler: fmt::Debug {
    fn handle(&self, req: &Request) -> Response;
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Core handler: the business logic
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ApiHandler;

impl Handler for ApiHandler {
    fn handle(&self, req: &Request) -> Response {
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/health") => Response::ok(r#"{"status": "healthy"}"#),
            ("GET", "/users") => Response::ok(r#"[{"id": 1, "name": "alice"}]"#),
            ("POST", "/users") => {
                if req.body.is_empty() {
                    Response::error(400, "request body required")
                } else {
                    Response::ok(r#"{"id": 2, "created": true}"#)
                        .with_header("Location", "/users/2")
                }
            }
            _ => Response::error(404, "not found"),
        }
    }

    fn name(&self) -> &str {
        "api"
    }
}

// ---------------------------------------------------------------------------
// Decorator 1: Logging
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct LoggingMiddleware<H: Handler> {
    inner: H,
    prefix: String,
}

impl<H: Handler> LoggingMiddleware<H> {
    fn new(inner: H, prefix: &str) -> Self {
        LoggingMiddleware {
            inner,
            prefix: prefix.to_string(),
        }
    }
}

impl<H: Handler> Handler for LoggingMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        println!("[{}] --> {} {}", self.prefix, req.method, req.path);
        let response = self.inner.handle(req);
        println!("[{}] <-- {} (status: {})", self.prefix, req.path, response.status);
        response
    }

    fn name(&self) -> &str {
        "logging"
    }
}

// ---------------------------------------------------------------------------
// Decorator 2: Timing
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TimingMiddleware<H: Handler> {
    inner: H,
}

impl<H: Handler> TimingMiddleware<H> {
    fn new(inner: H) -> Self {
        TimingMiddleware { inner }
    }
}

impl<H: Handler> Handler for TimingMiddleware<H> {
    fn handle(&self, req: &Request) -> Response {
        let start = Instant::now();
        let response = self.inner.handle(req);
        let elapsed = start.elapsed();
        println!("[timing] {} {} took {:?}", req.method, req.path, elapsed);
        response
            .with_header("X-Response-Time", &format!("{}us", elapsed.as_micros()))
    }

    fn name(&self) -> &str {
        "timing"
    }
}

// ---------------------------------------------------------------------------
// Decorator 3: Authentication
// ---------------------------------------------------------------------------

#[derive(Debug)]
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
        // Skip auth for health checks
        if req.path == "/health" {
            return self.inner.handle(req);
        }

        match req.headers.get("Authorization") {
            Some(token) if self.valid_tokens.contains(token) => {
                self.inner.handle(req)
            }
            Some(_) => Response::error(403, "invalid token"),
            None => Response::error(401, "authorization required"),
        }
    }

    fn name(&self) -> &str {
        "auth"
    }
}

// ---------------------------------------------------------------------------
// Dynamic dispatch version (for runtime-configurable stacks)
// ---------------------------------------------------------------------------

struct DynLoggingMiddleware {
    inner: Box<dyn Handler>,
    prefix: String,
}

impl fmt::Debug for DynLoggingMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynLogging")
            .field("prefix", &self.prefix)
            .field("inner", &self.inner.name())
            .finish()
    }
}

impl Handler for DynLoggingMiddleware {
    fn handle(&self, req: &Request) -> Response {
        println!("[dyn-{}] --> {} {}", self.prefix, req.method, req.path);
        let response = self.inner.handle(req);
        println!("[dyn-{}] <-- status {}", self.prefix, response.status);
        response
    }

    fn name(&self) -> &str {
        "dyn-logging"
    }
}

struct DynAuthMiddleware {
    inner: Box<dyn Handler>,
    valid_tokens: Vec<String>,
}

impl fmt::Debug for DynAuthMiddleware {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynAuth")
            .field("tokens", &self.valid_tokens.len())
            .field("inner", &self.inner.name())
            .finish()
    }
}

impl Handler for DynAuthMiddleware {
    fn handle(&self, req: &Request) -> Response {
        if req.path == "/health" {
            return self.inner.handle(req);
        }
        match req.headers.get("Authorization") {
            Some(token) if self.valid_tokens.contains(token) => {
                self.inner.handle(req)
            }
            Some(_) => Response::error(403, "invalid token"),
            None => Response::error(401, "authorization required"),
        }
    }

    fn name(&self) -> &str {
        "dyn-auth"
    }
}

/// Build a middleware stack from a simple configuration.
/// This is impossible with generics alone -- the stack shape
/// depends on runtime values.
fn build_dynamic_stack(enable_logging: bool, enable_auth: bool) -> Box<dyn Handler> {
    let mut handler: Box<dyn Handler> = Box::new(ApiHandler);

    if enable_auth {
        handler = Box::new(DynAuthMiddleware {
            inner: handler,
            valid_tokens: vec!["Bearer secret123".to_string()],
        });
    }

    if enable_logging {
        handler = Box::new(DynLoggingMiddleware {
            inner: handler,
            prefix: "app".to_string(),
        });
    }

    handler
}

// ---------------------------------------------------------------------------
// Helper: print middleware stack
// ---------------------------------------------------------------------------

fn print_stack_info(handler: &dyn Handler) {
    println!("Stack head: {} ({:?})", handler.name(), handler);
}

// ---------------------------------------------------------------------------
// Main: demonstrate both approaches
// ---------------------------------------------------------------------------

fn main() {
    println!("=== STATIC DISPATCH MIDDLEWARE STACK ===\n");

    // Build stack: Auth -> Timing -> Logging -> ApiHandler
    // Execution order (outermost first): Auth checks token, Timing starts clock,
    // Logging prints, ApiHandler processes, then unwinds.
    let handler = AuthMiddleware::new(
        TimingMiddleware::new(
            LoggingMiddleware::new(
                ApiHandler,
                "api",
            ),
        ),
        vec!["Bearer secret123"],
    );

    // The full type is: AuthMiddleware<TimingMiddleware<LoggingMiddleware<ApiHandler>>>
    // All dispatch is static -- zero vtable overhead.
    println!("Handler type (compile-time): AuthMiddleware<TimingMiddleware<LoggingMiddleware<ApiHandler>>>\n");

    // Test 1: Health check (skips auth)
    println!("--- Request: GET /health (no auth needed) ---");
    let req = Request::new("GET", "/health");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Test 2: Authenticated request
    println!("--- Request: GET /users (valid token) ---");
    let req = Request::new("GET", "/users")
        .with_header("Authorization", "Bearer secret123");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Test 3: Missing auth
    println!("--- Request: GET /users (no token) ---");
    let req = Request::new("GET", "/users");
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Test 4: Invalid token
    println!("--- Request: POST /users (bad token) ---");
    let req = Request::new("POST", "/users")
        .with_header("Authorization", "Bearer wrong")
        .with_body(r#"{"name": "bob"}"#);
    let resp = handler.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    println!("\n=== DYNAMIC DISPATCH MIDDLEWARE STACK ===\n");

    // Build stack from configuration (runtime decision)
    let stack = build_dynamic_stack(true, true);
    print_stack_info(stack.as_ref());
    println!();

    println!("--- Request: GET /users (valid token, dynamic stack) ---");
    let req = Request::new("GET", "/users")
        .with_header("Authorization", "Bearer secret123");
    let resp = stack.handle(&req);
    println!("Response: {} {}\n", resp.status, resp.body);

    // Stack without auth
    println!("--- Building stack with logging only (no auth) ---");
    let stack = build_dynamic_stack(true, false);
    print_stack_info(stack.as_ref());
    println!();

    let req = Request::new("GET", "/users");
    let resp = stack.handle(&req);
    println!("Response: {} {}", resp.status, resp.body);
}
