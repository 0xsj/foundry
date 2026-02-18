# Go Specification Reference — Generics

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> and [the Go blog on generics](https://go.dev/blog/intro-generics)
> for the `generics` module. Covers: type parameters, type constraints, instantiation,
> type inference, and generic declarations.

---

## Type Parameters

Source: [spec#Type_parameter_declarations](https://go.dev/ref/spec#Type_parameter_declarations)

A type parameter list declares one or more type parameters for a generic function or type. Each type parameter is an identifier followed by a type constraint.

### EBNF

```
TypeParameters  = "[" TypeParamList [ "," ] "]" .
TypeParamList   = TypeParamDecl { "," TypeParamDecl } .
TypeParamDecl   = IdentifierList TypeConstraint .
TypeConstraint  = TypeElem .
```

### Examples

```go
[P any]
[S interface{ ~[]byte | ~string }]
[S ~[]E, E any]
[P Constraint[int]]
[_ any]
```

---

## Generic Function Declarations

Source: [spec#Function_declarations](https://go.dev/ref/spec#Function_declarations)

A function declaration may include a type parameter list, making the function generic.

```
FunctionDecl = "func" FunctionName [ TypeParameters ] Signature [ FunctionBody ] .
```

### Examples

```go
// Generic function with one type parameter
func min[V constraints.Ordered](a, b V) V {
    if a < b {
        return a
    }
    return b
}

// Generic function with multiple type parameters
func apply[T, U any](s []T, f func(T) U) []U {
    r := make([]U, len(s))
    for i, v := range s {
        r[i] = f(v)
    }
    return r
}
```

---

## Generic Type Declarations

Source: [spec#Type_declarations](https://go.dev/ref/spec#Type_declarations)

A type declaration may include a type parameter list.

```
TypeDecl = "type" TypeSpec .
TypeSpec = AliasDecl | TypeDef .
TypeDef  = identifier [ TypeParameters ] Type .
```

### Examples

```go
type List[T any] struct {
    next *List[T]
    val  T
}

type Tree[T interface{ ~int | ~float64 }] struct {
    left, right *Tree[T]
    value       T
}

// Instantiation: provide type arguments
var s Stack[int]

type IntList = List[int]  // type alias (specific instantiation)
```

---

## Type Constraints

Source: [spec#Type_constraints](https://go.dev/ref/spec#Type_constraints)

A type constraint is an interface that defines the set of types a type parameter may be instantiated with.

```
InterfaceType = "interface" "{" { InterfaceElem ";" } "}" .
InterfaceElem = MethodElem | TypeElem .
TypeElem      = TypeTerm { "|" TypeTerm } .
TypeTerm      = [ "~" ] Type .
```

### Predeclared Constraints

| Constraint | Definition | Meaning |
|------------|-----------|---------|
| `any` | `interface{}` | Any type |
| `comparable` | built-in | Types supporting `==` and `!=` |

### Interface as Constraint

Any interface type can be used as a constraint:

```go
type Stringer interface {
    String() string
}

func Stringify[T Stringer](s []T) []string {
    result := make([]string, len(s))
    for i, v := range s {
        result[i] = v.String()
    }
    return result
}
```

### Union Elements

An interface used as a constraint may contain type terms separated by `|`. A type satisfies this constraint if it matches any of the terms:

```go
// Only int or string
type IntOrString interface {
    int | string
}
```

### The `~` (Tilde) Operator

A `~T` term matches any type whose **underlying type** is `T`:

```go
// ~int matches: int, and any named type with underlying type int (e.g., type MyInt int)
type MyInt int
var x MyInt = 3

// This constraint accepts MyInt
type IntLike interface {
    ~int
}

func double[T IntLike](v T) T {
    return v + v
}

double(MyInt(3))  // valid
double(3)         // valid — int has underlying type int
```

`~T` is **only valid inside a union element**. `interface{ ~int }` is valid; `~int` outside an interface is a syntax error.

---

## Instantiation

Source: [spec#Instantiations](https://go.dev/ref/spec#Instantiations)

An instantiation creates a concrete function or type by substituting type arguments for type parameters.

```go
// Generic declaration
func min[V constraints.Ordered](a, b V) V { ... }

// Explicit instantiation
min[int](2, 3)     // instantiated with int
min[string]("a", "b")

// Implicit instantiation (type inference)
min(2, 3)          // V inferred as int
min("a", "b")      // V inferred as string
```

Instantiation is performed at compile time. Each unique set of type arguments produces a separate compiled entity (subject to GC Shape Stenciling optimization).

---

## Type Inference

Source: [spec#Type_unification](https://go.dev/ref/spec#Type_unification)

The compiler infers type arguments from:
1. **Function arguments** — unify argument types against parameter types
2. **Return value context** — in assignment context, unify against the expected type
3. **Constraint type inference** — derive type arguments from other type arguments via constraints

### Rules

- Type inference is attempted when type arguments are omitted
- Inference fails if the type cannot be determined uniquely — you must then provide type arguments explicitly
- Type arguments can be partially provided; the rest are inferred

```go
func Map[T, U any](s []T, f func(T) U) []U { ... }

// Full inference: T = int (from []int), U = string (from func return)
Map([]int{1, 2}, func(n int) string { return strconv.Itoa(n) })

// Partial inference is not supported for functions — must provide all or none
// Map[int]([]int{1, 2}, ...)  // ERROR: must provide all type arguments
```

Type parameters that appear **only in the return type** cannot be inferred:

```go
func Zero[T any]() T {
    var z T
    return z
}

Zero[int]()      // must specify
// Zero()        // ERROR: cannot infer T
```

---

## Methods on Generic Types

Source: [spec#Method_declarations](https://go.dev/ref/spec#Method_declarations)

A method may be declared on a generic type. The method's receiver must include the type parameter list (without constraints).

```go
type Stack[T any] struct {
    items []T
}

// Receiver: (s *Stack[T])
// No new type parameters may be introduced on the method
func (s *Stack[T]) Push(v T) {
    s.items = append(s.items, v)
}

func (s *Stack[T]) Pop() (T, bool) {
    if len(s.items) == 0 {
        var zero T
        return zero, false
    }
    n := len(s.items) - 1
    v := s.items[n]
    s.items = s.items[:n]
    return v, true
}
```

**Restriction:** Methods cannot have their own type parameters. Only the type parameters of the containing type are in scope. For additional type-parameterized behavior, use package-level generic functions.

---

## Comparable Constraint

Source: [spec#Comparison_operators](https://go.dev/ref/spec#Comparison_operators)

`comparable` is a built-in interface. A type implements `comparable` if values of that type can be compared using `==` and `!=`.

| Type | Comparable? |
|------|------------|
| `bool`, numeric, `string`, `pointer`, `channel` | Yes |
| `interface` | Yes (at runtime, may panic if dynamic type is not comparable) |
| Struct | Yes if all fields are comparable |
| Array | Yes if element type is comparable |
| Slice | No |
| Map | No |
| Function | No |

```go
// comparable is required for map keys and equality checks in generic code
func Index[K comparable, V any](m map[K]V, key K) (V, bool) {
    v, ok := m[key]
    return v, ok
}
```

---

## `any` and `comparable` Built-ins

```go
// any is an alias for interface{}
// Defined in the universe block:
type any = interface{}

// comparable is an interface implemented by all comparable types
// Cannot be instantiated directly or used in union elements
```

**Key distinction:**
- `any` — no constraints, cannot use `==`
- `comparable` — enables `==` and `!=`, required for map keys

---

## Type Sets

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

An interface defines a **type set** — the set of all types that implement the interface. When used as a constraint, only types in the type set may be used as type arguments.

| Interface element | Adds to type set |
|------------------|-----------------|
| Method `M()` | Types that implement `M()` |
| Type term `T` | Exactly the type `T` |
| Type term `~T` | `T` and all types with underlying type `T` |
| Union `A \| B` | Types in set of `A` or set of `B` |

An interface with only method elements has an infinite type set (any type implementing those methods). An interface with type terms has a finite type set.

**Interface types with type terms cannot be used as ordinary interface values** — only as constraints:

```go
type Numeric interface {
    ~int | ~float64
}

var x Numeric  // ERROR: interface contains type constraints
func f(n Numeric) {}  // ERROR: cannot use as parameter type

func g[T Numeric](n T) {}  // OK: Numeric used as constraint
```

---

## Constraint Literals

A constraint can be written inline without a named interface:

```go
// Inline constraint
func Abs[T interface{ ~int | ~float64 }](x T) T {
    if x < 0 { return -x }
    return x
}

// Equivalent named constraint
type SignedNum interface {
    ~int | ~float64
}
func Abs[T SignedNum](x T) T { ... }
```

For one-off use, inline is fine. For reuse across three or more functions, name it.

---

## `slices` and `maps` Packages (Go 1.21)

Go 1.21 added two standard library packages that use generics:

```go
import (
    "slices"
    "maps"
)

// slices package — generic slice utilities
slices.Contains([]int{1, 2, 3}, 2)         // true
slices.Sort([]string{"c", "a", "b"})       // sorts in place
slices.Index([]string{"a", "b"}, "b")      // 1
slices.Equal([]int{1, 2}, []int{1, 2})     // true
slices.Reverse([]int{1, 2, 3})             // [3, 2, 1]

// maps package — generic map utilities
maps.Keys(map[string]int{"a": 1})          // []string{"a"}
maps.Values(map[string]int{"a": 1})        // []int{1}
maps.Clone(map[string]int{"a": 1})         // copy of map
maps.Equal(m1, m2)                         // true if same key-value pairs
```

These packages are the primary motivation for generics in the standard library. See `go doc slices` and `go doc maps` for the full API.

---

## Restrictions (as of Go 1.22)

The following are currently **not supported** in Go generics:

- Methods may not have their own type parameters
- Type parameters cannot be used as base types for method declarations from outside the package
- Type assertions to type parameters are not allowed: `x.(T)` where `T` is a type parameter
- No higher-kinded types (type parameters over type constructors, e.g., `F[_]`)
- No variadic type parameters (variable number of type arguments)
- No const generics (type parameters over constant values)
- Interfaces with type terms cannot be used as regular types (only as constraints)
- No covariance or contravariance

---

## References

- [The Go Programming Language Specification — Type Parameter Declarations](https://go.dev/ref/spec#Type_parameter_declarations)
- [The Go Programming Language Specification — Instantiations](https://go.dev/ref/spec#Instantiations)
- [The Go Programming Language Specification — Type Unification](https://go.dev/ref/spec#Type_unification)
- [An Introduction to Generics (Go Blog)](https://go.dev/blog/intro-generics)
- [When to Use Generics (Go Blog)](https://go.dev/blog/when-generics)
- [golang.org/x/exp/constraints package](https://pkg.go.dev/golang.org/x/exp/constraints)
- [slices package (Go 1.21)](https://pkg.go.dev/slices)
- [maps package (Go 1.21)](https://pkg.go.dev/maps)
