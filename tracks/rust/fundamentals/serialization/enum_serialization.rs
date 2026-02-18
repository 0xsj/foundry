// enum_serialization.rs — The four enum tagging strategies and when to use each
//
// This file demonstrates all four serde enum serialization strategies using
// manual JSON construction to show exactly what each one produces on the wire.
// In real code, serde generates this from #[serde(tag = "...")] attributes.
//
// Run: rustc enum_serialization.rs && ./enum_serialization

// ─────────────────────────────────────────────────────────────────────────────
// The domain: a webhook dispatcher that handles events from multiple providers.
// Each strategy represents a different JSON shape for the same data.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum WebhookEvent {
    // Unit variant — no associated data
    Ping,
    // Tuple variant — positional data
    RawData(String),
    // Struct variant — named fields
    PaymentSucceeded { order_id: String, amount_cents: u64, currency: String },
    PaymentFailed { order_id: String, reason: String, retryable: bool },
    UserCreated { user_id: u64, email: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// STRATEGY 1: Externally Tagged (serde default)
//
// The variant name wraps the content as a JSON key.
// #[derive(Serialize, Deserialize)]
// enum WebhookEvent { ... }  ← no tag attribute = externally tagged
//
// Good for: internal Rust-to-Rust serialization, self-describing formats
// Bad for: REST APIs (capitalized variant names look wrong externally)
// ─────────────────────────────────────────────────────────────────────────────

fn to_external_json(event: &WebhookEvent) -> String {
    match event {
        // Unit: "Ping"
        WebhookEvent::Ping => String::from(r#""Ping""#),

        // Tuple: {"RawData": "...content..."}
        WebhookEvent::RawData(data) => {
            format!(r#"{{"RawData":"{}"}}"#, escape(data))
        }

        // Struct variant: {"PaymentSucceeded": {"order_id": "...", ...}}
        WebhookEvent::PaymentSucceeded { order_id, amount_cents, currency } => {
            format!(
                r#"{{"PaymentSucceeded":{{"order_id":"{order_id}","amount_cents":{amount_cents},"currency":"{currency}"}}}}"#
            )
        }

        WebhookEvent::PaymentFailed { order_id, reason, retryable } => {
            format!(
                r#"{{"PaymentFailed":{{"order_id":"{order_id}","reason":"{reason}","retryable":{retryable}}}}}"#
            )
        }

        WebhookEvent::UserCreated { user_id, email } => {
            format!(r#"{{"UserCreated":{{"user_id":{user_id},"email":"{email}"}}}}"#)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STRATEGY 2: Internally Tagged
//
// A "type" field lives inside the JSON object alongside the data.
// #[serde(tag = "type")]
// #[serde(rename_all = "snake_case")]
//
// {"type": "payment_succeeded", "order_id": "...", "amount_cents": 4999}
//
// Good for: REST APIs, Stripe/GitHub webhook format, any external-facing API
// Bad for: tuple variants (the content isn't a map — serde will error)
// Limitation: only works with struct variants and unit variants
// ─────────────────────────────────────────────────────────────────────────────

fn to_internal_json(event: &WebhookEvent) -> String {
    match event {
        // Unit variant: just the type tag, no other fields
        WebhookEvent::Ping => String::from(r#"{"type":"ping"}"#),

        // Tuple variant: NOT SUPPORTED by internally tagged in serde
        // You'd get a compile error: tuple variants require adjacent tagging.
        // Here we show what you'd do instead (wrap in a struct variant):
        WebhookEvent::RawData(data) => {
            format!(r#"{{"type":"raw_data","data":"{}"}}"#, escape(data))
        }

        // Struct variants: type field + all struct fields at the same level
        WebhookEvent::PaymentSucceeded { order_id, amount_cents, currency } => {
            format!(
                r#"{{"type":"payment_succeeded","order_id":"{order_id}","amount_cents":{amount_cents},"currency":"{currency}"}}"#
            )
        }

        WebhookEvent::PaymentFailed { order_id, reason, retryable } => {
            format!(
                r#"{{"type":"payment_failed","order_id":"{order_id}","reason":"{reason}","retryable":{retryable}}}"#
            )
        }

        WebhookEvent::UserCreated { user_id, email } => {
            format!(r#"{{"type":"user_created","user_id":{user_id},"email":"{email}"}}"#)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STRATEGY 3: Adjacently Tagged
//
// The tag and the content are separate top-level fields.
// #[serde(tag = "kind", content = "data")]
//
// {"kind": "payment_succeeded", "data": {"order_id": "...", ...}}
//
// Good for: consistent envelope format, works with all variant types
//           (tuple variants work, unlike internal tagging)
// Bad for: more verbose than internal tagging for struct variants
// ─────────────────────────────────────────────────────────────────────────────

fn to_adjacent_json(event: &WebhookEvent) -> String {
    match event {
        WebhookEvent::Ping => {
            String::from(r#"{"kind":"ping","data":null}"#)
        }

        // Tuple variant: data is the value directly (string, number, etc.)
        WebhookEvent::RawData(data) => {
            format!(r#"{{"kind":"raw_data","data":"{}"}}"#, escape(data))
        }

        // Struct variants: data is a nested JSON object
        WebhookEvent::PaymentSucceeded { order_id, amount_cents, currency } => {
            format!(
                r#"{{"kind":"payment_succeeded","data":{{"order_id":"{order_id}","amount_cents":{amount_cents},"currency":"{currency}"}}}}"#
            )
        }

        WebhookEvent::PaymentFailed { order_id, reason, retryable } => {
            format!(
                r#"{{"kind":"payment_failed","data":{{"order_id":"{order_id}","reason":"{reason}","retryable":{retryable}}}}}"#
            )
        }

        WebhookEvent::UserCreated { user_id, email } => {
            format!(r#"{{"kind":"user_created","data":{{"user_id":{user_id},"email":"{email}"}}}}"#)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STRATEGY 4: Untagged
//
// No discriminant — serde tries each variant in definition order.
// #[serde(untagged)]
//
// {"order_id": "...", "amount_cents": 4999, "currency": "USD"}
//
// Good for: API responses that return different types by shape alone
//           (e.g., a field that is either a string OR a number)
// Bad for: ambiguous shapes (PaymentSucceeded and PaymentFailed both have
//          order_id — which would serde pick?)
// ─────────────────────────────────────────────────────────────────────────────

// For untagged, different variants MUST have structurally distinct shapes.
// Our webhook events are ambiguous. Here's a better use case:
#[derive(Debug)]
enum ConfigValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

// With #[serde(untagged)], serde would try Bool first (since "true"/"false"),
// then Int (since integers parse as Int), then Float, then Str.
// In manual form, this is what the generated code does:
fn config_value_to_json(v: &ConfigValue) -> String {
    match v {
        ConfigValue::Bool(b) => format!("{b}"),
        ConfigValue::Int(i) => format!("{i}"),
        ConfigValue::Float(f) => format!("{f}"),
        ConfigValue::Str(s) => format!(r#""{}""#, escape(s)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Enum dispatch — showing how internally-tagged enums enable type-safe dispatch
// ─────────────────────────────────────────────────────────────────────────────

fn dispatch_event(event: &WebhookEvent) -> String {
    // Pattern matching on enums IS the switch statement — exhaustive.
    // If you add a new variant, every match arm fails to compile until handled.
    match event {
        WebhookEvent::Ping => String::from("pong"),
        WebhookEvent::RawData(data) => format!("received {} bytes", data.len()),
        WebhookEvent::PaymentSucceeded { amount_cents, currency, .. } => {
            format!("recorded ${:.2} {currency}", *amount_cents as f64 / 100.0)
        }
        WebhookEvent::PaymentFailed { retryable: true, .. } => {
            String::from("queued for retry")
        }
        WebhookEvent::PaymentFailed { reason, retryable: false, .. } => {
            format!("permanent failure: {reason}")
        }
        WebhookEvent::UserCreated { email, .. } => {
            format!("sending welcome email to {email}")
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The #[serde(other)] pattern — catching unknown future variants
//
// In real serde:
//   #[serde(tag = "type")]
//   enum WebhookEvent {
//       PaymentSucceeded { ... },
//       #[serde(other)]
//       Unknown,
//   }
//
// Parsing {"type": "subscription_cancelled", ...} would give WebhookEvent::Unknown
// instead of a deserialization error. Safe to add new events without breaking old code.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum KnownEvent {
    PaymentSucceeded { amount_cents: u64 },
    PaymentFailed { reason: String },
    Unknown, // #[serde(other)] — catches future event types
}

fn parse_event_type(json: &str) -> KnownEvent {
    // Simulate what serde does with internally-tagged + #[serde(other)]
    if let Some(event_type) = extract_field_value(json, "type") {
        match event_type.as_str() {
            "payment_succeeded" => {
                let amount = extract_u64_field(json, "amount_cents").unwrap_or(0);
                KnownEvent::PaymentSucceeded { amount_cents: amount }
            }
            "payment_failed" => {
                let reason = extract_field_value(json, "reason")
                    .unwrap_or_default();
                KnownEvent::PaymentFailed { reason }
            }
            _ => KnownEvent::Unknown, // #[serde(other)]
        }
    } else {
        KnownEvent::Unknown
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Minimal JSON helpers
// ─────────────────────────────────────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn extract_field_value(json: &str, field: &str) -> Option<String> {
    let key = format!(r#""{field}":""#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_u64_field(json: &str, field: &str) -> Option<u64> {
    let key = format!(r#""{field}":"#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// main — compare all four strategies side by side
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let events = vec![
        WebhookEvent::Ping,
        WebhookEvent::RawData(String::from("binary payload here")),
        WebhookEvent::PaymentSucceeded {
            order_id: String::from("ord_abc123"),
            amount_cents: 4999,
            currency: String::from("USD"),
        },
        WebhookEvent::PaymentFailed {
            order_id: String::from("ord_def456"),
            reason: String::from("insufficient funds"),
            retryable: false,
        },
        WebhookEvent::UserCreated {
            user_id: 42,
            email: String::from("alice@example.com"),
        },
    ];

    println!("=== Enum Serialization Strategies ===\n");
    println!("The same event, four different wire formats.\n");

    for event in &events {
        println!("Event: {event:?}");
        println!("  External tagged: {}", to_external_json(event));
        println!("  Internal tagged: {}", to_internal_json(event));
        println!("  Adjacent tagged: {}", to_adjacent_json(event));
        println!("  Dispatch result: {}", dispatch_event(event));
        println!();
    }

    println!("=== Untagged (for structurally distinct types) ===\n");

    let values = vec![
        ConfigValue::Bool(true),
        ConfigValue::Int(-42),
        ConfigValue::Float(3.14),
        ConfigValue::Str(String::from("hello")),
    ];

    for v in &values {
        println!("  {:?} → {}", v, config_value_to_json(v));
    }

    println!("\n=== #[serde(other)] — Unknown future variants ===\n");

    let future_event = r#"{"type":"subscription_cancelled","plan":"pro"}"#;
    let known_event = r#"{"type":"payment_succeeded","amount_cents":9999}"#;

    println!("Future event: {future_event}");
    println!("  Parsed as: {:?}", parse_event_type(future_event));
    println!("Known event: {known_event}");
    println!("  Parsed as: {:?}", parse_event_type(known_event));

    println!("\n=== Strategy Comparison ===\n");
    println!("Strategy          | Tuple variants | Flat object  | Common in");
    println!("──────────────────┼────────────────┼──────────────┼──────────────────");
    println!("External (default)| Yes            | No (wrapped) | Rust-internal, debug");
    println!("Internal tagged   | No             | Yes          | REST APIs, webhooks");
    println!("Adjacent tagged   | Yes            | No (data:{}) | Envelope protocols");
    println!("Untagged          | Yes            | Yes          | Polymorphic fields");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_ping() {
        let json = to_external_json(&WebhookEvent::Ping);
        assert_eq!(json, r#""Ping""#);
    }

    #[test]
    fn test_internal_payment_succeeded() {
        let event = WebhookEvent::PaymentSucceeded {
            order_id: String::from("ord_1"),
            amount_cents: 100,
            currency: String::from("USD"),
        };
        let json = to_internal_json(&event);
        // type field appears at top level, not nested
        assert!(json.contains(r#""type":"payment_succeeded""#));
        assert!(json.contains(r#""order_id":"ord_1""#));
        assert!(!json.contains(r#""payment_succeeded":{"#)); // not wrapped
    }

    #[test]
    fn test_adjacent_separates_tag_and_data() {
        let event = WebhookEvent::PaymentFailed {
            order_id: String::from("ord_2"),
            reason: String::from("declined"),
            retryable: true,
        };
        let json = to_adjacent_json(&event);
        assert!(json.contains(r#""kind":"payment_failed""#));
        assert!(json.contains(r#""data":{"#)); // data is nested under "data" key
    }

    #[test]
    fn test_dispatch_exhaustive() {
        // If all these match without a wildcard, the match is exhaustive
        let events = vec![
            WebhookEvent::Ping,
            WebhookEvent::RawData(String::from("x")),
            WebhookEvent::PaymentSucceeded {
                order_id: String::from("o"),
                amount_cents: 100,
                currency: String::from("USD"),
            },
            WebhookEvent::PaymentFailed {
                order_id: String::from("o"),
                reason: String::from("r"),
                retryable: true,
            },
            WebhookEvent::UserCreated {
                user_id: 1,
                email: String::from("x@y.com"),
            },
        ];
        for event in events {
            let result = dispatch_event(&event);
            assert!(!result.is_empty(), "dispatch returned empty for {event:?}");
        }
    }

    #[test]
    fn test_serde_other_catches_unknown() {
        let unknown = r#"{"type":"completely_new_event","some_field":"value"}"#;
        let parsed = parse_event_type(unknown);
        assert!(matches!(parsed, KnownEvent::Unknown));
    }

    #[test]
    fn test_serde_other_still_parses_known() {
        let known = r#"{"type":"payment_succeeded","amount_cents":5000}"#;
        let parsed = parse_event_type(known);
        assert!(matches!(parsed, KnownEvent::PaymentSucceeded { amount_cents: 5000 }));
    }
}
