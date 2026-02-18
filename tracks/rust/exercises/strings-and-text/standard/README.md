# Standard Exercise: Structured Log Parser

## Scenario

Your team runs a distributed system where each service emits structured log lines in a
common text format. An ops tool needs to ingest these log lines, parse them into typed
structs, produce formatted reports, and look up entries efficiently. The tool must handle
log lines that may come from filenames with non-UTF-8 characters on some platforms, and
must avoid unnecessary allocations in the hot path where headers are forwarded unchanged.

## Brief

Implement a `LogLine` struct that models a single parsed log entry, a `LogParser` that
ingests raw lines, and a `ReportBuilder` that formats structured output. The `LogLine`
type must implement `Display` (user-facing format) and `FromStr` (parse from raw text).
The header forwarding utility must use `Cow<str>` to avoid allocating when no change is
needed.

## Acceptance Criteria

### 1. `LogLevel` enum

Variants: `Trace`, `Debug`, `Info`, `Warn`, `Error`

- `#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]`
- Implement `Display`: `Trace` → `"TRACE"`, etc.
- Implement `FromStr`: parse `"trace"`, `"TRACE"`, `"Trace"` (any case) → `Ok(LogLevel::Trace)`.
  Unknown strings → `Err(String)` with a message.

### 2. `LogLine` struct

Fields:
- `timestamp: String` — raw ISO 8601 timestamp string (e.g., `"2024-01-15T10:30:00Z"`)
- `level: LogLevel`
- `service: String` — emitting service name
- `message: String` — the log message body

Implement `Display`:
```
[ERROR] auth-service 2024-01-15T10:30:00Z: connection refused
```
Format: `[{level}] {service} {timestamp}: {message}`

Implement `FromStr` to parse this format:
```
2024-01-15T10:30:00Z [ERROR] auth-service: connection refused
```
Format: `{timestamp} [{level}] {service}: {message}`
Return `Err(String)` if the line does not match the expected structure.

### 3. `LogParser` struct

Fields (all private):
- `entries: Vec<LogLine>`
- `parse_errors: Vec<String>` — raw lines that failed to parse

Constructor: `LogParser::new() -> LogParser`

Methods:
- `ingest(&mut self, raw: &str)` — parses the line; on success pushes to `entries`, on
  failure pushes the raw line to `parse_errors`
- `ingest_many(&mut self, lines: &[&str])` — calls `ingest` for each line
- `entries(&self) -> &[LogLine]` — all successfully parsed entries
- `errors(&self) -> &[String]` — all raw lines that failed to parse
- `by_level(&self, level: LogLevel) -> Vec<&LogLine>` — all entries matching level,
  in insertion order, return references (no cloning)
- `by_service<'a>(&'a self, service: &str) -> Vec<&'a LogLine>` — all entries for a
  given service name

### 4. `sanitize_header<'a>(value: &'a str) -> Cow<'a, str>`

A free function (not a method). Returns:
- `Cow::Borrowed(value)` if the header contains no `\n` or `\r` characters
- `Cow::Owned(...)` with `\n` and `\r` replaced by a single space if any are present

The caller's code must be identical for both cases — they use `&*result` or pass the
result to any `&str` parameter.

### 5. `ReportBuilder` struct

Fields (all private):
- `parser: LogParser`

Constructor: `ReportBuilder::new(parser: LogParser) -> ReportBuilder`

Methods:
- `summary(&self) -> String` — a multi-line summary report formatted as:
  ```
  === Log Summary ===
  Total entries: 42
  Parse errors:  3

  By level:
    TRACE    0
    DEBUG    5
    INFO    22
    WARN    12
    ERROR    3

  Services seen: auth-service, api-gateway, db-pool
  ```
  Services are listed in alphabetical order, comma-separated.

- `format_entries(&self, entries: &[&LogLine]) -> String` — formats a slice of
  references as one `Display` line per entry, joined by `\n`.

## Constraints

- `sanitize_header` must return `Cow::Borrowed` (no allocation) when the input is clean.
  The test verifies this with `matches!(result, Cow::Borrowed(_))`.
- `by_level` and `by_service` must return references into `entries`, not clones.
- `format_entries` accepts `&[&LogLine]` — slices of references — not owned data.
- No external crates.
- All tests in `starter/main.rs` must pass.

## Hints

<details>
<summary>Hint 1: Implementing FromStr for LogLine</summary>

Use `split_once` to peel fields off the front of the string:

```rust
fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (timestamp, rest) = s.split_once(' ')
        .ok_or_else(|| format!("missing space after timestamp: {:?}", s))?;

    let rest = rest.strip_prefix('[')
        .ok_or_else(|| "expected '[' after timestamp".to_string())?;
    let (level_str, rest) = rest.split_once(']')
        .ok_or_else(|| "expected ']' after level".to_string())?;
    let rest = rest.strip_prefix(' ')
        .ok_or_else(|| "expected space after ']'".to_string())?;

    let (service, message) = rest.split_once(": ")
        .ok_or_else(|| format!("expected ': ' separator: {:?}", rest))?;

    let level = level_str.parse::<LogLevel>()?;

    Ok(LogLine {
        timestamp: timestamp.to_string(),
        level,
        service:   service.to_string(),
        message:   message.to_string(),
    })
}
```

`split_once` returns `Option<(&str, &str)>` — pair of slices into the original string.
`ok_or_else` converts `None` to an `Err(String)`.

</details>

<details>
<summary>Hint 2: Cow::Borrowed vs Cow::Owned</summary>

```rust
use std::borrow::Cow;

fn sanitize_header<'a>(value: &'a str) -> Cow<'a, str> {
    if value.contains('\n') || value.contains('\r') {
        Cow::Owned(value.replace(['\n', '\r'], " "))
    } else {
        Cow::Borrowed(value)
    }
}
```

`Cow<'a, str>` has lifetime `'a` tied to the input `&'a str`. When `Borrowed`, the
returned reference has the same lifetime as the input. When `Owned`, the `String` is
independent and has no borrowed lifetime.

</details>

<details>
<summary>Hint 3: by_level returning references</summary>

```rust
fn by_level(&self, level: LogLevel) -> Vec<&LogLine> {
    self.entries.iter()
        .filter(|e| e.level == level)
        .collect()
}
```

`self.entries.iter()` yields `&LogLine` references. After filter, `collect()` collects
them into `Vec<&LogLine>`. The lifetime of the references is tied to `&self`.

</details>

<details>
<summary>Hint 4: Building the summary string</summary>

Use `write!` / `writeln!` into a `String` buffer to avoid one-allocation-per-line:

```rust
use std::fmt::Write;

fn summary(&self) -> String {
    let mut out = String::with_capacity(256);
    writeln!(out, "=== Log Summary ===").unwrap();
    // ...
    out
}
```

For the services list, collect service names into a `BTreeSet` (auto-sorted, no
duplicates), then join:

```rust
use std::collections::BTreeSet;
let services: BTreeSet<&str> = self.parser.entries.iter()
    .map(|e| e.service.as_str())
    .collect();
let services_str = services.into_iter().collect::<Vec<_>>().join(", ");
```

</details>
