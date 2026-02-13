const std = @import("std");

// =============================================================================
// Sensor Data Pipeline — BUGGY VERSION
// =============================================================================
// This code has 3 bugs. It compiles, but produces incorrect results.
// Read the README.md for symptoms, then find and fix each bug.
//
// Run tests with: zig test buggy_test.zig
// =============================================================================

/// Convert a temperature from Celsius to Fahrenheit.
///
/// Formula: F = C * 9/5 + 32
///
/// BUG 1 IS HERE: This function uses integer arithmetic for a calculation
/// that needs float precision. For large values of celsius, the intermediate
/// result (celsius * 9) can overflow i16 range (-32768 to 32767). Even for
/// smaller values, integer division truncates the fractional part.
pub fn celsiusToFahrenheit(celsius: i16) i16 {
    return @divTrunc(celsius * 9, 5) + 32;
}

/// A running statistics tracker for sensor readings.
pub const SensorStats = struct {
    /// BUG 2 IS HERE: reading_count is u8, which can only hold values 0-255.
    /// The wrapping add (+%=) silently wraps to 0 after 255, instead of
    /// panicking or using a wider type.
    reading_count: u8,
    sum: i64,

    pub fn init() SensorStats {
        return .{
            .reading_count = 0,
            .sum = 0,
        };
    }

    /// Record a new sensor reading.
    pub fn addReading(self: SensorStats, temperature: i16) SensorStats {
        var new = self;
        new.reading_count +%= 1; // BUG: wrapping add silently wraps u8 past 255
        new.sum += @as(i64, temperature);
        return new;
    }

    /// Get the number of readings recorded.
    pub fn getCount(self: SensorStats) u8 {
        return self.reading_count;
    }

    /// Calculate the average temperature across all readings.
    ///
    /// BUG 3 IS HERE: Integer division truncates. sum / count discards the
    /// fractional part entirely, then @intCast converts to i16 (also integer).
    /// The function signature returns i16, so there's no way to represent
    /// fractional averages even if the division were correct.
    pub fn average(self: SensorStats) i16 {
        if (self.reading_count == 0) return 0;
        const count_i64: i64 = @as(i64, self.reading_count);
        return @intCast(@divTrunc(self.sum, count_i64));
    }
};

/// Process a batch of sensor readings: convert each to Fahrenheit,
/// track statistics, and return the stats.
pub fn processBatch(readings_celsius: []const i16) SensorStats {
    var stats = SensorStats.init();
    for (readings_celsius) |celsius| {
        const fahrenheit = celsiusToFahrenheit(celsius);
        stats = stats.addReading(fahrenheit);
    }
    return stats;
}
