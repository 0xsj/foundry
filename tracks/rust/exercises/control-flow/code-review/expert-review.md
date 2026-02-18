# Expert Review: Webhook Validator

## Critical Issues

### 1. Missing `"shopify"` source — test `test_validate_source_valid` fails

**Location:** `validate_source`, line with `"github" => Ok(())`

```rust
pub fn validate_source(source: &str) -> Result<(), ValidationError> {
    match source {
        "stripe" => Ok(()),
        "paypal" => Ok(()),
        "github" => Ok(()),
        // "shopify" is missing
        s if s.len() == 0 => Err(ValidationError::UnsupportedSource(...)),
        s => Err(ValidationError::UnsupportedSource(...)),
    }
}
```

The test `test_validate_source_valid` asserts `validate_source("shopify") == Ok(())`,
but the function returns `Err(UnsupportedSource("shopify"))` because `"shopify"`
is not listed.

**Fix:** Add `"shopify"` as a valid source:

```rust
match source {
    "stripe" | "paypal" | "github" | "shopify" => Ok(()),
    s if s.is_empty() => Err(ValidationError::UnsupportedSource(s.to_string())),
    s => Err(ValidationError::UnsupportedSource(s.to_string())),
}
```

**Lesson:** String matching has no compile-time exhaustiveness check — unlike
enum matching, adding a new source won't produce a compiler error. This is a
case where a test suite is your safety net, but the test correctly caught the
miss. Consider storing valid sources in a constant slice and using `.contains()`.

---

### 2. `event_type_name` silently misses new variants

**Location:** `event_type_name`

```rust
pub fn event_type_name(event_type: &EventType) -> &'static str {
    if *event_type == EventType::UserCreated {
        "user.created"
    } else if ... {
        ...
    } else {
        "unknown"  // catches EventType::Unknown but also any NEW variant
    }
}
```

The `if/else` chain compiles but does not use pattern matching. If a new variant
(e.g., `SubscriptionRenewed`) is added to `EventType`, this function returns
`"unknown"` for it with no compiler warning.

**Fix:** Use `match` to get exhaustiveness checking:

```rust
pub fn event_type_name(event_type: &EventType) -> &'static str {
    match event_type {
        EventType::UserCreated => "user.created",
        EventType::UserDeleted => "user.deleted",
        EventType::OrderPlaced => "order.placed",
        EventType::OrderCancelled => "order.cancelled",
        EventType::PaymentReceived => "payment.received",
        EventType::PaymentFailed => "payment.failed",
        EventType::Unknown => "unknown",
    }
}
```

Now adding `SubscriptionRenewed` to the enum produces a compile error here,
forcing the author to handle it explicitly.

**Lesson:** `if/else` chains on enum values lose Rust's signature safety
property. `match` on enums is exhaustive — the compiler checks it for you.
Prefer `match` for any dispatch on an enum value.

---

## Major Concerns

### 3. Redundant match arms in `validate_version`

**Location:** `validate_version`

```rust
match version {
    1 => Ok(()),
    2 => Ok(()),
    3 => Ok(()),
    _ => Err(ValidationError::InvalidVersion(version)),
}
```

Three arms that all do the same thing. Use an or-pattern:

```rust
match version {
    1 | 2 | 3 => Ok(()),
    _ => Err(ValidationError::InvalidVersion(version)),
}
```

This is more readable and easier to extend (adding version 4 is one character
change). The intent — "versions 1, 2, or 3 are valid" — is clearer as an
or-pattern.

---

### 4. Unnecessary intermediate variable and redundant arms in `requires_signature`

**Location:** `requires_signature`

```rust
pub fn requires_signature(event_type: &EventType) -> bool {
    let result = match event_type {
        EventType::PaymentReceived => true,
        EventType::PaymentFailed => true,
        _ => false,
    };
    result
}
```

Two issues:
1. The two `true` arms can be consolidated with an or-pattern.
2. The `result` binding is unnecessary — `match` is already an expression.

**Fix:**

```rust
pub fn requires_signature(event_type: &EventType) -> bool {
    matches!(event_type, EventType::PaymentReceived | EventType::PaymentFailed)
}
```

Or if you prefer the explicit match form:

```rust
pub fn requires_signature(event_type: &EventType) -> bool {
    match event_type {
        EventType::PaymentReceived | EventType::PaymentFailed => true,
        _ => false,
    }
}
```

The `matches!` macro is idiomatic for boolean pattern tests.

---

### 5. `validate_event` manually matches `Ok`/`Err` instead of using `?`

**Location:** `validate_event`

```rust
let version_result = validate_version(event.version);
match version_result {
    Ok(_) => {},
    Err(e) => return Err(e),
}

// ... then again:
let source_result = validate_source(&event.source);
match source_result {
    Ok(_) => {},
    Err(e) => return Err(e),
}
```

This is exactly the pattern that the `?` operator was designed to replace:

```rust
validate_version(event.version)?;
validate_source(&event.source)?;
```

The `?` operator means "if this is Err, return it from the current function;
if it's Ok, unwrap the value." It is far more readable and produces the same
behavior.

---

### 6. Magic number `10 * 1024 * 1024` in `validate_event`

**Location:** `validate_event`, payload size check

```rust
if event.payload_size > 10 * 1024 * 1024 {
    return Err(ValidationError::PayloadTooLarge);
}
```

The constant `10 * 1024 * 1024` (10 MB) has no name and appears only once, but
if the limit changes or is referenced elsewhere, it's easy to miss. Prefer:

```rust
const MAX_PAYLOAD_BYTES: usize = 10 * 1024 * 1024; // 10 MB

if event.payload_size > MAX_PAYLOAD_BYTES {
    return Err(ValidationError::PayloadTooLarge);
}
```

---

### 7. `loop { break value }` anti-pattern in `error_severity`

**Location:** `error_severity`

```rust
pub fn error_severity(err: &ValidationError) -> &'static str {
    let severity = loop {
        let s = match err { ... };
        break s;
    };
    severity
}
```

This is `loop { break expr }` — an infinite loop that immediately breaks with a
value. This is sometimes used intentionally (e.g., to use `break` as a labeled
expression exit), but here it's entirely unnecessary. The function should just
return the match value directly:

```rust
pub fn error_severity(err: &ValidationError) -> &'static str {
    match err {
        ValidationError::MissingSignature => "critical",
        ValidationError::InvalidVersion(_) => "high",
        ValidationError::PayloadTooLarge => "high",
        ValidationError::UnknownEventType => "medium",
        ValidationError::UnsupportedSource(_) => "low",
    }
}
```

---

## Minor Suggestions

### 8. `s.len() == 0` should be `s.is_empty()` in `validate_source`

```rust
s if s.len() == 0 => ...
```

`is_empty()` is more expressive and is the standard idiom. Clippy warns about
`len() == 0` with `clippy::len_zero`.

### 9. Arm order in `validate_source` — empty check should come first

Even after fixing the `is_empty()` style, the empty-string check is after the
literal arms. It only catches strings that didn't match `"stripe"`, `"paypal"`,
or `"github"`. This is technically correct (those literals are not empty strings),
but it reads as if the empty check is a "special case" when it's more of a
precondition. Moving it first makes intent clear:

```rust
match source {
    s if s.is_empty() => Err(ValidationError::UnsupportedSource(s.to_string())),
    "stripe" | "paypal" | "github" | "shopify" => Ok(()),
    s => Err(ValidationError::UnsupportedSource(s.to_string())),
}
```

---

## Positive Feedback

1. **`validate_event` structure is sound** — checking signature, version,
   payload size, and source in sequence makes the validation order clear.
   The logic is correct even if the syntax can be improved.

2. **`ValidationError` enum design is good** — distinct variants for each error
   type with relevant data (`InvalidVersion(u8)`, `UnsupportedSource(String)`)
   means callers can pattern-match on the error and handle each case
   specifically. This is idiomatic Rust error design.

3. **`requires_signature` correctly isolates the signature-required logic**
   — separating this into its own function makes it testable and readable.
   The logic is right even if the implementation can be condensed.

4. **Tests exist and catch the `"shopify"` bug** — the test
   `test_validate_source_valid` correctly asserts Shopify as valid, which
   caught the missing source. This is the test suite doing its job.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | Missing `"shopify"` in `validate_source` | String matching non-exhaustiveness |
| 2 | Critical | `if/else` chain on enum misses new variants | `match` exhaustiveness on enums |
| 3 | Major | Three identical match arms instead of or-pattern | `\|` in match patterns |
| 4 | Major | Unnecessary `result` binding + redundant arms in `requires_signature` | `match` as expression, or-patterns |
| 5 | Major | Manual `match Ok/Err` instead of `?` | The `?` operator |
| 6 | Major | Magic number `10 * 1024 * 1024` | Named constants |
| 7 | Major | `loop { break value }` when direct expression suffices | Loop expressions |
| 8 | Minor | `s.len() == 0` vs `s.is_empty()` | Idiomatic stdlib methods |
| 9 | Minor | Empty string guard after literal arms | Match arm ordering for clarity |

## Related Concepts

- [[fundamentals/rust/control-flow]] — match, if let, ? operator
- [[pitfalls/rust-match-arm-order]] — patterns evaluated top-to-bottom
- [[pitfalls/rust-ifelse-vs-match]] — when if/else loses exhaustiveness
