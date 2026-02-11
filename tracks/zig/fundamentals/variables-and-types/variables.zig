const std = @import("std");
const print = std.debug.print;

// ============================================================================
// Variables and Types — Zig
// ============================================================================
// Fill in each section. Run with: zig run variables.zig
// ============================================================================

pub fn main() void {
    // ========================================================================
    // 1. CONST vs VAR
    // Declare an immutable binding with `const` and a mutable one with `var`.
    // ========================================================================
    // a) a const i32
    // TODO

    // b) a var u8
    // TODO

    // c) reassign the var to a different value
    // TODO

    // d) try uncommenting this — what error do you get?
    // const immutable: i32 = 10;
    // immutable = 20;

    // Print them:
    // print("a = {d}, b = {d}\n", .{ a, b });

    // ========================================================================
    // 2. TYPE INFERENCE
    // Declare variables without explicit types. What does Zig infer?
    // ========================================================================
    // a) const inferred from an integer literal — what type is this?
    // TODO

    // b) const with explicit type
    // TODO

    // c) why does this fail? (uncomment to test, then re-comment)
    // var x = 42;   // hint: comptime_int can't exist at runtime without a type
    // _ = x;

    // ========================================================================
    // 3. INTEGER TYPES AND OVERFLOW
    // Zig catches overflow at compile time for comptime values,
    // and at runtime (in safe mode) for var values.
    // ========================================================================
    // a) this works — 255 fits in u8
    // const fits: u8 = 255;

    // b) uncomment this — what error do you get?
    // const overflow: u8 = 256;

    // c) wrapping arithmetic — what value does this produce?
    // var wrap: u8 = 255;
    // wrap +%= 1;
    // print("wrapped: {d}\n", .{wrap});

    // d) saturating arithmetic — what value does this produce?
    // var sat: u8 = 250;
    // sat +|= 10;
    // print("saturated: {d}\n", .{sat});

    // ========================================================================
    // 4. EXPLICIT CASTING
    // Zig never implicitly converts between types. You must use builtins.
    // ========================================================================
    // a) widen a u8 to a u32 — which builtin do you use?
    // const small: u8 = 42;
    // TODO: const big: u32 = ???

    // b) convert an integer to a float
    // const int_val: i32 = 100;
    // TODO: const float_val: f64 = ???

    // c) convert a float to an integer (truncates)
    // const pi: f64 = 3.14;
    // TODO: const truncated: i32 = ???

    // ========================================================================
    // 5. STRINGS AND SLICES
    // Strings in Zig are just byte slices: []const u8
    // ========================================================================
    // a) declare a string literal
    // TODO

    // b) get its length
    // print("length: {d}\n", .{greeting.len});

    // c) access a single byte and print it as a character
    // print("first byte: {c}\n", .{greeting[0]});

    // d) take a slice of the string (e.g., first 5 bytes)
    // const sub = greeting[0..5];
    // print("slice: {s}\n", .{sub});

    // ========================================================================
    // 6. ARRAYS
    // Fixed-size, value type. Copying an array copies all elements.
    // ========================================================================
    // a) declare an array of 5 i32 values
    // TODO

    // b) print its length
    // print("array len: {d}\n", .{arr.len});

    // c) copy the array to a new variable and modify element 0.
    //    does the original change? why or why not?
    // TODO

    // ========================================================================
    // 7. STRUCTS
    // The primary composite type in Zig. Like Go structs, no classes.
    // ========================================================================
    const Config = struct {
        port: u16 = 8080,
        debug: bool = false,
        name: []const u8 = "default",
    };

    // a) create a Config using all defaults
    // TODO

    // b) create a Config overriding just the port
    // TODO

    // c) print the config
    // print("port={d}, debug={}, name={s}\n", .{ cfg.port, cfg.debug, cfg.name });

    // ========================================================================
    // 8. ENUMS
    // Named set of values. Can have methods.
    // ========================================================================
    const LogLevel = enum {
        debug,
        info,
        warn,
        err,

        pub fn isImportant(self: LogLevel) bool {
            return switch (self) {
                .debug, .info => false,
                .warn, .err => true,
            };
        }
    };

    // a) create a LogLevel value using the shorthand .syntax
    // TODO

    // b) call the method on it
    // print("is important? {}\n", .{level.isImportant()});

    // c) get the tag name as a string
    // print("level name: {s}\n", .{@tagName(level)});

    // ========================================================================
    // 9. OPTIONALS
    // ?T means "T or null". You must unwrap before using.
    // ========================================================================
    // a) declare an optional i32 with a value
    // TODO

    // b) set it to null
    // TODO

    // c) unwrap with orelse
    // const value = maybe orelse 0;

    // d) unwrap with if-capture
    // if (maybe) |val| {
    //     print("got: {d}\n", .{val});
    // } else {
    //     print("was null\n", .{});
    // }

    // ========================================================================
    // 10. TAGGED UNIONS
    // A value that can be one of several types, with compile-time
    // exhaustiveness checking.
    // ========================================================================
    const Token = union(enum) {
        number: f64,
        string: []const u8,
        boolean: bool,
        null_val,
    };

    // a) create a Token holding a number
    // TODO

    // b) switch on it and print the value
    // switch (tok) {
    //     .number => |n| print("number: {d}\n", .{n}),
    //     .string => |s| print("string: {s}\n", .{s}),
    //     .boolean => |b| print("bool: {}\n", .{b}),
    //     .null_val => print("null\n", .{}),
    // }

    print("done\n", .{});
}
