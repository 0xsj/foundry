# Variables and Types — Zig

## The Zig Mindset

Before diving into syntax, it helps to understand *why* Zig does things the way it does. Zig was designed around a few core principles:

- **No hidden control flow.** If something can fail or branch, you see it in the code.
- **No hidden allocations.** Memory allocation never happens behind your back.
- **Explicit over implicit.** If there's ambiguity, the compiler makes you resolve it.

Coming from JS/TS, this is a fundamentally different contract. JavaScript does enormous amounts of work invisibly — type coercion, garbage collection, prototype chains, implicit returns. Zig takes the opposite stance: the code you read is the code that runs.

Coming from Go, you'll notice some overlap (static typing, compiled, explicit error handling) but Zig goes further — no garbage collector, no hidden allocations, and compile-time execution instead of code generation.

---

## How Variables Work Under the Hood

### Declaration

Zig has two keywords for binding values to names: `const` and `var`.

```zig
const x: i32 = 42;    // immutable — cannot be reassigned
var y: i32 = 42;       // mutable — can be reassigned
y = 100;               // OK
// x = 100;            // compile error: cannot assign to constant
```

This is closer to JavaScript's `const`/`let` than to Go's `:=`/`var`. But there's an important difference: **Zig's `const` means truly immutable**. In JS, `const obj = {}` prevents reassignment but the object itself is mutable. In Zig, `const` means the value cannot change at all, period.

| | JS/TS | Go | Zig |
|---|---|---|---|
| Immutable binding | `const` (shallow) | No direct equivalent | `const` (deep) |
| Mutable binding | `let` | `var` or `:=` | `var` |
| Reassignable but immutable contents? | `const` on objects | N/A | Not possible — `const` is all-or-nothing |

### Type Inference

Zig can infer types from the right-hand side, similar to Go's `:=` or TypeScript's type inference:

```zig
const x = 42;           // inferred as comptime_int (more on this below)
const y: i32 = 42;      // explicitly i32
var z: u8 = 0;          // var requires explicit type OR an initial value the compiler can infer from
```

When you omit the type, the compiler picks the type from the value. For integer literals like `42`, Zig doesn't pick `i32` by default — it uses a special type called `comptime_int`, which is an arbitrary-precision integer that only exists at compile time. It gets "materialized" into a concrete type when you use it in a runtime context.

```zig
const x = 42;            // comptime_int
const y: i32 = x;        // comptime_int coerces to i32
const z: u8 = x;         // comptime_int coerces to u8 — fits, so it's fine
const w: u8 = 300;       // compile error: 300 doesn't fit in u8
```

This is powerful — it means literal values are checked at compile time for overflow and range, not silently truncated like in C.

### Memory: Stack by Default

Every `var` and `const` in a function lives on the stack. There is no garbage collector. There is no implicit heap allocation. If something goes on the heap, *you* put it there explicitly using an allocator.

```zig
const std = @import("std");

pub fn main() void {
    var x: i32 = 42;      // stack-allocated, 4 bytes
    x = 100;              // modifies the same stack slot
    _ = x;
    // When main() returns, x's stack space is reclaimed automatically
}
```

Compare this mental model:

| | JS/TS | Go | Zig |
|---|---|---|---|
| Primitives | Stack (usually) | Stack (usually) | Stack (always, unless you allocate) |
| Objects/Structs | Heap (GC managed) | Stack or heap (escape analysis) | Stack (default), heap only if you explicitly allocate |
| Who decides heap? | Runtime (GC) | Compiler (escape analysis) | You (via allocator) |

In Go, the compiler decides via escape analysis. In Zig, **you decide**. This is more work but also more control — you always know exactly where your data lives.

### Your notes
<!-- User adds insights here during learning -->


---

## Type System

### Static, Nominal, Strong

Zig's type system is:
- **Static**: all types known at compile time
- **Nominal**: types are identified by name, not structure (unlike Go interfaces or TypeScript's structural typing)
- **Strong**: no implicit conversions between types. A `u8` is not an `i32` — you must convert explicitly.

```zig
const a: u8 = 42;
// const b: i32 = a;     // compile error — no implicit widening
const b: i32 = @intCast(a);  // explicit cast — you're saying "I know this fits"
```

This is stricter than Go (which also requires explicit conversion but between its own types) and *much* stricter than JS/TS (which coerces freely at runtime).

### Primitive Types

Zig has fixed-size integer types with explicit signedness. No platform-dependent `int` like Go — every type has a known size.

#### Integers

| Type | Size | Range | Notes |
|---|---|---|---|
| `u8` | 1 byte | 0 to 255 | Equivalent to Go's `byte` |
| `u16` | 2 bytes | 0 to 65,535 | |
| `u32` | 4 bytes | 0 to ~4.3 billion | |
| `u64` | 8 bytes | 0 to ~18.4 quintillion | |
| `i8` | 1 byte | -128 to 127 | |
| `i16` | 2 bytes | -32,768 to 32,767 | |
| `i32` | 4 bytes | -2.1B to 2.1B | |
| `i64` | 8 bytes | | |
| `usize` | pointer-sized | | Used for array/slice indexing, memory sizes |
| `isize` | pointer-sized | | Signed version of usize |
| `comptime_int` | arbitrary | | Compile-time only, infinite precision |

The naming convention is simple: `u` = unsigned, `i` = signed, number = bit width. Once you see the pattern, you can predict any type name. Zig even supports arbitrary bit-width integers like `u3` (0-7) or `i12` — useful for bit-packed structures and protocols.

```zig
const a: u3 = 5;    // 3-bit unsigned integer (0-7)
const b: i12 = -100; // 12-bit signed integer
```

#### Floats

| Type | Size | Notes |
|---|---|---|
| `f16` | 2 bytes | Half precision |
| `f32` | 4 bytes | Single precision (≈ C's `float`) |
| `f64` | 8 bytes | Double precision (≈ JS's `number`) |
| `f128` | 16 bytes | Quad precision |
| `comptime_float` | | Compile-time only |

#### Other Primitives

| Type | Size | Notes |
|---|---|---|
| `bool` | 1 byte | `true` or `false` |
| `void` | 0 bytes | No value; used for functions that return nothing |
| `noreturn` | | Function never returns (e.g., `@panic`, infinite loop) |
| `type` | | A type itself — only exists at comptime |

### Your notes
<!-- -->


---

## Strings and Arrays

### Strings Are Byte Arrays

In Zig, there is no dedicated `string` type. A string literal is a `*const [N:0]u8` — a pointer to a null-terminated array of bytes.

```zig
const greeting = "hello";
// Type is: *const [5:0]u8
// - pointer to a const array
// - 5 bytes of content
// - :0 means null-terminated (there's a 0 byte after 'o')
```

When you want to work with strings as a runtime value (pass them around, slice them), you typically use a **slice**: `[]const u8`.

```zig
const greeting: []const u8 = "hello";
// This is a slice — a pointer + length (like Go's string header)
```

A slice in Zig is a struct under the hood:

```
// Conceptually:
struct {
    ptr: [*]const u8,  // pointer to the data
    len: usize,        // number of elements
}
```

This is almost exactly Go's string header (pointer + length). And like Go, `len` gives you bytes, not characters. For UTF-8 character counting, you'd use `std.unicode.utf8CountCodepoints()`.

| | JS/TS | Go | Zig |
|---|---|---|---|
| String type | `string` (UTF-16) | `string` (immutable bytes) | `[]const u8` (byte slice) |
| Encoding | UTF-16 | UTF-8 bytes | Raw bytes (typically UTF-8) |
| Length | `.length` (UTF-16 code units) | `len()` (bytes) | `.len` (bytes) |
| Mutability | Immutable | Immutable | `const` = immutable, can have `[]u8` for mutable |
| Null terminated? | No | No | Literals are `:0`, slices are not |

### Arrays

Arrays in Zig are fixed-size, stack-allocated, and value types (like Go arrays, unlike JS arrays which are heap objects).

```zig
const a = [5]u8{ 1, 2, 3, 4, 5 };     // type: [5]u8
const b = [_]u8{ 1, 2, 3, 4, 5 };     // [_] lets compiler count — still [5]u8
const c: [3]bool = .{ true, false, true };

// Access
const first = a[0];    // 1
const length = a.len;  // 5 (compile-time known)
```

Arrays are **value types** — assigning or passing an array copies the entire thing:

```zig
var a = [3]i32{ 1, 2, 3 };
var b = a;       // b is a COPY
b[0] = 999;     // does NOT affect a
// a[0] is still 1
```

This is identical to Go arrays (but different from JS arrays, which are references).

### Slices

Slices are a *view* into an array — a pointer + length, without owning the data.

```zig
const a = [5]u8{ 1, 2, 3, 4, 5 };
const slice = a[1..4];    // type: []const u8, elements [2, 3, 4]
// slice.ptr points into a's memory
// slice.len == 3
```

Unlike Go slices, Zig slices do **not** have a capacity field. They're strictly pointer + length. If you need a growable dynamic array, you use `std.ArrayList` with an explicit allocator.

| | Go slice | Zig slice |
|---|---|---|
| Header | ptr + len + cap | ptr + len |
| Growable? | Yes (`append`) | No — use `std.ArrayList` |
| Owns data? | No (view into underlying array) | No (view into underlying array) |

### Your notes
<!-- -->


---

## Structs

Structs in Zig are the primary way to create composite types. Like Go, there are no classes.

```zig
const Point = struct {
    x: f64,
    y: f64,
};

const p = Point{ .x = 1.0, .y = 2.0 };
```

Notice the syntax: `.x = 1.0` uses the dot-prefix for field initialization. This is Zig's way of saying "this is a field of the expected type."

### Default Values

Zig structs can have default values (Go structs cannot — they always zero-initialize):

```zig
const Config = struct {
    port: u16 = 8080,
    host: []const u8 = "localhost",
    max_connections: u32 = 100,
};

const default_config = Config{};                        // all defaults
const custom = Config{ .port = 3000 };                  // override just port
```

This is closer to TypeScript's default parameter values or Python's dataclass defaults than to Go's zero values.

### Methods

Zig attaches methods to structs via declarations inside the struct namespace:

```zig
const Vec2 = struct {
    x: f64,
    y: f64,

    // Method — takes self as first parameter
    pub fn length(self: Vec2) f64 {
        return @sqrt(self.x * self.x + self.y * self.y);
    }

    // "Constructor" pattern — just a function that returns the type
    pub fn init(x: f64, y: f64) Vec2 {
        return Vec2{ .x = x, .y = y };
    }
};

const v = Vec2.init(3.0, 4.0);
const len = v.length();   // 5.0
```

The `self` parameter is explicit (like Python, unlike Go's implicit receiver). If you want to mutate, use a pointer:

```zig
pub fn scale(self: *Vec2, factor: f64) void {
    self.x *= factor;
    self.y *= factor;
}
```

Compare with Go:

| | Go | Zig |
|---|---|---|
| Method declaration | `func (v Vec2) Length() float64` | `pub fn length(self: Vec2) f64` |
| Pointer receiver | `func (v *Vec2) Scale(f float64)` | `pub fn scale(self: *Vec2, factor: f64) void` |
| Constructor convention | `func NewVec2(x, y float64) Vec2` | `pub fn init(x: f64, y: f64) Vec2` |

### Your notes
<!-- -->


---

## Enums

Zig enums are more powerful than Go's `iota`-based constants but less complex than Rust's enums (which are algebraic data types).

```zig
const Direction = enum {
    north,
    south,
    east,
    west,
};

const d: Direction = .north;   // shorthand — compiler infers the enum type
```

Enums can have explicit integer backing types and values:

```zig
const HttpStatus = enum(u16) {
    ok = 200,
    not_found = 404,
    internal_error = 500,
};

// Convert to integer
const code: u16 = @intFromEnum(HttpStatus.ok);  // 200

// Convert from integer
const status = @enumFromInt(HttpStatus, 200);     // .ok
```

Enums can also have methods, just like structs:

```zig
const Color = enum {
    red,
    green,
    blue,

    pub fn isWarm(self: Color) bool {
        return self == .red;
    }
};
```

### Your notes
<!-- -->


---

## Unions and Tagged Unions

This is where Zig gets interesting. A **tagged union** is a type that can hold one of several types, with a tag that tells you which one is active. Think TypeScript discriminated unions, but enforced at the language level.

```zig
const Value = union(enum) {
    integer: i64,
    float: f64,
    boolean: bool,
    none,         // no payload

    pub fn isNumeric(self: Value) bool {
        return switch (self) {
            .integer, .float => true,
            .boolean, .none => false,
        };
    }
};

const v = Value{ .integer = 42 };
```

The `switch` on a tagged union must be **exhaustive** — you must handle every variant or use `else`. The compiler enforces this. This is similar to Rust's `match` on enums and TypeScript's exhaustive switch patterns.

| | TS discriminated union | Rust enum | Zig tagged union |
|---|---|---|---|
| Enforcement | Runtime (type narrowing) | Compile time (exhaustive match) | Compile time (exhaustive switch) |
| Payload | Object fields | Enum variants | Union fields |
| Type safety | Optional (type guard) | Mandatory | Mandatory |

### Your notes
<!-- -->


---

## Optional Types

Zig has no null. Instead, it has **optional types** — a value that is either the thing or `null`.

```zig
var maybe: ?i32 = 42;    // ?i32 means "optional i32"
maybe = null;             // now it's null

// To use the value, you must unwrap it
if (maybe) |value| {
    // value is i32 here — guaranteed non-null
    _ = value;
}

// Or use orelse for a default
const result = maybe orelse 0;   // 0 if null, the value otherwise

// Dangerous: .? force-unwraps (panics if null, like ! in Swift)
// const dangerous = maybe.?;    // would panic at runtime since maybe is null
```

This is similar to:
- TypeScript's `T | undefined` with optional chaining
- Rust's `Option<T>`
- Go's pointer-as-optional pattern (where `*int` being nil means "no value")

But Zig's version is more explicit than Go's and lighter-weight than Rust's. The `?T` syntax is built into the language, and `if (optional) |val|` is the idiomatic way to unwrap.

### Your notes
<!-- -->


---

## Error Unions (Preview)

This will be covered in depth in the error-handling module, but you'll see it everywhere in Zig code, so here's the short version.

Zig uses **error unions** instead of exceptions, Result types, or Go's `(value, error)` tuples:

```zig
// This function can return an error OR a u64
fn parseNumber(input: []const u8) !u64 {
    return std.fmt.parseInt(u64, input, 10) catch |err| return err;
}

// The ! means "this can fail"
// The return type is actually: error{...}!u64
```

The `!` in a return type means the function can fail. Callers must handle the error with `try` (propagate) or `catch` (handle):

```zig
const value = try parseNumber("42");     // propagate error to caller
const safe = parseNumber("???") catch 0; // handle error with default
```

This is Zig's equivalent of:
- Go's `value, err := ...` then `if err != nil`
- Rust's `Result<T, E>` with `?` operator
- TypeScript's try/catch

We'll dig into error sets, error unions, and error handling patterns in the dedicated module.

### Your notes
<!-- -->


---

## Comptime (Preview)

One of Zig's unique features is **comptime** — the ability to run arbitrary code at compile time. This replaces generics, macros, and code generation in other languages.

```zig
// comptime parameter — resolved at compile time
fn multiply(comptime T: type, a: T, b: T) T {
    return a * b;
}

const result = multiply(f64, 3.0, 4.0);   // compiler generates f64 multiplication
const other = multiply(i32, 3, 4);          // compiler generates i32 multiplication
```

When you saw `comptime_int` earlier for integer literals — that's this system at work. The literal `42` exists at compile time with infinite precision. The compiler evaluates expressions involving `comptime_int` at compile time and only materializes a concrete type when needed at runtime.

We'll cover comptime in depth in its own module. For now, just know it exists and that it's why you see `comptime` and `@`-prefixed builtins throughout Zig code.

### Your notes
<!-- -->


---

## Undefined and Zero Initialization

Zig has two ways to initialize variables without a meaningful value:

```zig
var buffer: [1024]u8 = undefined;       // uninitialized — contains garbage
var zeroed: [1024]u8 = std.mem.zeroes([1024]u8);  // all zeros
var also_zeroed = [_]u8{0} ** 1024;     // also all zeros (repeat operator)
```

`undefined` is Zig's way of saying "I don't care what's in this memory yet." This is dangerous — reading from `undefined` memory is undefined behavior. But it's also efficient when you're about to fill a buffer anyway.

Compare to Go's zero values: Go **always** zero-initializes. A `var x int` is `0`, always. In Zig, you can opt into undefined for performance, but you take on the responsibility of initializing before reading.

| | JS/TS | Go | Zig |
|---|---|---|---|
| Default uninitialized | `undefined` | Zero value (always safe) | Must choose: value, zeroes, or `undefined` |
| Can you skip init? | Sort of (`let x;`) | No — always zeroed | Yes — `undefined` (unsafe) |
| Safety | Runtime errors | Always safe | Your responsibility |

This is the systems programming tradeoff: more control, more responsibility.

### Your notes
<!-- -->


---

## Putting It Together — Running Zig Code

Let's verify everything works. Here's a small program that exercises the types we've covered:

```zig
const std = @import("std");

const Sensor = struct {
    name: []const u8,
    value: f64,
    status: Status,

    const Status = enum {
        active,
        inactive,
        error_state,
    };

    pub fn format(self: Sensor) void {
        std.debug.print("{s}: {d:.2} [{s}]\n", .{
            self.name,
            self.value,
            @tagName(self.status),
        });
    }
};

pub fn main() void {
    // const and var
    const sensor = Sensor{
        .name = "temperature",
        .value = 23.5,
        .status = .active,
    };

    var reading: f64 = sensor.value;
    reading += 1.5;

    // Optional
    var maybe_threshold: ?f64 = null;
    maybe_threshold = 30.0;

    const threshold = maybe_threshold orelse 25.0;

    // Array and slice
    const readings = [_]f64{ 20.1, 22.3, 23.5, 25.0, reading };
    const recent = readings[2..]; // slice of last 3

    sensor.format();
    std.debug.print("threshold: {d:.1}\n", .{threshold});
    std.debug.print("recent readings: {d}\n", .{recent.*});
}
```

Save this as `main.zig` and run with `zig run main.zig`.

### Your notes
<!-- -->
