// Telemetry Aggregator — Debugging Exercise (Rust)
//
// This code has 4 bugs. Some cause compile errors, some cause runtime panics,
// some cause wrong results. Find and fix all of them.
//
// Run tests: rustc --test buggy.rs && ./buggy

use std::collections::HashMap;

/// A single metric sample from a telemetry agent.
#[derive(Debug, Clone)]
struct Sample {
    name: String,
    value: f64,
    agent_id: u32,
}

impl Sample {
    fn new(name: &str, value: f64, agent_id: u32) -> Sample {
        Sample {
            name: name.to_string(),
            value,
            agent_id,
        }
    }
}

// ----- Bug 1: [] indexing causes panic -----
//
// The function is supposed to return the highest value from a list of samples.
// For an empty input, it should return None. If there is at least one sample,
// it should return Some(highest_value).

fn aggregate(samples: &[Sample]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }

    // BUG: this panics when called with index out of bounds in test cases.
    // The test calls aggregate with a 3-element slice and asks for index 10.
    // Replace with safe access that can handle arbitrary lengths.
    let mut max = samples[10].value;
    for sample in samples {
        if sample.value > max {
            max = sample.value;
        }
    }
    Some(max)
}

// ----- Bug 2: Cannot iterate and mutate simultaneously -----
//
// This function should multiply every value in the Vec by the given factor.
// It does NOT compile. Fix it without removing the scaling behavior.

fn apply_multiplier(samples: &mut Vec<Sample>, factor: f64) {
    for sample in samples.iter() {          // BUG: immutable borrow while we try to mutate
        sample.value *= factor;
    }
}

// ----- Bug 3: Counting without entry API produces wrong counts -----
//
// This function should count how many samples came from each agent.
// The counts are consistently wrong — sometimes off, sometimes doubled.

fn build_index(samples: &[Sample]) -> HashMap<u32, u32> {
    let mut index: HashMap<u32, u32> = HashMap::new();

    for sample in samples {
        // BUG: This pattern checks for the key and then inserts, but the logic
        // is wrong — it always inserts 1 and never actually increments.
        if !index.contains_key(&sample.agent_id) {
            index.insert(sample.agent_id, 1);
        } else {
            index.insert(sample.agent_id, 1);  // BUG: should increment, not reset to 1
        }
    }

    index
}

// ----- Bug 4: collect into wrong type -----
//
// This function should return a Vec of (name, value) pairs for the top n
// samples sorted by value descending. It does NOT compile because collect()
// is being called without a type annotation and Rust cannot infer the target.
// Fix by adding the correct type.

fn top_metrics(samples: &[Sample], n: usize) -> Vec<(&str, f64)> {
    let mut pairs: Vec<(&str, f64)> = samples.iter()
        .map(|s| (s.name.as_str(), s.value))
        .collect();

    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    pairs.truncate(n);

    // BUG: this collects into the wrong type — it re-collects the already-built
    // Vec<(&str, f64)> into something Rust can't infer. Remove the spurious
    // collect() that wraps the result, or fix the type annotation.
    pairs.iter().collect()
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    use super::*;

    fn make_samples() -> Vec<Sample> {
        vec![
            Sample::new("cpu", 45.0, 1),
            Sample::new("cpu", 72.0, 2),
            Sample::new("memory", 38.0, 1),
            Sample::new("cpu", 91.0, 3),
            Sample::new("memory", 55.0, 2),
            Sample::new("memory", 80.0, 3),
        ]
    }

    // -- aggregate --

    #[test]
    fn test_aggregate_none_on_empty() {
        assert_eq!(aggregate(&[]), None);
    }

    #[test]
    fn test_aggregate_safe_access() {
        let samples = make_samples();
        // This should NOT panic — safe access regardless of index
        let result = aggregate(&samples);
        assert_eq!(result, Some(91.0));
    }

    // -- apply_multiplier --

    #[test]
    fn test_apply_multiplier() {
        let mut samples = vec![
            Sample::new("cpu", 50.0, 1),
            Sample::new("cpu", 80.0, 2),
        ];
        apply_multiplier(&mut samples, 2.0);
        assert_eq!(samples[0].value, 100.0);
        assert_eq!(samples[1].value, 160.0);
    }

    // -- build_index --

    #[test]
    fn test_build_index_counts() {
        let samples = make_samples();
        let index = build_index(&samples);

        // agent 1: cpu + memory = 2 samples
        assert_eq!(index[&1], 2, "agent 1 should have 2 samples");
        // agent 2: cpu + memory = 2 samples
        assert_eq!(index[&2], 2, "agent 2 should have 2 samples");
        // agent 3: cpu + memory = 2 samples
        assert_eq!(index[&3], 2, "agent 3 should have 2 samples");
    }

    // -- top_metrics --

    #[test]
    fn test_top_metrics_type() {
        let samples = make_samples();
        let top: Vec<(&str, f64)> = top_metrics(&samples, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("cpu", 91.0));
        assert_eq!(top[1], ("memory", 80.0));
    }
}

fn main() {
    println!("Telemetry Aggregator — run with: rustc --test buggy.rs && ./buggy");

    let samples = vec![
        Sample::new("cpu",    45.2, 1),
        Sample::new("memory", 72.8, 1),
        Sample::new("cpu",    91.5, 2),
    ];

    println!("max value: {:?}", aggregate(&samples));

    let index = build_index(&samples);
    println!("agent counts: {:?}", index);
}
