// Chain of Responsibility: Request Validation Chain
//
// Demonstrates: Trait-based handler chain with Box<dyn Handler>
//
// Scenario: An API gateway validates incoming requests through a chain of
// validators. Each validator checks one aspect (format, size, auth token,
// content type) and either passes the request to the next validator or
// short-circuits with an error.
//
// Run: rustc validation.rs && ./validation

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    fn new(method: &str, path: &str, body: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: body.to_string(),
        }
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ValidationError {
    validator: String,
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.validator, self.message)
    }
}

// ---------------------------------------------------------------------------
// Handler trait
// ---------------------------------------------------------------------------

/// Each validator in the chain implements this trait.
/// Object safe: no generics, no Self return, &self receiver.
trait Validator: fmt::Display {
    /// Validate the request. Returns Ok(()) to pass, Err to reject.
    fn validate(&self, request: &Request) -> Result<(), ValidationError>;

    /// Name of this validator (used in error messages and logging).
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Validation chain — owns a Vec of validators, runs them in order
// ---------------------------------------------------------------------------

/// Runs validators sequentially. Short-circuits on first error.
///
/// This uses a Vec rather than a linked list because:
/// 1. Validators don't need to know about each other
/// 2. The chain is always fully traversed (no branching)
/// 3. Vec is cache-friendly and easy to build dynamically
struct ValidationChain {
    validators: Vec<Box<dyn Validator>>,
}

impl ValidationChain {
    fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    fn add<V: Validator + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Box::new(validator));
        self
    }

    /// Run all validators. Returns Ok if all pass, Err on first failure.
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        for validator in &self.validators {
            println!("  Running validator: {}", validator.name());
            validator.validate(request)?; // ? short-circuits on Err
        }
        Ok(())
    }

    /// Run all validators, collecting ALL errors instead of short-circuiting.
    fn validate_all(&self, request: &Request) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for validator in &self.validators {
            if let Err(e) = validator.validate(request) {
                errors.push(e);
            }
        }
        errors
    }
}

// ---------------------------------------------------------------------------
// Concrete validators
// ---------------------------------------------------------------------------

/// Checks that the request body is valid JSON (naive check: starts with { or [).
struct FormatValidator;

impl Validator for FormatValidator {
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        if request.body.is_empty() {
            return Ok(()); // Empty body is allowed (e.g., GET requests)
        }
        let trimmed = request.body.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            Ok(())
        } else {
            Err(ValidationError {
                validator: self.name().to_string(),
                message: format!(
                    "expected JSON body, got: {}...",
                    &request.body[..request.body.len().min(20)]
                ),
            })
        }
    }

    fn name(&self) -> &str {
        "format"
    }
}

impl fmt::Display for FormatValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FormatValidator(JSON)")
    }
}

/// Checks that the request body does not exceed a size limit.
struct SizeLimitValidator {
    max_bytes: usize,
}

impl SizeLimitValidator {
    fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }
}

impl Validator for SizeLimitValidator {
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        if request.body.len() > self.max_bytes {
            Err(ValidationError {
                validator: self.name().to_string(),
                message: format!(
                    "body size {} exceeds limit of {} bytes",
                    request.body.len(),
                    self.max_bytes
                ),
            })
        } else {
            Ok(())
        }
    }

    fn name(&self) -> &str {
        "size_limit"
    }
}

impl fmt::Display for SizeLimitValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SizeLimitValidator(max={})", self.max_bytes)
    }
}

/// Checks that the request contains a valid auth token.
struct AuthTokenValidator {
    valid_tokens: Vec<String>,
}

impl AuthTokenValidator {
    fn new(tokens: Vec<&str>) -> Self {
        Self {
            valid_tokens: tokens.into_iter().map(String::from).collect(),
        }
    }
}

impl Validator for AuthTokenValidator {
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        let token = request.headers.get("authorization").ok_or_else(|| {
            ValidationError {
                validator: self.name().to_string(),
                message: "missing Authorization header".to_string(),
            }
        })?;

        if self.valid_tokens.iter().any(|t| t == token) {
            Ok(())
        } else {
            Err(ValidationError {
                validator: self.name().to_string(),
                message: "invalid authorization token".to_string(),
            })
        }
    }

    fn name(&self) -> &str {
        "auth_token"
    }
}

impl fmt::Display for AuthTokenValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AuthTokenValidator({} tokens)", self.valid_tokens.len())
    }
}

/// Checks that the Content-Type header matches an expected value.
struct ContentTypeValidator {
    expected: String,
}

impl ContentTypeValidator {
    fn new(expected: &str) -> Self {
        Self {
            expected: expected.to_string(),
        }
    }
}

impl Validator for ContentTypeValidator {
    fn validate(&self, request: &Request) -> Result<(), ValidationError> {
        if request.body.is_empty() {
            return Ok(()); // No body, no content type needed
        }

        let ct = request.headers.get("content-type").ok_or_else(|| {
            ValidationError {
                validator: self.name().to_string(),
                message: "missing Content-Type header".to_string(),
            }
        })?;

        if ct.starts_with(&self.expected) {
            Ok(())
        } else {
            Err(ValidationError {
                validator: self.name().to_string(),
                message: format!("expected {}, got {}", self.expected, ct),
            })
        }
    }

    fn name(&self) -> &str {
        "content_type"
    }
}

impl fmt::Display for ContentTypeValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentTypeValidator({})", self.expected)
    }
}

// ---------------------------------------------------------------------------
// Main — demonstrate the chain
// ---------------------------------------------------------------------------

fn main() {
    // Build the validation chain
    let chain = ValidationChain::new()
        .add(FormatValidator)
        .add(SizeLimitValidator::new(1024))
        .add(AuthTokenValidator::new(vec!["Bearer abc123", "Bearer xyz789"]))
        .add(ContentTypeValidator::new("application/json"));

    println!("=== Test 1: Valid request ===");
    let req = Request::new("POST", "/api/users", r#"{"name": "Alice"}"#)
        .with_header("authorization", "Bearer abc123")
        .with_header("content-type", "application/json");
    match chain.validate(&req) {
        Ok(()) => println!("  PASSED: all validators accepted the request\n"),
        Err(e) => println!("  REJECTED: {}\n", e),
    }

    println!("=== Test 2: Invalid JSON format ===");
    let req = Request::new("POST", "/api/users", "not json at all")
        .with_header("authorization", "Bearer abc123")
        .with_header("content-type", "text/plain");
    match chain.validate(&req) {
        Ok(()) => println!("  PASSED\n"),
        Err(e) => println!("  REJECTED: {} (chain stopped here)\n", e),
    }

    println!("=== Test 3: Missing auth token ===");
    let req = Request::new("POST", "/api/users", r#"{"name": "Bob"}"#)
        .with_header("content-type", "application/json");
    match chain.validate(&req) {
        Ok(()) => println!("  PASSED\n"),
        Err(e) => println!("  REJECTED: {} (chain stopped here)\n", e),
    }

    println!("=== Test 4: Body too large ===");
    let large_body = format!(r#"{{"data": "{}"}}"#, "x".repeat(2000));
    let req = Request::new("POST", "/api/upload", &large_body)
        .with_header("authorization", "Bearer abc123")
        .with_header("content-type", "application/json");
    match chain.validate(&req) {
        Ok(()) => println!("  PASSED\n"),
        Err(e) => println!("  REJECTED: {} (chain stopped here)\n", e),
    }

    println!("=== Test 5: Collect ALL errors (no short-circuit) ===");
    let req = Request::new("POST", "/api/users", "invalid body that is also quite long");
    let errors = chain.validate_all(&req);
    if errors.is_empty() {
        println!("  PASSED: no errors\n");
    } else {
        println!("  Found {} errors:", errors.len());
        for e in &errors {
            println!("    - {}", e);
        }
        println!();
    }

    println!("=== Test 6: GET request with empty body (should pass format + size + content-type) ===");
    let req = Request::new("GET", "/api/users", "")
        .with_header("authorization", "Bearer xyz789");
    match chain.validate(&req) {
        Ok(()) => println!("  PASSED: GET requests with empty body are fine\n"),
        Err(e) => println!("  REJECTED: {}\n", e),
    }
}
