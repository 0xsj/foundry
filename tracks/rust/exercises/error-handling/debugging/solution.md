# Solution: Metric Aggregator Bugs

## Bug 1: `From` impl has the conversion backwards

### Root Cause

The `From` impl reads:

```rust
impl From<MetricError> for ParseFloatError {
    fn from(_: MetricError) -> ParseFloatError { ... }
}
```

This says: "a `ParseFloatError` can be constructed from a `MetricError`." That is the opposite of what `?` needs. `lookup_sample` does:

```rust
let value: f64 = raw.parse()?;
// parse() returns Result<f64, ParseFloatError>
// ? must convert ParseFloatError -> MetricError
// That requires: From<ParseFloatError> for MetricError
```

The error message is exact: `"the trait From<ParseFloatError> is not implemented for MetricError"`.

### Fix

There are two correct approaches:

**Option A: Fix the `From` direction with `map_err` for context**

`From<ParseFloatError>` for `MetricError` cannot carry `source_id` and `raw_value` context, because `From` only receives the error itself. Use `map_err` to add context, and remove the backwards impl:

```rust
// Remove the backwards impl entirely.

pub fn lookup_sample(
    samples: &HashMap<String, String>,
    source_id: &str,
) -> Result<f64, MetricError> {
    let raw = samples.get(source_id)
        .ok_or_else(|| MetricError::NotFound(source_id.to_string()))?;

    let value: f64 = raw.parse().map_err(|e: ParseFloatError| MetricError::ParseError {
        source_id: source_id.to_string(),
        raw_value: raw.clone(),
        cause: e,
    })?;

    Ok(value)
}
```

**Option B: Implement `From<ParseFloatError> for MetricError` correctly (loses context)**

```rust
impl From<ParseFloatError> for MetricError {
    fn from(e: ParseFloatError) -> Self {
        MetricError::ParseError {
            source_id: String::from("unknown"),
            raw_value: String::from("unknown"),
            cause: e,
        }
    }
}
```

This compiles but loses the `source_id` and `raw_value` context. Option A is better.

### Lesson

`From<A> for B` means "B can be created from A." When you see a `From` impl, read it as: "the type after `for` can be made from the type inside `From<...>`." The `?` operator needs `From<source_error_type>` for the function's return error type — not the other way around.

When a `From` impl cannot carry sufficient context (because `From` only receives the source error), use `map_err` instead. This is the common case for errors with structured context fields.

---

## Bug 2: `?` in a function returning `Vec<f64>`

### Root Cause

```rust
pub fn parse_all_samples(raw_samples: &[(&str, &str)]) -> Vec<f64> {
    raw_samples.iter()
        .map(...)
        .collect::<Result<Vec<f64>, _>>()?  // ERROR: ? requires Result return type
}
```

`?` desugars to `match result { Ok(v) => v, Err(e) => return Err(From::from(e)) }`. The `return Err(...)` arm requires the enclosing function to return `Result`. A function returning `Vec<f64>` cannot use `?`.

The caller's intent (from the docstring: "if any sample fails to parse, propagate the first error") requires a `Result` return type. The `Vec<f64>` return type was the bug — not the use of `?`.

### Fix

Change the return type to `Result<Vec<f64>, MetricError>`:

```rust
pub fn parse_all_samples(raw_samples: &[(&str, &str)]) -> Result<Vec<f64>, MetricError> {
    raw_samples.iter()
        .map(|(id, val)| {
            val.parse::<f64>().map_err(|e| MetricError::ParseError {
                source_id: id.to_string(),
                raw_value: val.to_string(),
                cause: e,
            })
        })
        .collect::<Result<Vec<f64>, _>>()
    // No ? needed — collect already produces Result<Vec<f64>, MetricError>
}
```

Note: the trailing `?` on `collect()` is now unnecessary — `collect()` itself returns `Result<Vec<f64>, MetricError>`, which is the function's return type. Remove it.

### Lesson

When the compiler says "`?` cannot be used in a function that returns `Vec<T>`," the solution is almost never "remove `?`." It is "change the return type to `Result`." Ask: what should the caller do when this fails? If the answer is "handle the error," the return type should be `Result`.

---

## Bug 3: `map_err` discards error context

### Root Cause

```rust
return Err(MetricError::ComputeError("cannot average empty slice".to_string()));
```

The error message does not include the metric name. When this error appears in a log, the operator sees `"computation error: cannot average empty slice"` and has no idea which metric caused the problem. The test explicitly checks `msg.contains("latency")`.

### Fix

Include the metric name in the error message:

```rust
return Err(MetricError::ComputeError(
    format!("cannot average empty slice for metric '{}'", name)
));
```

### Lesson

Error messages should answer three questions: **what** happened, **where** (which resource, key, metric), and ideally **why**. "Cannot average empty slice" answers "what" but not "where." Adding the metric name makes the error actionable.

This is the same principle as `anyhow`'s `.with_context()` — every layer of the call stack should add the context it knows that the layer below does not. `compute_average` knows the metric name; the caller does not always know which metric triggered the failure.

In Go, this would be:
```go
return fmt.Errorf("cannot average empty slice for metric %q", name)
```

The habit of including identifiers in error messages is language-agnostic.

---

## Bug 4: `unwrap()` panics instead of returning `Err`

### Root Cause

```rust
let mut file = std::fs::File::create(path).unwrap();
// ...
writeln!(file, ...).unwrap();
```

`.unwrap()` on an `Err` value panics. The function's return type is `Result<(), MetricError>`, so the caller cannot catch the panic — it propagates up the thread stack, potentially crashing the entire service.

The test `test_write_report_to_bad_path` expects `result.is_err()` but gets a panic instead.

### Fix

Replace `unwrap()` with `?`. The `From<std::io::Error>` impl for `MetricError` already exists, so `?` will convert automatically:

```rust
pub fn write_report(
    path: &str,
    metrics: &HashMap<String, Vec<f64>>,
) -> Result<(), MetricError> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;  // io::Error -> MetricError::Io

    for (name, values) in metrics {
        let avg = if values.is_empty() {
            "N/A".to_string()
        } else {
            let sum: f64 = values.iter().sum();
            format!("{:.2}", sum / values.len() as f64)
        };
        writeln!(file, "{}: {}", name, avg)?;  // io::Error -> MetricError::Io
    }

    Ok(())
}
```

### Lesson

`unwrap()` in a function that returns `Result` is almost always a bug. It converts a handleable `Err` into an unrecoverable panic. The caller wrote `write_report(...)` expecting to handle errors — `unwrap()` silently takes that ability away.

The only acceptable `unwrap()` in non-test code is on values that are logically infallible and where a panic truly represents a programming bug (not an operational failure). Writing to a file path is an operational failure — the path might not exist, permissions might be wrong, the disk might be full.

**Pattern to internalize:** whenever a function returns `Result<_, E>` where `E: From<io::Error>` (or has a matching `From` impl), use `?` instead of `.unwrap()`. If no `From` impl exists, use `map_err` to add context.

---

## Summary

| Bug | Root Cause | Category | Fix |
|-----|-----------|----------|-----|
| 1 — `lookup_sample` | `From` impl is backwards (`From<MetricError> for ParseFloatError` instead of vice versa) | Wrong `From` direction | Remove the backwards impl; `map_err` was already correct |
| 2 — `parse_all_samples` | Return type is `Vec<f64>`, but `?` requires `Result` | Wrong return type | Change return to `Result<Vec<f64>, MetricError>` |
| 3 — `compute_average` | Error message missing metric name | Context-free error | Include `name` in the `ComputeError` message |
| 4 — `write_report` | `unwrap()` panics instead of returning `Err` | `unwrap()` misuse | Replace both `unwrap()` calls with `?` |

## Related Pitfalls

- [[rust-from-impl-direction]] — From<A> for B vs From<B> for A
- [[rust-question-mark-return-type]] — ? requires Result or Option return type
- [[rust-context-free-errors]] — always include identifiers in error messages
- [[rust-unwrap-in-result-functions]] — unwrap() in Result-returning functions is a bug
