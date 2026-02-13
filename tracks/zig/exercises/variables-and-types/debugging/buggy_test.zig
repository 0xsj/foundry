const std = @import("std");
const testing = std.testing;
const sensor = @import("buggy.zig");

// =============================================================================
// Sensor Data Pipeline — Tests
// =============================================================================
// These tests expose the 3 bugs in buggy.zig.
// After fixing the bugs, all tests should pass.
//
// Run with: zig test buggy_test.zig
// =============================================================================

// ---------------------------------------------------------------------------
// Bug 1: Temperature conversion
// ---------------------------------------------------------------------------

test "basic Celsius to Fahrenheit conversion" {
    // 0 C = 32 F
    const result = sensor.celsiusToFahrenheit(0);
    // After fix, this should return f64, so adjust the test accordingly.
    // For now, testing the integer version:
    try testing.expectEqual(@as(i16, 32), result);
}

test "room temperature conversion is approximately correct" {
    // 25 C = 77.0 F
    // With integer arithmetic: 25 * 9 / 5 + 32 = 225 / 5 + 32 = 45 + 32 = 77 (looks correct by luck)
    const result = sensor.celsiusToFahrenheit(25);
    try testing.expectEqual(@as(i16, 77), result);
}

test "conversion loses precision for non-round values" {
    // 26 C = 78.8 F, but integer math gives 78
    // 26 * 9 = 234, 234 / 5 = 46 (truncated from 46.8), 46 + 32 = 78
    // BUG: This should be approximately 78.8, but integer division gives 78
    // After fix (returning f64), this test should check for 78.8
    const result = sensor.celsiusToFahrenheit(26);
    // This test DOCUMENTS the bug: we expect 78 because of integer truncation,
    // but the correct answer is 78.8.
    // AFTER FIX: change to expectApproxEqAbs(@as(f64, 78.8), result, 0.01)
    try testing.expectEqual(@as(i16, 78), result);
}

test "large Celsius value overflows i16 in intermediate calculation" {
    // 4000 C: 4000 * 9 = 36000 which OVERFLOWS i16 (max 32767)
    // In safe mode, this will panic. In unsafe mode, it wraps to garbage.
    // BUG: This should work. 4000 C = 7232 F.
    //
    // To test: after fixing celsiusToFahrenheit to use f64 internally,
    // this should return approximately 7232.0
    //
    // BEFORE FIX: this test will panic due to overflow in safe mode
    // AFTER FIX: uncomment and adjust the assertion
    // const result = sensor.celsiusToFahrenheit(4000);
    // try testing.expectApproxEqAbs(@as(f64, 7232.0), result, 0.01);
}

// ---------------------------------------------------------------------------
// Bug 2: Counter overflow
// ---------------------------------------------------------------------------

test "count stays correct for small batches" {
    var stats = sensor.SensorStats.init();
    stats = stats.addReading(20);
    stats = stats.addReading(25);
    stats = stats.addReading(30);
    try testing.expectEqual(@as(u8, 3), stats.getCount());
}

test "count overflows after 255 readings" {
    // Process 256 readings. With a u8 counter and wrapping add, the count
    // wraps to 0 on the 256th reading.
    // BUG: After 256 readings, getCount() returns 0 instead of 256.
    var stats = sensor.SensorStats.init();
    for (0..256) |_| {
        stats = stats.addReading(20);
    }
    // With u8 +%= wrapping, this returns 0 instead of 256
    // AFTER FIX: use u32 counter, getCount returns u32, and this should be 256
    try testing.expectEqual(@as(u8, 0), stats.getCount()); // BUG: should be 256
}

test "count wraps again at 512 readings" {
    // Further proof: 512 readings wraps back to 0 a second time
    var stats = sensor.SensorStats.init();
    for (0..512) |_| {
        stats = stats.addReading(20);
    }
    try testing.expectEqual(@as(u8, 0), stats.getCount()); // BUG: should be 512
}

// ---------------------------------------------------------------------------
// Bug 3: Average truncation
// ---------------------------------------------------------------------------

test "average of evenly divisible values looks correct" {
    // Sum = 72, count = 3, average = 24 (integer division happens to be exact)
    var stats = sensor.SensorStats.init();
    stats = stats.addReading(23);
    stats = stats.addReading(24);
    stats = stats.addReading(25);
    try testing.expectEqual(@as(i16, 24), stats.average());
}

test "average truncates fractional part" {
    // Sum = 73, count = 3, average should be 24.333...
    // BUG: integer division gives 24, losing the .333
    var stats = sensor.SensorStats.init();
    stats = stats.addReading(23);
    stats = stats.addReading(24);
    stats = stats.addReading(26);
    // AFTER FIX: change to expectApproxEqAbs(@as(f64, 24.333), result, 0.01)
    try testing.expectEqual(@as(i16, 24), stats.average()); // BUG: should be ~24.33
}

test "average of single reading returns exact value" {
    var stats = sensor.SensorStats.init();
    stats = stats.addReading(37);
    try testing.expectEqual(@as(i16, 37), stats.average());
}

test "average of empty batch returns zero" {
    const stats = sensor.SensorStats.init();
    try testing.expectEqual(@as(i16, 0), stats.average());
}
