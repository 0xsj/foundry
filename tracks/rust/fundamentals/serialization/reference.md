# Serialization Reference — Rust

> Extracted from the serde documentation (https://serde.rs), serde_json crate docs, and the Rust Reference. Version current as of serde 1.0, serde_json 1.0.

---

## Core Crates

| Crate | Purpose | Cargo.toml |
|-------|---------|------------|
| `serde` | Traits, derive macros | `serde = { version = "1", features = ["derive"] }` |
| `serde_json` | JSON format | `serde_json = "1"` |
| `toml` | TOML format | `toml = "0.8"` |
| `bincode` | Binary format | `bincode = "1"` |
| `csv` | CSV format | `csv = "1"` |
| `rmp-serde` | MessagePack format | `rmp-serde = "1"` |

---

## The Serialize and Deserialize Traits

```rust
// The Serialize trait — types that can be converted to any format
pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

// The Deserialize trait — types that can be reconstructed from any format
pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}
```

The `'de` lifetime on `Deserialize` is the lifetime of the input data. For types that borrow from the input (zero-copy), `'de` flows into the type. For owned types, `'de` appears only in the trait bound.

---

## Derive Macros

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MyType { ... }

#[derive(Serialize, Deserialize)]
enum MyEnum { ... }
```

Requires `serde` with `features = ["derive"]` in `Cargo.toml`. Generates implementations of `Serialize` and `Deserialize` based on the type's structure.

---

## Container Attributes

Attributes placed on the struct/enum definition (before the `struct`/`enum` keyword).

### `#[serde(rename = "name")]`
Serialize and deserialize this struct/enum with the given name instead of its Rust name.

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename = "user_record")]
struct UserRecord { ... }
```

### `#[serde(rename_all = "...")]`
Rename all fields (of a struct) or variants (of an enum) according to the given case convention.

Allowed values: `"lowercase"`, `"UPPERCASE"`, `"PascalCase"`, `"camelCase"`, `"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"`, `"SCREAMING-KEBAB-CASE"`

```rust
#[serde(rename_all = "camelCase")]
struct ApiRequest {
    user_id: u64,       // → "userId"
    created_at: u64,    // → "createdAt"
}
```

### `#[serde(deny_unknown_fields)]`
Treat unknown fields as an error during deserialization. By default, unknown fields are ignored.

```rust
#[serde(deny_unknown_fields)]
struct StrictConfig {
    host: String,
    port: u16,
    // Any JSON field not listed here causes a deserialization error
}
```

### `#[serde(default)]`
Use `Default::default()` for any missing fields during deserialization. Applied at the container level, this affects all fields. Applied at the field level, it uses the field's default.

### `#[serde(bound = "T: Serialize")]`
Override the trait bounds that serde generates for generic types. The generated bounds are usually correct but occasionally need manual override for complex generic scenarios.

### `#[serde(tag = "type")]` — Internal Tagging (enums only)
Use an internally tagged representation. The tag field is a field inside the JSON object.

```rust
#[serde(tag = "type")]
enum Message {
    Request { id: u64, method: String },
    Response { id: u64, result: String },
}
// {"type": "Request", "id": 1, "method": "get_user"}
```

Only works with struct variants and newtype variants wrapping a struct (not tuple variants or variants containing non-struct types).

### `#[serde(tag = "t", content = "c")]` — Adjacent Tagging (enums only)
Use an adjacently tagged representation. Tag and content are separate fields.

```rust
#[serde(tag = "kind", content = "data")]
enum Event {
    Created { id: u64 },
    Deleted(u64),  // tuple variants work here
}
// {"kind": "Created", "data": {"id": 1}}
// {"kind": "Deleted", "data": 1}
```

### `#[serde(untagged)]` (enums only)
No tag. serde tries each variant in order and uses the first that deserializes successfully.

```rust
#[serde(untagged)]
enum StringOrNumber {
    Str(String),
    Num(f64),
}
```

---

## Field Attributes

Attributes placed on individual struct fields or enum variant fields.

### `#[serde(rename = "name")]`
Serialize and deserialize this field with the given name.

```rust
#[serde(rename = "user_id")]
id: u64,
```

### `#[serde(alias = "name")]`
Accept an alternative name during deserialization. Multiple aliases allowed.

```rust
#[serde(alias = "userId", alias = "user-id")]
user_id: u64,
```

### `#[serde(default)]`
If the field is missing during deserialization, use `Default::default()`.

```rust
#[serde(default)]
retry_count: u32,   // defaults to 0 if absent
```

### `#[serde(default = "path")]`
If the field is missing during deserialization, call `path()` to get the default value.

```rust
#[serde(default = "default_port")]
port: u16,

fn default_port() -> u16 { 8080 }
```

### `#[serde(skip)]`
Skip this field entirely — not serialized or deserialized.

```rust
#[serde(skip)]
internal_state: u64,
```

### `#[serde(skip_serializing)]`
Only skip during serialization. Still deserialized (uses `Default::default()` if absent).

### `#[serde(skip_deserializing)]`
Only skip during deserialization. Still serialized.

### `#[serde(skip_serializing_if = "path")]`
Skip serializing this field if `path(value)` returns true. `path` must be a function `fn(&T) -> bool`.

```rust
// Don't include None fields in JSON output
#[serde(skip_serializing_if = "Option::is_none")]
description: Option<String>,

// Don't include empty vecs
#[serde(skip_serializing_if = "Vec::is_empty")]
tags: Vec<String>,
```

### `#[serde(flatten)]`
Flatten the contents of this field into the container it's defined in. The field type must implement `Serialize`/`Deserialize` for map-like structures.

```rust
#[derive(Serialize, Deserialize)]
struct Pagination {
    page: u32,
    per_page: u32,
}

#[derive(Serialize, Deserialize)]
struct ListUsersRequest {
    filter: String,
    #[serde(flatten)]
    pagination: Pagination,
    // JSON: {"filter": "active", "page": 1, "per_page": 20}
}
```

### `#[serde(with = "module")]`
Use the specified module's `serialize` and `deserialize` functions for this field. The module must contain:

```rust
pub fn serialize<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<T, D::Error>
```

### `#[serde(serialize_with = "path")]` / `#[serde(deserialize_with = "path")]`
Like `with`, but for serialize or deserialize only.

### `#[serde(borrow)]`
Borrow string/bytes from the input during deserialization (zero-copy). The field type must be `&'de str`, `&'de [u8]`, `Cow<'de, str>`, or `Cow<'de, [u8]>`.

```rust
#[derive(Deserialize)]
struct Request<'de> {
    #[serde(borrow)]
    method: &'de str,
}
```

---

## Variant Attributes

Attributes placed on enum variant definitions.

### `#[serde(rename = "name")]`
Serialize this variant with the given name.

```rust
#[serde(rename = "payment_succeeded")]
PaymentSucceeded { amount: u64 },
```

### `#[serde(alias = "name")]`
Accept this alias name during deserialization for this variant.

### `#[serde(skip)]`
Never serialize or deserialize this variant. Returns an error if encountered during deserialization.

### `#[serde(other)]`
For tagged enums, map any unknown variant to this one during deserialization. The variant must be a unit variant.

```rust
#[serde(tag = "type")]
enum WebhookEvent {
    PaymentSucceeded { amount: u64 },
    PaymentFailed { reason: String },
    #[serde(other)]
    Unknown,  // catches any future event types
}
```

---

## serde_json API Reference

### Serialization

```rust
// Serialize to String
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error>

// Serialize to pretty-printed String
pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String, Error>

// Serialize to Vec<u8>
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, Error>

// Serialize to any io::Write
pub fn to_writer<W: io::Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error>
pub fn to_writer_pretty<W: io::Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error>
```

### Deserialization

```rust
// Deserialize from &str
pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, Error>

// Deserialize from &[u8]
pub fn from_slice<'a, T: Deserialize<'a>>(v: &'a [u8]) -> Result<T, Error>

// Deserialize from any io::Read
pub fn from_reader<R: io::Read, T: DeserializeOwned>(rdr: R) -> Result<T, Error>
```

### Value Type

```rust
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),  // i64, u64, or f64 depending on the JSON
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

**Indexing:** `value["key"]` and `value[0]` return `&Value`. Returns `&Value::Null` if the index doesn't exist (no panic).

**Conversion methods:**

```rust
value.as_str() -> Option<&str>
value.as_u64() -> Option<u64>
value.as_i64() -> Option<i64>
value.as_f64() -> Option<f64>
value.as_bool() -> Option<bool>
value.as_array() -> Option<&Vec<Value>>
value.as_object() -> Option<&Map<String, Value>>
value.is_null() -> bool
value.is_string() -> bool
// etc.
```

**`json!` macro:**

```rust
let v = serde_json::json!({
    "key": "value",
    "number": 42,
    "array": [1, 2, 3],
    "nested": { "flag": true }
});
```

### Converting between typed and Value

```rust
// From T to Value (infallible for Serialize types)
let v: Value = serde_json::to_value(&my_struct)?;

// From Value to T
let my_struct: MyStruct = serde_json::from_value(v)?;
```

### Error Type

```rust
pub struct Error { ... }

impl Error {
    pub fn line(&self) -> usize   // line number in the input (1-indexed)
    pub fn column(&self) -> usize // column number in the input (1-indexed)
    pub fn is_io(&self) -> bool
    pub fn is_syntax(&self) -> bool
    pub fn is_data(&self) -> bool  // type mismatch, missing field, etc.
    pub fn is_eof(&self) -> bool
}
```

---

## The serde Data Model

serde's abstract data model has 29 types. Your Rust types map to these; formats encode/decode them.

| Category | Types |
|----------|-------|
| Primitives | bool, i8–i128, u8–u128, f32, f64, char |
| Strings | string (owned), str (borrowed, zero-copy) |
| Bytes | bytes (owned), byte_buf (borrowed) |
| Options | option (Some/None) |
| Unit | unit, unit_struct, unit_variant |
| Newtype | newtype_struct, newtype_variant |
| Sequences | seq, tuple, tuple_struct, tuple_variant |
| Maps | map, struct, struct_variant |

---

## Enum Tagging Summary

| Strategy | Attribute | JSON Output | Works With |
|----------|-----------|-------------|------------|
| External (default) | (none) | `{"VariantName": {...}}` | All variants |
| Internal | `#[serde(tag = "t")]` | `{"t": "Variant", ...fields...}` | Struct variants, unit variants |
| Adjacent | `#[serde(tag = "t", content = "c")]` | `{"t": "Variant", "c": {...}}` | All variants |
| Untagged | `#[serde(untagged)]` | `{...fields...}` | All variants (try each in order) |

---

## Implementing Custom Serialize

```rust
use serde::{Serialize, Serializer};
use serde::ser::{SerializeStruct, SerializeMap, SerializeSeq};

impl Serialize for MyType {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // For a struct with N fields:
        let mut state = s.serialize_struct("MyType", N)?;
        state.serialize_field("field_name", &self.field)?;
        state.end()

        // For a map:
        // let mut map = s.serialize_map(Some(self.map.len()))?;
        // for (k, v) in &self.map { map.serialize_entry(k, v)?; }
        // map.end()

        // For a sequence:
        // let mut seq = s.serialize_seq(Some(self.items.len()))?;
        // for item in &self.items { seq.serialize_element(item)?; }
        // seq.end()

        // For a primitive:
        // s.serialize_str("some string")
        // s.serialize_u64(42)
    }
}
```

---

## Implementing Custom Deserialize

```rust
use serde::{Deserialize, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use std::fmt;

struct MyTypeVisitor;

impl<'de> Visitor<'de> for MyTypeVisitor {
    type Value = MyType;

    // Human-readable description of what this visitor expects
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a MyType object")
    }

    // Called when the format provides a map (JSON object)
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<MyType, A::Error> {
        let mut field: Option<String> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "field" => { field = Some(map.next_value()?); }
                _ => { let _: de::IgnoredAny = map.next_value()?; }
            }
        }
        let field = field.ok_or_else(|| de::Error::missing_field("field"))?;
        Ok(MyType { field })
    }
}

impl<'de> Deserialize<'de> for MyType {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<MyType, D::Error> {
        d.deserialize_map(MyTypeVisitor)
    }
}
```

---

## DeserializeOwned

A convenience alias for `for<'de> Deserialize<'de>` — types that can be deserialized from any lifetime, meaning they own all their data (no borrowed fields).

```rust
// This constraint means T cannot borrow from the input:
pub fn from_reader<R: io::Read, T: DeserializeOwned>(rdr: R) -> Result<T, Error>

// vs from_str which allows borrowed fields:
pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, Error>
```

When you use zero-copy deserialization (`&'de str` fields), you must use `from_str`/`from_slice`, not `from_reader`, because the reader doesn't have a stable buffer to borrow from.

---

## Common Patterns

### Optional fields with no null in JSON

```rust
#[serde(skip_serializing_if = "Option::is_none")]
field: Option<String>,
```

### Unknown enum variants as a catch-all

```rust
#[serde(tag = "type")]
enum Event {
    Known { id: u64 },
    #[serde(other)]
    Unknown,
}
```

### Flattening extra fields into a HashMap

```rust
#[derive(Deserialize)]
struct Request {
    id: String,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}
```

### Transparently wrapping a value (newtype)

```rust
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct UserId(u64);
// Serializes as just 42, not {"0": 42} or similar
```

### Deserializing either a string or a number

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrU64 {
    Str(String),
    Int(u64),
}
```

---

## References

- [serde.rs — The official serde documentation](https://serde.rs)
- [serde API docs](https://docs.rs/serde)
- [serde_json API docs](https://docs.rs/serde_json)
- [serde attributes reference](https://serde.rs/attributes.html)
- [serde data model](https://serde.rs/data-model.html)
- [Custom serialization guide](https://serde.rs/custom-serialization.html)
- [serde examples](https://serde.rs/examples.html)
