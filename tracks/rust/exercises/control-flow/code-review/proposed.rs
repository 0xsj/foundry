// Webhook Validator — Proposed Code for Review
//
// Validates incoming webhook events before they are dispatched to handlers.

// ---------- Types ----------

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    UserCreated,
    UserDeleted,
    OrderPlaced,
    OrderCancelled,
    PaymentReceived,
    PaymentFailed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub event_type: EventType,
    pub source: String,
    pub payload_size: usize,
    pub version: u8,
    pub signature: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    MissingSignature,
    InvalidVersion(u8),
    PayloadTooLarge,
    UnknownEventType,
    UnsupportedSource(String),
}

// ---------- Validation functions ----------

/// Returns the human-readable name for an event type.
/// Issue: should use match, not if/else chain.
pub fn event_type_name(event_type: &EventType) -> &'static str {
    if *event_type == EventType::UserCreated {
        "user.created"
    } else if *event_type == EventType::UserDeleted {
        "user.deleted"
    } else if *event_type == EventType::OrderPlaced {
        "order.placed"
    } else if *event_type == EventType::OrderCancelled {
        "order.cancelled"
    } else if *event_type == EventType::PaymentReceived {
        "payment.received"
    } else if *event_type == EventType::PaymentFailed {
        "payment.failed"
    } else {
        "unknown"
    }
    // Issue: EventType::Unknown is handled by the else, but if a new variant
    // is added, this function silently falls through to "unknown" with no
    // compile-time warning. A match arm would catch missing variants.
}

/// Validates the webhook event version.
/// Issue: redundant match arms that could be consolidated.
pub fn validate_version(version: u8) -> Result<(), ValidationError> {
    match version {
        1 => Ok(()),
        2 => Ok(()),
        3 => Ok(()),
        // Issue: versions 1, 2, 3 all return Ok(()) — should be 1 | 2 | 3 => Ok(())
        _ => Err(ValidationError::InvalidVersion(version)),
    }
}

/// Determines whether an event requires a signature check.
/// Issue: match used where if let would be cleaner for a single-variant check.
pub fn requires_signature(event_type: &EventType) -> bool {
    // Payment events always require a signature; others are optional.
    let result = match event_type {
        EventType::PaymentReceived => true,
        EventType::PaymentFailed => true,
        _ => false,
    };
    result
    // Issue: the intermediate `result` binding is unnecessary — the match
    // expression can be returned directly. Also, the two `true` arms could
    // be consolidated with an or-pattern.
}

/// Validates the source of the webhook.
/// Issue: missing a case, and the guard logic is inverted.
pub fn validate_source(source: &str) -> Result<(), ValidationError> {
    match source {
        "stripe" => Ok(()),
        "paypal" => Ok(()),
        "github" => Ok(()),
        // Issue: "shopify" is a valid source used in tests but not listed here.
        // This function is non-exhaustive in a different way from enums —
        // string matching has no compile-time exhaustiveness check.
        s if s.len() == 0 => {
            // Issue: the guard condition is overcomplicated.
            // `s.is_empty()` is cleaner than `s.len() == 0`.
            // But more importantly: this arm is AFTER the literal arms,
            // so it only catches empty strings not matched above.
            // It should be the FIRST arm to catch empty input before
            // trying to match it as a valid source.
            Err(ValidationError::UnsupportedSource(s.to_string()))
        }
        s => Err(ValidationError::UnsupportedSource(s.to_string())),
    }
}

/// Validates a complete webhook event.
/// Issue: nested if/else instead of early returns, and missing guard use.
pub fn validate_event(event: &WebhookEvent) -> Result<(), ValidationError> {
    // Issue: the author checks requires_signature and then checks signature
    // with a nested if inside an if. This should use an if let or a guard.
    if requires_signature(&event.event_type) {
        if event.signature.is_none() {
            return Err(ValidationError::MissingSignature);
        }
    }

    // Issue: validate_version is called but the result is stored in a
    // variable and then matched. The ? operator would be cleaner.
    let version_result = validate_version(event.version);
    match version_result {
        Ok(_) => {},
        Err(e) => return Err(e),
    }

    // Issue: magic number (10 * 1024 * 1024) with no named constant.
    if event.payload_size > 10 * 1024 * 1024 {
        return Err(ValidationError::PayloadTooLarge);
    }

    let source_result = validate_source(&event.source);
    match source_result {
        Ok(_) => {},
        Err(e) => return Err(e),
    }

    if event.event_type == EventType::Unknown {
        return Err(ValidationError::UnknownEventType);
    }

    Ok(())
}

/// Returns a severity label for a validation error.
/// Issue: loop { break } anti-pattern when an expression suffices.
pub fn error_severity(err: &ValidationError) -> &'static str {
    // Issue: this is written as a loop with break, but it's just an
    // expression that should return directly.
    let severity = loop {
        let s = match err {
            ValidationError::MissingSignature => "critical",
            ValidationError::InvalidVersion(_) => "high",
            ValidationError::PayloadTooLarge => "high",
            ValidationError::UnknownEventType => "medium",
            ValidationError::UnsupportedSource(_) => "low",
        };
        break s;
    };
    severity
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_name() {
        assert_eq!(event_type_name(&EventType::UserCreated), "user.created");
        assert_eq!(event_type_name(&EventType::OrderCancelled), "order.cancelled");
        assert_eq!(event_type_name(&EventType::Unknown), "unknown");
    }

    #[test]
    fn test_validate_version_valid() {
        assert_eq!(validate_version(1), Ok(()));
        assert_eq!(validate_version(2), Ok(()));
        assert_eq!(validate_version(3), Ok(()));
    }

    #[test]
    fn test_validate_version_invalid() {
        assert_eq!(validate_version(4), Err(ValidationError::InvalidVersion(4)));
        assert_eq!(validate_version(0), Err(ValidationError::InvalidVersion(0)));
    }

    #[test]
    fn test_requires_signature() {
        assert!(requires_signature(&EventType::PaymentReceived));
        assert!(requires_signature(&EventType::PaymentFailed));
        assert!(!requires_signature(&EventType::UserCreated));
        assert!(!requires_signature(&EventType::OrderPlaced));
    }

    #[test]
    fn test_validate_source_valid() {
        assert_eq!(validate_source("stripe"), Ok(()));
        assert_eq!(validate_source("github"), Ok(()));
        // This test will FAIL because "shopify" is missing from validate_source
        assert_eq!(validate_source("shopify"), Ok(()));
    }

    #[test]
    fn test_validate_source_invalid() {
        assert!(validate_source("unknown-provider").is_err());
        assert!(validate_source("").is_err());
    }

    #[test]
    fn test_validate_event_ok() {
        let event = WebhookEvent {
            event_type: EventType::UserCreated,
            source: "github".to_string(),
            payload_size: 1024,
            version: 2,
            signature: None,
        };
        assert_eq!(validate_event(&event), Ok(()));
    }

    #[test]
    fn test_validate_event_missing_signature() {
        let event = WebhookEvent {
            event_type: EventType::PaymentReceived,
            source: "stripe".to_string(),
            payload_size: 512,
            version: 1,
            signature: None,  // missing — required for payment events
        };
        assert_eq!(validate_event(&event), Err(ValidationError::MissingSignature));
    }

    #[test]
    fn test_validate_event_payload_too_large() {
        let event = WebhookEvent {
            event_type: EventType::OrderPlaced,
            source: "stripe".to_string(),
            payload_size: 11 * 1024 * 1024,
            version: 1,
            signature: None,
        };
        assert_eq!(validate_event(&event), Err(ValidationError::PayloadTooLarge));
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(error_severity(&ValidationError::MissingSignature), "critical");
        assert_eq!(error_severity(&ValidationError::PayloadTooLarge), "high");
        assert_eq!(error_severity(&ValidationError::UnsupportedSource("x".to_string())), "low");
    }
}

fn main() {
    let events = vec![
        WebhookEvent {
            event_type: EventType::PaymentReceived,
            source: "stripe".to_string(),
            payload_size: 512,
            version: 2,
            signature: Some("sha256=abc123".to_string()),
        },
        WebhookEvent {
            event_type: EventType::UserCreated,
            source: "github".to_string(),
            payload_size: 256,
            version: 1,
            signature: None,
        },
        WebhookEvent {
            event_type: EventType::OrderPlaced,
            source: "shopify".to_string(),  // will fail validation
            payload_size: 1024,
            version: 4,                     // invalid version
            signature: None,
        },
    ];

    for event in &events {
        let name = event_type_name(&event.event_type);
        match validate_event(event) {
            Ok(()) => println!("[OK]    {} from {}", name, event.source),
            Err(e) => println!("[ERROR] {} from {} — {:?} (severity: {})",
                               name, event.source, e, error_severity(&e)),
        }
    }
}
