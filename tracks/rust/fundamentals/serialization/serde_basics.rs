// serde_basics.rs — What serde does and how field attributes work
//
// This file uses manual JSON construction/parsing (no external crates)
// to demonstrate the same patterns that serde automates. After reading
// this, you'll understand what serde's derive macros generate for you.
//
// Run: rustc serde_basics.rs && ./serde_basics

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 1: What serde automates
//
// Without serde, serializing a struct requires this kind of boilerplate.
// Every field, every type, every nesting level — by hand.
// ─────────────────────────────────────────────────────────────────────────────

struct ServiceConfig {
    host: String,
    port: u16,
    timeout_ms: u64,
    description: Option<String>,
    retry_count: u32,
}

impl ServiceConfig {
    // Manual "to_json" — what serde_json::to_string generates automatically
    fn to_json(&self) -> String {
        let desc_field = match &self.description {
            // serde(skip_serializing_if = "Option::is_none") — omit null fields
            None => String::new(),
            Some(d) => format!(r#","description":"{}""#, escape_json(d)),
        };
        format!(
            r#"{{"host":"{host}","port":{port},"timeout_ms":{timeout_ms}{desc_field},"retry_count":{retry}}}"#,
            host = escape_json(&self.host),
            port = self.port,
            timeout_ms = self.timeout_ms,
            desc_field = desc_field,
            retry = self.retry_count,
        )
    }

    // Manual "from_json" — what serde's Deserialize generates automatically
    fn from_json(json: &str) -> Result<ServiceConfig, String> {
        // In reality, serde generates a Visitor that traverses the format's
        // token stream. We're doing a simplified version here.
        let host = extract_string_field(json, "host")
            .ok_or("missing field: host")?;
        let port = extract_number_field(json, "port")
            .ok_or("missing field: port")? as u16;
        let timeout_ms = extract_number_field(json, "timeout_ms")
            .ok_or("missing field: timeout_ms")?;
        let description = extract_string_field(json, "description");
        let retry_count = extract_number_field(json, "retry_count")
            .unwrap_or(0) as u32; // serde(default) — use 0 if absent

        Ok(ServiceConfig { host, port, timeout_ms, description, retry_count })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 2: Field renaming and camelCase APIs
//
// APIs often use camelCase externally. In Go you'd write:
//   `json:"userId"` or `json:"createdAt"`
// In serde:
//   #[serde(rename = "userId")] or #[serde(rename_all = "camelCase")]
// ─────────────────────────────────────────────────────────────────────────────

struct ApiUser {
    // In real serde: #[serde(rename_all = "camelCase")] on the struct
    user_id: u64,       // → "userId" in JSON
    display_name: String, // → "displayName" in JSON
    created_at: u64,    // → "createdAt" in JSON
}

impl ApiUser {
    // Manual camelCase rename — serde rename_all does this automatically
    fn to_json_camel_case(&self) -> String {
        format!(
            r#"{{"userId":{user_id},"displayName":"{display_name}","createdAt":{created_at}}}"#,
            user_id = self.user_id,
            display_name = escape_json(&self.display_name),
            created_at = self.created_at,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 3: Flatten — inlining nested structs
//
// A struct with #[serde(flatten)] on a field inlines that field's keys
// directly into the parent object, rather than nesting.
// ─────────────────────────────────────────────────────────────────────────────

struct Timestamps {
    created_at: u64,
    updated_at: u64,
}

struct OrderRecord {
    order_id: String,
    amount_cents: u64,
    timestamps: Timestamps, // In serde: #[serde(flatten)]
}

impl OrderRecord {
    // Without flatten: {"order_id": "...", "timestamps": {"created_at": ...}}
    fn to_json_nested(&self) -> String {
        format!(
            r#"{{"order_id":"{id}","amount_cents":{amt},"timestamps":{{"created_at":{ca},"updated_at":{ua}}}}}"#,
            id = self.order_id,
            amt = self.amount_cents,
            ca = self.timestamps.created_at,
            ua = self.timestamps.updated_at,
        )
    }

    // With flatten: {"order_id": "...", "created_at": ..., "updated_at": ...}
    fn to_json_flattened(&self) -> String {
        format!(
            r#"{{"order_id":"{id}","amount_cents":{amt},"created_at":{ca},"updated_at":{ua}}}"#,
            id = self.order_id,
            amt = self.amount_cents,
            ca = self.timestamps.created_at,
            ua = self.timestamps.updated_at,
        )
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 4: Validation after deserialization
//
// serde handles structural correctness. Domain correctness is your job.
// ─────────────────────────────────────────────────────────────────────────────

struct CreateOrderRequest {
    item_id: String,
    quantity: u32,
    unit_price_cents: u64,
}

#[derive(Debug)]
enum ValidationError {
    InvalidField { field: String, reason: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidField { field, reason } => {
                write!(f, "{field}: {reason}")
            }
        }
    }
}

impl CreateOrderRequest {
    fn from_json(json: &str) -> Result<CreateOrderRequest, String> {
        // Step 1: structural parsing (what serde does)
        let item_id = extract_string_field(json, "item_id")
            .ok_or("missing field: item_id")?;
        let quantity = extract_number_field(json, "quantity")
            .ok_or("missing field: quantity")? as u32;
        let unit_price_cents = extract_number_field(json, "unit_price_cents")
            .ok_or("missing field: unit_price_cents")?;

        Ok(CreateOrderRequest { item_id, quantity, unit_price_cents })
    }

    // Step 2: domain validation (your responsibility, not serde's)
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.item_id.is_empty() {
            errors.push(ValidationError::InvalidField {
                field: String::from("item_id"),
                reason: String::from("cannot be empty"),
            });
        }
        if self.item_id.len() > 64 {
            errors.push(ValidationError::InvalidField {
                field: String::from("item_id"),
                reason: String::from("cannot exceed 64 characters"),
            });
        }
        if self.quantity == 0 {
            errors.push(ValidationError::InvalidField {
                field: String::from("quantity"),
                reason: String::from("must be at least 1"),
            });
        }
        if self.unit_price_cents == 0 {
            errors.push(ValidationError::InvalidField {
                field: String::from("unit_price_cents"),
                reason: String::from("must be positive"),
            });
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    // Two-step ingestion: parse then validate
    fn ingest(json: &str) -> Result<CreateOrderRequest, String> {
        let req = CreateOrderRequest::from_json(json)?;
        req.validate()
            .map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))?;
        Ok(req)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Minimal JSON helpers (what serde and serde_json replace entirely)
// ─────────────────────────────────────────────────────────────────────────────

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn extract_string_field(json: &str, field: &str) -> Option<String> {
    let key = format!(r#""{field}":""#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_number_field(json: &str, field: &str) -> Option<u64> {
    let key = format!(r#""{field}":"#);
    let start = json.find(&key)? + key.len();
    let rest = &json[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// ─────────────────────────────────────────────────────────────────────────────
// main — run all examples
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Section 1: What serde automates ===\n");

    let config = ServiceConfig {
        host: String::from("api.internal"),
        port: 8080,
        timeout_ms: 30_000,
        description: Some(String::from("Primary API endpoint")),
        retry_count: 3,
    };

    let json = config.to_json();
    println!("Serialized:\n  {json}\n");

    let parsed = ServiceConfig::from_json(&json).expect("failed to parse");
    println!("Deserialized:");
    println!("  host: {}", parsed.host);
    println!("  port: {}", parsed.port);
    println!("  timeout_ms: {}", parsed.timeout_ms);
    println!("  description: {:?}", parsed.description);
    println!("  retry_count: {}", parsed.retry_count);

    let no_desc = ServiceConfig {
        host: String::from("db.internal"),
        port: 5432,
        timeout_ms: 10_000,
        description: None,      // skip_serializing_if = "Option::is_none"
        retry_count: 0,
    };
    let json_no_desc = no_desc.to_json();
    // Note: "description" key is absent, not "description": null
    println!("\nNo description (key omitted, not null):\n  {json_no_desc}");

    println!("\n=== Section 2: Field renaming (camelCase API) ===\n");

    let user = ApiUser {
        user_id: 42,
        display_name: String::from("Alice Liddell"),
        created_at: 1708300000,
    };
    let camel_json = user.to_json_camel_case();
    println!("camelCase JSON:\n  {camel_json}");
    println!("  (serde: #[serde(rename_all = \"camelCase\")] on the struct)");

    println!("\n=== Section 3: Flatten ===\n");

    let order = OrderRecord {
        order_id: String::from("ord_001"),
        amount_cents: 4999,
        timestamps: Timestamps {
            created_at: 1708300000,
            updated_at: 1708300060,
        },
    };

    println!("Nested (no flatten):");
    println!("  {}", order.to_json_nested());
    println!("\nFlattened (#[serde(flatten)]):");
    println!("  {}", order.to_json_flattened());
    println!("  (timestamps fields appear at the top level, not nested)");

    println!("\n=== Section 4: Validation after deserialization ===\n");

    let valid_json = r#"{"item_id":"sku_abc","quantity":2,"unit_price_cents":999}"#;
    match CreateOrderRequest::ingest(valid_json) {
        Ok(req) => println!("Valid order: {} x {} @ {} cents",
            req.quantity, req.item_id, req.unit_price_cents),
        Err(e) => println!("Error: {e}"),
    }

    // Structural error: missing field
    let bad_json = r#"{"item_id":"sku_abc"}"#;
    match CreateOrderRequest::ingest(bad_json) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Structural error caught: {e}"),
    }

    // Semantic error: quantity is 0 (structurally valid, semantically invalid)
    let invalid_json = r#"{"item_id":"sku_abc","quantity":0,"unit_price_cents":999}"#;
    match CreateOrderRequest::ingest(invalid_json) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Validation error caught: {e}"),
    }

    // Multiple semantic errors at once
    let multi_error_json = r#"{"item_id":"","quantity":0,"unit_price_cents":0}"#;
    match CreateOrderRequest::ingest(multi_error_json) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Multiple errors: {e}"),
    }

    println!("\n=== Key Takeaways ===\n");
    println!("1. serde automates the serialization boilerplate shown in Section 1.");
    println!("2. #[serde(rename_all)] handles camelCase/snake_case boundary automatically.");
    println!("3. #[serde(flatten)] inlines nested struct fields into the parent JSON.");
    println!("4. serde handles structure; YOU handle domain validation (Section 4).");
    println!("5. All of this is generated at compile time — zero runtime reflection.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_round_trip() {
        let config = ServiceConfig {
            host: String::from("test.internal"),
            port: 9090,
            timeout_ms: 5000,
            description: Some(String::from("test instance")),
            retry_count: 2,
        };
        let json = config.to_json();
        let parsed = ServiceConfig::from_json(&json).unwrap();
        assert_eq!(parsed.host, "test.internal");
        assert_eq!(parsed.port, 9090);
        assert_eq!(parsed.timeout_ms, 5000);
        assert_eq!(parsed.description.as_deref(), Some("test instance"));
        assert_eq!(parsed.retry_count, 2);
    }

    #[test]
    fn test_skip_none_description() {
        let config = ServiceConfig {
            host: String::from("h"),
            port: 80,
            timeout_ms: 1000,
            description: None,
            retry_count: 0,
        };
        let json = config.to_json();
        assert!(!json.contains("description"));
        assert!(!json.contains("null"));
    }

    #[test]
    fn test_camel_case_rename() {
        let user = ApiUser {
            user_id: 1,
            display_name: String::from("Bob"),
            created_at: 1000,
        };
        let json = user.to_json_camel_case();
        assert!(json.contains("\"userId\""));
        assert!(json.contains("\"displayName\""));
        assert!(json.contains("\"createdAt\""));
        assert!(!json.contains("user_id"));
    }

    #[test]
    fn test_flatten_no_nesting() {
        let order = OrderRecord {
            order_id: String::from("o1"),
            amount_cents: 100,
            timestamps: Timestamps { created_at: 1, updated_at: 2 },
        };
        let flat = order.to_json_flattened();
        // Fields appear at top level, not nested under "timestamps"
        assert!(!flat.contains("\"timestamps\""));
        assert!(flat.contains("\"created_at\""));
        assert!(flat.contains("\"updated_at\""));
    }

    #[test]
    fn test_validation_collects_all_errors() {
        let req = CreateOrderRequest {
            item_id: String::new(),
            quantity: 0,
            unit_price_cents: 0,
        };
        let errors = req.validate().unwrap_err();
        // Three separate errors: empty item_id, zero quantity, zero price
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_validation_passes_for_valid_request() {
        let req = CreateOrderRequest {
            item_id: String::from("sku-abc-123"),
            quantity: 5,
            unit_price_cents: 1999,
        };
        assert!(req.validate().is_ok());
    }
}
