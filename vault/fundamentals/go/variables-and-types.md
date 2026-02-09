---
title: Variables and Types (Go Deep Dive)
category: fundamentals
tags: [variables, types, memory, stack, heap, pointers, zero-values, escape-analysis]
languages: [go]
status: completed
created: 2026-02-07
updated: 2026-02-08
related: [[fundamentals/variables-and-types]], [[typescript/variables-and-types]], [[rust/variables-and-types]], [[memory-and-ownership]]
---

> **Cross-Language Comparison:** See [[fundamentals/variables-and-types]] for how Go compares to TypeScript, Rust, Python, Java, and C#.

---

# Variables and Types (Go Deep Dive)

## Overview

Variables in Go are statically typed, explicitly declared, and backed by real memory with predictable layout. Go's type system is static, structural (for interfaces), and strong -- no implicit conversions between types, even between `int` and `int64`. Understanding how variables map to memory (stack vs heap, escape analysis, string internals) is foundational for writing performant Go.

## Core Concepts

### Declaration Forms

- **`var x int = 42`** -- explicit type and value
- **`var x int`** -- explicit type, initialized to zero value
- **`x := 42`** -- short declaration, type inferred (functions only)
- **`var ( ... )`** -- grouped declarations for multiple variables

Short declarations (`:=`) can redeclare variables in the same block if at least one variable in the statement is new.

### Memory Model: Stack vs Heap

Go decides allocation via **escape analysis** at compile time:
- **Stack**: fast, cleaned up automatically when the function returns
- **Heap**: slower (GC-managed), value survives after the function returns

A variable escapes to the heap when the compiler can't prove it stays local -- returning a pointer, storing it in an interface, passing it somewhere that outlives the scope.

```go
func stackAlloc() int {
    x := 42    // stays on stack -- returned by value (copy)
    return x
}

func heapAlloc() *int {
    x := 42    // escapes to heap -- pointer outlives function
    return &x
}
```

Inspect with: `go build -gcflags="-m" .`

### Type System: Static, Structural, Strong

- Types checked at compile time (static)
- Interfaces are structural -- no `implements` keyword needed
- No implicit conversions: `int` and `int64` are distinct types requiring explicit cast

```go
var a int = 42
var b int64 = int64(a)  // explicit conversion required
```

### Primitive Types and Sizes

| Type | Size | Zero Value | Notes |
|---|---|---|---|
| `bool` | 1 byte | `false` | |
| `int` | 8 bytes (64-bit) | `0` | Platform-dependent |
| `int8/16/32/64` | 1/2/4/8 bytes | `0` | Fixed-width |
| `float32/64` | 4/8 bytes | `0.0` | IEEE 754 |
| `string` | 16 bytes | `""` | Header: pointer (8) + length (8) |
| `byte` | 1 byte | `0` | Alias for `uint8` |
| `rune` | 4 bytes | `0` | Alias for `int32` |

### String Internals

A `string` is a two-word struct (pointer + length), not the character data itself:

```go
// Internal representation:
type stringHeader struct {
    Data uintptr  // pointer to byte array
    Len  int      // length in bytes
}
```

- The 16-byte header lives on the stack; character data lives elsewhere (read-only segment for literals, heap for constructed strings)
- Strings are **immutable** -- every modification creates a new string
- `len(s)` is O(1) -- reads the Len field, no iteration
- `len(s)` returns **bytes, not characters** -- matters for non-ASCII

### Zero Values

Every type has a usable zero value. No uninitialized memory.

| Type | Zero Value | Usable? |
|---|---|---|
| Numeric | `0` | Yes |
| `bool` | `false` | Yes |
| `string` | `""` | Yes |
| Pointer | `nil` | Dereference panics |
| Slice | `nil` | Yes -- `append` works on nil slices |
| Map | `nil` | Read OK, **write panics** |
| Channel | `nil` | Blocks forever |
| Interface | `nil` | Method call panics |
| Struct | All fields zeroed | Yes |

Compare with TypeScript: TS has `undefined` (declared but unassigned) and `null` (explicit absence). Go has no equivalent -- variables always have a concrete zero value. See [[variables-and-types-typescript]].

### Pointers

Go has pointers but no pointer arithmetic.

```go
x := 42
p := &x       // p is *int, holds address of x
fmt.Println(*p) // 42 -- dereference
*p = 100
fmt.Println(x)  // 100 -- x changed through pointer
```

- `&x` = address of x
- `*p` = value at address p
- `*int` = type: pointer to int

### Value Semantics vs Pointer Semantics

| | JS/TS | Go |
|---|---|---|
| Primitives | copied | copied |
| Objects/Structs | reference (always) | copied (default), reference (with `*`) |

Go makes you choose explicitly. Struct assignment copies the entire struct. To share, pass a pointer.

```go
type Config struct { Port int }

func updateByValue(c Config)  { c.Port = 9090 }   // caller unchanged
func updateByPointer(c *Config) { c.Port = 9090 } // caller sees change
```

### Constants and iota

```go
const maxRetries = 3         // untyped -- adapts to context
const port int = 8080        // typed -- locked to int

const (
    StatusPending = iota     // 0
    StatusActive             // 1
    StatusClosed             // 2
)
```

Untyped constants have arbitrary precision and adapt to their usage context. `const x = 1` can be used as `int`, `float64`, etc. without explicit conversion.

## Key Insights

> [!tip] `len()` on strings returns bytes, not characters
> `len("cafe\u0301")` might surprise you. For character counts, use `utf8.RuneCountInString(s)`. This is different from JS/TS where `.length` returns UTF-16 code units.

> [!tip] Escape analysis is observable
> Run `go build -gcflags="-m" .` to see exactly which variables escape to the heap. `fmt.Printf` with `%p` or interface args can itself cause escapes -- the value gets boxed into `interface{}`.

> [!tip] nil map vs nil slice -- different behavior
> A nil slice is safe to `append` to. A nil map panics on write. Both look "zero-valued" but have different safety profiles. Always `make` your maps.

> [!tip] Untyped constants are more flexible than typed ones
> `const x = 1` can be used anywhere an integer-compatible type is expected without conversion. `const x int = 1` is locked to `int`. Prefer untyped unless you need to enforce a specific type.

## Common Pitfalls

- **Writing to a nil map** -- `var m map[string]int; m["key"] = 1` panics. Must use `make`.
- **Assuming `len()` counts characters** -- it counts bytes. Use `utf8.RuneCountInString` for rune count.
- **Mixing numeric types** -- `int` and `int64` are distinct. The compiler won't convert for you.
- **Ignoring escape analysis** -- returning `&localVar` forces heap allocation. Not always bad, but worth understanding for hot paths.
- **Slice header copies share backing arrays** -- `s2 := s1` copies the header, not the data. Mutating `s2[0]` changes `s1[0]`. Use `copy()` or `slices.Clone()` for independence.

## Spec References

- [Variables](https://go.dev/ref/spec#Variables) -- storage locations and static vs dynamic types
- [Variable Declarations](https://go.dev/ref/spec#Variable_declarations) -- `var` syntax and initialization rules
- [Short Variable Declarations](https://go.dev/ref/spec#Short_variable_declarations) -- `:=` syntax and redeclaration rules
- [The Zero Value](https://go.dev/ref/spec#The_zero_value) -- default values for all types
- [Numeric Types](https://go.dev/ref/spec#Numeric_types) -- full table of int/float/complex types
- [String Types](https://go.dev/ref/spec#String_types) -- immutability, byte semantics, indexing
- [Constants](https://go.dev/ref/spec#Constants) -- untyped constants, `iota`, arbitrary precision
- [Pointer Types](https://go.dev/ref/spec#Pointer_types) -- pointer syntax and nil
- [Address Operators](https://go.dev/ref/spec#Address_operators) -- `&` and addressability rules

## Related Patterns

- [[variables-and-types-typescript]] -- compare zero values, structural typing, type erasure
- [[memory-and-ownership]] -- deeper dive into stack/heap, GC, and ownership (Rust comparison)
- [[interfaces-and-traits]] -- Go's structural interface matching
- [[functions-and-closures]] -- value vs pointer receivers on methods
- [[error-handling]] -- zero values play into Go's error patterns (`if err != nil`)
