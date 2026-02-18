# Debugging Solution: Config Loader

## Bug 1 — `unwrap()` on a missing field causes panic

**Location:** `parse_service_config`, the `retry_count` line

**Buggy code:**
```rust
let retry_count = get_u64(json, "retry_count").unwrap() as u32;
```

**What happens:** `get_u64` returns `Option<u64>`. When `retry_count` is absent from the JSON, it returns `None`. Calling `.unwrap()` on `None` panics at runtime with `called unwrap on a None value`.

**Root cause:** This is the difference between `.unwrap()` (panics) and `.unwrap_or(default)` (returns default). In serde, `#[serde(default)]` on a field calls `Default::default()` when the field is absent — the equivalent of `.unwrap_or_default()` or `.unwrap_or(0)`.

**Fix:**
```rust
let retry_count = get_u64(json, "retry_count").unwrap_or(0) as u32;
```

Or more explicitly:
```rust
let retry_count = get_u64(json, "retry_count")
    .map(|n| n as u32)
    .unwrap_or(0);
```

**Prevention:** In serde, `#[serde(default)]` handles this automatically. When writing parsers by hand, treat every `.unwrap()` call as a potential panic — replace with `.unwrap_or`, `.unwrap_or_else`, or `?` depending on whether absence is an error or a default.

---

## Bug 2 — Enum variant name mismatch (case sensitivity)

**Location:** `LogLevel::from_str`

**Buggy code:**
```rust
fn from_str(s: &str) -> Result<LogLevel, String> {
    match s {
        "debug" => Ok(LogLevel::Debug),
        "info"  => Ok(LogLevel::Info),
        "warn"  => Ok(LogLevel::Warn),
        "error" => Ok(LogLevel::Error),
        other   => Err(format!("unknown log level: '{other}'")),
    }
}
```

**What happens:** The match arms use lowercase strings (`"debug"`, `"info"`, etc.) but the test JSON uses uppercase (`"INFO"`, `"WARN"`, `"DEBUG"`). The match falls through to the `other` arm and returns an error.

**Root cause:** String matching in Rust is case-sensitive. When serde encounters `#[serde(rename_all = "UPPERCASE")]` or `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`, it generates the match arms with the right casing. Manual code must handle casing explicitly.

**Fix — Option A:** Match uppercase strings:
```rust
match s {
    "DEBUG" => Ok(LogLevel::Debug),
    "INFO"  => Ok(LogLevel::Info),
    "WARN"  => Ok(LogLevel::Warn),
    "ERROR" => Ok(LogLevel::Error),
    other   => Err(format!("unknown log level: '{other}'")),
}
```

**Fix — Option B:** Case-insensitive matching via `to_lowercase()`:
```rust
match s.to_lowercase().as_str() {
    "debug" => Ok(LogLevel::Debug),
    "info"  => Ok(LogLevel::Info),
    "warn"  => Ok(LogLevel::Warn),
    "error" => Ok(LogLevel::Error),
    other   => Err(format!("unknown log level: '{other}'")),
}
```

Option B is more defensive — it accepts `"INFO"`, `"info"`, and `"Info"`.

**In serde, this is `#[serde(rename_all = "UPPERCASE")]` on the enum or individual variants.** The compiler catches the mismatch at code-generation time rather than at test time.

**Prevention:** When your JSON uses a different casing than your Rust identifiers, use serde's `rename_all`. When writing manual string matching, always include a test that uses the actual format from your data source.

---

## Bug 3 — Wrong type for nested optional field

**Location:** `ServiceConfig.description` field definition and `parse_service_config`

**Buggy code:**
```rust
struct ServiceConfig {
    description: Option<String>,  // wrong
    ...
}
```

**What happens:** The intent is to distinguish:
- Field absent from JSON → `None` (outer None)
- Field present in JSON with a value → `Some(Some("the value"))` (outer Some, inner Some)
- Field present in JSON but explicitly `null` → `Some(None)` (outer Some, inner None)

With `Option<String>`, you can only represent "absent" or "present with value". You cannot represent "present but explicitly null".

**Root cause:** This is the `Option<Option<T>>` pattern for fields that can be omitted or explicitly set to null. In serde:
```rust
#[serde(skip_serializing_if = "Option::is_none")]
description: Option<Option<String>>,
// Absent JSON → None
// "description": "text" → Some(Some("text"))
// "description": null   → Some(None)
```

**Fix:**
```rust
// In the struct:
description: Option<Option<String>>,

// In parse_service_config:
let description: Option<Option<String>> = get_str(json, "description").map(Some);
// When field is absent: get_str returns None → description = None
// When field is present: get_str returns Some("text") → .map(Some) = Some(Some("text"))
// Note: our helpers can't represent "present but null" — that's a serde-level feature.
// For the exercise, treating "present with value" as Some(Some(value)) is sufficient.
```

**Prevention:** Whenever a field needs to distinguish between "not provided" and "explicitly cleared", use `Option<Option<T>>` with appropriate serde attributes. This pattern appears frequently in PATCH endpoints where you want to differentiate "don't update this field" from "set this field to null".

---

## Bug 4 — Timestamp format mismatch

**Location:** `parse_timestamp` function

**Buggy code:**
```rust
fn parse_timestamp(s: &str) -> Result<u64, String> {
    let parts: Vec<&str> = s.split('T').collect();
    if parts.len() != 2 {
        return Err(format!("invalid timestamp, expected YYYY-MM-DDTHH:MM:SSZ, got '{s}'"));
    }
    if !parts[1].ends_with('Z') {
        return Err(format!("timestamp must end with Z, got '{s}'"));
    }
    ...
}
```

**What happens:** The function expects ISO 8601 format with `T` separator and `Z` suffix: `"2026-02-18T14:30:00Z"`. The test provides a space-separated format without the Z: `"2026-02-18 14:30:00"`. The `split('T')` produces one element instead of two, triggering the error.

**Root cause:** A mismatch between the documented format expectation and the actual data. This is the kind of bug that appears constantly in real systems — a field is described as "ISO 8601" but different sources implement different variants.

**Fix:**
```rust
fn parse_timestamp(s: &str) -> Result<u64, String> {
    // Accept both "YYYY-MM-DDTHH:MM:SSZ" and "YYYY-MM-DD HH:MM:SS"
    let sep = if s.contains('T') { 'T' } else { ' ' };
    let parts: Vec<&str> = s.splitn(2, sep).collect();
    if parts.len() != 2 {
        return Err(format!("invalid timestamp format, got '{s}'"));
    }
    let year: u64 = parts[0][..4].parse()
        .map_err(|_| format!("invalid year in timestamp '{s}'"))?;
    Ok(year * 365 * 24 * 3600)
}
```

**In serde, this is the `#[serde(with = "module")]` pattern** — you provide a custom serialize/deserialize module that handles your specific timestamp format. The `chrono` crate integrates with serde and handles many timestamp variants out of the box:
```rust
#[serde(with = "chrono::serde::ts_seconds")]
last_updated: chrono::DateTime<Utc>,
```

**Prevention:** When you own the data format, document exactly which variant you use. When consuming external data, test against real samples, not assumed formats. The `chrono` crate's serde integration handles the most common variants.

---

## Summary

| # | Bug | Concept |
|---|-----|---------|
| 1 | `.unwrap()` on optional field panics when absent | `#[serde(default)]` vs `.unwrap_or()` |
| 2 | Enum variant name case mismatch (lowercase vs UPPERCASE) | `#[serde(rename_all)]`, case-sensitive matching |
| 3 | `Option<String>` can't represent "present but null" | `Option<Option<T>>` for nullable optional fields |
| 4 | Timestamp format mismatch (`T` vs space separator) | `#[serde(with = "module")]`, format tolerance |

## Related Concepts

- [[fundamentals/serialization]] — serde attribute reference, custom deserializers
- [[pitfalls/rust-unwrap-on-none]] — when to use unwrap vs unwrap_or vs ?
- [[fundamentals/rust/structs-methods-and-enums]] — enum pattern matching, derive macros
