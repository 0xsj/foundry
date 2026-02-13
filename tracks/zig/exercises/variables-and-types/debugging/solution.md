# Solution: Sensor Data Pipeline Bugs

## Bug 1: Integer Overflow and Truncation in Temperature Conversion

### Root Cause

The `celsiusToFahrenheit` function uses `i16` arithmetic for the formula `celsius * 9 / 5 + 32`. This has two problems:

1. **Overflow**: For `celsius = 4000`, the intermediate result `4000 * 9 = 36000` exceeds `i16` max (32767). In Zig's safe mode (default for debug builds), this panics with "integer overflow". In release/unsafe mode, it wraps silently to garbage values.

2. **Truncation**: Even when the multiplication doesn't overflow, integer division truncates. For `celsius = 26`: `26 * 9 = 234`, `234 / 5 = 46` (should be 46.8), `46 + 32 = 78` (should be 78.8).

### The Buggy Code

```zig
pub fn celsiusToFahrenheit(celsius: i16) i16 {
    return @divTrunc(celsius * 9, 5) + 32;
}
```

### The Fix

Use `f64` for the calculation and return type:

```zig
pub fn celsiusToFahrenheit(celsius: i16) f64 {
    const c: f64 = @floatFromInt(celsius);
    return c * 9.0 / 5.0 + 32.0;
}
```

### Why This Matters

This is a classic "works in testing, breaks in production" bug. Small test values (0, 25, 100) often divide cleanly or don't overflow, so tests pass. Real-world data with extreme values (industrial sensors, weather stations at altitude) hits both problems.

**Zig concept**: Zig's explicit casting philosophy exists precisely to prevent this. The compiler forces you to think about `@floatFromInt` — the act of writing it should trigger the question "should I be doing this computation in float from the start?"

**Cross-language note**: Go has the same issue — `int` arithmetic truncates. But Go won't panic on overflow (it silently wraps), making this even harder to catch. Rust's behavior is closest to Zig: debug builds panic on overflow, release builds wrap.

### Prevention

- Use `f64` for any calculation that involves division or could have fractional results
- Use `@floatFromInt` at the start of a computation, not at the end
- Choose numeric types based on the domain's value range, not the minimum size

---

## Bug 2: Counter Overflow with Wrapping Arithmetic

### Root Cause

The `reading_count` field is `u8` (range 0-255), and the code uses `+%=` (wrapping add) to increment it. The wrapping operator silently wraps `255 + 1` back to `0` instead of reporting an error. After 256 readings, the pipeline thinks it has processed 0 readings.

The developer likely chose `+%=` thinking it was the "safe" way to add (it won't panic!). But wrapping is not safe — it's just silent corruption. The whole reason Zig's default `+` operator panics on overflow is to catch exactly this kind of bug.

### The Buggy Code

```zig
reading_count: u8,
// ...
new.reading_count +%= 1;  // wraps to 0 after 255
```

### The Fix

Use a wider type and normal addition:

```zig
reading_count: u32,
// ...
new.reading_count += 1;  // panics if u32 overflows (at 4 billion — unlikely)

pub fn getCount(self: SensorStats) u32 {
    return self.reading_count;
}
```

### Why This Matters

Wrapping arithmetic (`+%`, `-%`, `*%`) exists for specific use cases: hash functions, checksums, ring buffers, bitwise operations. Using it for a simple counter is a misuse. The "safety" of not panicking becomes a liability — you get silent data corruption instead of a clear crash.

**Zig concept**: Zig provides four arithmetic modes for a reason:
- `+` — default, panics on overflow in safe mode (use this unless you have a reason not to)
- `+%` — wrapping, silently wraps (for hash functions, ring buffers)
- `+|` — saturating, clamps to max/min (for audio, color values)
- `@addWithOverflow` — returns a tuple with the result and an overflow flag (for manual handling)

**Cross-language note**: In Go, all integer arithmetic wraps silently — there's no overflow detection. In Rust, debug builds panic like Zig's `+`, and release builds wrap like Zig's `+%`. Zig makes you choose explicitly.

### Prevention

- Default to `+` (panicking add) — it catches bugs
- Only use `+%` when wrapping is the desired behavior (and comment why)
- Choose integer widths based on maximum expected values, not minimum storage

---

## Bug 3: Integer Division in Average Calculation

### Root Cause

The `average` function divides two integers (`sum / count`), which performs integer division (truncation toward zero). The result `73 / 3 = 24` instead of `24.333...`. Then it casts to `i16` via `@intCast`, which also cannot represent fractional values.

The function signature `fn average() i16` makes it structurally impossible to return a correct fractional average.

### The Buggy Code

```zig
pub fn average(self: SensorStats) i16 {
    if (self.reading_count == 0) return 0;
    const count_i64: i64 = @as(i64, self.reading_count);
    return @intCast(@divTrunc(self.sum, count_i64));
}
```

### The Fix

Convert to float before dividing, and return `f64`:

```zig
pub fn average(self: SensorStats) f64 {
    if (self.reading_count == 0) return 0.0;
    const sum_f: f64 = @floatFromInt(self.sum);
    const count_f: f64 = @floatFromInt(self.reading_count);
    return sum_f / count_f;
}
```

### Why This Matters

This is the most common numeric bug across all languages. Integer division silently discards information. In many cases (like Bug 1's test with `24`), the truncation is invisible because the result happens to be exact.

**Zig concept**: Zig forces explicit conversions with `@floatFromInt`. This friction is intentional — writing the conversion function name should make you think about whether you're converting at the right point in the computation. Converting the *result* of integer division to float preserves the truncation error. You must convert *before* dividing.

**Cross-language note**:
- Python 3 fixed this permanently: `/` always returns float, `//` is explicit integer division
- Go has the same trap: `sum / count` with int operands truncates
- JavaScript avoids it entirely (all numbers are float64) — but then you get floating-point precision issues instead

### Prevention

- If a function computes an average, ratio, or percentage, it should return `f64`
- Convert to float *before* dividing, not after
- Be suspicious of any `@intCast` applied to a division result

---

## Summary of Fixes

| Bug | Root Cause | Key Concept | Fix |
|-----|-----------|-------------|-----|
| 1 | i16 overflow + truncation in `celsius * 9` | Integer overflow, explicit casting | Use f64 for computation, return f64 |
| 2 | u8 counter with wrapping add (+%=) | Wrapping vs safe arithmetic, type sizing | Use u32 with normal += |
| 3 | Integer division in sum/count | Integer division truncation | Convert to f64 before dividing |

## Related Concepts

- [[fundamentals/variables-and-types]] — Numeric types and their ranges
- [[pitfalls/zig-wrapping-arithmetic-misuse]] — When to use +% vs +
- [[pitfalls/integer-division-truncation]] — Cross-language issue
- [[rosetta/common-operations#numeric-conversion]] — Type casting across languages
