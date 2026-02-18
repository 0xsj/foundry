# Expert Review: String Utilities

## Critical Issues

### 1. `short_id` can panic on non-ASCII input

**Location:** `short_id`, the slice `s[..n]`

```rust
pub fn short_id(s: &str, n: usize) -> String {
    if s.len() < n {
        return String::new();
    }
    s[..n].to_string()   // panics if byte n is not a char boundary
}
```

**Problem:** `s.len()` returns the byte length. `s[..n]` slices the first `n` bytes. If
byte `n` falls in the middle of a multibyte character — for example, an emoji or
accented character — this panics at runtime with "byte index N is not a char boundary."

The test only passes because the test input `"request-12345"` is pure ASCII. Any caller
that passes a string with non-ASCII content will hit a panic in production.

**Fix:** Use `char_indices` to find the byte boundary after `n` characters:

```rust
pub fn short_id(s: &str, n: usize) -> String {
    match s.char_indices().nth(n) {
        Some((byte_pos, _)) => s[..byte_pos].to_string(),
        None => s.to_string(),   // string has fewer than n chars — return all of it
    }
}
```

Or, if the intent is truly "first n bytes" (e.g., for a binary ID), be explicit:

```rust
pub fn short_id_bytes(s: &str, n: usize) -> &str {
    // Caller's responsibility to pass n that lands on a char boundary.
    // Document this precondition clearly.
    &s[..n.min(s.len())]
}
```

**Concept:** Never slice a `&str` at an integer byte index unless you have verified
the byte is a character boundary. `char_indices()` gives you safe byte positions for
each character.

---

## Major Concerns

### 2. `Display` for `FieldSummary` uses `Debug` format for the preview

**Location:** `impl fmt::Display for FieldSummary`

```rust
impl fmt::Display for FieldSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({} words)", self.preview, self.word_count)
    }
}
```

**Problem:** `{:?}` invokes `Debug` formatting on `self.preview`, which wraps it in
quotes: `"The quick brown fox..." (9 words)`. This is developer output, not user output.
A user-facing `Display` implementation should not include debug quotes. It confuses
callers who display the summary in a UI or log it to a user-visible report.

**Fix:**

```rust
impl fmt::Display for FieldSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} words)", self.preview, self.word_count)
    }
}
```

`{}` uses `Display` for `self.preview` (which is a `String`), producing clean output
without quotation marks.

**Concept:** `{:?}` (Debug) is for developers inspecting internal state. `{}` (Display)
is for human-readable output. A `Display` impl that uses `{:?}` internally is almost
always a mistake. Rule of thumb: `Display` impls should never contain `{:?}`.

---

### 3. `truncate` allocates even when no truncation is needed

**Location:** `truncate`, the short-circuit branch

```rust
pub fn truncate(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()    // allocates a new String even though the input is fine
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        truncated + "..."
    }
}
```

**Problem:** The common case — where the string is short enough — still allocates a new
`String`. For a function used in a hot path (per-log-line summary), this adds an
allocation on every call even when the string is already within bounds.

`Cow<str>` is the idiomatic solution: borrow when no change is needed, own when you must
construct new data.

**Fix:**

```rust
use std::borrow::Cow;

pub fn truncate(s: &str, max_chars: usize) -> Cow<str> {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        Cow::Borrowed(s)    // zero allocation — just returns a reference
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        Cow::Owned(truncated + "...")
    }
}
```

Callers use `&*truncate(s, 10)` or pass it to any `&str` parameter via deref coercion.
`FieldSummary::preview` can be updated to `Cow<'a, str>` if the struct needs to reflect
the lifetime — or, if the struct owns its data, call `.into_owned()` at construction.

**Concept:** `Cow<str>` is the right return type for "sometimes borrow, sometimes own."
Returning `String` unconditionally from a function that often has nothing to change is a
common source of unnecessary allocation in hot paths.

---

### 4. `normalize_service_name` takes `String` by value — forces allocation at every call site

**Location:** `normalize_service_name` parameter

```rust
pub fn normalize_service_name(name: String) -> String {
    name.trim().to_lowercase()
}
```

**Problem:** The function takes ownership of `name`. Every caller must either:
- Have an owned `String` they are willing to give up, OR
- Allocate a new `String` just to pass it:
  ```rust
  normalize_service_name("Auth-Service".to_string())  // needless allocation
  ```

The function doesn't need ownership — it only reads `name` to produce a new lowercased
string. Accepting `&str` is strictly more flexible.

**Fix:**

```rust
pub fn normalize_service_name(name: &str) -> String {
    name.trim().to_lowercase()
}
```

Callers can now pass string literals, `&String`, or `&str` directly. The `to_lowercase()`
call still allocates — that's necessary since we're producing new data — but the
allocation at the call site is eliminated.

**Concept:** A function that only reads its input should accept a reference. Taking
ownership of `String` in a parameter forces callers to do more work (and allocate more)
than necessary. This is the `&str` vs `String` parameter principle applied.

---

### 5. `values_with_prefix` clones every matching value — callers may only need references

**Location:** `values_with_prefix`, the `.cloned()` call

```rust
pub fn values_with_prefix(map: &HashMap<String, String>, prefix: &str) -> Vec<String> {
    map.values()
       .filter(|v| v.starts_with(prefix))
       .cloned()    // clones every matching value
       .collect()
}
```

**Problem:** The caller receives `Vec<String>` — fully owned copies of every matching
value. If the caller only needs to read the values (print them, compare them, count
them), they have paid for clones they don't need. This function is described as being
used in a streaming pipeline — cloning every matching log field on every log line adds
up.

**Fix:** Return references when the map is alive:

```rust
pub fn values_with_prefix<'a>(map: &'a HashMap<String, String>, prefix: &str) -> Vec<&'a str> {
    map.values()
       .filter(|v| v.starts_with(prefix))
       .map(|v| v.as_str())
       .collect()
}
```

Callers that need owned copies can call `.to_string()` on the references they care about.
Callers that don't need ownership (the majority) get cheap references. The lifetimes are
tied to the map's borrow — `'a` ensures the references are valid as long as `map` is.

**Concept:** Return references when the underlying data already exists and the caller
may not need ownership. Return owned data only when the caller genuinely needs to take
ownership or when the data is newly created inside the function.

---

### 6. `build_field_index` always allocates — `value.trim()` could borrow when no trimming needed

**Location:** `build_field_index`, the `value.trim().to_string()` call

```rust
index.insert(
    name.to_string(),
    value.trim().to_string(),   // always allocates a new String for the value
);
```

**Problem:** If `*value` is already trimmed (no leading/trailing whitespace), `trim()`
returns the same `&str` slice pointing into `*value`. Then `.to_string()` allocates
anyway. For already-trimmed values (common in practice), this is a needless allocation.

**Fix:** Use `Cow<str>` for the value:

```rust
use std::borrow::Cow;

pub fn build_field_index<'a>(fields: &[(&'a str, &'a str)]) -> HashMap<&'a str, Cow<'a, str>> {
    fields.iter()
        .map(|(name, value)| {
            let trimmed = value.trim();
            let cow = if trimmed.len() == value.len() {
                Cow::Borrowed(*value)   // no trimming needed — borrow
            } else {
                Cow::Owned(trimmed.to_string())   // trimmed — must own
            };
            (*name, cow)
        })
        .collect()
}
```

Alternatively, if you don't want to introduce `Cow` into the return type (a valid
tradeoff), at minimum use `Cow` internally and call `.into_owned()` before inserting,
so you only allocate when the value is actually modified.

**Concept:** `str::trim()` returns a `&str` slice into the original. If the lengths
match, no trimming occurred and you can borrow. Use `Cow<str>` to express "sometimes I
borrow, sometimes I own" without forcing an allocation on the common path.

---

## Minor Suggestions

### 7. `is_valid_level` builds a new `String` on every call via `to_uppercase()`

**Location:** `is_valid_level`

```rust
pub fn is_valid_level(s: &str) -> bool {
    let upper = s.to_uppercase();
    upper == "DEBUG" || upper == "INFO" || upper == "WARN"
        || upper == "ERROR" || upper == "TRACE"
}
```

`to_uppercase()` allocates a `String`. For a function that only compares to five known
ASCII strings, this is unnecessary. Use `eq_ignore_ascii_case`:

```rust
pub fn is_valid_level(s: &str) -> bool {
    ["debug", "info", "warn", "error", "trace"]
        .iter()
        .any(|&l| s.eq_ignore_ascii_case(l))
}
```

Zero allocation. `eq_ignore_ascii_case` compares ASCII character-by-character without
allocating. For non-ASCII Unicode, use `unicase` crate — but log levels are always ASCII.

---

### 8. `summarize` is fine but could note that `truncate` allocates

The `summarize` function calls `truncate`, which returns a `String`. If `truncate` is
changed to return `Cow<str>` (see issue #3), `summarize` would call `.into_owned()` when
storing in `FieldSummary::preview`. This is worth noting in comments for future
maintainers.

---

## Positive Feedback

1. **`truncate` correctly counts characters, not bytes** — using `s.chars().count()` and
   `s.chars().take(max_chars)` is the right approach for UTF-8 text. The test with
   `"café"` confirms this works correctly.

2. **`build_field_index` uses `name.to_string()` for keys** — since the keys need to
   be owned (HashMap requires owned keys), this is correct. No lazy borrowing here.

3. **`is_valid_level` is case-insensitive** — normalizing to uppercase before comparison
   is correct. The fix (issue #7) just removes the unnecessary allocation.

4. **`summarize` computes `word_count` from the original text, not the truncated preview**
   — this is the correct behavior. The word count reflects the full content.

5. **Test coverage** — tests cover truncation with Unicode, normalization, and the
   `values_with_prefix` function. Edge cases like empty input could be added, but the
   existing coverage is solid for a first PR.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `short_id` panics on non-ASCII at byte index `n` | Always use `char_indices` or `split_at_char` — never bare `&s[..n]` |
| 2 | Major | `Display` impl uses `{:?}` — wraps in quotes | `{:?}` is Debug; `Display` should use `{}` |
| 3 | Major | `truncate` allocates even when no truncation needed | Return `Cow<str>`: borrow when clean, own when modified |
| 4 | Major | `normalize_service_name` takes `String` by value | Accept `&str` — the function only reads, not owns |
| 5 | Major | `values_with_prefix` clones all matches | Return `Vec<&str>` — let callers decide if they need ownership |
| 6 | Major | `build_field_index` always allocates values | Use `Cow<str>` for values that may not need modification |
| 7 | Minor | `is_valid_level` allocates via `to_uppercase()` | Use `eq_ignore_ascii_case` — zero allocation for ASCII |
| 8 | Minor | `summarize` note on `truncate` allocation | Documentation gap |

## Related Concepts

- [[fundamentals/rust/strings-and-text]] — String vs &str, Cow<str>, byte vs char indexing
- [[fundamentals/strings-and-text]] — Cross-language string comparison
- [[pitfalls/rust-str-slice-boundary]] — slicing at non-char boundaries
- [[pitfalls/rust-string-vs-str-params]] — accept &str, not String, in parameters
- [[pitfalls/rust-display-vs-debug]] — when to use {} vs {:?} in Display impls
