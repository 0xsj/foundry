const std = @import("std");
const testing = std.testing;

// =============================================================================
// Metrics Collector — Starter
// =============================================================================
// Build a type-safe metrics collector using Zig's tagged unions, optionals,
// and explicit casting. Fill in the TODO sections below.
//
// Run tests with: zig test main.zig
// =============================================================================

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/// A fixed-capacity histogram that stores up to 64 observations.
pub const Histogram = struct {
    observations: [64]f64,
    count: usize,

    pub fn init() Histogram {
        return .{
            .observations = [_]f64{0.0} ** 64,
            .count = 0,
        };
    }

    /// Add an observation. Returns error if at capacity.
    pub fn observe(self: Histogram, value: f64) error{HistogramFull}!Histogram {
        if (self.count >= 64) return error.HistogramFull;
        var new = self;
        new.observations[self.count] = value;
        new.count = self.count + 1;
        return new;
    }
};

/// Tagged union representing the three metric types.
/// - counter: monotonically increasing integer (e.g., total requests)
/// - gauge: point-in-time float value (e.g., current CPU usage)
/// - histogram: distribution of observed values (e.g., request latencies)
pub const MetricType = union(enum) {
    counter: u64,
    gauge: f64,
    histogram: Histogram,
};

/// A single metric data point.
pub const Metric = struct {
    name: []const u8,
    value: MetricType,
    tags: ?[]const u8, // optional comma-separated tags, e.g. "env:prod,region:us"
    timestamp: u64, // unix epoch in seconds
};

/// Errors that metric operations can produce.
pub const MetricError = error{NotACounter};

// -----------------------------------------------------------------------------
// Functions — implement these
// -----------------------------------------------------------------------------

/// Create a counter metric with the given name, value, and timestamp.
/// Tags default to null.
pub fn createCounter(name: []const u8, value: u64, timestamp: u64) Metric {
    // TODO: return a Metric with a counter MetricType and null tags
    _ = name;
    _ = value;
    _ = timestamp;
    unreachable;
}

/// Create a gauge metric with the given name, value, and timestamp.
/// Tags default to null.
pub fn createGauge(name: []const u8, value: f64, timestamp: u64) Metric {
    // TODO: return a Metric with a gauge MetricType and null tags
    _ = name;
    _ = value;
    _ = timestamp;
    unreachable;
}

/// Increment a counter metric by the given amount.
/// Returns a new Metric with the updated value (value semantics).
/// Returns error.NotACounter if the metric is not a counter.
pub fn increment(metric: Metric, amount: u64) MetricError!Metric {
    // TODO: switch on metric.value
    //   - if counter, return a new Metric with counter + amount
    //   - otherwise, return error.NotACounter
    _ = metric;
    _ = amount;
    unreachable;
}

/// Extract the numeric value from a metric as f64.
/// - counter: cast u64 to f64
/// - gauge: return the f64 directly
/// - histogram: return null (no single numeric value)
pub fn getNumericValue(metric: Metric) ?f64 {
    // TODO: switch on metric.value and return the appropriate f64 or null
    _ = metric;
    unreachable;
}

/// Check if a metric's numeric value exceeds the given threshold.
/// Returns null for histogram metrics (no single numeric value).
pub fn isAboveThreshold(metric: Metric, threshold: f64) ?bool {
    // TODO: use getNumericValue, then compare with threshold
    //   - if getNumericValue returns null, return null
    //   - otherwise return whether the value > threshold
    _ = metric;
    _ = threshold;
    unreachable;
}

/// Write a human-readable representation of the metric into the buffer.
/// Returns the slice of buf that was written to.
///
/// Format:
///   counter:  "<name> counter=<value> ts=<timestamp>"
///   gauge:    "<name> gauge=<value> ts=<timestamp>"
///   histogram: "<name> histogram count=<count> ts=<timestamp>"
///
/// If tags are present, append " [<tags>]" at the end.
pub fn formatMetric(metric: Metric, buf: []u8) ![]u8 {
    // TODO: use std.fmt.bufPrint to write into buf
    //   1. switch on metric.value to get the type-specific part
    //   2. if tags are non-null, append them
    //   3. return the written slice
    _ = metric;
    _ = buf;
    unreachable;
}

// =============================================================================
// Tests
// =============================================================================

test "createCounter returns a counter metric" {
    const m = createCounter("http.requests", 42, 1700000000);
    try testing.expectEqualStrings("http.requests", m.name);
    try testing.expectEqual(@as(u64, 42), m.value.counter);
    try testing.expectEqual(@as(u64, 1700000000), m.timestamp);
    try testing.expect(m.tags == null);
}

test "createGauge returns a gauge metric" {
    const m = createGauge("cpu.usage", 73.5, 1700000000);
    try testing.expectEqualStrings("cpu.usage", m.name);
    try testing.expectApproxEqAbs(@as(f64, 73.5), m.value.gauge, 0.001);
    try testing.expectEqual(@as(u64, 1700000000), m.timestamp);
    try testing.expect(m.tags == null);
}

test "createCounter with zero value" {
    const m = createCounter("cache.misses", 0, 1700000000);
    try testing.expectEqual(@as(u64, 0), m.value.counter);
}

test "createGauge with zero value" {
    const m = createGauge("mem.free", 0.0, 1700000000);
    try testing.expectApproxEqAbs(@as(f64, 0.0), m.value.gauge, 0.001);
}

test "createGauge with negative value" {
    const m = createGauge("temperature.delta", -12.5, 1700000000);
    try testing.expectApproxEqAbs(@as(f64, -12.5), m.value.gauge, 0.001);
}

test "increment counter succeeds" {
    const m = createCounter("http.requests", 10, 1700000000);
    const incremented = try increment(m, 5);
    try testing.expectEqual(@as(u64, 15), incremented.value.counter);
    // name and timestamp should be preserved
    try testing.expectEqualStrings("http.requests", incremented.name);
    try testing.expectEqual(@as(u64, 1700000000), incremented.timestamp);
}

test "increment counter from zero" {
    const m = createCounter("new.counter", 0, 1700000000);
    const incremented = try increment(m, 1);
    try testing.expectEqual(@as(u64, 1), incremented.value.counter);
}

test "increment gauge returns error" {
    const m = createGauge("cpu.usage", 50.0, 1700000000);
    const result = increment(m, 1);
    try testing.expectError(error.NotACounter, result);
}

test "increment histogram returns error" {
    const m = Metric{
        .name = "latency",
        .value = .{ .histogram = Histogram.init() },
        .tags = null,
        .timestamp = 1700000000,
    };
    const result = increment(m, 1);
    try testing.expectError(error.NotACounter, result);
}

test "getNumericValue for counter" {
    const m = createCounter("req.count", 100, 1700000000);
    const val = getNumericValue(m);
    try testing.expect(val != null);
    try testing.expectApproxEqAbs(@as(f64, 100.0), val.?, 0.001);
}

test "getNumericValue for gauge" {
    const m = createGauge("cpu.temp", 65.3, 1700000000);
    const val = getNumericValue(m);
    try testing.expect(val != null);
    try testing.expectApproxEqAbs(@as(f64, 65.3), val.?, 0.001);
}

test "getNumericValue for histogram returns null" {
    const m = Metric{
        .name = "latency",
        .value = .{ .histogram = Histogram.init() },
        .tags = null,
        .timestamp = 1700000000,
    };
    try testing.expect(getNumericValue(m) == null);
}

test "getNumericValue counter casts u64 to f64" {
    // Large counter value — tests that the cast from u64 to f64 works
    const m = createCounter("bytes.sent", 1_000_000_000, 1700000000);
    const val = getNumericValue(m);
    try testing.expect(val != null);
    try testing.expectApproxEqAbs(@as(f64, 1_000_000_000.0), val.?, 1.0);
}

test "isAboveThreshold for counter above" {
    const m = createCounter("errors", 50, 1700000000);
    const result = isAboveThreshold(m, 25.0);
    try testing.expect(result != null);
    try testing.expect(result.? == true);
}

test "isAboveThreshold for counter below" {
    const m = createCounter("errors", 5, 1700000000);
    const result = isAboveThreshold(m, 25.0);
    try testing.expect(result != null);
    try testing.expect(result.? == false);
}

test "isAboveThreshold for gauge" {
    const m = createGauge("cpu.usage", 95.2, 1700000000);
    const result = isAboveThreshold(m, 90.0);
    try testing.expect(result != null);
    try testing.expect(result.? == true);
}

test "isAboveThreshold for histogram returns null" {
    const m = Metric{
        .name = "latency",
        .value = .{ .histogram = Histogram.init() },
        .tags = null,
        .timestamp = 1700000000,
    };
    try testing.expect(isAboveThreshold(m, 10.0) == null);
}

test "isAboveThreshold at exact threshold returns false" {
    const m = createGauge("cpu.usage", 90.0, 1700000000);
    const result = isAboveThreshold(m, 90.0);
    try testing.expect(result != null);
    try testing.expect(result.? == false); // > threshold, not >=
}

test "formatMetric counter without tags" {
    const m = createCounter("http.requests", 42, 1700000000);
    var buf: [256]u8 = undefined;
    const output = try formatMetric(m, &buf);
    try testing.expectEqualStrings("http.requests counter=42 ts=1700000000", output);
}

test "formatMetric gauge without tags" {
    const m = createGauge("cpu.usage", 73.5, 1700000000);
    var buf: [256]u8 = undefined;
    const output = try formatMetric(m, &buf);
    // Zig default float formatting — accept the output as long as it starts correctly
    try testing.expect(std.mem.startsWith(u8, output, "cpu.usage gauge="));
    try testing.expect(std.mem.endsWith(u8, output, "ts=1700000000"));
}

test "formatMetric with tags" {
    var m = createCounter("http.requests", 42, 1700000000);
    m.tags = "env:prod,region:us";
    var buf: [256]u8 = undefined;
    const output = try formatMetric(m, &buf);
    try testing.expectEqualStrings("http.requests counter=42 ts=1700000000 [env:prod,region:us]", output);
}

test "formatMetric histogram" {
    var hist = Histogram.init();
    hist = try hist.observe(1.5);
    hist = try hist.observe(2.3);
    hist = try hist.observe(0.8);
    const m = Metric{
        .name = "request.latency",
        .value = .{ .histogram = hist },
        .tags = null,
        .timestamp = 1700000000,
    };
    var buf: [256]u8 = undefined;
    const output = try formatMetric(m, &buf);
    try testing.expectEqualStrings("request.latency histogram count=3 ts=1700000000", output);
}

test "histogram observe tracks count" {
    var hist = Histogram.init();
    hist = try hist.observe(1.0);
    hist = try hist.observe(2.0);
    try testing.expectEqual(@as(usize, 2), hist.count);
    try testing.expectApproxEqAbs(@as(f64, 1.0), hist.observations[0], 0.001);
    try testing.expectApproxEqAbs(@as(f64, 2.0), hist.observations[1], 0.001);
}

test "tags are optional and default to null" {
    const m = createCounter("test", 1, 0);
    try testing.expect(m.tags == null);

    // Can set tags on a copy
    var with_tags = m;
    with_tags.tags = "env:staging";
    try testing.expect(with_tags.tags != null);
    try testing.expectEqualStrings("env:staging", with_tags.tags.?);
}
