# Expert Review: CSV Sales Report Pipeline

## Critical Issues

### 1. `run_pipeline` aborts on the first file error — wrong for a batch job

**Location:** `run_pipeline`, the `Err(e) => { return Err(e); }` branch

```rust
match load_csv_file(&path_str) {
    Ok(records) => all_records.extend(records),
    Err(e) => {
        eprintln!("error processing {}: {}", path_str, e);
        return Err(e);  // abort on first file error
    }
}
```

**Problem:** The PR description says the pipeline "processes daily sales reports" and "errors should be logged and the job should continue processing other files if possible." But the implementation aborts on the first bad file. If one of 50 files has a single bad CSV row, none of the other 49 files are processed. This is exactly the opposite of the stated requirement.

**Fix:** Collect file-level errors and continue processing. Return a result that includes both the successes and the failures:

```rust
pub fn run_pipeline(data_dir: &str) -> Result<PipelineResult, String> {
    // ...
    let mut file_errors: Vec<String> = Vec::new();

    for entry in entries {
        // ...
        match load_csv_file(&path_str) {
            Ok(records) => all_records.extend(records),
            Err(e) => {
                eprintln!("skipping {}: {}", path_str, e);
                file_errors.push(format!("{}: {}", path_str, e));
                // continue — don't abort
            }
        }
    }

    Ok(PipelineResult {
        summaries: aggregate_by_region(&all_records),
        errors: file_errors,
    })
}
```

The caller can then decide whether partial results with errors are acceptable for its use case (e.g., alert if more than 10% of files failed).

**Concept:** `?` is a blunt instrument for orchestration code. Sometimes you want to continue on error and collect failures — this is a different pattern from propagation. The choice between "stop on first error" and "collect errors and continue" is a product decision that must match the stated requirements.

---

### 2. `String` as the error type — callers cannot match on error variants

**Location:** All function signatures — `parse_record`, `parse_csv`, `load_csv_file`, `run_pipeline`

```rust
pub fn parse_record(line: &str, line_num: usize) -> Result<SalesRecord, String>
pub fn parse_csv(content: &str, filename: &str) -> Result<Vec<SalesRecord>, String>
pub fn load_csv_file(path: &str) -> Result<Vec<SalesRecord>, String>
pub fn run_pipeline(data_dir: &str) -> Result<HashMap<String, RegionSummary>, String>
```

**Problem:** Using `String` as the error type means callers cannot programmatically distinguish between error categories. If downstream code needs to retry on I/O errors but not on parse errors (a common pattern), it cannot — both are opaque `String` values.

Additionally, string comparison to determine error type is brittle and untestable. The test `test_parse_csv_propagates_filename` checks `err.contains("sales_2026_01.csv")` — this will silently break if the error message format changes.

**Fix:** Define a structured error enum:

```rust
#[derive(Debug)]
pub enum PipelineError {
    Io { path: String, source: std::io::Error },
    Parse { path: String, line: usize, message: String },
}
```

Then tests can assert on variants:
```rust
assert!(matches!(err, PipelineError::Parse { line: 2, .. }));
```

And callers can branch on error type:
```rust
match run_pipeline(dir) {
    Err(PipelineError::Io { .. }) => retry(),
    Err(PipelineError::Parse { path, .. }) => alert_data_team(path),
    Ok(_) => {}
}
```

**When `String` errors are acceptable:** Rapid prototyping, `main()` functions, or internal code that will never be called by external code. Once a function is part of a library or called from multiple places, structured errors pay for themselves.

**Concept:** `String` as an error type is a code smell in library-style code. It provides human-readable messages at the cost of machine-readable structure. Use `thiserror` to get both.

---

## Major Concerns

### 3. `parse_csv` short-circuits on the first bad row — partial data may be preferable

**Location:** `parse_csv`, the `map_err(...)?.` chain

```rust
let record = parse_record(line, idx + 1).map_err(|e| {
    format!("error in file '{}': {}", filename, e)
})?;  // returns on first parse error
```

**Problem:** If a 10,000-row file has one bad row at line 5000, the entire file is rejected. The `run_pipeline` level will then log the error and (after the fix from issue #1) skip the file entirely. The business may prefer "process 9,999 good rows and log the 1 bad one."

**Fix:** Change `parse_csv` to return both the good records and a list of row-level errors:

```rust
pub fn parse_csv(content: &str, filename: &str)
    -> (Vec<SalesRecord>, Vec<String>)
{
    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() { continue; }
        match parse_record(line.trim(), idx + 1) {
            Ok(r) => records.push(r),
            Err(e) => errors.push(format!("file '{}': {}", filename, e)),
        }
    }

    (records, errors)
}
```

Whether to skip individual rows vs abort the file is again a product decision — but the current behavior of aborting an entire file on one bad row is almost never right for a batch data pipeline.

---

### 4. `map_err(|e| e.to_string())` discards the error type

**Location:** `load_csv_file`, `run_pipeline`

```rust
let content = std::fs::read_to_string(path)
    .map_err(|e| e.to_string())?;
```

**Problem:** `e.to_string()` converts `io::Error` to a `String`. This discards `io::ErrorKind`, which the caller might need (e.g., to distinguish `NotFound` from `PermissionDenied`). Once converted to `String`, the distinction is gone — the caller would have to do unreliable string matching.

This is consistent with the `String` error type problem (issue #2), but worth calling out explicitly: `e.to_string()` is a lossy conversion. It is never the right way to convert errors unless you are genuinely at the end of the error handling chain (e.g., logging to a file).

**Fix:** Part of the structured error type fix. With `PipelineError::Io { path, source }`, the `io::Error` is preserved:

```rust
std::fs::read_to_string(path).map_err(|e| PipelineError::Io {
    path: path.to_string(),
    source: e,
})?
```

---

### 5. `print_report` uses `.unwrap()` on a float comparison

**Location:** `print_report`

```rust
regions.sort_by(|a, b| b.total_sales.partial_cmp(&a.total_sales).unwrap());
```

**Problem:** `f64::partial_cmp` returns `None` when either value is `NaN`. `.unwrap()` on `None` panics. If any `total_sales` is `NaN` (possible if `unit_price` or `quantity` fields in the CSV are NaN-producing values), this sort will panic in production.

**Fix:** Use `total_cmp` (stable since Rust 1.62) which provides a total ordering over floats and handles NaN consistently:

```rust
regions.sort_by(|a, b| b.total_sales.total_cmp(&a.total_sales));
```

Or use `unwrap_or` with a fallback ordering:
```rust
regions.sort_by(|a, b| {
    b.total_sales.partial_cmp(&a.total_sales)
        .unwrap_or(std::cmp::Ordering::Equal)
});
```

**Concept:** Floating point operations on user-provided data can produce NaN. Any sort, comparison, or arithmetic involving `f64` from external input should handle the NaN case. `partial_cmp().unwrap()` on floats is a latent panic.

---

## Minor Suggestions

### 6. `parse_csv` skips only the first line as a header — fragile

```rust
for (idx, line) in content.lines().enumerate() {
    if idx == 0 { continue; }
```

This skips the first line unconditionally. If the CSV has no header, the first data row is lost. If the CSV has two header rows (some Excel exports do this), the second header row becomes a data row and causes a parse error.

**Better:** Check that the first line matches expected column names, and return a clear error if not:

```rust
if idx == 0 {
    if !line.to_lowercase().contains("region") {
        return Err(format!("file '{}': unexpected header '{}'", filename, line));
    }
    continue;
}
```

---

### 7. Test assertions check string content — fragile

```rust
assert!(err.contains("line 3"), ...);
assert!(err.contains("quantity"), ...);
assert!(err.contains("sales_2026_01.csv"), ...);
```

String-contains assertions are brittle. They'll fail if the error message wording changes but the behavior is still correct. With structured error types:

```rust
assert!(matches!(err, PipelineError::Parse { line: 3, .. }));
```

This is exact and refactoring-safe.

---

## Positive Feedback

1. **Error messages include line numbers and field names.** `"line 3: invalid quantity 'abc'"` is immediately actionable. Many engineers write errors like `"parse failed"` — this PR did better.

2. **`map_err` is used to add context at the parse layer.** `parse_csv` wraps `parse_record`'s error with the filename. The principle of "add context as errors propagate up" is applied correctly, even if the error type should be structured rather than `String`.

3. **Header row is skipped.** A small detail but easy to forget.

4. **`aggregate_by_region` returns a `HashMap` without `Result` — correct.** Aggregation itself cannot fail on valid data; it should not pretend to be fallible. Keeping infallible functions free of `Result` is good hygiene.

5. **Tests cover the basic cases.** The tests verify parse errors and filename propagation. A good start even if assertions could be more precise.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | Pipeline aborts on first file error — contradicts batch-job requirements | Error handling strategy: fail-fast vs collect-and-continue |
| 2 | Critical | `String` error type prevents callers from matching on error variants | Structured errors with `thiserror` |
| 3 | Major | `parse_csv` aborts on first bad row — entire file rejected for one bad line | Partial success / error collection |
| 4 | Major | `map_err(|e| e.to_string())` discards error type information | Lossy error conversion |
| 5 | Major | `partial_cmp().unwrap()` panics on NaN | Float comparison safety |
| 6 | Minor | Header detection is fragile | Defensive parsing |
| 7 | Minor | String-contains test assertions break on message changes | Structured error assertions |

## Related Concepts

- [[fundamentals/rust/error-handling]] — `String` vs structured errors, `thiserror`
- [[fundamentals/error-handling]] — Cross-language comparison of error handling
- [[pitfalls/rust-string-error-type]] — When String errors are and aren't appropriate
- [[pitfalls/rust-question-mark-fail-fast]] — `?` is fail-fast; not always what batch jobs need
- [[pitfalls/rust-partial-cmp-unwrap]] — Float comparison panics on NaN
