# Go Specification Reference — Structs, Methods & Enums

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> for the `structs-methods-and-enums` module. Covers: struct types, field tags,
> method declarations, method sets, receiver types, promoted methods, and constants.

---

## Struct Types

Source: [spec#Struct_types](https://go.dev/ref/spec#Struct_types)

A struct is a sequence of named elements, called **fields**, each of which has a name and a type. Field names may be explicit (an `IdentifierList`) or implicit (an `EmbeddedField`). Within a struct, non-blank field names must be unique.

```go
struct {}

struct {
    x, y int
    u float32
    _ float32       // padding
    A *[]int
    F func()
}
```

### Field Declarations

A field declared with a type but no explicit field name is called an **embedded field**. An embedded field must be specified as a type name `T` or as a pointer to a non-interface type name `*T`, and `T` itself may not be a pointer type.

The unqualified type name acts as the field name:

```go
struct {
    T1             // field name is T1
    *T2            // field name is T2
    P.T3           // field name is T3
    *P.T4          // field name is T4
    x, y int       // field names are x and y
}
```

The following declaration is **illegal** because field names must be unique:

```go
struct {
    T     // conflicts with embedded field T
    *T    // conflicts with *T and embedded field T
    *P.T  // conflicts with *P.T and embedded field T
}
```

### Field Tags

A field declaration may be followed by an optional string literal **tag**, which becomes an attribute for all the fields in the corresponding field declaration. An empty tag string is equivalent to an absent tag.

```go
struct {
    x, y float64 ""   // an empty tag string is like an absent tag
    name string  "any string is permitted as a tag"
    _    [4]byte "ceci n'est pas un champ de structure"
}

// A Tag used as documentation
struct {
    Model        string `json:"model" validate:"required"`
    Version      string `json:"version,omitempty"`
    MaxRetries   int    `json:"max_retries" validate:"min=0,max=10"`
}
```

Tags are visible through the reflection interface and take part in type identity for structs but are otherwise ignored. The `reflect.StructTag` type provides methods for interpreting tags formatted as key:"value" pairs.

### Promoted Fields and Methods

For a struct `S` containing an embedded field `T`:
- If `S` contains an embedded field `T`, the **method sets** of `S` and `*S` both include promoted methods with receiver `T`. The method set of `*S` also includes promoted methods with receiver `*T`.
- If `S` contains an embedded field `*T`, the method sets of `S` and `*S` both include promoted methods with receiver `T` or `*T`.

A field or method `f` of an embedded field in a struct `x` is called **promoted** if `x.f` is a legal selector that denotes that field or method `f`.

Promoted fields act like ordinary fields of a struct except that they cannot be used as field names in composite literals of the struct:

```go
type innerType struct {
    x int
}
type S struct {
    innerType
}
s := S{innerType{1}} // OK — explicit
s := S{x: 1}         // error: cannot use promoted field as key
```

Given a struct type `S` and a named type `T`, promoted methods are included in the method set of the struct:

- If `S` contains an embedded field `T`, the method sets of `S` and `*S` both include promoted methods with receiver `T`. The method set of `*S` also includes promoted methods with receiver `*T`.
- If `S` contains an embedded field `*T`, the method sets of `S` and `*S` both include promoted methods with receiver `T` or `*T`.

### Struct Comparability

Struct values are comparable if all their fields are comparable. Two struct values are equal if their corresponding non-blank fields are equal. Fields are compared in source order.

---

## Method Declarations

Source: [spec#Method_declarations](https://go.dev/ref/spec#Method_declarations)

A method is a function with a **receiver**. A method declaration binds a method name to a base type and associates the method with that base type.

```go
func (p *Point) Length() float64 {
    return math.Sqrt(p.X*p.X + p.Y*p.Y)
}

func (p *Point) Scale(factor float64) {
    p.X *= factor
    p.Y *= factor
}
```

### Receiver

The receiver is specified via an extra parameter section before the method name. That parameter section must declare a single non-variadic parameter, the receiver. Its type must be a **defined type** `T` or a pointer to a defined type `*T` and `T` must be defined in the same package as the method.

The receiver type must not be a pointer or interface type, and it must be defined in the same package as the method declaration.

```go
func (p *Point) Scale(factor float64) { /* ... */ }
//     ^^^^^^^
//     receiver: *Point
```

A non-blank receiver name must be unique in the method signature. If the receiver's value is not referenced inside the body of the method, its name may be omitted in the declaration.

### Method Sets

Source: [spec#Method_sets](https://go.dev/ref/spec#Method_sets)

The method set of a type determines the interfaces that the type implements and the methods that can be called using a receiver of that type.

| Type | Method Set |
|------|------------|
| `T` | All methods declared with receiver `T` |
| `*T` | All methods declared with receiver `T` or `*T` |
| Interface `I` | The set of methods of `I` |

Further rules for named types:
- The method set of an interface type is its interface.
- The method set of any other named type `T` consists of all methods declared with receiver type `T`.
- The method set of a pointer to a named type `*T` consists of all methods declared with receiver type `T` or `*T`.

**In practice:** Go automatically takes the address of an addressable value when calling a pointer-receiver method. However, **map elements are not addressable**, so you cannot call pointer-receiver methods directly on values retrieved from a map.

```go
type Counter struct{ n int }
func (c *Counter) Inc() { c.n++ }

m := map[string]Counter{"a": {}}
m["a"].Inc()          // compile error: cannot take address of m["a"]

m2 := map[string]*Counter{"a": {}}
m2["a"].Inc()         // OK: m2["a"] is already *Counter
```

---

## Selectors

Source: [spec#Selectors](https://go.dev/ref/spec#Selectors)

For a primary expression `x` that is not a package name, the selector expression `x.f` denotes the field or method `f` of the value `x` (or sometimes `*x`).

A selector `f` may denote a field or method `f` of a type `T`, or it may refer to a field or method `f` of a nested embedded field of `T`. The number of embedded fields traversed to reach `f` is called its **depth** in `T`.

The depth of a field or method `f` declared in `T` is zero. The depth of a field or method `f` declared in an embedded field `A` in `T` is the depth of `f` in `A` plus one.

**Selector rules:**

1. For a value `x` of type `T` or `*T` where `T` is not a pointer or interface type, `x.f` denotes the field or method at the shallowest depth in `T` where there is such an `f`.
2. For a value `x` of interface type `I`, `x.f` denotes the actual method with name `f` of the dynamic value of `x`.
3. As an exception, if `x` is a defined pointer type and `(*x).f` is a valid selector expression denoting a field (but not a method), `x.f` is shorthand for `(*x).f`.

If there is not exactly one `f` with shallowest depth, the selector expression is **illegal** (ambiguous):

```go
type A struct{ x int }
type B struct{ x int }
type C struct {
    A
    B
}

var c C
c.x  // ambiguous: both A.x and B.x at same depth — compile error
c.A.x  // OK: explicit disambiguation
```

---

## Interface Types

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

An interface type defines a type set. A variable of interface type can store a value of any type with a method set that is a superset of the interface.

```go
// Interface satisfied by EmailChannel, SMSChannel, etc.
type Notifier interface {
    Send(recipient, body string) error
    Name() string
}

// Implicit satisfaction — no "implements" declaration needed
type EmailChannel struct { /* ... */ }
func (e *EmailChannel) Send(r, b string) error { /* ... */ }
func (e *EmailChannel) Name() string           { return "email" }

// *EmailChannel satisfies Notifier — checked at compile time
var n Notifier = &EmailChannel{}
```

---

## Type Assertions and Type Switches

Source: [spec#Type_assertions](https://go.dev/ref/spec#Type_assertions)

For a value `x` of interface type and a type `T`, the primary expression `x.(T)` asserts that `x` is not nil and that the value stored in `x` is of type `T`.

```go
// Single-return: panics if assertion fails
e := ch.(EmailChannel)

// Two-return: safe form
e, ok := ch.(*EmailChannel)
if ok {
    // e is *EmailChannel
}
```

Source: [spec#Type_switches](https://go.dev/ref/spec#Type_switches)

A type switch compares types rather than values. The variable in the TypeSwitchGuard may optionally be bound to a name in each TypeCaseClause:

```go
switch v := ch.(type) {
case *EmailChannel:
    fmt.Println(v.SMTPHost)  // v is *EmailChannel here
case *SMSChannel:
    fmt.Println(v.Provider)  // v is *SMSChannel here
case nil:
    fmt.Println("nil channel")
default:
    fmt.Printf("unknown: %T\n", v)
}
```

A case clause with multiple types binds `v` to the interface type (not a specific type):

```go
case *EmailChannel, *SMSChannel:
    // v is the interface type ch.(type)
```

---

## Constants and iota

Source: [spec#Constant_declarations](https://go.dev/ref/spec#Constant_declarations)

Within a constant declaration, the predeclared identifier `iota` represents successive untyped integer constants. Its value is the index of the respective ConstSpec in that constant declaration, starting at 0.

```go
const (
    c0 = iota  // 0
    c1          // 1
    c2          // 2
)

const (
    a = 1 << iota  // a == 1  (iota == 0)
    b = 1 << iota  // b == 2  (iota == 1)
    c = 3           // c == 3  (iota == 2, unused)
    d = 1 << iota  // d == 8  (iota == 3)
)
```

The expression list may be omitted from all but the first ConstSpec — it implicitly repeats the preceding list:

```go
const (
    Sunday = iota  // 0
    Monday          // 1
    Tuesday         // 2
    Wednesday       // 3
    Thursday        // 4
    Friday          // 5
    Saturday        // 6
)
```

Within an ExpressionList, the value of each iota is the same because it is only incremented after each ConstSpec:

```go
const (
    bit0, mask0 = 1 << iota, 1<<iota - 1  // bit0 == 1, mask0 == 0  (iota == 0)
    bit1, mask1                             // bit1 == 2, mask1 == 1  (iota == 1)
    _, _                                    //                         (iota == 2, unused)
    bit3, mask3                             // bit3 == 8, mask3 == 7  (iota == 3)
)
```

---

## Composite Literals

Source: [spec#Composite_literals](https://go.dev/ref/spec#Composite_literals)

Composite literals construct new composite values each time they are evaluated. They consist of the type of the literal followed by a brace-bound list of elements.

```go
// Struct literal with all fields named
Point{x: 1, y: 2}

// Struct literal — field names optional if all fields listed in order
Point{1, 2}

// Nested struct literal
LineSegment{
    start: Point{x: 0, y: 0},
    end:   Point{x: 1, y: 1},
}
```

**Rules for struct literals:**
- A key must be a field name declared in the struct type.
- An element list that does not contain any keys must list an element for each struct field in the order in which the fields are declared.
- It is an error to specify an element for a non-exported field belonging to a different package.
- A literal may omit the field list — such a literal evaluates to the zero value for its type.
- It is an error to specify an element for a blank `_` field.

**Taking the address of a composite literal** allocates a new struct instance and returns a pointer:

```go
p := &Point{x: 1, y: 2}  // p is *Point
```

This is idiomatic Go for constructing heap-allocated values.

---

## Struct Memory Layout

Source: [spec#Size_and_alignment_guarantees](https://go.dev/ref/spec#Size_and_alignment_guarantees)

The size of a struct is at least the sum of the sizes of its fields. The compiler may insert **padding** between fields to satisfy alignment requirements.

```go
// Well-ordered: no padding needed
type Compact struct {
    a int64  // 8 bytes
    b int32  // 4 bytes
    c int16  // 2 bytes
    d int8   // 1 byte
    e int8   // 1 byte
}
// Total: 16 bytes

// Poorly ordered: padding inserted for alignment
type Padded struct {
    a int8   // 1 byte + 7 bytes padding
    b int64  // 8 bytes
    c int8   // 1 byte + 7 bytes padding
}
// Total: 24 bytes (8 bytes of padding wasted)
```

Use `unsafe.Sizeof(s)` to see the actual struct size. Use `unsafe.Offsetof(s.f)` to see the byte offset of field `f`.

The `go vet` tool checks some alignment issues. For performance-critical structs, order fields from largest to smallest alignment requirement.
