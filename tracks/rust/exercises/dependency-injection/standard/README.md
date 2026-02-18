# Metrics Collection Pipeline

## Scenario

Your team is building an internal observability platform. Application services push metrics (CPU usage, request latency, error counts) to a central collector. The collector processes metrics through a configurable pipeline -- normalizing values, filtering outliers -- then exports to one or more backends (stdout for dev, file for staging, remote API for production). The entire pipeline must be testable without real I/O.

## Brief

Implement a `MetricsPipeline` that is generic over three dependency traits:
- `MetricsSource` -- provides raw metrics
- `Processor` -- transforms/filters metrics
- `Exporter` -- writes processed metrics to a backend

The pipeline collects from the source, runs each metric through the processor, and exports the results.

## Acceptance Criteria

- [ ] Define `Metric` struct with fields: `name` (String), `value` (f64), `timestamp` (u64), `tags` (HashMap<String, String>)
- [ ] Define `MetricsSource` trait with `fn collect(&self) -> Vec<Metric>`
- [ ] Define `Processor` trait with `fn process(&self, metric: &Metric) -> Option<Metric>` (returns None to filter out)
- [ ] Define `Exporter` trait with `fn export(&self, metrics: &[Metric]) -> Result<usize, String>` (returns count exported)
- [ ] Implement `MetricsPipeline<S: MetricsSource, P: Processor, E: Exporter>` with:
  - `fn new(source: S, processor: P, exporter: E) -> Self`
  - `fn run(&self) -> Result<PipelineResult, String>` that collects, processes, and exports
- [ ] `PipelineResult` contains: `collected` (total from source), `processed` (after filtering), `exported` (confirmed exported)
- [ ] Implement `StdoutExporter` that prints metrics and returns count
- [ ] Implement `ThresholdProcessor` that filters out metrics below a configurable threshold
- [ ] Implement `StaticSource` that returns a pre-configured list of metrics (for testing and demos)
- [ ] All tests pass: `rustc --test main.rs && ./main`

## Constraints

- No external crates -- use only std library
- Each file must be independently compilable with `rustc`
- Processor should be composable: a `ChainProcessor` that runs metrics through multiple processors in sequence (bonus)
- The pipeline must report accurate counts at each stage

## Files

- `starter/main.rs` -- Scaffold with traits and struct definitions, TODOs for implementation
- `starter/tests.rs` -- Full test suite (copy into main.rs or compile separately)

## Getting Started

```bash
rustc --test starter/main.rs && ./main
# All tests will fail initially
```

## Hints

<details>
<summary>Hint 1: Structuring the pipeline</summary>

The `run()` method follows a simple flow:
1. `let raw = self.source.collect();`
2. Filter through processor: `raw.iter().filter_map(|m| self.processor.process(m)).collect()`
3. `let exported = self.exporter.export(&processed)?;`
4. Return counts

</details>

<details>
<summary>Hint 2: ThresholdProcessor</summary>

Store the threshold as a field. In `process()`, check if `metric.value >= self.threshold`. If so, return `Some(metric.clone())`. If not, return `None`.

</details>

<details>
<summary>Hint 3: ChainProcessor (bonus)</summary>

A `ChainProcessor` holds a `Vec<Box<dyn Processor>>`. Its `process()` method feeds the metric through each processor in sequence. If any processor returns `None`, the chain short-circuits and returns `None`.

Note: this requires the `Processor` trait to be object-safe.

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
