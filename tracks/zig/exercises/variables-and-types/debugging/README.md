# Debugging: Sensor Data Pipeline

## Context

A teammate wrote a sensor data pipeline that reads temperature values from industrial sensors, converts between Celsius and Fahrenheit, and tracks running statistics (count, sum, average). The code compiles without errors and even passes some basic tests, but production reports reveal incorrect behavior under certain conditions.

Your job: find and fix the bugs using only the symptoms below. Do not read `solution.md` until you've identified all three issues.

## Symptoms

### Symptom 1: Wildly wrong temperature conversions
Users report that converting large Celsius values (e.g., industrial furnace readings around 1500-5000 C) produces completely wrong Fahrenheit values. Smaller values like room temperature seem fine. In debug builds, the program sometimes crashes with an "integer overflow" message.

### Symptom 2: Sensor count resets to zero
After processing exactly 256 sensor readings in a batch, the `reading_count` resets to 0. The pipeline then reports "0 readings processed" despite having processed hundreds. This was caught when a factory with 300 sensors in a batch reported empty statistics.

### Symptom 3: Average temperature is always a whole number
The average temperature calculation always returns a round integer (e.g., 24 instead of 24.33). For batches where the sum divides evenly by the count, the result looks correct. For everything else, the fractional part is silently lost.

## Instructions

1. Open `buggy.zig` and read through the code
2. Run `zig test buggy_test.zig` to see the failing tests
3. For each symptom, identify the root cause and fix it
4. All tests in `buggy_test.zig` should pass after your fixes
5. Compare your fixes with `solution.md`

## Concepts Tested

- Integer overflow behavior (safe arithmetic vs wrapping operators)
- Integer type sizing (u8, u16, u32 range limits)
- Integer vs float arithmetic (truncation in division)
- Explicit type casting (`@floatFromInt`, `@intFromFloat`)
- Choosing the right numeric type for the domain
