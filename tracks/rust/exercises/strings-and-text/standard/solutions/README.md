# Solution Notes: Structured Log Parser

## Approach

The solution is built around three composable types:

1. `LogLevel` — a typed enum rather than a `&'static str`, enabling `Ord` comparisons,
   `Display` for human output, and `FromStr` for parsing from raw text.
2. `LogLine` — a parsed log entry with `Display` and `FromStr` that round-trip cleanly.
3. `LogParser` — stores entries as `Vec<LogLine>` and returns `&LogLine` references
   from query methods, avoiding any cloning of log data.

`sanitize_header` demonstrates `Cow<str>`: the common case (clean header) returns
`Cow::Borrowed` with zero allocation; the uncommon case (dirty header) allocates exactly
once.

## Key Decisions

### split_once for parsing

```rust
let (timestamp, rest) = s.split_once(' ')
    .ok_or_else(|| format!("missing space after timestamp: {:?}", s))?;
```

`split_once` is the right tool for structured log parsing: it splits at the first
occurrence and returns `Option<(&str, &str)>`. All returned slices borrow from `s` —
no allocation until the final `.to_string()` calls when constructing `LogLine`.

Using `?` with `.ok_or_else()` propagates parse failures cleanly. The error message
includes the problematic input, which is critical for debugging production log streams.

### Cow::Borrowed in the common path

```rust
fn sanitize_header<'a>(value: &'a str) -> Cow<'a, str> {
    if value.contains('\n') || value.contains('\r') {
        Cow::Owned(value.replace(['\n', '\r'], " "))
    } else {
        Cow::Borrowed(value)
    }
}
```

The key insight: the check (`contains`) is O(n) in bytes but there is no allocation.
The fast path does one scan and returns a borrowed reference. Only the slow path
allocates. Callers use `&*result` or pass to `&str` parameters — the code is identical
for both cases.

### BTreeSet for alphabetical services

```rust
let services: BTreeSet<&str> = p.entries.iter()
    .map(|e| e.service.as_str())
    .collect();
let services_list = services.into_iter().collect::<Vec<_>>().join(", ");
```

`BTreeSet` gives us deduplication and alphabetical order in one pass. The alternative
(collect into `Vec`, sort, dedup) requires two extra passes and a sort call.
`e.service.as_str()` borrows from the `String` field — no allocation for the set keys.

### `write!` for report building

```rust
let mut out = String::with_capacity(512);
writeln!(out, "=== Log Summary ===").unwrap();
writeln!(out, "Total entries: {}", p.entries.len()).unwrap();
```

Pre-allocating with `with_capacity` avoids reallocations during the write loop.
`writeln!` returns `fmt::Result` — `.unwrap()` is acceptable here because writing to a
`String` never fails (the only error would be an out-of-memory condition, which panics
anyway).

### by_level returning references

```rust
fn by_level(&self, level: LogLevel) -> Vec<&LogLine> {
    self.entries.iter()
        .filter(|e| e.level == level)
        .collect()
}
```

`self.entries.iter()` yields `&LogLine`. `.collect()` collects `&LogLine` references
into `Vec<&LogLine>`. The lifetimes are inferred: the returned references live as long as
`&self`. No cloning occurs — querying a large log set is cheap.

## Complexity

| Method | Time | Allocation |
|---|---|---|
| `ingest` | O(n) parse + O(1) push | One `LogLine` struct |
| `by_level` | O(n) scan | `Vec<&LogLine>` (pointers only) |
| `by_service` | O(n) scan | `Vec<&LogLine>` (pointers only) |
| `sanitize_header` (clean) | O(n) scan | Zero |
| `sanitize_header` (dirty) | O(n) scan + O(n) replace | One `String` |
| `summary` | O(n) | One `String` buffer |

## Variants

### Variant A: Incremental level counts

Instead of scanning `entries` in `summary` to count per level, maintain a
`HashMap<LogLevel, u32>` in `LogParser` and increment at ingest time (like
`LogAnalyzer` in the collections exercise). This makes `summary` O(1) per level at the
cost of 5 extra HashMap entries. Worth it for high-query workloads.

### Variant B: Cow<str> for service names

If service names are always static strings (known at compile time), change
`service: String` to `service: Cow<'static, str>` and store `Cow::Borrowed("auth-service")`
for known services. Eliminates the per-entry allocation for the service field. Overkill
for most use cases but instructive.

### Variant C: Error type instead of String

Replace `type Err = String` in `FromStr` impls with a proper enum:

```rust
#[derive(Debug)]
enum ParseError {
    MissingTimestamp,
    InvalidLevel(String),
    MissingServiceSeparator,
}
```

This lets callers pattern-match on the error type rather than parsing an error string.
The tradeoff: more boilerplate upfront, better ergonomics for callers.
