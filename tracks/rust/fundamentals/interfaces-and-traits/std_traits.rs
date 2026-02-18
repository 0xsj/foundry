// Standard Traits — Interfaces & Traits (Rust)
//
// Covers: Display, Debug, From/Into, Default, Clone/Copy,
//         PartialEq/Eq/Hash, Iterator, operator overloading.
//
// Run: rustc std_traits.rs && ./std_traits

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Add, Mul};

// -------------------------------------------------------------------------
// 1. Display and Debug
// -------------------------------------------------------------------------

/// A structured API error. Debug is auto-derived. Display is hand-written.
#[derive(Debug, Clone, PartialEq)]
struct ApiError {
    status: u16,
    code: String,
    message: String,
}

// Display: human-readable, for logs and end-user messages.
// Use {} in format strings.
impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.status, self.code, self.message)
    }
}

// A type with sensitive data — custom Debug to redact secrets.
struct ServiceCredentials {
    client_id: String,
    client_secret: String,   // never log this
    token_endpoint: String,
}

impl fmt::Debug for ServiceCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceCredentials")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("token_endpoint", &self.token_endpoint)
            .finish()
    }
}

// -------------------------------------------------------------------------
// 2. From and Into
// -------------------------------------------------------------------------

/// A newtype for typed IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RequestId(u64);

// Implementing From<u64> for UserId gives us Into<UserId> for u64 automatically.
impl From<u64> for UserId {
    fn from(n: u64) -> UserId {
        UserId(n)
    }
}

impl From<u64> for RequestId {
    fn from(n: u64) -> RequestId {
        RequestId(n)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user:{}", self.0)
    }
}

// From also works for error conversions with the ? operator.
#[derive(Debug)]
enum AppError {
    NotFound(String),
    InvalidInput(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg)     => write!(f, "not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            AppError::Internal(msg)     => write!(f, "internal error: {}", msg),
        }
    }
}

// Converting std::num::ParseIntError into AppError via From — enables ? in functions.
impl From<std::num::ParseIntError> for AppError {
    fn from(e: std::num::ParseIntError) -> AppError {
        AppError::InvalidInput(e.to_string())
    }
}

fn parse_user_id(s: &str) -> Result<UserId, AppError> {
    let n: u64 = s.parse()?;  // ? converts ParseIntError -> AppError via From
    Ok(UserId(n))
}

// Accept impl Into<UserId> to allow callers to pass either u64 or UserId:
fn load_user(id: impl Into<UserId>) -> Option<String> {
    let id = id.into();
    // Pretend we look it up
    if id.0 == 1 {
        Some(String::from("alice"))
    } else {
        None
    }
}

// -------------------------------------------------------------------------
// 3. Default
// -------------------------------------------------------------------------

/// Service configuration with sensible production defaults.
/// Derive Default for the zero-value version, then override with manual impl.
#[derive(Debug, Clone, PartialEq)]
struct ServiceConfig {
    host: String,
    port: u16,
    timeout_ms: u64,
    max_retries: u8,
    enable_tls: bool,
}

// The derived Default would give ("", 0, 0, 0, false).
// We want sensible non-zero defaults for a production service.
impl Default for ServiceConfig {
    fn default() -> Self {
        ServiceConfig {
            host: String::from("localhost"),
            port: 8080,
            timeout_ms: 5000,
            max_retries: 3,
            enable_tls: false,
        }
    }
}

// Pattern: ..Default::default() to fill in the fields you don't override.
fn make_tls_config(host: &str, port: u16) -> ServiceConfig {
    ServiceConfig {
        host: host.to_string(),
        port,
        enable_tls: true,
        ..ServiceConfig::default()
    }
}

// -------------------------------------------------------------------------
// 4. Clone and Copy
// -------------------------------------------------------------------------

// Copy: small, stack-only type. No String, no Vec.
// Values are implicitly copied (like integers) on assignment or function call.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Endpoint {
    port: u16,
    protocol: Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Protocol {
    Http,
    Https,
    Grpc,
}

// Clone only: contains a String (heap-allocated) — cannot be Copy.
#[derive(Debug, Clone, PartialEq)]
struct NamedEndpoint {
    name: String,
    port: u16,
}

// -------------------------------------------------------------------------
// 5. PartialEq, Eq, Hash — for collections
// -------------------------------------------------------------------------

// Full derive chain for use as HashMap keys:
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RouteKey {
    method: String,
    path: String,
}

impl RouteKey {
    fn new(method: &str, path: &str) -> Self {
        RouteKey {
            method: method.to_uppercase(),
            path: path.to_string(),
        }
    }
}

// -------------------------------------------------------------------------
// 6. Iterator
// -------------------------------------------------------------------------

/// A rate-limited sequence of token bucket refills.
/// Implements Iterator: yields u64 token counts at each step.
struct TokenRefillIter {
    current_tokens: u64,
    capacity: u64,
    refill_amount: u64,
    steps_remaining: u32,
}

impl TokenRefillIter {
    fn new(start: u64, capacity: u64, refill: u64, steps: u32) -> Self {
        TokenRefillIter {
            current_tokens: start,
            capacity,
            refill_amount: refill,
            steps_remaining: steps,
        }
    }
}

impl Iterator for TokenRefillIter {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.steps_remaining == 0 {
            return None;
        }
        self.steps_remaining -= 1;
        // Refill — cap at capacity
        self.current_tokens = (self.current_tokens + self.refill_amount).min(self.capacity);
        Some(self.current_tokens)
    }
}

// -------------------------------------------------------------------------
// 7. Operator overloading
// -------------------------------------------------------------------------

/// An HTTP rate limit budget — tracks allowed requests per window.
/// Overloads + to combine two budgets and * to scale a budget.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RateLimit {
    requests_per_second: u32,
    burst_capacity: u32,
}

impl RateLimit {
    fn new(rps: u32, burst: u32) -> Self {
        RateLimit { requests_per_second: rps, burst_capacity: burst }
    }
}

impl Add for RateLimit {
    type Output = RateLimit;

    // Add two limits (e.g., combine two upstream pools)
    fn add(self, rhs: RateLimit) -> RateLimit {
        RateLimit {
            requests_per_second: self.requests_per_second + rhs.requests_per_second,
            burst_capacity: self.burst_capacity + rhs.burst_capacity,
        }
    }
}

impl Mul<u32> for RateLimit {
    type Output = RateLimit;

    // Scale a limit (e.g., double capacity during peak hours)
    fn mul(self, factor: u32) -> RateLimit {
        RateLimit {
            requests_per_second: self.requests_per_second * factor,
            burst_capacity: self.burst_capacity * factor,
        }
    }
}

impl fmt::Display for RateLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}rps (burst: {})", self.requests_per_second, self.burst_capacity)
    }
}

// -------------------------------------------------------------------------
// main
// -------------------------------------------------------------------------

fn main() {
    println!("=== 1. Display and Debug ===\n");

    let err = ApiError {
        status: 404,
        code: String::from("USER_NOT_FOUND"),
        message: String::from("no user with id 42"),
    };
    println!("Display: {}", err);
    println!("Debug:   {:?}", err);

    let creds = ServiceCredentials {
        client_id: String::from("svc-payments"),
        client_secret: String::from("s3cr3t-t0k3n"),
        token_endpoint: String::from("https://auth.internal/token"),
    };
    println!("Redacted debug: {:?}", creds);

    println!("\n=== 2. From and Into ===\n");

    // From
    let id = UserId::from(42);
    println!("UserId::from(42) = {:?}", id);

    // Into (derived from From)
    let id2: UserId = 99_u64.into();
    println!("99_u64.into() = {:?}", id2);

    // load_user accepts impl Into<UserId>
    println!("load_user(1u64) = {:?}", load_user(1_u64));
    println!("load_user(UserId(1)) = {:?}", load_user(UserId(1)));

    // From for error conversion
    match parse_user_id("abc") {
        Ok(id) => println!("parsed: {:?}", id),
        Err(e) => println!("error (converted via From): {}", e),
    }
    match parse_user_id("7") {
        Ok(id) => println!("parsed: {}", id),
        Err(e) => println!("error: {}", e),
    }

    println!("\n=== 3. Default ===\n");

    let default_cfg = ServiceConfig::default();
    println!("default: {:?}", default_cfg);

    let tls_cfg = make_tls_config("api.example.com", 443);
    println!("tls override: {:?}", tls_cfg);

    println!("\n=== 4. Clone and Copy ===\n");

    // Copy: implicit — ep1 still usable after assignment
    let ep1 = Endpoint { port: 8080, protocol: Protocol::Http };
    let ep2 = ep1;  // bitwise copy — ep1 not moved
    println!("ep1 = {:?}", ep1);
    println!("ep2 = {:?}", ep2);

    // Clone: explicit — named_ep1 still usable after .clone()
    let named_ep1 = NamedEndpoint { name: String::from("primary"), port: 443 };
    let named_ep2 = named_ep1.clone();
    println!("named_ep1 = {:?}", named_ep1);
    println!("named_ep2 = {:?}", named_ep2);

    println!("\n=== 5. PartialEq, Eq, Hash ===\n");

    let mut routing_table: HashMap<RouteKey, String> = HashMap::new();
    routing_table.insert(RouteKey::new("GET", "/api/users"), String::from("UserListHandler"));
    routing_table.insert(RouteKey::new("POST", "/api/users"), String::from("UserCreateHandler"));
    routing_table.insert(RouteKey::new("GET", "/api/orders"), String::from("OrderListHandler"));

    let lookup = RouteKey::new("get", "/api/users");  // normalized to uppercase
    println!("GET /api/users -> {:?}", routing_table.get(&lookup));

    let mut seen_ids: HashSet<UserId> = HashSet::new();
    seen_ids.insert(UserId(1));
    seen_ids.insert(UserId(2));
    seen_ids.insert(UserId(1));  // duplicate
    println!("unique user IDs seen: {}", seen_ids.len()); // 2

    println!("\n=== 6. Iterator ===\n");

    let refills = TokenRefillIter::new(0, 100, 20, 7);
    let levels: Vec<u64> = refills.collect();
    println!("token refill levels: {:?}", levels);

    // Iterator adapters — all provided by the Iterator trait for free:
    let above_50_count = TokenRefillIter::new(0, 100, 20, 7)
        .filter(|&t| t > 50)
        .count();
    println!("steps above 50 tokens: {}", above_50_count);

    let total_tokens: u64 = TokenRefillIter::new(10, 100, 15, 5).sum();
    println!("total tokens over 5 steps: {}", total_tokens);

    println!("\n=== 7. Operator overloading ===\n");

    let primary = RateLimit::new(100, 500);
    let secondary = RateLimit::new(50, 200);

    let combined = primary + secondary;
    println!("primary + secondary = {}", combined);

    let scaled = primary * 3;
    println!("primary * 3 = {}", scaled);
}
