---
title: Variables and Types (Cross-Language Comparison)
category: fundamentals
tags: [variables, types, memory, primitives, type-systems]
languages: [go, typescript, rust, python, scala, csharp, zig, haskell]
status: not-started
created: 2026-02-08
updated: 2026-02-12
related: [[memory-and-ownership]], [[control-flow]], [[functions-and-closures]]
---

# Variables and Types

> **Language-Specific Deep Dives:**
> - [[go/variables-and-types]] — Go: escape analysis, zero values, string internals
> - [[typescript/variables-and-types]] — TypeScript: type erasure, var/let/const
> - [[rust/variables-and-types]] — Rust: ownership, borrowing, move semantics
> - [[python/variables-and-types]] — Python: duck typing, object model
> - [[scala/variables-and-types]] — Scala: unified type hierarchy, case classes, Option
> - [[csharp/variables-and-types]] — C#: value vs reference types, nullable
> - [[zig/variables-and-types]] — Zig: comptime, no hidden allocations, explicit memory
> - [[haskell/variables-and-types]] — Haskell: immutability, algebraic data types, type inference

---

## Overview

Variables are named storage locations that hold values. The way variables work varies dramatically across languages — from Go's explicit stack/heap distinction to Python's "everything is an object" model to Rust's ownership system to Haskell's pure immutability. Understanding these differences is key to writing idiomatic code in each language.

## Core Concepts

### What is a Variable?

At the lowest level, a variable is:
1. **A name** (compile-time label)
2. **A memory location** (address where data lives)
3. **A type** (determines size, operations, and semantics)

Different languages expose these layers differently:
- **Go/C#/Rust/Zig**: All three layers visible
- **Scala**: JVM manages memory, but type system is rich and expressive
- **TypeScript**: Type system is compile-time only (erased at runtime)
- **Python**: Types are on objects, not variables (duck typing)
- **Haskell**: Variables are immutable bindings; types are inferred and enforced at compile time

### Type Systems

| Language | Typing Discipline | Checked When | Conversion |
|---|---|---|---|
| Go | Static, structural (interfaces), strong | Compile time | Explicit |
| TypeScript | Static, structural, strong | Compile time (erased) | Implicit widening |
| Rust | Static, nominal, strong | Compile time | Explicit |
| Python | Dynamic, duck typing | Runtime | Implicit |
| Scala | Static, nominal + structural, strong | Compile time | Explicit (with implicit conversions) |
| C# | Static, nominal, strong | Compile time | Explicit (with implicit conversions) |
| Zig | Static, nominal, strong | Compile time + comptime | Explicit |
| Haskell | Static, nominal, strong | Compile time | Explicit (type classes) |

**Key Insight:** Static typing catches errors before runtime but requires more ceremony. Dynamic typing is flexible but defers errors to runtime. Most modern languages (Go, Rust, C#, Scala) add type inference to reduce ceremony while keeping safety. Haskell has perhaps the most powerful type inference (Hindley-Milner).

### Memory Models

| Language | Stack/Heap | Value vs Reference | Who Decides |
|---|---|---|---|
| Go | Both | Escape analysis | Compiler |
| TypeScript | N/A (JS runtime) | Primitives copied, objects referenced | Runtime |
| Rust | Both | Explicit (ownership) | Programmer + Compiler |
| Python | Heap (all objects) | Everything is a reference | Runtime (GC) |
| Scala | Both (JVM) | Primitives (stack), Objects (heap) | JVM |
| C# | Both | Value types (stack), Reference types (heap) | CLR |
| Zig | Both | Explicit | Programmer (no hidden allocations) |
| Haskell | Heap (lazily evaluated thunks) | Everything is a thunk/reference | Runtime (GC) |

**Key Insight:** Languages with manual control (Go, Rust, C#, Zig) give you performance levers. Languages with automatic management (Python, TypeScript, Haskell) prioritize developer ergonomics over control. Scala and C# sit in the middle — managed runtime but with value types available.

---

## Language Comparison

### Variable Declaration

| Language | Syntax | Type Inference |
|---|---|---|
| Go | `var x int = 42` or `x := 42` | Yes (`:=`) |
| TypeScript | `let x: number = 42` or `const x = 42` | Yes |
| Rust | `let x: i32 = 42` or `let x = 42` | Yes |
| Python | `x = 42` | N/A (dynamically typed) |
| Scala | `val x: Int = 42` or `val x = 42` | Yes |
| C# | `int x = 42` or `var x = 42` | Yes (`var`) |
| Zig | `const x: i32 = 42` or `var x: i32 = 42` | Partial (must annotate when ambiguous) |
| Haskell | `x :: Int; x = 42` or `x = 42` | Yes (Hindley-Milner) |

### Mutability

| Language | Default | Mutable | Immutable |
|---|---|---|---|
| Go | Mutable | `var x = 42` | `const x = 42` (compile-time only) |
| TypeScript | Depends | `let x = 42` | `const x = 42` (binding only) |
| Rust | Immutable | `let mut x = 42` | `let x = 42` |
| Python | Mutable | `x = 42` | N/A (convention only) |
| Scala | Depends | `var x = 42` | `val x = 42` |
| C# | Mutable | `int x = 42` | `const int X = 42` / `readonly` |
| Zig | Depends | `var x: i32 = 42` | `const x: i32 = 42` |
| Haskell | Immutable | N/A (use IORef/STRef) | `x = 42` (always) |

**Key Insight:** Rust, Scala, and Haskell default to immutable — you opt in to mutation. Go, Python, and C# default to mutable. This reflects different philosophies about state management and safety.

### Primitives

#### Integers

| Language | Types Available | Default Size | Overflow Behavior |
|---|---|---|---|
| Go | `int`, `int8/16/32/64`, `uint`, `uint8/16/32/64` | Platform-dependent (`int`) | Wraps (unchecked) |
| TypeScript | `number` (IEEE 754 double), `bigint` | 64-bit float | Wraps (for bitwise) |
| Rust | `i8/16/32/64/128`, `u8/16/32/64/128`, `isize`, `usize` | `i32` (default) | Panic (debug), wrap (release) |
| Python | `int` (arbitrary precision) | Unlimited | Never overflows |
| Scala | `Byte`, `Short`, `Int`, `Long` | `Int` (32-bit) | Wraps |
| C# | `sbyte`, `byte`, `short`, `ushort`, `int`, `uint`, `long`, `ulong` | `int` (32-bit) | Wraps (unchecked) or throws (checked) |
| Zig | `i8`-`i128`, `u8`-`u128`, `usize`, `isize` | N/A (must specify) | Safety-checked (panic in safe, undefined in unsafe) |
| Haskell | `Int` (fixed), `Integer` (arbitrary) | `Integer` in literals | `Int` wraps, `Integer` never overflows |

**Key Insight:** Python and Haskell's `Integer` never overflow. Rust and Zig panic on overflow in debug/safe modes. Go/Scala/C# wrap silently by default.

#### Floating Point

| Language | Types | Default | Notes |
|---|---|---|---|
| Go | `float32`, `float64` | `float64` | IEEE 754 |
| TypeScript | `number` | 64-bit double | No separate int/float |
| Rust | `f32`, `f64` | `f64` | IEEE 754, no NaN comparison |
| Python | `float` | 64-bit double | IEEE 754 |
| Scala | `Float`, `Double` | `Double` | IEEE 754, JVM |
| C# | `float`, `double`, `decimal` | `double` | `decimal` for financial |
| Zig | `f16`, `f32`, `f64`, `f80`, `f128` | N/A (must specify) | IEEE 754 |
| Haskell | `Float`, `Double` | `Double` | IEEE 754 |

**Unique:** C#'s `decimal` type for exact decimal arithmetic (financial calculations). Zig offers f16 and f128.

### Strings

| Language | Mutability | Encoding | Representation |
|---|---|---|---|
| Go | Immutable | UTF-8 | Header (ptr + len) |
| TypeScript | Immutable | UTF-16 | JS primitive |
| Rust | Immutable (`&str`) / Owned (`String`) | UTF-8 | Slice (`&str`) or heap (`String`) |
| Python | Immutable | Unicode (flexible internal) | Object (interned literals) |
| Scala | Immutable | UTF-16 | Java String (interned) |
| C# | Immutable | UTF-16 | Object (interned literals) |
| Zig | `[]const u8` (byte slice) | UTF-8 (by convention) | Pointer + length |
| Haskell | Immutable (`String` = `[Char]`) | Unicode | Linked list of chars / `Text` for efficiency |

**Key Insight:** All modern languages use immutable strings by default. Rust's split between `&str` (borrowed slice) and `String` (owned) is unique. Zig treats strings as plain byte slices. Haskell's default `String` is a linked list (use `Text` in practice).

### Null/None/Nil

| Language | Representation | Type System Integration |
|---|---|---|
| Go | `nil` (for pointers, slices, maps, channels) | No type-level null safety |
| TypeScript | `null`, `undefined` | Opt-in (`strictNullChecks`) |
| Rust | `None` (in `Option<T>`) | Built-in via `Option` |
| Python | `None` | Runtime checks only |
| Scala | `None` (in `Option[A]`) | Built-in via `Option` |
| C# | `null` (reference types) | Opt-in (`nullable` reference types, C# 8+) |
| Zig | `null` (for optionals) | Built-in via `?T` optional type |
| Haskell | `Nothing` (in `Maybe a`) | Built-in via `Maybe` |

**Key Insight:** Rust's `Option<T>`, Scala's `Option[A]`, Haskell's `Maybe a`, and Zig's `?T` are the gold standard — null is not a value but a type-level concept. TypeScript and C# have opt-in null safety, but it's a retrofit.

---

## Idioms

### Go

```go
// Short declaration (idiomatic for locals)
count := 42

// Zero values are useful (no nil panics)
var m map[string]int
_ = m["key"]  // returns zero value, doesn't panic

// Escape analysis — return pointer is OK
func newCounter() *int {
    x := 42
    return &x  // escapes to heap automatically
}
```

### TypeScript

```typescript
// const by default, let when reassignment needed
const user = { name: "Alice" };
let count = 0;

// Type annotations when inference isn't enough
const users: User[] = [];

// Null safety with optional chaining
const name = user?.profile?.name;
```

### Rust

```rust
// Immutable by default
let x = 42;
let mut y = 10;  // explicitly mutable

// String slices vs owned strings
let borrowed: &str = "hello";
let owned: String = String::from("hello");

// Pattern matching on Option (no null)
match value {
    Some(v) => println!("{}", v),
    None => println!("nothing"),
}
```

### Python

```python
# Dynamic typing — check with isinstance() or duck typing
x = 42
if isinstance(x, int):
    print(x + 1)

# Type hints for documentation/tooling (not enforced)
def greet(name: str) -> str:
    return f"Hello, {name}"
```

### Scala

```scala
// val (immutable) by default
val count = 42
var mutable = 10  // explicitly mutable

// Option instead of null
val name: Option[String] = Some("Alice")
name.getOrElse("Unknown")

// Case classes for data
case class Config(host: String, port: Int, retries: Int = 3)
val cfg = Config("localhost", 8080)
```

### C#

```csharp
// var for type inference
var count = 42;
var name = "Alice";

// Nullable value types
int? maybeCount = null;
int count = maybeCount ?? 0;  // null-coalescing

// Value types vs reference types
struct Point { ... }  // value type (copied)
class Person { ... }  // reference type (shared)

// String interpolation
var message = $"Hello, {name}!";
```

### Zig

```zig
// const by default, var for mutable
const count: u32 = 42;
var mutable: u32 = 10;

// Optional types instead of null
const maybe_value: ?u32 = 42;
if (maybe_value) |value| {
    std.debug.print("got: {}\n", .{value});
}

// No hidden allocations — strings are byte slices
const greeting: []const u8 = "hello";
```

### Haskell

```haskell
-- All bindings are immutable
count :: Int
count = 42

-- Maybe instead of null
lookupUser :: String -> Maybe User
lookupUser name = case findUser name of
    Just user -> Just user
    Nothing   -> Nothing

-- Type inference is powerful (annotations optional)
double x = x * 2  -- inferred: Num a => a -> a
```

---

## Common Pitfalls

### Go: Nil Map Panic

See [[go-nil-map-panic]]

```go
var m map[string]int
m["key"] = 42  // PANIC — nil map
```

**Fix:** Always use `make()` for maps.

### TypeScript: Implicit any

```typescript
function process(data) {  // implicitly `any`
    return data.foo();    // no type checking
}
```

**Fix:** Enable `noImplicitAny` in `tsconfig.json`.

### Rust: Moving Out of Borrowed Context

```rust
let s = String::from("hello");
let borrowed = &s;
drop(s);          // ERROR — can't move while borrowed
println!("{}", borrowed);
```

**Fix:** Understand ownership rules — can't drop while borrowed references exist.

### Python: Mutable Default Arguments

```python
def append_to(element, target=[]):
    target.append(element)
    return target

append_to(1)  # [1]
append_to(2)  # [1, 2] — same list!
```

**Fix:** Use `None` as default and create new list inside function.

### Scala: Null From Java Interop

```scala
val result: String = javaMethod()  // might return null!
result.length  // NullPointerException
```

**Fix:** Wrap Java interop results in `Option(result)` which converts null to None.

### C#: Unboxing Null

```csharp
int? wrapped = null;
int x = (int)wrapped;  // throws InvalidOperationException
```

**Fix:** Check `HasValue` before unboxing nullable types, or use `??` operator.

### Zig: Ignoring Optional

```zig
const maybe: ?u32 = null;
const value = maybe.?;  // PANIC at runtime
```

**Fix:** Always use `if` or `orelse` to safely unwrap optionals.

### Haskell: Partial Functions

```haskell
head []  -- throws exception at runtime!
```

**Fix:** Use pattern matching or safe alternatives like `listToMaybe`.

---

## Related Patterns

- [[memory-and-ownership]] — Deep dive into stack/heap, ownership, borrowing
- [[control-flow]] — How variables interact with conditionals and loops
- [[functions-and-closures]] — Variable scope and lifetime
- [[interfaces-and-traits]] — How types define contracts
- [[generics]] — Parameterizing code over types

---

## References

- [Go Specification - Variables](https://go.dev/ref/spec#Variables)
- [TypeScript Handbook - Variable Declarations](https://www.typescriptlang.org/docs/handbook/variable-declarations.html)
- [The Rust Reference - Variables](https://doc.rust-lang.org/reference/variables.html)
- [Python Data Model](https://docs.python.org/3/reference/datamodel.html)
- [Scala 3 Reference - Types](https://docs.scala-lang.org/scala3/reference/)
- [C# Language Specification - Variables](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/variables)
- [Zig Language Reference](https://ziglang.org/documentation/master/)
- [Haskell Report - Types](https://www.haskell.org/onlinereport/haskell2010/)
