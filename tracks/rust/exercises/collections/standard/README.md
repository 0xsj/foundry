# Standard Exercise: Log Analyzer

## Scenario

Your team ingests structured log entries from a distributed system — dozens of services
emitting millions of lines per day. You need a log analyzer that can ingest raw entries,
compute frequency distributions by log level and source service, surface the most
frequent error messages, and produce sorted summary reports for dashboards.

## Brief

Implement a `LogEntry` struct, a `LogAnalyzer` struct with an ingestion pipeline, and
a set of analysis methods. The analyzer stores entries in a `Vec`, counts occurrences
using a `HashMap`, and produces sorted output using a `BTreeMap`.

## Acceptance Criteria

### 1. `LogEntry` struct

Fields:
- `level: &'static str` — one of `"DEBUG"`, `"INFO"`, `"WARN"`, `"ERROR"`
- `source: &'static str` — the emitting service name (e.g., `"auth-service"`)
- `message: String` — the log message
- `timestamp_ms: u64` — Unix timestamp in milliseconds

Constructor: `LogEntry::new(level, source, message: &str, timestamp_ms) -> LogEntry`

### 2. `LogAnalyzer` struct

Fields (all private):
- `entries: Vec<LogEntry>` — all ingested log entries in insertion order
- `level_counts: HashMap<&'static str, u32>` — running count per level
- `source_counts: HashMap<&'static str, u32>` — running count per source

Constructor: `LogAnalyzer::new() -> LogAnalyzer` (empty, all collections initialized)

### 3. `LogAnalyzer::ingest(&mut self, entry: LogEntry)`

- Appends the entry to `entries`
- Increments the count in `level_counts` for `entry.level`
- Increments the count in `source_counts` for `entry.source`
- Use the entry API — no `contains_key` + `insert` pattern

### 4. `LogAnalyzer::level_summary(&self) -> BTreeMap<&'static str, u32>`

- Returns a `BTreeMap` with the count for every level that appeared in the logs
- Keys are sorted alphabetically by level name
- Levels with zero entries should not appear in the output

### 5. `LogAnalyzer::top_sources(&self, n: usize) -> Vec<(&'static str, u32)>`

- Returns the top `n` sources by entry count, sorted by count descending
- Ties in count are broken by source name alphabetically ascending
- If there are fewer than `n` sources, return all of them

### 6. `LogAnalyzer::errors(&self) -> Vec<&LogEntry>`

- Returns references to all entries where `level == "ERROR"`, in insertion order
- No cloning — return `&LogEntry` references into the internal `Vec`

### 7. `LogAnalyzer::most_frequent_errors(&self, n: usize) -> Vec<(&str, u32)>`

- Counts how many times each distinct error message appears (among ERROR entries only)
- Returns the top `n` (message, count) pairs sorted by count descending
- Ties broken by message alphabetically ascending

### 8. `LogAnalyzer::sources_with_errors(&self) -> Vec<&'static str>`

- Returns a sorted list of source names that have at least one ERROR entry
- No duplicates — each source appears once even if it emitted multiple errors
- Hint: `HashSet` or `BTreeSet` can help

### 9. `LogAnalyzer::entries_in_range(&self, start_ms: u64, end_ms: u64) -> Vec<&LogEntry>`

- Returns references to all entries where `start_ms <= timestamp_ms <= end_ms`
- Preserves insertion order
- Both bounds are inclusive

## Constraints

- No external crates — stdlib only
- All tests in the starter file must pass
- Use the entry API in `ingest` — do not use `contains_key` + `insert`
- `errors()` and `entries_in_range()` must return references, not clones
- `sources_with_errors()` must not return duplicates

## Hints

<details>
<summary>Hint 1: Entry API in ingest</summary>

```rust
*self.level_counts.entry(entry.level).or_insert(0) += 1;
*self.source_counts.entry(entry.source).or_insert(0) += 1;
```

The `*` dereferences the `&mut u32` returned by `or_insert`. Without it, you'd be
incrementing the reference, not the value.

</details>

<details>
<summary>Hint 2: Converting HashMap to sorted Vec for top_sources</summary>

Collect the HashMap's entries into a Vec, then sort. Rust's `sort_by` takes a closure
that returns an `Ordering`:

```rust
let mut pairs: Vec<(&'static str, u32)> = self.source_counts.iter().map(|(&k, &v)| (k, v)).collect();
pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
pairs.truncate(n);
```

`then` chains a secondary sort: if counts are equal (`.cmp` returns `Equal`), fall back
to alphabetical source name.

</details>

<details>
<summary>Hint 3: most_frequent_errors — build a local HashMap</summary>

```rust
let mut error_counts: HashMap<&str, u32> = HashMap::new();
for entry in self.entries.iter().filter(|e| e.level == "ERROR") {
    *error_counts.entry(entry.message.as_str()).or_insert(0) += 1;
}
```

`entry.message.as_str()` converts `&String` to `&str` so you can use it as a HashMap key.
The lifetime is tied to `&self` so this compiles fine.

</details>

<details>
<summary>Hint 4: sources_with_errors without duplicates</summary>

Collect source names into a `BTreeSet` (automatically sorted, no duplicates), then
convert to `Vec`:

```rust
use std::collections::BTreeSet;
let unique: BTreeSet<&'static str> = self.entries.iter()
    .filter(|e| e.level == "ERROR")
    .map(|e| e.source)
    .collect();
unique.into_iter().collect()
```

</details>
