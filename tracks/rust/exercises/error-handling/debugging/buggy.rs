// Metric Aggregator — Debugging Exercise
//
// There are 4 bugs — one per function.
// Compile with: rustc --test buggy.rs
// Read each compiler error carefully before jumping to a fix.

use std::collections::HashMap;
use std::fmt;
use std::num::ParseFloatError;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum MetricError {
    /// A sample value could not be parsed as a float.
    ParseError { source_id: String, raw_value: String, cause: ParseFloatError },
    /// A named metric was requested but does not exist.
    NotFound(String),
    /// A computation failed (e.g., average of an empty slice).
    ComputeError(String),
    /// Writing the report failed.
    Io(std::io::Error),
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricError::ParseError { source_id, raw_value, cause } => write!(
                f,
                "could not parse sample '{}' from source '{}': {}",
                raw_value, source_id, cause
            ),
            MetricError::NotFound(name)  => write!(f, "metric '{}' not found", name),
            MetricError::ComputeError(s) => write!(f, "computation error: {}", s),
            MetricError::Io(e)           => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for MetricError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MetricError::ParseError { cause, .. } => Some(cause),
            MetricError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MetricError {
    fn from(e: std::io::Error) -> Self { MetricError::Io(e) }
}

// ---- Bug 1: From impl exists but ? still fails ----
//
// The author wants ? to automatically convert ParseFloatError -> MetricError.
// They wrote a From impl, but something is wrong. The compiler reports:
// "the trait `From<ParseFloatError>` is not implemented for `MetricError`"
// even though there's an impl below.
//
// Look carefully at the impl direction.
// Hint: `From<T> for U` means "U can be created from T".
// Which direction does this impl go?

impl From<MetricError> for ParseFloatError {
    // BUG: the conversion is backwards — this implements:
    //   ParseFloatError can be created from MetricError
    // But ? needs:
    //   MetricError can be created from ParseFloatError
    // i.e., From<ParseFloatError> for MetricError
    fn from(_: MetricError) -> ParseFloatError {
        "0.0".parse::<f64>().unwrap_err()
    }
}

/// Look up a raw sample by source ID and parse it as f64.
pub fn lookup_sample(
    samples: &HashMap<String, String>,
    source_id: &str,
) -> Result<f64, MetricError> {
    let raw = samples.get(source_id)
        .ok_or_else(|| MetricError::NotFound(source_id.to_string()))?;

    // This bare ? relies on From<ParseFloatError> for MetricError.
    // The parse() call returns Result<f64, ParseFloatError>.
    // ? must convert ParseFloatError -> MetricError using From.
    // But the From impl above is backwards, so this fails to compile.
    let value: f64 = raw.parse()?;

    Ok(value)
}

// ---- Bug 2: ? in a function that returns Vec<f64>, not Result ----
//
// The author forgot that ? requires the enclosing function to return Result or Option.
// The fix is NOT to remove ? — it's to change the return type so ? is valid.
// Think about what the caller actually needs from this function.

/// Parse all sample strings and return the successfully parsed values.
///
/// The caller wants: if any sample fails to parse, propagate the first error.
pub fn parse_all_samples(raw_samples: &[(&str, &str)]) -> Vec<f64> {
    // BUG: ? cannot be used here — this function returns Vec<f64>, not Result<Vec<f64>, _>
    raw_samples
        .iter()
        .map(|(id, val)| {
            val.parse::<f64>().map_err(|e| MetricError::ParseError {
                source_id: id.to_string(),
                raw_value: val.to_string(),
                cause: e,
            })
        })
        .collect::<Result<Vec<f64>, _>>()?  // ? needs Result return type
}

// ---- Bug 3: map_err discards the original error information ----
//
// This function compiles and runs without crashing, but the test
// test_average_preserves_error_info fails because the error message does not
// include the metric name. The author's map_err replaces the whole error with
// a generic string, losing context.
//
// Fix: add the metric name to the error message without discarding the inner
// error's message. Do not change the return type or the ComputeError variant.

/// Compute the average of a named metric's values.
pub fn compute_average(
    metrics: &HashMap<String, Vec<f64>>,
    name: &str,
) -> Result<f64, MetricError> {
    let values = metrics
        .get(name)
        .ok_or_else(|| MetricError::NotFound(name.to_string()))?;

    if values.is_empty() {
        // BUG: the error message says "cannot average empty slice" but does NOT
        // include the metric name. When this error appears in logs, the operator
        // has no idea which metric caused the problem.
        return Err(MetricError::ComputeError("cannot average empty slice".to_string()));
    }

    let sum: f64 = values.iter().sum();
    Ok(sum / values.len() as f64)
}

// ---- Bug 4: unwrap() instead of ? — panic instead of Err ----
//
// write_report should return Result<(), MetricError> and propagate any I/O
// error as an Err. Instead, it uses unwrap() — so when writing fails (e.g.,
// a bad path), the whole process panics instead of returning an error to the
// caller.
//
// Fix: replace the unwrap() calls with ? so errors are propagated correctly.
// The function signature is already correct.

/// Write a metrics report to a file.
pub fn write_report(
    path: &str,
    metrics: &HashMap<String, Vec<f64>>,
) -> Result<(), MetricError> {
    use std::io::Write;

    // BUG: if this fails (bad path, permissions, etc.), it panics instead of returning Err
    let mut file = std::fs::File::create(path).unwrap();

    for (name, values) in metrics {
        let avg = if values.is_empty() {
            "N/A".to_string()
        } else {
            let sum: f64 = values.iter().sum();
            format!("{:.2}", sum / values.len() as f64)
        };
        // BUG: same issue — panics on I/O error instead of returning Err
        writeln!(file, "{}: {}", name, avg).unwrap();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn test_lookup_sample_valid() {
        let samples = sample_map(&[("cpu", "0.75"), ("mem", "0.42")]);
        assert!((lookup_sample(&samples, "cpu").unwrap() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_lookup_sample_not_found() {
        let samples = sample_map(&[("cpu", "0.75")]);
        assert!(matches!(
            lookup_sample(&samples, "disk").unwrap_err(),
            MetricError::NotFound(_)
        ));
    }

    #[test]
    fn test_lookup_sample_parse_error() {
        let samples = sample_map(&[("cpu", "not_a_float")]);
        assert!(matches!(
            lookup_sample(&samples, "cpu").unwrap_err(),
            MetricError::ParseError { .. }
        ));
    }

    #[test]
    fn test_parse_all_samples_valid() {
        let raw = vec![("a", "1.0"), ("b", "2.5"), ("c", "3.0")];
        let result = parse_all_samples(&raw).expect("all valid");
        assert_eq!(result, vec![1.0, 2.5, 3.0]);
    }

    #[test]
    fn test_parse_all_samples_error() {
        let raw = vec![("a", "1.0"), ("b", "not_a_float")];
        assert!(matches!(
            parse_all_samples(&raw).unwrap_err(),
            MetricError::ParseError { .. }
        ));
    }

    #[test]
    fn test_average_valid() {
        let mut metrics = HashMap::new();
        metrics.insert("latency".to_string(), vec![10.0, 20.0, 30.0]);
        let avg = compute_average(&metrics, "latency").unwrap();
        assert!((avg - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_average_preserves_error_info() {
        let mut metrics = HashMap::new();
        metrics.insert("latency".to_string(), vec![]);
        let err = compute_average(&metrics, "latency").unwrap_err();
        let msg = err.to_string();
        // The error message must identify WHICH metric is empty
        assert!(
            msg.contains("latency"),
            "error message should mention metric name 'latency', got: '{}'",
            msg
        );
    }

    #[test]
    fn test_write_report_to_bad_path() {
        let metrics = HashMap::new();
        // This path cannot be created — write_report must return Err, not panic
        let result = write_report("/nonexistent/deeply/nested/path/report.txt", &metrics);
        assert!(
            result.is_err(),
            "expected Err for unwritable path, got Ok"
        );
    }
}

fn main() {
    println!("Fix the bugs, then run: rustc --test buggy.rs && ./buggy");
}
