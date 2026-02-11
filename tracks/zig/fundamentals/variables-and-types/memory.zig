const std = @import("std");
const print = std.debug.print;

// ============================================================================
// Memory and Types — Zig
// ============================================================================
// Explore how Zig lays out data in memory.
// Run with: zig run memory.zig
// ============================================================================

pub fn main() void {
    // ========================================================================
    // 1. SIZES OF TYPES
    // Use @sizeOf to check how big each type is in bytes.
    // Compare with your expectations from the lesson.
    // ========================================================================

    print("=== Type Sizes ===\n", .{});
    print("u8:     {d} bytes\n", .{@sizeOf(u8)});
    print("u16:    {d} bytes\n", .{@sizeOf(u16)});
    print("u32:    {d} bytes\n", .{@sizeOf(u32)});
    print("u64:    {d} bytes\n", .{@sizeOf(u64)});
    print("i32:    {d} bytes\n", .{@sizeOf(i32)});
    print("f32:    {d} bytes\n", .{@sizeOf(f32)});
    print("f64:    {d} bytes\n", .{@sizeOf(f64)});
    print("bool:   {d} bytes\n", .{@sizeOf(bool)});
    print("usize:  {d} bytes\n", .{@sizeOf(usize)});

    // Q: Why is bool 1 byte and not 1 bit?
    //    (answer here)

    // Q: What is usize on your machine? Why that size?
    //    (answer here)

    // ========================================================================
    // 2. SLICES ARE FAT POINTERS
    // A slice is a pointer + length. Check its size.
    // ========================================================================

    const greeting: []const u8 = "hello";

    print("\n=== Slice Layout ===\n", .{});
    print("[]const u8 size: {d} bytes\n", .{@sizeOf([]const u8)});
    print("slice.len:       {d}\n", .{greeting.len});
    print("slice.ptr:       {*}\n", .{greeting.ptr});

    // Q: Why is []const u8 the same size as two pointers (16 bytes on 64-bit)?
    //    How does this compare to Go's string header?
    //    (answer here)

    // ========================================================================
    // 3. ARRAYS ARE VALUE TYPES
    // Assigning an array copies the entire thing.
    // ========================================================================

    var a = [3]i32{ 10, 20, 30 };
    var b = a; // copy

    b[0] = 999;

    print("\n=== Array Value Semantics ===\n", .{});
    print("a[0] = {d}\n", .{a[0]});
    print("b[0] = {d}\n", .{b[0]});

    // Q: a[0] is still 10. Why?
    //    How is this the same as Go structs? How is it different from JS arrays?
    //    (answer here)

    // ========================================================================
    // 4. STRUCT LAYOUT
    // Zig structs have a defined size. Check the size and see
    // if padding is added for alignment.
    // ========================================================================

    const Compact = struct {
        a: u8,
        b: u8,
    };

    const Padded = struct {
        a: u8,
        b: u64,
    };

    const Reordered = struct {
        a: u8,
        b: u64,
        c: u8,
    };

    print("\n=== Struct Sizes ===\n", .{});
    print("Compact  (u8, u8):      {d} bytes\n", .{@sizeOf(Compact)});
    print("Padded   (u8, u64):     {d} bytes\n", .{@sizeOf(Padded)});
    print("Reordered (u8, u64, u8): {d} bytes\n", .{@sizeOf(Reordered)});

    // Q: Why is Padded bigger than 9 bytes (1 + 8)?
    //    The u64 needs 8-byte alignment, so the compiler inserts padding
    //    after the u8 to align the u64. This is the same concept as in C/Go.
    //    (write your understanding here)

    // Q: Is Reordered bigger or smaller than you'd expect?
    //    Hint: Zig is allowed to reorder struct fields for optimal layout
    //    (unlike C, which must preserve declaration order).
    //    (answer here)

    // ========================================================================
    // 5. OPTIONAL SIZE
    // ?T uses a flag bit. Check how big optionals are.
    // ========================================================================

    print("\n=== Optional Sizes ===\n", .{});
    print("i32:    {d} bytes\n", .{@sizeOf(i32)});
    print("?i32:   {d} bytes\n", .{@sizeOf(?i32)});
    print("*i32:   {d} bytes\n", .{@sizeOf(*i32)});
    print("?*i32:  {d} bytes\n", .{@sizeOf(?*i32)});

    // Q: ?i32 is bigger than i32 — where does the extra space come from?
    //    (answer here)

    // Q: ?*i32 is the SAME size as *i32. Why?
    //    Hint: what value can a pointer never be?
    //    (answer here)

    // ========================================================================
    // 6. ENUM AND TAGGED UNION SIZE
    // Enums are just integers. Tagged unions are the tag + the largest variant.
    // ========================================================================

    const Direction = enum { north, south, east, west };
    const SmallEnum = enum(u8) { a, b, c };

    const Value = union(enum) {
        integer: i64,
        float: f64,
        boolean: bool,
        none,
    };

    print("\n=== Enum and Union Sizes ===\n", .{});
    print("Direction (4 variants):  {d} bytes\n", .{@sizeOf(Direction)});
    print("SmallEnum (u8 backing):  {d} bytes\n", .{@sizeOf(SmallEnum)});
    print("Value (tagged union):    {d} bytes\n", .{@sizeOf(Value)});

    // Q: Why is Value bigger than the largest variant (i64 = 8 bytes)?
    //    What's the extra space for?
    //    (answer here)

    // ========================================================================
    // 7. UNDEFINED vs ZEROED
    // ========================================================================

    var zeroed: [8]u8 = std.mem.zeroes([8]u8);
    // var undef: [8]u8 = undefined;  // dangerous — would contain garbage

    print("\n=== Zero Init ===\n", .{});
    print("zeroed: ", .{});
    for (zeroed) |byte| {
        print("{d} ", .{byte});
    }
    print("\n", .{});

    // Q: In Go, all variables are zero-initialized automatically.
    //    In Zig, you must choose: explicit value, zeroes, or undefined.
    //    When would you use `undefined` instead of zeroes?
    //    (answer here)

    // Prevent unused variable errors
    _ = &a;
    _ = &b;
}
