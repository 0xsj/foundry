# Serialization — Rust

## What Is Serialization and Why Does Rust Make It Special?

Serialization is the process of converting an in-memory data structure into a format that can be stored or transmitted — JSON, TOML, binary, CSV, MessagePack — and deserialization is the reverse. Every backend service does this constantly: reading config files, decoding incoming HTTP bodies, encoding API responses.

In most languages this is tedious boilerplate. In Go you annotate struct fields with backtick tags. In TypeScript you reach for `JSON.parse` and hope the shape matches. In Python you use `json.dumps` and get back a `dict` you have to squint at.

Rust has **serde** — a serialization framework unlike anything in other languages. It's not a JSON library. It's a data model + type-driven machinery that any format can plug into. The same derive macro that serializes to JSON can serialize to TOML, YAML, MessagePack, BSON, and dozens of other formats with zero code changes. The format is a plugin; your types are the schema.

The deeper reason Rust's approach works is its trait system. `Serialize` and `Deserialize` are traits. Any type that implements them can be encoded/decoded by any format. The macro generates the implementation mechanically from your struct and enum definitions. No reflection, no runtime type inspection — pure compile-time code generation.

---

## The serde Mental Model

Before diving into syntax, understand the three layers:

```
Your types                    serde data model                  Format
─────────────────────────────────────────────────────────────────────
struct User {        →  Serialize trait  →  14 types:         →  serde_json
  name: String,                            map, seq, str,        serde_toml
  age: u32,                                u64, bool, ...        bincode
}                    ←  Deserialize trait ←                    ←  serde_yaml
```

**The serde data model** is a set of 14 abstract types (bool, i8-i128, u8-u128, f32/f64, char, string, bytes, none, some, unit, seq, map, struct, enum). Every Rust type maps to one or more of these. Every format knows how to encode/decode them.

This is why serde is language-level powerful: the coupling between your types and the wire format runs through this abstract layer, not through format-specific code in your struct definitions.

---

## The Derive Macros

The entry point to serde is two derive macros:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WebhookEvent {
    id: String,
    event_type: String,
    timestamp: u64,
    payload: EventPayload,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventPayload {
    source: String,
    data: String,
}
```

`#[derive(Serialize)]` generates an implementation that visits each field in order, telling the serializer "here's a struct named 'WebhookEvent' with field 'id' (a string), field 'event_type' (a string)..." and so on. The format (JSON, TOML, etc.) decides how to encode each piece.

`#[derive(Deserialize)]` generates the reverse: a visitor that expects a map-like structure, looks for each field name, and reconstructs the struct. It handles missing fields, unknown fields, and type mismatches.

To actually produce JSON, you use `serde_json`:

```rust
// serde_json::to_string — serialize to a JSON String
let event = WebhookEvent {
    id: String::from("evt_001"),
    event_type: String::from("payment.succeeded"),
    timestamp: 1708300000,
    payload: EventPayload {
        source: String::from("stripe"),
        data: String::from("{\"amount\": 4999}"),
    },
};

let json = serde_json::to_string(&event)?;
// {"id":"evt_001","event_type":"payment.succeeded","timestamp":1708300000,...}

let pretty = serde_json::to_string_pretty(&event)?;
// {
//   "id": "evt_001",
//   "event_type": "payment.succeeded",
//   ...
// }

// serde_json::from_str — deserialize from a &str
let decoded: WebhookEvent = serde_json::from_str(&json)?;
```

The `?` operator propagates `serde_json::Error`. Always handle deserialization errors — external JSON is never guaranteed to match your type.

Compare to Go:

```go
// Go: struct tags
type WebhookEvent struct {
    ID        string       `json:"id"`
    EventType string       `json:"event_type"`
    Timestamp int64        `json:"timestamp"`
    Payload   EventPayload `json:"payload"`
}

data, _ := json.Marshal(event)
var decoded WebhookEvent
json.Unmarshal(data, &decoded)  // error silently swallowed here
```

Go's `encoding/json` uses reflection at runtime. Serde generates the traversal code at compile time. The result: serde is typically 2-4x faster, and the mapping is verified at compile time rather than discovered at runtime.

### Your notes
<!-- -->


---

## Field Attributes

serde attributes on fields control how each field is serialized. They go between `#[serde(...)]` above the field.

### Renaming

JSON APIs use snake_case internally but camelCase externally, or have legacy names that don't match your clean Rust field names:

```rust
#[derive(Serialize, Deserialize)]
struct ApiUser {
    #[serde(rename = "userId")]
    user_id: u64,

    #[serde(rename = "displayName")]
    display_name: String,

    #[serde(rename = "createdAt")]
    created_at: u64,
}
// Serializes to: {"userId": 1, "displayName": "alice", "createdAt": 1708300000}
```

For the common case of converting all fields, `rename_all` is cleaner:

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiUser {
    user_id: u64,         // → "userId"
    display_name: String, // → "displayName"
    created_at: u64,      // → "createdAt"
}
```

`rename_all` options: `"camelCase"`, `"PascalCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`.

### Skip and Default

```rust
#[derive(Serialize, Deserialize)]
struct ServiceConfig {
    host: String,
    port: u16,

    // Omit this field from serialization entirely (e.g., internal state)
    #[serde(skip)]
    connection_count: u64,

    // Skip serializing if the value is None (cleaner JSON without null fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    // Use a default value if the field is missing during deserialization
    // Default::default() is called — for u32, that's 0
    #[serde(default)]
    retry_count: u32,

    // Use a specific function for the default
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 { 30_000 }
```

The `skip_serializing_if` pattern is important for optional fields. Without it, `description: None` would serialize as `"description": null`. With it, the key is absent entirely — exactly what most REST APIs expect.

### Flatten

`flatten` inlines a nested struct's fields into the parent. This is essential when an API returns a flat object but you want to model it with separate concerns:

```rust
#[derive(Serialize, Deserialize)]
struct Timestamps {
    created_at: u64,
    updated_at: u64,
}

#[derive(Serialize, Deserialize)]
struct Record {
    id: String,
    name: String,
    #[serde(flatten)]
    timestamps: Timestamps,
}

// JSON: {"id": "r1", "name": "foo", "created_at": 1708300000, "updated_at": 1708300001}
// Not: {"id": "r1", "name": "foo", "timestamps": {"created_at": ...}}
```

This keeps the wire format flat while letting you organize your Rust code around separate structs.

### The `with` Attribute

For total control over how a single field is serialized/deserialized without writing a full custom serializer:

```rust
mod timestamp_seconds {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(ts: &std::time::SystemTime, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let secs = ts.duration_since(std::time::UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?
            .as_secs();
        s.serialize_u64(secs)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<std::time::SystemTime, D::Error>
    where D: Deserializer<'de> {
        // covered in the advanced section
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
struct Event {
    id: String,
    #[serde(with = "timestamp_seconds")]
    created_at: std::time::SystemTime,
}
```

The `with` attribute points to a module that must have `serialize` and `deserialize` functions matching specific signatures. This is the right tool when you need a field-specific custom encoding without making the whole type custom.

### Your notes
<!-- -->


---

## Enum Serialization

This is where serde's design really shines. Enums in Rust have four serialization strategies, each corresponding to a different JSON pattern you'll see in real APIs.

### Externally Tagged (Default)

The default for Rust enums. The variant name is the key in a JSON object:

```rust
#[derive(Serialize, Deserialize)]
enum Notification {
    Email { to: String, subject: String },
    Push { device_token: String },
    Sms(String),
}

// Notification::Email { to: "a@b.com", subject: "hi" }
// → {"Email": {"to": "a@b.com", "subject": "hi"}}

// Notification::Sms("555-1234")
// → {"Sms": "555-1234"}
```

This is self-describing and round-trips perfectly, but the outer key is the Rust variant name (capitalized). Not typical for REST APIs.

### Internally Tagged

A `type` field (or whatever name you choose) lives inside the object alongside the data:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Notification {
    Email { to: String, subject: String },
    Push { device_token: String },
}

// → {"type": "Email", "to": "a@b.com", "subject": "hi"}
// → {"type": "Push", "device_token": "tok_abc"}
```

This is the most common pattern for REST API event payloads. The discriminant is a field called `"type"` right in the flat JSON. You'll see this in Stripe webhooks, GitHub events, etc.

Note: internally tagged doesn't work with tuple variants or newtype variants that contain non-map types.

### Adjacently Tagged

The tag and the data are separate fields in the same object:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
enum Notification {
    Email { to: String, subject: String },
    Push { device_token: String },
    Sms(String),   // tuple variants work here, unlike internal tagging
}

// → {"kind": "Email", "data": {"to": "a@b.com", "subject": "hi"}}
// → {"kind": "Sms", "data": "555-1234"}
```

Works with all variant types. Used when you want a consistent envelope structure where data is always nested under a `"data"` key.

### Untagged

No discriminant at all. serde tries each variant in order and picks the first one that parses successfully:

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrInt {
    Str(String),
    Int(i64),
}

// "hello" → Str("hello")
// 42      → Int(42)
```

Useful for APIs that return different types depending on context. Fragile for complex types because the "try each variant" approach can be slow and error-prone — if two variants have similar shapes, the wrong one might match. Use untagged only for structurally distinct types.

### Renaming Enum Variants

Just like `rename_all` on structs, you can rename variants:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WebhookEvent {
    PaymentSucceeded { amount: u64 },    // → "payment_succeeded"
    PaymentFailed { reason: String },    // → "payment_failed"
    SubscriptionCreated { plan: String }, // → "subscription_created"
}

// → {"type": "payment_succeeded", "amount": 4999}
```

This is the combination you'll use for real webhook handlers: internally tagged with snake_case variant names.

### Your notes
<!-- -->


---

## serde_json::Value — Dynamic JSON

Sometimes you don't know the shape of the JSON at compile time. Maybe you're proxying requests, logging raw payloads, or handling a field that can be "anything." serde_json's `Value` type is an untyped JSON tree:

```rust
use serde_json::Value;

// Parse any JSON into a Value
let raw = r#"{"user": {"name": "alice", "age": 30}, "tags": ["admin", "beta"]}"#;
let v: Value = serde_json::from_str(raw)?;

// Navigate with indexing — returns Value::Null if path doesn't exist
let name = &v["user"]["name"];  // Value::String("alice")
let tag0 = &v["tags"][0];       // Value::String("admin")
let missing = &v["user"]["phone"]; // Value::Null (no panic)

// Check type
if let Value::String(s) = &v["user"]["name"] {
    println!("name: {s}");
}

// Extract with .as_str(), .as_u64(), etc. — returns Option
let age = v["user"]["age"].as_u64();  // Some(30)
let bad = v["user"]["missing"].as_str();  // None

// Build a Value programmatically
let response = serde_json::json!({
    "status": "ok",
    "count": 42,
    "items": ["a", "b", "c"],
    "nested": {
        "flag": true
    }
});

// Mix typed and untyped: a struct with a Value field
#[derive(Deserialize)]
struct ApiResponse {
    status: String,
    data: Value,  // the data payload has an unknown schema
}
```

The `json!` macro is a shorthand for constructing `Value` trees. It accepts JSON syntax directly.

**When to use Value:** Proxying or logging raw JSON, testing/debugging, fields with truly dynamic schemas (like Stripe's `metadata` map or webhook `data.object`). Prefer typed structs when you know the shape — the compiler will catch API changes.

### Your notes
<!-- -->


---

## Error Handling in Deserialization

Deserialization can fail. The external world does not conform to your type definitions. Handle it.

```rust
let result: Result<WebhookEvent, serde_json::Error> = serde_json::from_str(raw_json);

match result {
    Ok(event) => handle_event(event),
    Err(e) => {
        // serde_json::Error has rich context:
        eprintln!("parse error at line {}, col {}: {}", e.line(), e.column(), e);
        // Return a 400 Bad Request, log the error, etc.
    }
}
```

`serde_json::Error` tells you exactly where in the JSON the error occurred. Use this in API handlers to return descriptive 400 responses.

**The `?` pattern in a function:**

```rust
fn parse_webhook(body: &str) -> Result<WebhookEvent, serde_json::Error> {
    serde_json::from_str(body)
}

// In an HTTP handler that returns Result<Response, AppError>:
fn handle_request(body: &str) -> Result<Response, AppError> {
    let event = serde_json::from_str::<WebhookEvent>(body)
        .map_err(|e| AppError::BadRequest(format!("invalid payload: {e}")))?;

    process_event(event)?;
    Ok(Response::ok())
}
```

The turbofish `::<WebhookEvent>` tells the compiler which type to deserialize into when it can't be inferred from context.

### Validation After Deserialization

serde handles structural correctness (is the JSON valid? do the types match?) but not domain correctness (is the amount positive? is the email valid?). Validate after parsing:

```rust
#[derive(Deserialize)]
struct CreateOrderRequest {
    item_id: String,
    quantity: u32,
    unit_price_cents: u64,
}

impl CreateOrderRequest {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.item_id.is_empty() {
            errors.push(String::from("item_id cannot be empty"));
        }
        if self.quantity == 0 {
            errors.push(String::from("quantity must be at least 1"));
        }
        if self.unit_price_cents == 0 {
            errors.push(String::from("unit_price_cents must be positive"));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn handle_create_order(body: &str) -> Result<(), AppError> {
    let req: CreateOrderRequest = serde_json::from_str(body)
        .map_err(|e| AppError::BadRequest(format!("malformed JSON: {e}")))?;

    req.validate()
        .map_err(|errs| AppError::Validation(errs))?;

    // req is now structurally and semantically valid
    process_order(req)
}
```

The two-step pattern — serde for structure, manual `validate()` for semantics — is idiomatic. Libraries like `validator` add a derive-based validation layer on top, but the two-step model is the foundation.

### Your notes
<!-- -->


---

## Zero-Copy Deserialization

Most deserialization allocates: it copies string data from the input buffer into owned `String` values. For high-throughput services receiving thousands of JSON payloads per second, these allocations add up.

serde supports **zero-copy deserialization**: borrowing string slices directly from the input buffer rather than copying them. The deserialized type holds `&str` references into the original JSON bytes.

```rust
// 'de is the lifetime of the deserializer (the input buffer)
#[derive(Deserialize)]
struct LogEntry<'de> {
    #[serde(borrow)]
    request_id: &'de str,   // points into the original JSON bytes
    #[serde(borrow)]
    path: &'de str,         // no allocation — just a pointer and length
    status: u16,            // integers always zero-copy anyway
}

fn parse_log_entry(json: &str) -> Result<LogEntry<'_>, serde_json::Error> {
    serde_json::from_str(json)
}

fn process_logs(raw: &str) {
    if let Ok(entry) = parse_log_entry(raw) {
        println!("{} {} {}", entry.request_id, entry.path, entry.status);
    }
    // entry cannot outlive raw — the lifetime ensures this at compile time
}
```

The `'de` lifetime is the lifetime of the input data. `LogEntry<'de>` can only live as long as the JSON string it was parsed from. If you try to store `entry` somewhere that outlives `raw`, the compiler rejects it.

**When to use zero-copy:** Log processors, high-frequency event pipelines, anywhere you parse and immediately use string data without needing to store it. For most API handlers where the parsed struct outlives the request body buffer, just use `String`.

### Your notes
<!-- -->


---

## Custom Serializers and Deserializers

When derive doesn't cover your needs, implement the traits manually.

### Custom Serializer

```rust
use serde::{Serialize, Serializer, ser::SerializeMap};

struct RateLimit {
    requests: u32,
    window_seconds: u32,
}

impl Serialize for RateLimit {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Serialize as a human-readable string like "100/60s" instead of an object
        s.serialize_str(&format!("{}/{}", self.requests, self.window_seconds))
    }
}

// Custom map serialization:
impl Serialize for SomeComplexType {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(2))?;
        map.serialize_entry("version", &self.version)?;
        map.serialize_entry("data", &self.data)?;
        map.end()
    }
}
```

### Custom Deserializer

```rust
use serde::{Deserialize, Deserializer, de::{self, Visitor}};
use std::fmt;

struct RateLimit {
    requests: u32,
    window_seconds: u32,
}

struct RateLimitVisitor;

impl<'de> Visitor<'de> for RateLimitVisitor {
    type Value = RateLimit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a rate limit string like '100/60'")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<RateLimit, E> {
        let parts: Vec<&str> = v.split('/').collect();
        if parts.len() != 2 {
            return Err(E::custom(format!("expected 'N/M', got '{v}'")));
        }
        let requests = parts[0].parse::<u32>().map_err(E::custom)?;
        let window = parts[1].trim_end_matches('s').parse::<u32>().map_err(E::custom)?;
        Ok(RateLimit { requests, window_seconds: window })
    }
}

impl<'de> Deserialize<'de> for RateLimit {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<RateLimit, D::Error> {
        d.deserialize_str(RateLimitVisitor)
    }
}
```

The Visitor pattern is serde's extension point. The visitor says "I want a string" via `deserialize_str`. The format calls `visit_str` with the string it found. Error reporting uses `E::custom(msg)` — the format-specific error type fills in the details.

### Your notes
<!-- -->


---

## Other Formats: TOML, Binary, CSV

serde's crate ecosystem has format implementations for everything. The API is always the same: `to_string`/`from_str` (or equivalent), your type just needs `Serialize`/`Deserialize`.

```rust
// TOML — config files
// Cargo.toml: [dependencies] toml = "0.8"
//
// let config: ServerConfig = toml::from_str(include_str!("config.toml"))?;
// let toml_str = toml::to_string(&config)?;

// bincode — compact binary format for inter-service communication
// Cargo.toml: [dependencies] bincode = "1"
//
// let bytes: Vec<u8> = bincode::serialize(&event)?;
// let event: WebhookEvent = bincode::deserialize(&bytes)?;

// csv — tabular data
// Cargo.toml: [dependencies] csv = "1"
//
// let mut writer = csv::Writer::from_writer(io::stdout());
// writer.serialize(&record)?;
//
// let mut reader = csv::Reader::from_reader(io::stdin());
// for result in reader.deserialize::<Record>() {
//     let record = result?;
// }
```

The key insight: your `#[derive(Serialize, Deserialize)]` types work with all of these without modification. The format plugs in from the outside. If your internal event type needs to go to JSON for the REST API, TOML for the config system, and bincode for the internal message queue — one type, three formats, zero code duplication.

### Choosing a Format

| Format | Use when | Notes |
|--------|----------|-------|
| JSON (serde_json) | REST APIs, external-facing | Human-readable, widely supported |
| TOML (toml) | Config files | Human-readable, good for hierarchical config |
| bincode | Internal services, caching | Binary, fast, compact — not human-readable |
| MessagePack (rmp-serde) | Cross-language binary | Like JSON but binary |
| CSV (csv) | Reports, data exports | Tabular data only |
| CBOR (ciborium) | IoT, constrained environments | Binary, self-describing |

### Your notes
<!-- -->


---

## Comparison to Go: struct tags vs serde attributes

Go uses struct tags — raw string literals parsed at runtime by reflection:

```go
type User struct {
    Name      string    `json:"name" validate:"required"`
    Email     string    `json:"email,omitempty"`
    CreatedAt time.Time `json:"-"`           // skip
    InternalID int      `json:"internal_id"` // rename
}
```

Rust uses attributes parsed at compile time by a procedural macro:

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip)]
    internal_state: u64,
    #[serde(rename = "internal_id")]
    internal_id: u32,
}
```

Key differences:

| Aspect | Go (`encoding/json`) | Rust (serde) |
|--------|---------------------|--------------|
| Tag parsing | Runtime reflection | Compile-time macro expansion |
| Type safety | Limited — silent zero values on missing fields | Strong — `Option<T>` for optionals, errors on missing required fields |
| Performance | Reflection overhead | Zero overhead — generated code |
| Multi-format | Separate packages per format | Single derive, multiple format crates |
| Enum handling | Manual with `MarshalJSON`/`UnmarshalJSON` | Built-in tagging strategies |
| Custom logic | Implement `json.Marshaler`/`json.Unmarshaler` | Implement `Serialize`/`Deserialize` traits |
| Error messages | Terse — often just "cannot unmarshal" | Location (line, column), field path |

Go's `encoding/json` silently ignores unknown fields and fills missing fields with zero values. This is ergonomic but hides bugs — you might parse a payload successfully but have zero values where the field was simply absent or misspelled.

Rust's serde will return an error for required fields that are missing (unless you use `#[serde(default)]`). This is stricter and catches more bugs at the boundary.

### Your notes
<!-- -->
