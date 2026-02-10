# Zig Track

## Why Zig?

Zig is a systems programming language that provides:
- **Manual memory management** with safety features
- **Comptime** — compile-time code execution for metaprogramming
- **C interop** — seamless integration with C code
- **No hidden control flow** — explicit is better than implicit
- **Minimal language** — small, understandable core

Zig complements Go (GC vs manual), TypeScript (high-level vs low-level), and Rust (different approach to safety).

## Learning Focus

**Systems programming perspective:**
- Memory allocators and allocation strategies
- Comptime metaprogramming
- Error handling without exceptions
- Building fast, small binaries
- C interop and FFI

**Key differences from other languages:**
- No hidden allocations (all memory allocation is explicit)
- No RAII (Resource Acquisition Is Initialization)
- Comptime instead of macros/generics
- Error unions instead of Result types or exceptions
- Build system is part of the language

## Getting Started

### Installation

```bash
# Install Zig
brew install zig  # macOS
# or download from https://ziglang.org/download/

# Verify installation
zig version
```

### Hello World

```zig
const std = @import("std");

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Hello, {s}!\n", .{"World"});
}
```

### Running Code

```bash
# Run directly
zig run main.zig

# Build executable
zig build-exe main.zig

# Run tests
zig test main.zig
```

## Module Structure

```
zig/
├── fundamentals/
│   ├── variables-and-types/
│   ├── control-flow/
│   ├── functions-and-closures/
│   ├── error-handling/
│   ├── memory-management/        # Core Zig concept
│   ├── comptime/                 # Core Zig concept
│   └── testing-fundamentals/
├── patterns/
│   ├── allocator-pattern/        # Zig-specific
│   ├── comptime-polymorphism/    # Zig-specific
│   └── ...
└── exercises/
    ├── debugging/
    ├── refactoring/
    ├── code-review/
    └── api-design/
```

## Resources

- [Zig Language Reference](https://ziglang.org/documentation/master/)
- [Zig Learn](https://ziglearn.org/)
- [Zig Standard Library](https://ziglang.org/documentation/master/std/)
- [Zig GitHub Discussions](https://github.com/ziglang/zig/discussions)

## Notes

Zig is pre-1.0, so expect some changes. This track focuses on core concepts that will remain stable.
