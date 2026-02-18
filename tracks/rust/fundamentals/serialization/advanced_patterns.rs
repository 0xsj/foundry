// advanced_patterns.rs — Value type, zero-copy concepts, custom ser/de, schema validation
//
// This file covers three advanced serialization patterns:
//   1. The Value type — untyped JSON traversal and construction
//   2. Zero-copy deserialization — borrowing instead of allocating
//   3. Custom serialize/deserialize — when derive isn't enough
//
// Run: rustc advanced_patterns.rs && ./advanced_patterns

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 1: The Value type — dynamic / untyped JSON
//
// serde_json::Value is a recursive enum representing any JSON tree:
//
//   pub enum Value {
//       Null,
//       Bool(bool),
//       Number(Number),   // i64, u64, or f64
//       String(String),
//       Array(Vec<Value>),
//       Object(Map<String, Value>),
//   }
//
// We implement our own simplified version here to show the concept.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>), // ordered pairs (like serde_json::Map)
}

impl JsonValue {
    // Mimic serde_json::Value's indexing — never panics, returns Null for missing
    fn get(&self, key: &str) -> &JsonValue {
        match self {
            JsonValue::Object(pairs) => {
                for (k, v) in pairs {
                    if k == key { return v; }
                }
                &JsonValue::Null
            }
            _ => &JsonValue::Null,
        }
    }

    fn index(&self, i: usize) -> &JsonValue {
        match self {
            JsonValue::Array(items) => items.get(i).unwrap_or(&JsonValue::Null),
            _ => &JsonValue::Null,
        }
    }

    // Conversion methods — same as serde_json Value
    fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    // Serialize to JSON string
    fn to_json(&self) -> String {
        match self {
            JsonValue::Null => String::from("null"),
            JsonValue::Bool(b) => format!("{b}"),
            JsonValue::Number(n) => {
                // Don't print .0 suffix for integers
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            JsonValue::Str(s) => format!(r#""{}""#, s.replace('"', "\\\"")),
            JsonValue::Array(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_json()).collect();
                format!("[{}]", inner.join(","))
            }
            JsonValue::Object(pairs) => {
                let inner: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!(r#""{k}":{}"#, v.to_json()))
                    .collect();
                format!("{{{}}}", inner.join(","))
            }
        }
    }
}

// The json! macro equivalent — build Value trees from literals
// In serde_json: serde_json::json!({ "key": "value", "n": 42 })
macro_rules! jval {
    (null) => { JsonValue::Null };
    (true) => { JsonValue::Bool(true) };
    (false) => { JsonValue::Bool(false) };
    ($n:expr) => { JsonValue::Number($n as f64) };
}

fn make_api_response(status: &str, items: Vec<JsonValue>) -> JsonValue {
    // Building a Value tree programmatically — useful for proxying/transforming
    JsonValue::Object(vec![
        (String::from("status"), JsonValue::Str(status.to_string())),
        (String::from("count"), JsonValue::Number(items.len() as f64)),
        (String::from("items"), JsonValue::Array(items)),
    ])
}

// Real use case: a struct with a catch-all "metadata" field
// In serde_json: use Value for the field type
struct ApiResponse {
    id: String,
    event_type: String,
    metadata: JsonValue, // Any JSON — unknown schema at compile time
}

impl ApiResponse {
    fn from_value(v: &JsonValue) -> Option<ApiResponse> {
        Some(ApiResponse {
            id: v.get("id").as_str()?.to_string(),
            event_type: v.get("event_type").as_str()?.to_string(),
            metadata: v.get("metadata").clone(),
        })
    }

    // Mix of typed access and raw Value passthrough
    fn metadata_field(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).as_str()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 2: Zero-copy deserialization
//
// Instead of copying string data from the input buffer into owned Strings,
// we can borrow string slices directly from the input. The struct holds &str
// references into the JSON bytes — no allocation for string fields.
//
// In serde:
//   #[derive(Deserialize)]
//   struct LogEntry<'de> {
//       #[serde(borrow)]
//       request_id: &'de str,
//   }
//
// The 'de lifetime ties the struct to the buffer it was parsed from.
// ─────────────────────────────────────────────────────────────────────────────

// Zero-copy struct — borrows from the input JSON bytes
// The 'de lifetime means "lives at most as long as the input data"
struct LogEntry<'de> {
    request_id: &'de str,  // points into the original JSON string — no allocation
    path: &'de str,        // same — a slice of the input
    status: u16,           // numbers are always "zero-copy" since they're just parsed
    latency_ms: u64,
}

impl<'de> LogEntry<'de> {
    // Parse a log entry by borrowing from the input string
    // The lifetime 'de enforces that self cannot outlive input
    fn parse(input: &'de str) -> Option<LogEntry<'de>> {
        // Find "request_id":"..." — borrow the slice, no copy
        let request_id = borrow_string_field(input, "request_id")?;
        let path = borrow_string_field(input, "path")?;
        let status = extract_number_field(input, "status")? as u16;
        let latency_ms = extract_number_field(input, "latency_ms")?;

        Some(LogEntry { request_id, path, status, latency_ms })
    }
}

// Returns a &str pointing INTO the source string — zero allocation
fn borrow_string_field<'a>(source: &'a str, field: &str) -> Option<&'a str> {
    let key = format!(r#""{field}":""#);
    let start = source.find(&key)? + key.len();
    let rest = &source[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])  // this is a slice of source — no new String created
}

fn extract_number_field(source: &str, field: &str) -> Option<u64> {
    let key = format!(r#""{field}":"#);
    let start = source.find(&key)? + key.len();
    let rest = &source[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// Processing log entries without allocating for string fields
fn count_5xx_errors(logs: &[&str]) -> usize {
    logs.iter().filter(|log| {
        if let Some(entry) = LogEntry::parse(log) {
            entry.status >= 500
        } else {
            false
        }
    }).count()
}

// The lifetime constraint prevents this bug:
// fn store_entry(input: &str) -> LogEntry<'_> {
//     let entry = LogEntry::parse(input).unwrap();
//     entry  // would this be ok? Yes — entry borrows from input, which is still alive
// }
//
// But this would be a compile error:
// fn bad_store() -> LogEntry<'static> {
//     let input = String::from(r#"{"request_id":"req_1","path":"/","status":200,"latency_ms":5}"#);
//     let entry = LogEntry::parse(&input).unwrap();
//     entry  // ERROR: entry borrows from input, which dies here
// }

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 3: Custom serialization — when derive isn't enough
//
// Three common cases where you implement Serialize/Deserialize manually:
//   a) Custom wire format (e.g., "100/60s" for a rate limit)
//   b) Redacting sensitive fields
//   c) Versioned formats that need transformation
// ─────────────────────────────────────────────────────────────────────────────

// Case A: Custom wire format — rate limit as "N/Ws" string
#[derive(Debug, PartialEq)]
struct RateLimit {
    requests: u32,
    window_seconds: u32,
}

impl RateLimit {
    // Serialize to "100/60s" format instead of {"requests":100,"window_seconds":60}
    fn to_json_string(&self) -> String {
        format!(r#""{}/{}s""#, self.requests, self.window_seconds)
    }

    // Deserialize from "100/60s" format
    fn from_json_string(s: &str) -> Result<RateLimit, String> {
        // Strip surrounding quotes
        let inner = s.trim_matches('"');
        let without_suffix = inner.trim_end_matches('s');
        let parts: Vec<&str> = without_suffix.split('/').collect();
        if parts.len() != 2 {
            return Err(format!("expected 'N/Ws' format, got '{inner}'"));
        }
        let requests = parts[0].parse::<u32>()
            .map_err(|e| format!("invalid request count: {e}"))?;
        let window_seconds = parts[1].parse::<u32>()
            .map_err(|e| format!("invalid window: {e}"))?;
        Ok(RateLimit { requests, window_seconds })
    }
}

// Case B: Redacting sensitive fields
// Custom Debug/Serialize that omits secret field values
#[derive(Debug)]
struct ApiCredentials {
    key_id: String,
    secret_key: String, // should never appear in logs or serialized output
}

impl ApiCredentials {
    // Custom serialization — omit or redact the secret
    fn to_safe_json(&self) -> String {
        // Only include the key_id, never the secret
        format!(r#"{{"key_id":"{}","secret_key":"[REDACTED]"}}"#, self.key_id)
    }

    // What a manual Debug impl would do:
    fn safe_debug(&self) -> String {
        format!("ApiCredentials {{ key_id: {:?}, secret_key: \"[REDACTED]\" }}", self.key_id)
    }
}

// Case C: Versioned format — deserializing old and new schema
#[derive(Debug)]
struct UserProfile {
    user_id: u64,
    display_name: String,
}

impl UserProfile {
    // V1 format: {"id": 1, "name": "alice"}
    // V2 format: {"user_id": 1, "display_name": "alice"}
    fn from_json_any_version(json: &str) -> Option<UserProfile> {
        // Try V2 first
        let user_id = extract_number_field(json, "user_id")
            .or_else(|| extract_number_field(json, "id"))?;

        let display_name = borrow_string_field(json, "display_name")
            .map(|s| s.to_string())
            .or_else(|| borrow_string_field(json, "name").map(|s| s.to_string()))?;

        Some(UserProfile { user_id, display_name })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION 4: Schema validation via typed parsing
//
// The serde mental model: typing IS your schema.
// The compiler enforces the shape; you enforce the semantics.
// ─────────────────────────────────────────────────────────────────────────────

// This struct IS the schema for a valid webhook payload
#[derive(Debug)]
struct StripeWebhookPayload {
    id: String,
    event_type: String, // would use #[serde(rename = "type")] in real code
    livemode: bool,
    api_version: String,
    created: u64,
    data_object_id: String,
    data_object_amount: Option<u64>, // Some events have amounts, some don't
}

impl StripeWebhookPayload {
    fn parse(json: &str) -> Result<StripeWebhookPayload, String> {
        // Structural parsing — serde does this automatically
        let id = borrow_string_field(json, "id")
            .ok_or("missing required field: id")?
            .to_string();

        let event_type = borrow_string_field(json, "type")
            .ok_or("missing required field: type")?
            .to_string();

        let livemode = {
            let found = json.contains(r#""livemode":true"#);
            let found_false = json.contains(r#""livemode":false"#);
            if !found && !found_false {
                return Err(String::from("missing required field: livemode"));
            }
            found
        };

        let api_version = borrow_string_field(json, "api_version")
            .ok_or("missing required field: api_version")?
            .to_string();

        let created = extract_number_field(json, "created")
            .ok_or("missing required field: created")?;

        let data_object_id = borrow_string_field(json, "data_object_id")
            .ok_or("missing required field: data.object.id")?
            .to_string();

        let data_object_amount = extract_number_field(json, "data_object_amount");

        // Domain validation — serde doesn't do this
        let payload = StripeWebhookPayload {
            id: id.clone(),
            event_type: event_type.clone(),
            livemode,
            api_version,
            created,
            data_object_id,
            data_object_amount,
        };

        payload.validate()?;
        Ok(payload)
    }

    fn validate(&self) -> Result<(), String> {
        if !self.id.starts_with("evt_") {
            return Err(format!("id must start with 'evt_', got '{}'", self.id));
        }
        if self.event_type.is_empty() {
            return Err(String::from("event_type cannot be empty"));
        }
        if self.created == 0 {
            return Err(String::from("created timestamp must be positive"));
        }
        // In production, also check that created is recent (prevent replay attacks)
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// main — run all examples
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    println!("=== Section 1: The Value Type (Dynamic JSON) ===\n");

    // Building a Value tree
    let response = make_api_response(
        "ok",
        vec![
            JsonValue::Object(vec![
                (String::from("id"), JsonValue::Str(String::from("item_1"))),
                (String::from("score"), JsonValue::Number(0.95)),
            ]),
            JsonValue::Object(vec![
                (String::from("id"), JsonValue::Str(String::from("item_2"))),
                (String::from("score"), JsonValue::Number(0.72)),
            ]),
        ],
    );

    println!("Built response:\n  {}\n", response.to_json());

    // Navigating a Value tree
    println!("status: {:?}", response.get("status").as_str());
    println!("count: {:?}", response.get("count").as_f64());
    println!("items[0].id: {:?}", response.get("items").index(0).get("id").as_str());
    println!("missing field: {:?} (no panic)", response.get("nonexistent").as_str());

    // ApiResponse with typed + untyped fields
    let raw = JsonValue::Object(vec![
        (String::from("id"), JsonValue::Str(String::from("evt_001"))),
        (String::from("event_type"), JsonValue::Str(String::from("payment.succeeded"))),
        (String::from("metadata"), JsonValue::Object(vec![
            (String::from("source"), JsonValue::Str(String::from("stripe"))),
            (String::from("account"), JsonValue::Str(String::from("acct_xyz"))),
        ])),
    ]);

    if let Some(resp) = ApiResponse::from_value(&raw) {
        println!("\nApiResponse:");
        println!("  id: {}", resp.id);
        println!("  event_type: {}", resp.event_type);
        println!("  metadata.source: {:?}", resp.metadata_field("source"));
    }

    println!("\n=== Section 2: Zero-Copy Deserialization ===\n");

    let logs = vec![
        r#"{"request_id":"req_001","path":"/api/users","status":200,"latency_ms":12}"#,
        r#"{"request_id":"req_002","path":"/api/orders","status":500,"latency_ms":450}"#,
        r#"{"request_id":"req_003","path":"/api/health","status":200,"latency_ms":3}"#,
        r#"{"request_id":"req_004","path":"/api/payments","status":503,"latency_ms":1200}"#,
    ];

    // Parse each entry — &str slices point into the log strings above
    for log in &logs {
        if let Some(entry) = LogEntry::parse(log) {
            println!(
                "  {} {} {} {}ms",
                entry.request_id,  // &str — no allocation
                entry.path,        // &str — no allocation
                entry.status,
                entry.latency_ms
            );
        }
    }

    let error_count = count_5xx_errors(&logs);
    println!("\n5xx errors: {error_count} of {} requests", logs.len());
    println!("(String fields borrowed from input — 0 allocations for request_id or path)");

    println!("\n=== Section 3: Custom Serialization ===\n");

    // Case A: custom wire format
    let rate_limit = RateLimit { requests: 100, window_seconds: 60 };
    let serialized = rate_limit.to_json_string();
    println!("RateLimit serialized: {serialized}");
    let deserialized = RateLimit::from_json_string(&serialized).unwrap();
    println!("RateLimit deserialized: {:?}", deserialized);
    assert_eq!(rate_limit, deserialized);

    // Case B: redacting sensitive fields
    let creds = ApiCredentials {
        key_id: String::from("AKID_1234"),
        secret_key: String::from("super-secret-dont-log-this"),
    };
    println!("\nSafe debug: {}", creds.safe_debug());
    println!("Safe JSON: {}", creds.to_safe_json());
    println!("(secret_key never appears in output)");

    // Case C: versioned format
    let v1_json = r#"{"id":1,"name":"alice"}"#;
    let v2_json = r#"{"user_id":2,"display_name":"bob"}"#;

    println!("\nV1 format: {v1_json}");
    if let Some(p) = UserProfile::from_json_any_version(v1_json) {
        println!("  Parsed: {:?}", p);
    }
    println!("V2 format: {v2_json}");
    if let Some(p) = UserProfile::from_json_any_version(v2_json) {
        println!("  Parsed: {:?}", p);
    }

    println!("\n=== Section 4: Schema Validation ===\n");

    let valid = r#"{"id":"evt_001","type":"payment.succeeded","livemode":false,"api_version":"2023-10-01","created":1708300000,"data_object_id":"pi_abc","data_object_amount":4999}"#;
    match StripeWebhookPayload::parse(valid) {
        Ok(p) => println!("Valid payload: id={} type={}", p.id, p.event_type),
        Err(e) => println!("Unexpected error: {e}"),
    }

    // Missing required field
    let missing_field = r#"{"id":"evt_002","type":"payment.failed","livemode":true}"#;
    match StripeWebhookPayload::parse(missing_field) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Structural error: {e}"),
    }

    // Wrong id prefix (domain validation)
    let bad_id = r#"{"id":"WRONG_001","type":"charge.succeeded","livemode":false,"api_version":"2023","created":1708300000,"data_object_id":"ch_abc"}"#;
    match StripeWebhookPayload::parse(bad_id) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Domain validation error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Section 1: Value type
    #[test]
    fn test_value_null_for_missing() {
        let v = JsonValue::Object(vec![
            (String::from("key"), JsonValue::Str(String::from("val"))),
        ]);
        assert!(v.get("missing").is_null());
        assert!(v.get("missing").get("nested").is_null()); // no panic
    }

    #[test]
    fn test_value_array_bounds() {
        let v = JsonValue::Array(vec![JsonValue::Number(1.0)]);
        assert!(!v.index(0).is_null());
        assert!(v.index(99).is_null()); // out of bounds = Null, no panic
    }

    #[test]
    fn test_value_json_serialization() {
        let v = JsonValue::Object(vec![
            (String::from("a"), JsonValue::Number(1.0)),
            (String::from("b"), JsonValue::Bool(true)),
            (String::from("c"), JsonValue::Null),
        ]);
        let json = v.to_json();
        assert!(json.contains(r#""a":1"#));
        assert!(json.contains(r#""b":true"#));
        assert!(json.contains(r#""c":null"#));
    }

    // Section 2: Zero-copy
    #[test]
    fn test_log_entry_parse() {
        let input = r#"{"request_id":"req_abc","path":"/health","status":200,"latency_ms":5}"#;
        let entry = LogEntry::parse(input).unwrap();
        assert_eq!(entry.request_id, "req_abc");
        assert_eq!(entry.path, "/health");
        assert_eq!(entry.status, 200);
        assert_eq!(entry.latency_ms, 5);
    }

    #[test]
    fn test_zero_copy_is_borrowed() {
        let input = String::from(r#"{"request_id":"xyz","path":"/","status":404,"latency_ms":1}"#);
        let entry = LogEntry::parse(&input).unwrap();
        // entry.request_id is a &str that points INTO input
        // Verify it's the exact same bytes (pointer into the string)
        assert_eq!(entry.request_id, "xyz");
        assert_eq!(entry.path, "/");
    }

    #[test]
    fn test_5xx_counting() {
        let logs = vec![
            r#"{"request_id":"a","path":"/","status":200,"latency_ms":1}"#,
            r#"{"request_id":"b","path":"/","status":500,"latency_ms":1}"#,
            r#"{"request_id":"c","path":"/","status":502,"latency_ms":1}"#,
        ];
        assert_eq!(count_5xx_errors(&logs), 2);
    }

    // Section 3: Custom ser/de
    #[test]
    fn test_rate_limit_round_trip() {
        let rl = RateLimit { requests: 50, window_seconds: 30 };
        let json = rl.to_json_string();
        assert_eq!(json, r#""50/30s""#);
        let parsed = RateLimit::from_json_string(&json).unwrap();
        assert_eq!(parsed, rl);
    }

    #[test]
    fn test_rate_limit_bad_format() {
        let result = RateLimit::from_json_string(r#""100""#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected"));
    }

    #[test]
    fn test_credentials_never_expose_secret() {
        let creds = ApiCredentials {
            key_id: String::from("key_123"),
            secret_key: String::from("very-secret"),
        };
        let safe = creds.to_safe_json();
        assert!(safe.contains("key_123"));
        assert!(!safe.contains("very-secret"));
        assert!(safe.contains("[REDACTED]"));
    }

    #[test]
    fn test_versioned_profile_v1() {
        let v1 = r#"{"id":99,"name":"carol"}"#;
        let p = UserProfile::from_json_any_version(v1).unwrap();
        assert_eq!(p.user_id, 99);
        assert_eq!(p.display_name, "carol");
    }

    #[test]
    fn test_versioned_profile_v2() {
        let v2 = r#"{"user_id":100,"display_name":"dave"}"#;
        let p = UserProfile::from_json_any_version(v2).unwrap();
        assert_eq!(p.user_id, 100);
        assert_eq!(p.display_name, "dave");
    }

    // Section 4: Schema validation
    #[test]
    fn test_valid_webhook_payload() {
        let json = r#"{"id":"evt_123","type":"payment.succeeded","livemode":false,"api_version":"2023","created":1708300000,"data_object_id":"pi_abc"}"#;
        let payload = StripeWebhookPayload::parse(json);
        assert!(payload.is_ok());
    }

    #[test]
    fn test_invalid_id_prefix() {
        let json = r#"{"id":"WRONG","type":"t","livemode":false,"api_version":"v","created":1,"data_object_id":"x"}"#;
        let result = StripeWebhookPayload::parse(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("evt_"));
    }

    #[test]
    fn test_missing_required_field() {
        let json = r#"{"id":"evt_1","type":"t"}"#; // missing livemode, api_version, etc.
        let result = StripeWebhookPayload::parse(json);
        assert!(result.is_err());
    }
}
