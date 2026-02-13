# Exercise: Metrics Collector

## Scenario

You're building a lightweight metrics collector for a monitoring service. The collector receives named metrics from various subsystems — CPU counters, memory gauges, request latency histograms — and needs to store them in a type-safe way. Each metric has a typed value (counter, gauge, or histogram), optional tags for filtering, and a timestamp. The system needs to support querying and basic aggregation without losing type information.

## Brief

Implement a metrics collector that stores typed metric entries and provides query/aggregation operations. The core challenge is modeling the different metric types using Zig's tagged unions while keeping the API clean and type-safe.

## Acceptance Criteria

- [ ] `MetricType` is a tagged union with three variants:
  - `counter` holding a `u64`
  - `gauge` holding an `f64`
  - `histogram` holding a bounded array of `f64` observations (up to 64 entries) plus a count
- [ ] `Metric` struct has fields: `name` (`[]const u8`), `value` (`MetricType`), `tags` (optional `[]const u8` for a comma-separated tag string), `timestamp` (`u64`)
- [ ] `createCounter(name, value, timestamp)` returns a `Metric` with a counter value and null tags
- [ ] `createGauge(name, value, timestamp)` returns a `Metric` with a gauge value and null tags
- [ ] `increment(metric, amount)` returns a new `Metric` with the counter incremented, or `error.NotACounter` if the metric is not a counter type
- [ ] `getNumericValue(metric)` returns `?f64` — extracts the numeric value from a counter (cast to f64) or gauge, returns `null` for histograms
- [ ] `isAboveThreshold(metric, threshold)` returns `?bool` — `true`/`false` for numeric metrics, `null` for histograms
- [ ] `formatMetric(metric, buffer)` writes a formatted string representation into the provided buffer slice, returns the written slice

## Constraints

- Use only the standard library
- No heap allocation — all data lives on the stack or in fixed-size buffers
- Histogram observations use a fixed-size array (`[64]f64`) with a separate count field
- All numeric conversions must be explicit (`@floatFromInt`, `@intFromFloat`, `@as`)
- The `increment` function must return an error union, not panic

## Concepts Exercised

- `const` vs `var` declarations
- Struct definition with default fields
- Tagged unions (`union(enum)`) for type-safe variants
- Optionals (`?T`) for nullable fields
- Explicit type casting (`@floatFromInt`, `@as`)
- Error unions for fallible operations
- Switch expressions on tagged unions
- Slice and buffer operations
- Value semantics (returning new structs, not mutating)

## Hints

<details>
<summary>Hint 1: Modeling the histogram</summary>

Use a fixed-size array with a separate count to track how many observations have been added:

```zig
const Histogram = struct {
    observations: [64]f64,
    count: usize,
};
```

Then the tagged union variant holds this struct.
</details>

<details>
<summary>Hint 2: Error unions for increment</summary>

Zig error unions let you return either a value or an error:

```zig
const MetricError = error{NotACounter};

fn increment(metric: Metric, amount: u64) MetricError!Metric {
    switch (metric.value) {
        .counter => |c| { ... },
        else => return error.NotACounter,
    }
}
```
</details>

<details>
<summary>Hint 3: Formatting into a buffer</summary>

Use `std.fmt.bufPrint` to write formatted text into a caller-provided buffer:

```zig
fn formatMetric(metric: Metric, buf: []u8) ![]u8 {
    return std.fmt.bufPrint(buf, "...", .{ ... });
}
```

The `![]u8` return type means it returns a slice on success or an error if the buffer is too small.
</details>

<details>
<summary>Hint 4: Converting counter u64 to f64</summary>

Zig requires explicit conversion between integer and float types:

```zig
const float_val: f64 = @floatFromInt(counter_value);
```
</details>
