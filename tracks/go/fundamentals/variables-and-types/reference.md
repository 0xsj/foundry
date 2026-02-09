# Go Specification Reference — Variables and Types

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> for the `variables-and-types` module. Covers: variable declarations, zero values,
> type system, numeric types, strings, composite types, constants, and pointers.

---

## Variables

Source: [spec#Variables](https://go.dev/ref/spec#Variables)

A **variable** is a storage location for holding a value. The set of permissible values
is determined by the variable's type.

- A variable declaration reserves storage for a named variable.
- Calling `new` or taking the address of a composite literal allocates storage at run time.
- Variables of array, slice, and struct types have elements and fields that may be
  addressed individually.
- A variable's value is retrieved by referring to the variable in an expression.
  If not yet assigned, its value is the **zero value** for its type.

### Static vs Dynamic Types

- The **static type** (or just "type") is the type given in its declaration.
- Variables of interface type also have a **dynamic type** — the concrete type of the
  value assigned at run time. The dynamic type may vary during execution.

```go
var x interface{}  // x is nil, static type interface{}
var v *T           // v has value nil, static type *T
x = 42             // x has value 42 and dynamic type int
x = v              // x has value (*T)(nil) and dynamic type *T
```

---

## Variable Declarations

Source: [spec#Variable_declarations](https://go.dev/ref/spec#Variable_declarations)

```go
var i int
var U, V, W float64
var k = 0
var x, y float32 = -1, -2
var (
    i       int
    u, v, s = 2.0, 3.0, "bar"
)
var re, im = complexSqrt(-1)
var _, found = entries[name]  // map lookup; only interested in "found"
```

### Initialization Rules

- If an expression list is given, variables are initialized by assignment.
- Otherwise, each variable is initialized to its **zero value**.
- If a type is present, each variable has that type.
- If no type is present, each variable takes the type of its initialization value.
- For untyped constants, they are first implicitly converted to their default type.
- `nil` cannot be used to initialize a variable with no explicit type.

```go
var d = math.Sin(0.5)  // d is float64
var i = 42             // i is int
var t, ok = x.(T)      // t is T, ok is bool
var n = nil            // illegal
```

---

## Short Variable Declarations

Source: [spec#Short_variable_declarations](https://go.dev/ref/spec#Short_variable_declarations)

Shorthand for a regular declaration with initializer but no type:

```go
i, j := 0, 10
f := func() int { return 7 }
ch := make(chan int)
r, w, _ := os.Pipe()
```

### Redeclaration

A short variable declaration may **redeclare** variables provided:
- They were originally declared in the same block (or parameter list)
- With the same type
- At least one of the non-blank variables is new

```go
field1, offset := nextField(str, 0)
field2, offset := nextField(str, offset)  // redeclares offset
x, y, x := 1, 2, 3                        // illegal: x repeated
```

Short declarations may appear only inside functions.

---

## Zero Values

Source: [spec#The_zero_value](https://go.dev/ref/spec#The_zero_value)

When storage is allocated for a variable and no explicit initialization is provided,
the variable is given a **zero value**:

| Type | Zero Value |
|---|---|
| `bool` | `false` |
| Numeric types | `0` |
| `string` | `""` (empty string) |
| Pointer, function, slice, map, channel | `nil` |
| Array | All elements set to zero value |
| Struct | All fields set to zero value |
| Interface | `nil` |

---

## Type System Overview

Source: [spec#Types](https://go.dev/ref/spec#Types)

A type determines a set of values together with operations and methods specific to
those values. Types can be denoted by a **type name** or specified using a **type
literal** that composes a type from existing types.

### Type Definitions (Named Types)

A type definition creates a **new, distinct type** with the same underlying type and
operations:

```go
type Point struct{ x, y float64 }  // Point and struct{ x, y float64 } are different
type polar Point                   // polar and Point are different types
type TimeZone int
```

### Type Aliases

An alias declaration binds an identifier to a type; they are identical:

```go
type nodeList = []*Node   // nodeList and []*Node are identical types
type Polar = polar        // Polar and polar denote identical types
```

### Type Identity

Two types are either **identical** or **different**. A named type is always different
from any other type. Otherwise, types are identical if their underlying type literals
are structurally equivalent:

```go
type (
    A0 = []string
    A1 = A0
    B0 A0       // B0 is a new type — different from A0 even though same underlying
    B1 []string // B1 is a new type — different from B0
)
// A0, A1, and []string are identical
// B0 and B1 are different (distinct type definitions)
```

---

## Boolean Type

Source: [spec#Boolean_types](https://go.dev/ref/spec#Boolean_types)

- Represents the set of Boolean truth values: `true` and `false`.
- The predeclared boolean type is `bool`.
- A defined boolean type is a distinct type.

---

## Numeric Types

Source: [spec#Numeric_types](https://go.dev/ref/spec#Numeric_types)

### Architecture-Independent Integer Types

| Type | Size | Range |
|---|---|---|
| `uint8` / `byte` | 1 byte | 0 to 255 |
| `uint16` | 2 bytes | 0 to 65,535 |
| `uint32` | 4 bytes | 0 to 4,294,967,295 |
| `uint64` | 8 bytes | 0 to 2^64 - 1 |
| `int8` | 1 byte | -128 to 127 |
| `int16` | 2 bytes | -32,768 to 32,767 |
| `int32` / `rune` | 4 bytes | -2,147,483,648 to 2,147,483,647 |
| `int64` | 8 bytes | -2^63 to 2^63 - 1 |

### Architecture-Dependent Types

| Type | Size | Notes |
|---|---|---|
| `uint` | 32 or 64 bits | Platform-dependent |
| `int` | same as `uint` | Platform-dependent |
| `uintptr` | pointer-width | Large enough to store uninterpreted bits of a pointer |

`byte` is an alias for `uint8`. `rune` is an alias for `int32`.

All numeric types are **defined types** and thus distinct. Explicit conversions are
required when different numeric types are mixed.

### Floating-Point Types

| Type | Size | Spec |
|---|---|---|
| `float32` | 4 bytes | IEEE 754 32-bit |
| `float64` | 8 bytes | IEEE 754 64-bit |

### Complex Types

| Type | Size | Components |
|---|---|---|
| `complex64` | 8 bytes | `float32` real + `float32` imaginary |
| `complex128` | 16 bytes | `float64` real + `float64` imaginary |

---

## String Type

Source: [spec#String_types](https://go.dev/ref/spec#String_types)

- A string value is a (possibly empty) sequence of bytes.
- Strings are **immutable**: once created, contents cannot be changed.
- `len(s)` returns the length in bytes (not characters/runes).
- `s[i]` accesses the byte at index `i`.
- `&s[i]` is **illegal** — cannot take the address of a string element.

---

## Array Types

Source: [spec#Array_types](https://go.dev/ref/spec#Array_types)

An array is a numbered sequence of elements of a single type, called the element type.
The number of elements is the array **length** and is never negative.

```go
[32]byte
[2*N] struct { x, y int32 }
[1000]*float64
[3][5]int
```

- The length is **part of the array's type** — `[5]int` and `[10]int` are different types.
- The length must evaluate to a non-negative constant representable as `int`.

---

## Slice Types

Source: [spec#Slice_types](https://go.dev/ref/spec#Slice_types)

A slice is a descriptor for a contiguous segment of an underlying array. It provides
access to a numbered sequence of elements from that array.

```go
[]T                          // slice of T
make([]T, length, capacity)  // create with make
```

- `len(s)` — current number of elements (may change during execution).
- `cap(s)` — maximum length the slice can grow to without reallocation.
- Distinct slices may share the same underlying array storage.
- An uninitialized slice has value `nil` (length and capacity zero).

---

## Struct Types

Source: [spec#Struct_types](https://go.dev/ref/spec#Struct_types)

A struct is a sequence of named elements called **fields**, each with a name and type.

```go
struct {}                           // empty struct (0 bytes)
struct {
    x, y int
    u float32
    _ float32                       // padding
    A *[]int
    F func()
}
```

### Embedded Fields

A field declared with a type but no explicit name is an **embedded field**. The
unqualified type name acts as the field name:

```go
struct {
    T1        // field name is T1
    *T2       // field name is T2
    P.T3      // field name is T3
}
```

### Field Tags

Tags are string literals that become attributes visible through reflection:

```go
struct {
    microsec  uint64 `protobuf:"1"`
    serverIP6 uint64 `protobuf:"2"`
}
```

---

## Pointer Types

Source: [spec#Pointer_types](https://go.dev/ref/spec#Pointer_types)

A pointer type denotes the set of all pointers to variables of a given type (the
**base type**). An uninitialized pointer has value `nil`.

```go
*Point
*[4]int
```

---

## Map Types

Source: [spec#Map_types](https://go.dev/ref/spec#Map_types)

An unordered group of elements of one type, indexed by unique keys of another type.

```go
map[string]int
map[*T]struct{ x, y float64 }
```

- Key type must support `==` and `!=` — cannot be function, map, or slice.
- An uninitialized map has value `nil`.
- Create with `make(map[K]V)` or `make(map[K]V, capacity_hint)`.

---

## Constants

Source: [spec#Constants](https://go.dev/ref/spec#Constants)

Constants represent **exact values of arbitrary precision** that do not overflow.

### Typed vs Untyped

- **Untyped constants**: literal values, `true`, `false`, `iota`, and expressions of
  only untyped operands. They have a **default type** used when context requires one.
- **Typed constants**: explicitly given a type via declaration or conversion.

| Constant Kind | Default Type |
|---|---|
| Boolean | `bool` |
| Rune | `rune` |
| Integer | `int` |
| Floating-point | `float64` |
| Complex | `complex128` |
| String | `string` |

### Declarations

```go
const Pi float64 = 3.14159265358979323846
const zero = 0.0          // untyped floating-point constant
const (
    size int64 = 1024
    eof        = -1        // untyped integer constant
)
const a, b, c = 3, 4, "foo"
```

### iota

Within a `const` declaration, `iota` represents successive untyped integer constants.
Its value is the index of the `ConstSpec` in the block, starting at zero. Resets at
each new `const` keyword.

```go
const (
    c0 = iota  // 0
    c1 = iota  // 1
    c2 = iota  // 2
)

const (
    a = 1 << iota  // 1  (iota == 0)
    b = 1 << iota  // 2  (iota == 1)
    c = 3          // 3  (iota == 2, unused)
    d = 1 << iota  // 8  (iota == 3)
)
```

Expression lists may be omitted after the first `ConstSpec` — they repeat the previous:

```go
const (
    Sunday = iota  // 0
    Monday         // 1
    Tuesday        // 2
    Wednesday      // 3
    Thursday       // 4
    Friday         // 5
    Saturday       // 6
)
```

### Properties

- Represent exact values of arbitrary precision.
- Never overflow.
- No constants for IEEE 754 negative zero, infinity, or NaN.
- Implementation: at least 256-bit integers, 256-bit mantissa for floats.

---

## Addressability

Source: [spec#Address_operators](https://go.dev/ref/spec#Address_operators)

The `&` operator takes the address of an addressable operand:

```go
var x int
p := &x        // p is *int, points to x

arr := [3]int{1, 2, 3}
p := &arr[0]   // pointer to first element

s := "hello"
// p := &s[0]  // illegal: cannot take address of string bytes
```
