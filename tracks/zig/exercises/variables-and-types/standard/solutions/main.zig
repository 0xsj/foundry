const std = @import("std");
const testing = std.testing;

// =============================================================================
// Metrics Collector — Reference Solution
// =============================================================================
// A type-safe metrics collector using tagged unions, optionals, and explicit
// casting. Demonstrates core Zig variable/type concepts.
//
// Run tests with: zig test main.zig
// =============================================================================

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/// A fixed-capacity histogram that stores up to 64 observations.
/// Uses value semantics — observe() returns a new Histogram rather than
/// mutating in place, matching Zig's preference for explicit mutation.
pub const Histogram = struct {
    observations: [64]f64,
    count: usize,

    pub fn init() Histogram {
        return .{
            .observations = [_]f64{0.0} ** 64,
            .count = 0,
        };
    }

    /// Add an observation to the histogram.
    /// Returns a new Histogram with the observation appended.
    /// Returns error.HistogramFull if at capacity (64 observations).
    pub fn observe(self: Histogram, value: f64) error{HistogramFull}!Histogram {
        if (self.count >= 64) return error.HistogramFull;
        var new = self;
        new.observations[self.count] = value;
        new.count = self.count + 1;
        return new;
    }
};

/// Tagged union representing the three metric types.
///
/// This is the core type design decision. A tagged union (discriminated union)
/// ensures that every consumer must handle all variants — the compiler enforces
/// exhaustive switches. Compare to:
///   - Go: you'd use an interface or a struct with a "type" string field
///   - TypeScript: discriminated unions with a "kind" field
///   - Rust: enums with data (nearly identical to Zig's approach)
///
/// The tag is stored alongside the data, so MetricType knows at runtime
/// which variant it holds. No separate "type" field needed.
pub const MetricType = union(enum) {
    counter: u64,
    gauge: f64,
    histogram: Histogram,
};

/// A single metric data point.
///
/// Note the optional tags field (?[]const u8). This is Zig's way of expressing
/// "this value may or may not be present" — similar to Go's pointer-for-optional
/// pattern (*string) but without heap allocation. The ? adds a single bit of
/// overhead (or uses a sentinel null pointer for pointer/slice types).
pub const Metric = struct {
    name: []const u8,
    value: MetricType,
    tags: ?[]const u8,
    timestamp: u64,
};

/// Errors that metric operations can produce.
/// In Zig, errors are values — they're part of the type system, not exceptions.
/// This is similar to Go's explicit error returns but with compiler-enforced handling.
pub const MetricError = error{NotACounter};

// -----------------------------------------------------------------------------
// Functions
// -----------------------------------------------------------------------------

/// Create a counter metric with the given name, value, and timestamp.
/// Tags default to null — the caller can set them afterward if needed.
///
/// This demonstrates:
///   - Struct literal syntax with .{} shorthand
///   - Tagged union initialization (.counter = value)
///   - Optional null assignment
pub fn createCounter(name: []const u8, value: u64, timestamp: u64) Metric {
    return .{
        .name = name,
        .value = .{ .counter = value },
        .tags = null,
        .timestamp = timestamp,
    };
}

/// Create a gauge metric with the given name, value, and timestamp.
/// Gauges represent point-in-time measurements (CPU%, temperature, etc.)
pub fn createGauge(name: []const u8, value: f64, timestamp: u64) Metric {
    return .{
        .name = name,
        .value = .{ .gauge = value },
        .tags = null,
        .timestamp = timestamp,
    };
}

/// Increment a counter metric by the given amount.
///
/// Returns a NEW Metric with the updated value (value semantics — the original
/// is not mutated). This is idiomatic Zig: functions that "modify" data return
/// new copies rather than mutating through pointers, unless performance requires it.
///
/// Returns error.NotACounter if the metric is not a counter type.
/// The caller MUST handle this error — Zig won't let you ignore it.
///
/// This demonstrates:
///   - Error unions (MetricError!Metric means "Metric or MetricError")
///   - Switch on tagged union with payload capture (|c|)
///   - Value semantics (returning a new struct)
pub fn increment(metric: Metric, amount: u64) MetricError!Metric {
    switch (metric.value) {
        .counter => |c| {
            return .{
                .name = metric.name,
                .value = .{ .counter = c + amount },
                .tags = metric.tags,
                .timestamp = metric.timestamp,
            };
        },
        .gauge, .histogram => return error.NotACounter,
    }
}

/// Extract the numeric value from a metric as f64.
///
/// This demonstrates explicit type casting with @floatFromInt. Zig never
/// does implicit numeric conversions — you must spell out every cast.
/// Compare to Go where int-to-float requires float64(x), or JS where
/// everything is already a float.
///
/// Returns null for histograms since they don't have a single numeric value.
/// The ?f64 return type (optional f64) forces the caller to handle the null case.
pub fn getNumericValue(metric: Metric) ?f64 {
    return switch (metric.value) {
        .counter => |c| @floatFromInt(c),
        .gauge => |g| g,
        .histogram => null,
    };
}

/// Check if a metric's numeric value exceeds the given threshold.
///
/// This chains two optional operations:
///   1. getNumericValue might return null (histogram)
///   2. If we got a value, compare it to the threshold
///
/// Returns ?bool (optional bool):
///   - true if value > threshold
///   - false if value <= threshold
///   - null if the metric has no numeric value
///
/// Note: uses strict > (greater than), not >= (greater than or equal).
pub fn isAboveThreshold(metric: Metric, threshold: f64) ?bool {
    const value = getNumericValue(metric) orelse return null;
    return value > threshold;
}

/// Write a human-readable representation of the metric into the buffer.
///
/// Format:
///   counter:   "http.requests counter=42 ts=1700000000"
///   gauge:     "cpu.usage gauge=7.35e+01 ts=1700000000"
///   histogram: "request.latency histogram count=3 ts=1700000000"
///
/// If tags are present, appends " [env:prod,region:us]" at the end.
///
/// This demonstrates:
///   - Buffer-based formatting (no heap allocation)
///   - std.fmt.bufPrint for type-safe string formatting
///   - Optional unwrapping with if-capture syntax
///   - Error propagation with ! return type
pub fn formatMetric(metric: Metric, buf: []u8) ![]u8 {
    // First, format the type-specific part
    const base = switch (metric.value) {
        .counter => |c| try std.fmt.bufPrint(buf, "{s} counter={d} ts={d}", .{ metric.name, c, metric.timestamp }),
        .gauge => |g| try std.fmt.bufPrint(buf, "{s} gauge={d} ts={d}", .{ metric.name, g, metric.timestamp }),
        .histogram => |h| try std.fmt.bufPrint(buf, "{s} histogram count={d} ts={d}", .{ metric.name, h.count, metric.timestamp }),
    };

    // If tags are present, append them
    if (metric.tags) |tags| {
        // Calculate how much of the buffer we've used, then append into the remainder
        const used = base.len;
        const suffix = try std.fmt.bufPrint(buf[used..], " [{s}]", .{tags});
        // Return the full slice: base + suffix
        return buf[0 .. used + suffix.len];
    }

    return base;
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
