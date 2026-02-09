---
title: Variables and Types (Cross-Language Comparison)
category: fundamentals
tags: [variables, types, memory, primitives, type-systems]
languages: [go, typescript, rust, python, java, csharp]
status: in-progress
completed: [go]
created: 2026-02-08
updated: 2026-02-08
related: [[memory-and-ownership]], [[control-flow]], [[functions-and-closures]]
---

# Variables and Types

> **Language-Specific Deep Dives:**
> - [[go/variables-and-types]] — Go: escape analysis, zero values, string internals
> - [[typescript/variables-and-types]] — TypeScript: type erasure, var/let/const
> - [[rust/variables-and-types]] — Rust: ownership, borrowing, move semantics
> - [[python/variables-and-types]] — Python: duck typing, object model
> - [[java/variables-and-types]] — Java: primitives vs objects, autoboxing
> - [[csharp/variables-and-types]] — C#: value vs reference types, nullable

---

## Overview

Variables are named storage locations that hold values. The way variables work varies dramatically across languages — from Go's explicit stack/heap distinction to Python's "everything is an object" model to Rust's ownership system. Understanding these differences is key to writing idiomatic code in each language.

## Core Concepts

### What is a Variable?

At the lowest level, a variable is:
1. **A name** (compile-time label)
2. **A memory location** (address where data lives)
3. **A type** (determines size, operations, and semantics)

Different languages expose these layers differently:
- **Go/C#/Rust**: All three layers visible
- **TypeScript**: Type system is compile-time only (erased at runtime)
- **Python**: Types are on objects, not variables (duck typing)

### Type Systems

| Language | Typing Discipline | Checked When | Conversion |
|---|---|---|---|
| Go | Static, structural (interfaces), strong | Compile time | Explicit |
| TypeScript | Static, structural, strong | Compile time (erased) | Implicit widening |
| Rust | Static, nominal, strong | Compile time | Explicit |
| Python | Dynamic, duck typing | Runtime | Implicit |
| Java | Static, nominal, strong | Compile time | Explicit (with autoboxing) |
| C# | Static, nominal, strong | Compile time | Explicit (with implicit conversions) |

**Key Insight:** Static typing catches errors before runtime but requires more ceremony. Dynamic typing is flexible but defers errors to runtime. Most modern languages (Go, Rust, C#) add type inference to reduce ceremony while keeping safety.

### Memory Models

| Language | Stack/Heap | Value vs Reference | Who Decides |
|---|---|---|---|
| Go | Both | Escape analysis | Compiler |
| TypeScript | N/A (JS runtime) | Primitives copied, objects referenced | Runtime |
| Rust | Both | Explicit (ownership) | Programmer + Compiler |
| Python | Heap (all objects) | Everything is a reference | Runtime (GC) |
| Java | Both | Primitives (stack), Objects (heap) | JVM |
| C# | Both | Value types (stack), Reference types (heap) | CLR |

**Key Insight:** Languages with manual control (Go, Rust, C#) give you performance levers. Languages with automatic management (Python, TypeScript) prioritize developer ergonomics over control.

---

## Language Comparison

### Variable Declaration

| Language | Syntax | Type Inference |
|---|---|---|
| Go | `var x int = 42` or `x := 42` | Yes (`:=`) |
| TypeScript | `let x: number = 42` or `const x = 42` | Yes |
| Rust | `let x: i32 = 42` or `let x = 42` | Yes |
| Python | `x = 42` | N/A (dynamically typed) |
| Java | `int x = 42` or `var x = 42` | Yes (`var`, Java 10+) |
| C# | `int x = 42` or `var x = 42` | Yes (`var`) |

### Primitives

#### Integers

| Language | Types Available | Default Size | Overflow Behavior |
|---|---|---|---|
| Go | `int`, `int8/16/32/64`, `uint`, `uint8/16/32/64` | Platform-dependent (`int`) | Wraps (unchecked) |
| TypeScript | `number` (IEEE 754 double), `bigint` | 64-bit float | Wraps (for bitwise) |
| Rust | `i8/16/32/64/128`, `u8/16/32/64/128`, `isize`, `usize` | `i32` (default) | Panic (debug), wrap (release) |
| Python | `int` (arbitrary precision) | Unlimited | Never overflows |
| Java | `byte`, `short`, `int`, `long` | `int` (32-bit) | Wraps |
| C# | `sbyte`, `byte`, `short`, `ushort`, `int`, `uint`, `long`, `ulong` | `int` (32-bit) | Wraps (unchecked) or throws (checked) |

**Key Insight:** Python's unlimited integers and Rust's panic-on-overflow (debug) are the safest. Go/Java/C# wrap silently by default.

#### Floating Point

| Language | Types | Default | Notes |
|---|---|---|---|
| Go | `float32`, `float64` | `float64` | IEEE 754 |
| TypeScript | `number` | 64-bit double | No separate int/float |
| Rust | `f32`, `f64` | `f64` | IEEE 754, no NaN comparison |
| Python | `float` | 64-bit double | IEEE 754 |
| Java | `float`, `double` | `double` | IEEE 754 |
| C# | `float`, `double`, `decimal` | `double` | `decimal` for financial (no rounding errors) |

**Unique:** C#'s `decimal` type for exact decimal arithmetic (financial calculations).

### Strings

| Language | Mutability | Encoding | Representation |
|---|---|---|---|
| Go | Immutable | UTF-8 | Header (ptr + len) |
| TypeScript | Immutable | UTF-16 | JS primitive |
| Rust | Immutable (`&str`) / Owned (`String`) | UTF-8 | Slice (`&str`) or heap (`String`) |
| Python | Immutable | Unicode (flexible internal) | Object (interned literals) |
| Java | Immutable | UTF-16 | Object (interned literals) |
| C# | Immutable | UTF-16 | Object (interned literals) |

**Key Insight:** All modern languages use immutable strings by default. Rust's split between `&str` (borrowed slice) and `String` (owned) is unique.

### Null/None/Nil

| Language | Representation | Type System Integration |
|---|---|---|---|
| Go | `nil` (for pointers, slices, maps, channels) | No type-level null safety |
| TypeScript | `null`, `undefined` | Opt-in (`strictNullChecks`) |
| Rust | `None` (in `Option<T>`) | Built-in via `Option` |
| Python | `None` | Runtime checks only |
| Java | `null` (reference types only) | No type-level safety |
| C# | `null` (reference types) | Opt-in (`nullable` reference types, C# 8+) |

**Key Insight:** Rust's `Option<T>` is the gold standard — null is not a value but a type-level concept. TypeScript and C# have opt-in null safety, but it's a retrofit.

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
// Dynamic typing — check with isinstance() or duck typing
x = 42
if isinstance(x, int):
    print(x + 1)

// Type hints for documentation/tooling (not enforced)
def greet(name: str) -> str:
    return f"Hello, {name}"
```

### Java

```java
// var for local type inference (Java 10+)
var count = 42;         // int
var name = "Alice";     // String

// Wrapper classes for generics (autoboxing)
List<Integer> numbers = new ArrayList<>();
numbers.add(42);        // autoboxing int → Integer

// final for immutable references
final String CONFIG_PATH = "/etc/config";
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

### Java: NullPointerException

```java
String s = null;
s.length();  // throws NullPointerException at runtime
```

**Fix:** Check for null or use `Optional<T>`.

### C#: Unboxing Null

```csharp
Integer wrapped = null;
int x = wrapped;  // throws NullPointerException
```

**Fix:** Check `HasValue` before unboxing nullable types.

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
- [Java Language Specification - Variables](https://docs.oracle.com/javase/specs/jls/se17/html/jls-4.html#jls-4.12)
- [C# Language Specification - Variables](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/variables)
