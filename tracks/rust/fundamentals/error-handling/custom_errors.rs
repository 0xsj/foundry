// custom_errors.rs — Custom error types, From implementations, and thiserror
//
// Run: rustc custom_errors.rs && ./custom_errors
//
// NOTE: This file does NOT use external crates (no thiserror).
// It shows the manual implementation to build understanding, then at the bottom
// shows what thiserror generates for you. To use thiserror in a real project,
// add it to Cargo.toml: thiserror = "2"
//
// The goal: understand what you're asking thiserror to generate, not just copy
// the derive syntax blindly.

use std::fmt;
use std::num::ParseIntError;

// ---------------------------------------------------------------------------
// 1. A minimal custom error type — for a webhook validation service
// ---------------------------------------------------------------------------

/// Errors that can occur when processing an incoming webhook payload.
#[derive(Debug)]
pub enum WebhookError {
    /// The payload JSON could not be parsed.
    InvalidJson(String),
    /// A required field is missing from the payload.
    MissingField(String),
    /// A field value is present but invalid.
    InvalidField { field: String, value: String, reason: String },
    /// The signature header does not match the expected HMAC.
    SignatureMismatch,
    /// An I/O error occurred reading the request body.
    Io(std::io::Error),
}

// Step 1: Implement Display — this is the human-readable error message.
// It answers: "what went wrong, expressed as a complete sentence."
impl fmt::Display for WebhookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebhookError::InvalidJson(msg) => {
                write!(f, "invalid JSON payload: {}", msg)
            }
            WebhookError::MissingField(field) => {
                write!(f, "required field '{}' is missing from payload", field)
            }
            WebhookError::InvalidField { field, value, reason } => {
                write!(f, "invalid value '{}' for field '{}': {}", value, field, reason)
            }
            WebhookError::SignatureMismatch => {
                write!(f, "webhook signature verification failed")
            }
            WebhookError::Io(e) => {
                write!(f, "I/O error reading webhook body: {}", e)
            }
        }
    }
}

// Step 2: Implement std::error::Error.
// The source() method chains to the underlying cause where one exists.
impl std::error::Error for WebhookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // The Io variant wraps a std::io::Error — expose it as the source
            WebhookError::Io(e) => Some(e),
            // Other variants don't have an underlying cause
            _ => None,
        }
    }
}

// Step 3: From impls — enable ? operator to convert lower-level errors.
// With this impl, any function returning Result<_, WebhookError> can use ?
// on any operation that returns Result<_, std::io::Error>.
impl From<std::io::Error> for WebhookError {
    fn from(e: std::io::Error) -> Self {
        WebhookError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// 2. Layered error types — a service that wraps domain errors
// ---------------------------------------------------------------------------

/// A parsed webhook payload (simplified for demonstration).
#[derive(Debug)]
pub struct WebhookPayload {
    pub event_type: String,
    pub source_id: u64,
    pub retry_count: u32,
}

/// Parse and validate a raw webhook payload represented as key=value pairs.
pub fn parse_webhook_payload(
    raw: &[(&str, &str)],
) -> Result<WebhookPayload, WebhookError> {
    let map: std::collections::HashMap<&str, &str> = raw.iter().cloned().collect();

    // ok_or_else converts Option -> Result; ? propagates WebhookError
    let event_type = map
        .get("event_type")
        .ok_or_else(|| WebhookError::MissingField("event_type".to_string()))?
        .to_string();

    let source_id_str = map
        .get("source_id")
        .ok_or_else(|| WebhookError::MissingField("source_id".to_string()))?;

    let source_id: u64 = source_id_str.parse().map_err(|_: ParseIntError| {
        WebhookError::InvalidField {
            field: "source_id".to_string(),
            value: source_id_str.to_string(),
            reason: "must be a positive integer".to_string(),
        }
    })?;

    let retry_count: u32 = map
        .get("retry_count")
        .unwrap_or(&"0")  // optional field with default
        .parse()
        .map_err(|_: ParseIntError| WebhookError::InvalidField {
            field: "retry_count".to_string(),
            value: map.get("retry_count").unwrap_or(&"0").to_string(),
            reason: "must be a non-negative integer".to_string(),
        })?;

    Ok(WebhookPayload { event_type, source_id, retry_count })
}

// ---------------------------------------------------------------------------
// 3. A higher-level error that wraps WebhookError
// ---------------------------------------------------------------------------

/// Top-level errors for the webhook relay service.
#[derive(Debug)]
pub enum RelayError {
    /// Failed to parse or validate the incoming webhook.
    Webhook {
        endpoint: String,
        source: WebhookError,
    },
    /// Failed to forward the webhook to a downstream system.
    Delivery { target: String, reason: String },
    /// Rate limit exceeded — caller should retry after delay.
    RateLimited { retry_after_secs: u64 },
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelayError::Webhook { endpoint, source } => {
                write!(f, "webhook validation failed for {}: {}", endpoint, source)
            }
            RelayError::Delivery { target, reason } => {
                write!(f, "delivery to '{}' failed: {}", target, reason)
            }
            RelayError::RateLimited { retry_after_secs } => {
                write!(f, "rate limited — retry after {}s", retry_after_secs)
            }
        }
    }
}

impl std::error::Error for RelayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RelayError::Webhook { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Process a webhook at the relay level, wrapping validation errors with context.
pub fn relay_webhook(
    endpoint: &str,
    raw: &[(&str, &str)],
) -> Result<WebhookPayload, RelayError> {
    parse_webhook_payload(raw).map_err(|e| RelayError::Webhook {
        endpoint: endpoint.to_string(),
        source: e,
    })
}

// ---------------------------------------------------------------------------
// 4. Printing the error chain
// ---------------------------------------------------------------------------

fn print_error_chain(e: &dyn std::error::Error) {
    eprint!("error: {}", e);
    let mut source = e.source();
    while let Some(cause) = source {
        eprint!("\n  caused by: {}", cause);
        source = cause.source();
    }
    eprintln!();
}

// ---------------------------------------------------------------------------
// 5. What thiserror generates for you (commented out — requires thiserror crate)
//
// The code below is the thiserror equivalent of the manual implementations above.
// It is left as a reference so you can see what the derive macro produces.
// ---------------------------------------------------------------------------

/*
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("invalid JSON payload: {0}")]
    InvalidJson(String),

    #[error("required field '{0}' is missing from payload")]
    MissingField(String),

    #[error("invalid value '{value}' for field '{field}': {reason}")]
    InvalidField {
        field: String,
        value: String,
        reason: String,
    },

    #[error("webhook signature verification failed")]
    SignatureMismatch,

    #[error("I/O error reading webhook body: {0}")]
    Io(#[from] std::io::Error),   // #[from] generates From<io::Error> AND sets source()
}

// That's it. ~10 lines instead of ~50. Same behavior.
//
// Key differences from manual:
// - #[from] on a field: generates From impl + sets source()
// - #[source] on a field: sets source() only (no From)
// - Named fields in format strings: {field_name}
// - Positional: {0}
*/

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Valid payload ===");
    let valid = &[
        ("event_type", "payment.completed"),
        ("source_id", "12345"),
        ("retry_count", "2"),
    ];
    match parse_webhook_payload(valid) {
        Ok(p) => println!("{:?}", p),
        Err(e) => eprintln!("error: {}", e),
    }

    println!("\n=== Missing field ===");
    let missing_source = &[("event_type", "payment.completed")];
    match relay_webhook("/webhooks/payment", missing_source) {
        Ok(p) => println!("{:?}", p),
        Err(e) => {
            print_error_chain(&e);
        }
    }

    println!("\n=== Invalid field value ===");
    let bad_value = &[
        ("event_type", "payment.completed"),
        ("source_id", "not_a_number"),
    ];
    match relay_webhook("/webhooks/payment", bad_value) {
        Ok(p) => println!("{:?}", p),
        Err(e) => {
            println!("top-level: {}", e);
            if let RelayError::Webhook { source, .. } = &e {
                println!("inner:     {}", source);
            }
        }
    }

    println!("\n=== Display shows full context ===");
    let error = RelayError::Webhook {
        endpoint: "/webhooks/inventory".to_string(),
        source: WebhookError::InvalidField {
            field: "quantity".to_string(),
            value: "-1".to_string(),
            reason: "must be a non-negative integer".to_string(),
        },
    };
    print_error_chain(&error);
}
