# Zig Reference — Variables and Types

> Extracted from [Zig Language Reference](https://ziglang.org/documentation/master/)
> for the `variables-and-types` module. Covers: variable declarations, primitive types, arrays, slices, structs, enums, unions, optionals.

---

## Variable Declarations

Source: [Zig Language Reference — Variables](https://ziglang.org/documentation/master/#toc-Variables)

### `const`

Declares an immutable binding. The value cannot be changed after initialization.

```zig
const x: i32 = 42;
const y = 42;        // type inferred as comptime_int
```

A `const` in a function is a compile-time or runtime immutable value. A `const` at the top level (outside any function) is a **compile-time** value.

### `var`

Declares a mutable binding. The value can be changed after initialization.

```zig
var x: i32 = 42;
x = 100;     // OK
```

A `var` requires either an explicit type or an initializer from which the type can be inferred. `var` cannot be `comptime`-known if it is mutated at runtime.

### `undefined`

Any `var` or `const` can be initialized to `undefined`:

```zig
var buf: [1024]u8 = undefined;
```

This leaves the memory uninitialized. Reading from `undefined` is **safety-checked undefined behavior** — in safe build modes, it will be detected. In release modes, it is true undefined behavior.

---

## Integer Types

Source: [Zig Language Reference — Integers](https://ziglang.org/documentation/master/#toc-Integers)

### Standard Integer Types

| Type | Bits | Signed | Range |
|---|---|---|---|
| `u8` | 8 | No | 0 to 255 |
| `u16` | 16 | No | 0 to 65,535 |
| `u32` | 32 | No | 0 to 4,294,967,295 |
| `u64` | 64 | No | 0 to 18,446,744,073,709,551,615 |
| `u128` | 128 | No | |
| `i8` | 8 | Yes | -128 to 127 |
| `i16` | 16 | Yes | -32,768 to 32,767 |
| `i32` | 32 | Yes | -2,147,483,648 to 2,147,483,647 |
| `i64` | 64 | Yes | |
| `i128` | 128 | Yes | |
| `usize` | ptr | No | Platform pointer size (64-bit on 64-bit systems) |
| `isize` | ptr | Yes | Signed pointer size |

### Arbitrary Bit-Width Integers

Zig supports integers of any bit width from 0 to 65535:

```zig
const x: u3 = 5;     // 3-bit unsigned (0-7)
const y: i12 = -100;  // 12-bit signed
const z: u1 = 1;      // single bit
```

These are useful for packed structs, protocol implementations, and bit manipulation.

### Integer Overflow

Integer overflow is **illegal** in Zig. In safe build modes, overflow is detected and causes a panic. In release modes, it is undefined behavior.

For intentional wrapping arithmetic, use the `+%`, `-%`, `*%` operators:

```zig
var x: u8 = 255;
// x += 1;           // panic in safe mode: overflow
x +%= 1;             // wrapping add: x is now 0
```

Saturating arithmetic uses `+|`, `-|`, `*|`:

```zig
var x: u8 = 250;
x +|= 10;            // saturating add: x is now 255 (clamped)
```

---

## Floating Point Types

Source: [Zig Language Reference — Floats](https://ziglang.org/documentation/master/#toc-Floats)

| Type | Bits | Notes |
|---|---|---|
| `f16` | 16 | IEEE 754 half precision |
| `f32` | 32 | IEEE 754 single precision |
| `f64` | 64 | IEEE 754 double precision |
| `f80` | 80 | x86 extended precision |
| `f128` | 128 | IEEE 754 quad precision |
| `comptime_float` | N/A | Compile-time arbitrary precision |

Float literals:

```zig
const x = 3.14;              // comptime_float
const y: f64 = 3.14;         // f64
const z: f32 = 1.0e-5;       // scientific notation
const w: f64 = 0x1.0p10;     // hex float: 1024.0
```

---

## Boolean

Source: [Zig Language Reference — Primitives](https://ziglang.org/documentation/master/#toc-Primitive-Types)

```zig
const a: bool = true;
const b: bool = false;
```

Booleans are 1 byte (`u8`-sized). They support `and`, `or`, `!` operators. Zig does **not** allow using integers as booleans — there is no implicit truthiness.

```zig
// const x: bool = 1;      // compile error — not a bool
// if (42) { ... }          // compile error — condition must be bool
```

---

## Arrays

Source: [Zig Language Reference — Arrays](https://ziglang.org/documentation/master/#toc-Arrays)

Fixed-size, homogeneous, stack-allocated by default. Value type (copying an array copies all elements).

```zig
const a = [5]u8{ 'h', 'e', 'l', 'l', 'o' };
const b = [_]u32{ 1, 2, 3 };   // compiler infers length: [3]u32

// Access
const first = a[0];       // 'h'
const len = a.len;         // 5 (comptime-known)

// Sentinel-terminated arrays
const c = [_:0]u8{ 1, 2, 3 };  // [3:0]u8 — has a 0 byte at index 3
```

### Array Operations

```zig
// Concatenation (comptime only)
const hello = "hello " ++ "world";   // *const [11:0]u8

// Repetition (comptime only)
const zeros = [_]u8{0} ** 10;        // [10]u8 of zeros

// Iteration
for (a) |elem| {
    _ = elem;
}

// Iteration with index
for (a, 0..) |elem, i| {
    _ = elem;
    _ = i;
}
```

---

## Slices

Source: [Zig Language Reference — Slices](https://ziglang.org/documentation/master/#toc-Slices)

A slice is a pointer + length pair that references a contiguous region of an array.

```zig
const a = [5]u8{ 1, 2, 3, 4, 5 };
const slice: []const u8 = a[1..4];  // elements at indices 1, 2, 3
// slice.ptr — pointer to a[1]
// slice.len — 3
```

### Slice Types

| Type | Meaning |
|---|---|
| `[]T` | Mutable slice of T |
| `[]const T` | Immutable slice of T |
| `[*]T` | Many-item pointer (no length, for C interop) |
| `[*:0]T` | Many-item pointer, sentinel-terminated |

### Slicing Syntax

```zig
const a = [5]i32{ 10, 20, 30, 40, 50 };
const s1 = a[0..3];     // [10, 20, 30]
const s2 = a[2..];      // [30, 40, 50]  — open-ended
const s3 = a[0..a.len]; // entire array
```

---

## Strings

Source: [Zig Language Reference — String Literals](https://ziglang.org/documentation/master/#toc-String-Literals-and-Unicode)

String literals are `*const [N:0]u8` — a pointer to a compile-time known, null-terminated byte array. They coerce to `[]const u8` (a slice).

```zig
const hello = "hello";
// Type: *const [5:0]u8
// Coerces to: []const u8

const slice: []const u8 = hello;  // implicit coercion
```

### Multiline Strings

```zig
const multiline =
    \\This is line 1
    \\This is line 2
    \\This is line 3
;
```

### String Formatting

```zig
const std = @import("std");
// Print to stderr (debug)
std.debug.print("x = {d}, name = {s}\n", .{ 42, "hello" });
```

Format specifiers: `{d}` for integers, `{s}` for strings/byte slices, `{any}` for any type, `{x}` for hex, `{e}` for scientific float notation.

---

## Structs

Source: [Zig Language Reference — Structs](https://ziglang.org/documentation/master/#toc-struct)

```zig
const Point = struct {
    x: f64,
    y: f64,
};

const p = Point{ .x = 1.0, .y = 2.0 };
```

### Features

- **Default values**: `field: Type = default_value`
- **Methods**: Functions declared inside the struct namespace
- **Packed structs**: `packed struct` for exact memory layout
- **Extern structs**: `extern struct` for C-compatible layout

### Default Values

```zig
const Config = struct {
    port: u16 = 8080,
    debug: bool = false,
};

const c = Config{};             // uses all defaults
const d = Config{ .port = 3000 }; // override port, debug stays false
```

### Anonymous Struct Literals

```zig
fn makePoint(x: f64, y: f64) Point {
    return .{ .x = x, .y = y };   // type inferred from return type
}
```

---

## Enums

Source: [Zig Language Reference — Enums](https://ziglang.org/documentation/master/#toc-enum)

```zig
const Direction = enum {
    north,
    south,
    east,
    west,
};

const d: Direction = .north;
```

### Features

- **Explicit tag type**: `enum(u8) { ... }`
- **Explicit values**: `ok = 200`
- **Methods**: Functions declared inside the enum
- **Conversion**: `@intFromEnum(val)`, `@enumFromInt(Type, int)`
- **Tag name**: `@tagName(val)` returns a `[]const u8`

### Exhaustive Switch

```zig
const result = switch (d) {
    .north => "up",
    .south => "down",
    .east => "right",
    .west => "left",
};
// Removing any arm is a compile error
```

---

## Unions

Source: [Zig Language Reference — Unions](https://ziglang.org/documentation/master/#toc-union)

### Bare Unions

```zig
const Value = union {
    int: i64,
    float: f64,
    bool_val: bool,
};
// Accessing the wrong field is undefined behavior
```

### Tagged Unions

```zig
const Value = union(enum) {
    int: i64,
    float: f64,
    boolean: bool,
    none,
};

const v = Value{ .int = 42 };

switch (v) {
    .int => |i| std.debug.print("int: {}\n", .{i}),
    .float => |f| std.debug.print("float: {}\n", .{f}),
    .boolean => |b| std.debug.print("bool: {}\n", .{b}),
    .none => std.debug.print("none\n", .{}),
}
```

Tagged unions carry a discriminant that tracks which field is active. Accessing the wrong field is safely caught in safe build modes.

---

## Optional Types

Source: [Zig Language Reference — Optionals](https://ziglang.org/documentation/master/#toc-Optionals)

An optional type `?T` can hold either a value of type `T` or `null`.

```zig
var x: ?i32 = 42;
x = null;
```

### Unwrapping

```zig
// Payload capture (safe)
if (x) |value| {
    // value is i32
}

// orelse (default value)
const y = x orelse 0;

// .? (force unwrap — panics if null)
const z = x.?;
```

### Optional Pointers

`?*T` is guaranteed to be the same size as `*T` — the null representation uses the 0 address. This is a zero-cost abstraction.

---

## Type Coercion

Source: [Zig Language Reference — Type Coercion](https://ziglang.org/documentation/master/#toc-Type-Coercion)

Zig has limited, safe implicit coercions:

| From | To | Example |
|---|---|---|
| `*[N]T` | `[]T` | Array pointer to slice |
| `T` | `?T` | Value to optional |
| `T` | `E!T` | Value to error union |
| `comptime_int` | any integer type | If value fits |
| `comptime_float` | any float type | |
| `*T` | `*const T` | Mutable to const pointer |
| `[N:s]T` | `[N]T` | Sentinel-terminated to plain array |

All other conversions require explicit builtins: `@intCast`, `@floatCast`, `@intFromFloat`, `@ptrCast`, etc.

---

## Builtin Functions (Type-Related)

Source: [Zig Language Reference — Builtins](https://ziglang.org/documentation/master/#toc-Builtin-Functions)

| Builtin | Purpose | Example |
|---|---|---|
| `@intCast` | Convert between integer types | `@intCast(u8, x)` |
| `@floatCast` | Convert between float types | `@floatCast(f32, x)` |
| `@intFromFloat` | Float to integer | `@intFromFloat(i32, 3.14)` |
| `@floatFromInt` | Integer to float | `@floatFromInt(f64, 42)` |
| `@intFromEnum` | Enum to integer | `@intFromEnum(Status.ok)` |
| `@enumFromInt` | Integer to enum | `@enumFromInt(Status, 200)` |
| `@intFromBool` | Bool to integer (0 or 1) | `@intFromBool(true)` |
| `@sizeOf` | Size of type in bytes | `@sizeOf(i32)` → 4 |
| `@alignOf` | Alignment of type | `@alignOf(i64)` → 8 |
| `@typeInfo` | Reflect on type at comptime | `@typeInfo(i32)` |
| `@typeName` | Type name as string | `@typeName(i32)` → `"i32"` |
| `@tagName` | Enum/union variant name | `@tagName(Dir.north)` → `"north"` |

---

## Comptime Types

Source: [Zig Language Reference — Comptime](https://ziglang.org/documentation/master/#toc-comptime)

| Type | Description |
|---|---|
| `comptime_int` | Arbitrary-precision integer, compile-time only |
| `comptime_float` | Arbitrary-precision float, compile-time only |
| `type` | Represents a type itself — only exists at comptime |

These types cannot exist at runtime. They must be resolved to concrete types before code generation.

```zig
const x = 42;           // comptime_int
const y: i32 = x;       // coerced to i32 at compile time
// var z = x;            // error: comptime_int cannot exist at runtime
```
