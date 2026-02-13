# Standard Exercise: Log Filter

## Scenario

You're building a log filtering library for a microservice observability stack. The service
receives structured log entries from multiple sources, each with a severity level, message,
origin, timestamp, and optional metadata. Operators need to filter logs by minimum severity,
search for keywords, and generate summaries — all without copying data unnecessarily.

## Brief

Implement a `LogLevel` enum, a `LogEntry` struct, and a set of functions that demonstrate
Rust's ownership model, enums with derived traits, pattern matching, and the distinction
between `String` (owned) and `&str` (borrowed).

## Acceptance Criteria

1. **`LogLevel` enum** with variants `Debug`, `Info`, `Warn`, `Error`
   - Derives `PartialEq`, `PartialOrd`, `Clone`, `Debug`
   - Ordering: `Debug < Info < Warn < Error` (variant declaration order)

2. **`LogEntry` struct** with fields:
   - `level: LogLevel`
   - `message: String`
   - `source: String`
   - `timestamp: u64`
   - `metadata: Option<String>`

3. **`LogEntry::new(level, message, source, timestamp)`** — constructor
   - Takes `&str` for message and source, converts to owned `String`
   - Sets `metadata` to `None`

4. **`LogEntry::with_metadata(self, metadata: &str)`** — builder method
   - Consumes `self` (move semantics), returns a new `LogEntry` with metadata set
   - Demonstrates the builder pattern with ownership transfer

5. **`LogEntry::matches_level(&self, min_level: &LogLevel)`** — level check
   - Returns `true` if the entry's level is >= the minimum level

6. **`LogEntry::contains(&self, keyword: &str)`** — keyword search
   - Returns `true` if `keyword` appears in the message OR in metadata (if present)
   - Case-sensitive

7. **`filter_logs(logs: &[LogEntry], min_level: &LogLevel)`** — filter by level
   - Borrows a slice of log entries, returns `Vec<&LogEntry>`
   - Only includes entries that match the minimum level

8. **`summarize(logs: &[LogEntry])`** — generate summary string
   - Returns a `String` with counts per level
   - Format: `"Debug: N, Info: N, Warn: N, Error: N"`

9. **`parse_level(s: &str)`** — parse a string into a LogLevel
   - Returns `Result<LogLevel, String>`
   - Case-insensitive: "debug", "DEBUG", "Debug" all work
   - Returns `Err` with a descriptive message for invalid input

## Constraints

- No external crates — stdlib only
- All tests in the starter file must pass against your implementation
- No `.unwrap()` in `parse_level` — use proper `Result` handling

## Hints

<details>
<summary>Hint 1: Enum ordering</summary>

When you `#[derive(PartialOrd)]`, Rust orders variants by their declaration order.
If you declare `Debug` first and `Error` last, then `Debug < Info < Warn < Error`
automatically. No need for a manual impl.

</details>

<details>
<summary>Hint 2: Builder pattern with move</summary>

`with_metadata(self, ...)` takes ownership of `self` (no `&`). This means the caller
cannot use the original `LogEntry` after calling this method — Rust enforces it at
compile time. Return a modified version of self:

```rust
fn with_metadata(mut self, metadata: &str) -> LogEntry {
    self.metadata = Some(metadata.to_string());
    self
}
```

</details>

<details>
<summary>Hint 3: Borrowing in filter_logs</summary>

`filter_logs` takes `&[LogEntry]` — a borrowed slice. It returns `Vec<&LogEntry>` —
a vec of references into the original slice. This means no cloning. Use `.iter().filter().collect()`.

</details>

<details>
<summary>Hint 4: Pattern matching in parse_level</summary>

Use `match` on `s.to_lowercase().as_str()`. The `.as_str()` converts the owned
`String` from `to_lowercase()` into a `&str` that you can match against string
literals.

</details>
