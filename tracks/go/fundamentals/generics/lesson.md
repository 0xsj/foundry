# Generics — Go

## Why Generics Exist

Before Go 1.18, writing a function that worked on multiple types meant one of three options:

1. **Copy-paste the function for each type** — works, but the duplication is painful and diverges over time
2. **Use `interface{}`** — works, but you lose type safety and pay for runtime type assertions
3. **Use code generation** — works, but adds tooling complexity

```go
// Pre-generics: you'd write this for int, int64, float64, string...
func maxInt(a, b int) int {
    if a > b { return a }
    return b
}

// Or this abomination — works, but not safe
func max(a, b interface{}) interface{} {
    // ... runtime type switch ...
}
```

Generics solve this with **type parameters**: you write the algorithm once, parameterized by a type, and the compiler instantiates concrete versions for each type you actually use.

The core insight: you're writing code that is *generic over types* the same way a regular function is generic over values.

### Your notes
<!-- -->

---

## Type Parameters: The Basics

A type parameter is declared in square brackets `[T any]` between the function name and its regular parameters. `T` is a placeholder for a concrete type that will be determined at call time.

```go
// Generic function: works for any type T
func First[T any](slice []T) (T, bool) {
    if len(slice) == 0 {
        var zero T  // zero value of T
        return zero, false
    }
    return slice[0], true
}

// The compiler infers T from the argument
v, ok := First([]int{1, 2, 3})       // T = int,    v = 1
s, ok := First([]string{"a", "b"})    // T = string, s = "a"
b, ok := First([]bool{})              // T = bool,   b = false, ok = false

// Or you can specify T explicitly (rarely needed)
v, ok := First[int]([]int{1, 2, 3})
```

You can have multiple type parameters:

```go
func Map[T, U any](slice []T, fn func(T) U) []U {
    result := make([]U, len(slice))
    for i, v := range slice {
        result[i] = fn(v)
    }
    return result
}

lengths := Map([]string{"hello", "world"}, func(s string) int {
    return len(s)
})
// lengths: []int{5, 5}
```

### Type Inference

The compiler infers type parameters from the arguments — you rarely need to write them explicitly. The rules:

- If the type parameter appears in a function argument, it's inferred from that argument
- If the type parameter only appears in the return type, you must specify it

```go
func Zero[T any]() T {
    var z T
    return z
}

// T cannot be inferred — no argument to infer from
z := Zero[int]()     // must specify int explicitly
z := Zero[string]()
```

The zero value trick (`var z T`) is the idiomatic way to get the zero value of a type parameter. You can't write `0` or `""` because you don't know what T is.

### Your notes
<!-- -->

---

## Constraints

`any` is the widest possible constraint — it allows any type. But with `any`, you can only do things every type supports: assign, pass around, store in slices. You can't call methods, compare with `>`, or add with `+`.

**Constraints restrict what types can be used as a type parameter, and in return unlock operations on those types.**

```go
// Won't compile: < is not defined for T constrained by any
func Max[T any](a, b T) T {
    if a > b { return a }  // ERROR: cannot use > with T
    return b
}
```

A constraint is an interface. The constraint lists what you require of T:

```go
// Ordered is a constraint satisfied by all types with < > <= >=
// (from the golang.org/x/exp/constraints package, or define inline)
type Ordered interface {
    ~int | ~int8 | ~int16 | ~int32 | ~int64 |
    ~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 | ~uintptr |
    ~float32 | ~float64 |
    ~string
}

func Max[T Ordered](a, b T) T {
    if a > b { return a }
    return b
}

Max(3, 5)         // T = int
Max(3.14, 2.71)   // T = float64
Max("apple", "banana")  // T = string
```

### The `comparable` Constraint

The built-in `comparable` constraint is satisfied by any type that supports `==` and `!=`. This is required for map keys and for equality checks:

```go
func Contains[T comparable](slice []T, item T) bool {
    for _, v := range slice {
        if v == item {  // only works because T is comparable
            return true
        }
    }
    return false
}

Contains([]int{1, 2, 3}, 2)           // true
Contains([]string{"a", "b"}, "c")     // false
```

Without `comparable`, the `== item` line won't compile. This is the compiler enforcing correctness at the constraint level — not at runtime.

### Your notes
<!-- -->

---

## The `~` Operator (Underlying Type Constraints)

Consider a type alias:

```go
type Celsius float64
type Fahrenheit float64
```

These are distinct types. `Celsius` is not `float64` — it's a named type with underlying type `float64`. Without `~`, the `Ordered` constraint above would not accept `Celsius`:

```go
// Won't work: Celsius is not listed in Ordered
func toFahrenheit[T Ordered](c T) Fahrenheit {
    return Fahrenheit(c * 9/5 + 32)
}
```

The `~` operator means "this type, or any type whose *underlying type* is this type":

```go
type Temperature interface {
    ~float64  // accepts float64 AND any type with underlying type float64
}

// Now works with Celsius and Fahrenheit
func Convert[T ~float64](val T) float64 {
    return float64(val)
}

Convert(Celsius(100))     // works
Convert(Fahrenheit(212))  // works
Convert(3.14)             // works — float64 itself
```

This is critical for building constraints that work with domain types. In production, you'll frequently have `type UserID int64`, `type Score float32`, `type EventName string` — and your generic utilities need to work with these, not just the primitive types.

```go
// A generic "newtype" constraint for string-based identifiers
type StringID interface {
    ~string
}

func ParseID[T StringID](raw string) (T, error) {
    if raw == "" {
        var zero T
        return zero, fmt.Errorf("empty ID")
    }
    return T(raw), nil  // T(raw) works because T's underlying type is string
}

type UserID string
type OrderID string

uid, _ := ParseID[UserID]("user-123")    // UserID("user-123")
oid, _ := ParseID[OrderID]("order-456")  // OrderID("order-456")
```

### Your notes
<!-- -->

---

## Union Constraints

Constraints can express "T must be one of these types" using union syntax `|`:

```go
// Only int or float64 — nothing else
type Number interface {
    int | float64
}

func Sum[T Number](nums []T) T {
    var total T
    for _, n := range nums {
        total += n
    }
    return total
}

Sum([]int{1, 2, 3})          // 6 (int)
Sum([]float64{1.1, 2.2})     // 3.3 (float64)
// Sum([]string{...})        // compile error — string not in union
```

Combine `~` with `|` for unions that include derived types:

```go
type Numeric interface {
    ~int | ~int64 | ~float64
}
```

### Inline vs Named Constraints

You can write constraints inline (useful for one-off type parameters) or as named interface types (useful for reuse):

```go
// Inline constraint — used once
func Clamp[T interface{ ~int | ~float64 }](val, min, max T) T {
    if val < min { return min }
    if val > max { return max }
    return val
}

// Named constraint — reused across functions
type Numeric interface {
    ~int | ~int32 | ~int64 | ~float32 | ~float64
}

func Clamp[T Numeric](val, min, max T) T { ... }
func Abs[T Numeric](val T) T { ... }
func Round[T Numeric](val T) T { ... }
```

Named constraints are the right choice when you use the same set of types in multiple function signatures.

### Your notes
<!-- -->

---

## The `constraints` Package

Go provides the `golang.org/x/exp/constraints` package (experimental, likely to be standardized) with pre-built constraints you'll use constantly:

```go
import "golang.org/x/exp/constraints"

// constraints.Ordered — all types that support < > <= >=
// (integers, floats, strings)

// constraints.Signed — signed integer types
// ~int | ~int8 | ~int16 | ~int32 | ~int64

// constraints.Unsigned — unsigned integer types
// ~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 | ~uintptr

// constraints.Integer — all integers (Signed | Unsigned)

// constraints.Float — ~float32 | ~float64

// constraints.Complex — ~complex64 | ~complex128
```

```go
func Min[T constraints.Ordered](a, b T) T {
    if a < b { return a }
    return b
}

func Abs[T constraints.Signed](n T) T {
    if n < 0 { return -n }
    return n
}
```

In practice, you often define your own constraints that are specific to your domain and use the stdlib constraints as building blocks.

> **Note on `slices` and `maps` packages:** Go 1.21 added `slices` and `maps` packages to the standard library that use generics. `slices.Contains`, `slices.Sort`, `maps.Keys` — these are the real-world payoff of generics. Browse their source code to see idiomatic constraint usage.

### Your notes
<!-- -->

---

## Generic Structs

Type parameters apply to types (structs, interfaces) too, not just functions. This is where generics become truly powerful.

```go
// A type-safe optional value — like Option<T> in Rust or Maybe in Haskell
type Option[T any] struct {
    value T
    valid bool
}

func Some[T any](v T) Option[T] {
    return Option[T]{value: v, valid: true}
}

func None[T any]() Option[T] {
    return Option[T]{}
}

func (o Option[T]) Unwrap() (T, bool) {
    return o.value, o.valid
}

// Usage
name := Some("Alice")
v, ok := name.Unwrap()  // "Alice", true

missing := None[string]()
v, ok = missing.Unwrap()  // "", false
```

### Methods on Generic Types

Methods on generic types use the type parameter in the receiver, but the method cannot introduce *new* type parameters — only use the ones declared on the type:

```go
type Stack[T any] struct {
    items []T
}

// Receiver uses [T] to reference the type parameter
func (s *Stack[T]) Push(item T) {
    s.items = append(s.items, item)
}

func (s *Stack[T]) Pop() (T, bool) {
    if len(s.items) == 0 {
        var zero T
        return zero, false
    }
    item := s.items[len(s.items)-1]
    s.items = s.items[:len(s.items)-1]
    return item, true
}

func (s *Stack[T]) Len() int {
    return len(s.items)
}

// Usage — type inferred from first Push
var s Stack[int]
s.Push(1)
s.Push(2)
v, _ := s.Pop()  // 2 — LIFO
```

Note: you cannot write `func (s *Stack[T]) Method[U any](...)` — methods cannot have their own type parameters. If you need that, use a package-level generic function.

### Your notes
<!-- -->

---

## Generic Interfaces

An interface can use type parameters to describe generic behaviors:

```go
// A collection that can report its size
type Sizer[T any] interface {
    Len() int
    Add(T)
    Get(int) (T, bool)
}

// Stringer is not generic — it's the standard library interface
// But you can combine them:
type StringSizer interface {
    Sizer[string]
    fmt.Stringer
}
```

You can also write generic functions that work on any type implementing a generic interface:

```go
func Drain[T any](src Sizer[T]) []T {
    result := make([]T, 0, src.Len())
    for i := 0; i < src.Len(); i++ {
        if v, ok := src.Get(i); ok {
            result = append(result, v)
        }
    }
    return result
}
```

### Your notes
<!-- -->

---

## When NOT to Use Generics

This is the most important section. Generics add complexity. The readability cost is real. Use them when the benefit is clear.

### Use Generics When

**1. You're writing algorithms over collections of unknown element type**

```go
// Generic: works for any T — the algorithm is the same regardless of T
func Filter[T any](slice []T, fn func(T) bool) []T { ... }
func GroupBy[T any, K comparable](slice []T, key func(T) K) map[K][]T { ... }
```

**2. You're writing a container type**

```go
// Stack, Queue, Set, Cache — the data structure is the same regardless of what it holds
type Set[T comparable] struct { items map[T]struct{} }
type Result[T any] struct { value T; err error }
```

**3. You're eliminating copy-paste that differs only in type**

```go
// You have maxInt, maxFloat64, maxString — generics eliminate the duplication
func Max[T constraints.Ordered](a, b T) T { ... }
```

### Don't Use Generics When

**1. You're calling methods specific to T**

If your algorithm calls `t.Serialize()` or `t.Validate()` — that's what interfaces are for. A regular interface is more idiomatic and more flexible than a constraint.

```go
// WRONG: using generics when interface is correct
func Process[T interface{ Validate() error }](item T) error {
    return item.Validate()
}

// RIGHT: regular interface
type Validator interface { Validate() error }
func Process(item Validator) error {
    return item.Validate()
}
```

**2. The function has only one or two call sites**

If `Filter` is only called with `[]User`, don't make it generic. Write `filterUsers([]User, func(User) bool) []User`. Premature generalization adds cognitive load with no payoff.

**3. The abstraction makes the code harder to read**

```go
// Over-engineered: three type parameters, unclear benefit
func Transform[A, B, C any](a A, f func(A) B, g func(B) C) C {
    return g(f(a))
}

// Clearer: just write the two operations
result := g(f(a))
```

**4. You're trying to abstract over different struct shapes**

Go generics don't support field access (`t.Name` where Name is a field). If you need to access struct fields generically, use embedding or interfaces, not generics.

### The Rule of Thumb

> **Write the concrete version first.** When you see that you've written the same algorithm three times for three different types, and the only difference is the type, *then* generify it.

### Your notes
<!-- -->

---

## Comparison to TypeScript Generics

TypeScript's generics are similar in concept but much more powerful — and more complex.

**Similarities:**
- Type parameters in angle brackets (TS `<T>`) vs square brackets (Go `[T]`)
- Constraints (TS `extends`) vs Go constraints (interfaces)
- Type inference at call sites

**TypeScript-exclusive features Go lacks:**

```typescript
// Conditional types — no Go equivalent
type NonNullable<T> = T extends null | undefined ? never : T;
type ReturnType<T extends (...args: any) => any> = T extends (...args: any) => infer R ? R : any;

// Mapped types
type Partial<T> = { [P in keyof T]?: T[P] };
type Required<T> = { [P in keyof T]-?: T[P] };

// Template literal types
type EventName<T extends string> = `on${Capitalize<T>}`;
```

Go has none of this. No conditional types, no mapped types, no `infer`, no `keyof`. Go's type system is deliberately simpler.

**Why the difference matters:** TypeScript was built for a dynamic language runtime (JavaScript). Its type system needs to express "the type of what JavaScript would return here." Go's generics are purely for eliminating boilerplate — not for type-level computation.

**The practical consequence:** In TypeScript you can write incredibly expressive type utilities (`Partial<T>`, `Record<K, V>`, `Awaited<T>`). In Go, you write more concrete types and accept a little repetition. Go trades expressive power for readability and simplicity.

```typescript
// TypeScript: expressive constraint
function merge<T extends object, U extends object>(t: T, u: U): T & U {
    return { ...t, ...u };
}
```

```go
// Go: no equivalent — you'd use map[string]interface{} or code generation
// Go generics cannot express "T must have all fields of U"
```

### Your notes
<!-- -->

---

## Comparison to Rust Generics

Rust generics and Go generics are solving the same problem but with very different tradeoffs.

### Constraint Mechanism

```rust
// Rust: trait bounds
fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Or using where clause for readability
fn max<T>(a: T, b: T) -> T
where T: PartialOrd
{
    if a > b { a } else { b }
}
```

```go
// Go: interface constraints
func Max[T constraints.Ordered](a, b T) T {
    if a > b { return a }
    return b
}
```

The key similarity: both use their existing abstraction mechanism (interfaces vs traits) for constraints. It's a clean, unified approach.

### Monomorphization

Both Go and Rust use **monomorphization**: the compiler generates a concrete version of the function for each type it's called with. `Max[int]` and `Max[float64]` become separate compiled functions.

The runtime cost: binary size grows. The benefit: **zero runtime overhead** — no boxing, no dynamic dispatch. `Max[int]` is as fast as `maxInt`.

This is different from Java/C# generics which use erasure (type parameters erased at runtime, with boxing for primitives).

```
Rust/Go generics:
  Max[int]    → concrete function using int instructions
  Max[float]  → separate concrete function using float instructions
  No overhead at runtime

Java generics (erasure):
  max<T>      → single function operating on Object
  Primitives  → boxed to Integer, Float — heap allocation per call
```

### Where Rust Goes Further

```rust
// Rust: higher-kinded types via traits
trait Functor {
    fn map<A, B>(fa: Self<A>, f: impl Fn(A) -> B) -> Self<B>;
}

// Rust: associated types in traits
trait Iterator {
    type Item;  // associated type — specified by the implementor
    fn next(&mut self) -> Option<Self::Item>;
}

// Rust: const generics — generics over values, not just types
fn zeros<const N: usize>() -> [i32; N] {
    [0; N]
}
let arr: [i32; 5] = zeros::<5>();
```

Go has none of this (yet). No higher-kinded types, no associated types in generic interfaces, no const generics. Go's generics are deliberately limited to keep the language simple.

**Practical takeaway:** Rust lets you build extremely powerful abstractions (`Iterator`, `Future`, `Serialize/Deserialize` frameworks) that Go cannot express generically. Go makes you write more concrete code, which is often the right tradeoff for a systems language focused on readability and maintainability.

### Your notes
<!-- -->

---

## How Generics Work Under the Hood

Go uses a hybrid approach called **GC Shape Stenciling**:

1. **Monomorphization for pointer types**: All pointer types share the same compiled code (since a pointer is always the same size). `*int` and `*string` use the same generated code, with the type information passed as a dictionary.

2. **Monomorphization for value types of different sizes**: `int64` and `int32` generate different code because they have different sizes/layouts.

3. **Dictionary passing**: The runtime type information (methods, size, alignment) is passed as a hidden "dictionary" argument alongside the generic function call. This is how the compiler enables method dispatch on type parameters.

The practical consequence: Go generics have slightly more overhead than hand-written concrete functions (the dictionary lookup), but far less than interface-based dynamic dispatch. For tight numeric loops, a concrete function may outperform a generic one — benchmark if it matters.

```
Concrete:   Max(3, 5)        → direct int comparison instructions
Generic:    Max[int](3, 5)   → int comparison + possible dictionary overhead
Interface:  max(3, 5)        → virtual dispatch, heap allocation for boxing
```

For most code, the difference is immeasurable. For hot paths processing millions of items per second, profile before using generics.

### Your notes
<!-- -->
