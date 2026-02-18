# Go Specification Reference — Interfaces

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> and [the standard library](https://pkg.go.dev/) for the `interfaces-and-traits` module.
> Covers: interface types, type sets, implicit satisfaction, method sets, type assertions,
> type switches, and the empty interface.

---

## Interface Types

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

An interface type defines a **type set**. A variable of interface type can store a value of any type in that type set. Such a type is said to **implement** the interface. The value of an uninitialized variable of interface type is `nil`.

```go
// An interface type with three method elements
type Storer interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
}
```

### Interface Elements

An interface type may declare:

1. **Method elements** — method signatures
2. **Type elements** — type terms (for type constraints in generics; `~T` or `T|U`)
3. **Embedded interface types** — the union of the embedded interface's elements

```go
// Method elements
type Reader interface {
    Read(p []byte) (n int, err error)
}

// Embedded interface (interface composition)
type ReadWriter interface {
    Reader  // embedded — includes Read method
    Writer  // embedded — includes Write method
}

// Type elements (constraints — for generics, not for values)
type Integer interface {
    ~int | ~int8 | ~int16 | ~int32 | ~int64
}
```

### Basic Interfaces

An interface whose type set is defined entirely by a set of methods (no type terms) is called a **basic interface**. Basic interfaces can be used as types (not just as constraints):

```go
var r io.Reader = os.Stdin        // basic interface — usable as a type
var rw io.ReadWriter = os.Stdout  // basic interface
```

An interface with type terms (like `Integer` above) is a **general interface** and can only be used as a type constraint in generics. Attempting to use it as a variable type is a compile error.

### Implicit Implementation

In Go, there is no explicit `implements` keyword. A type implements an interface if it has all the methods in the interface's type set. Implementation is verified by the compiler, not declared in source.

```go
type Stringer interface {
    String() string
}

type NotificationStatus int

// NotificationStatus implicitly implements Stringer.
// No "implements Stringer" anywhere.
func (s NotificationStatus) String() string {
    // ...
    return "pending"
}

var _ fmt.Stringer = NotificationStatus(0) // compile-time assertion
```

---

## Method Sets

Source: [spec#Method_sets](https://go.dev/ref/spec#Method_sets)

The **method set** of a type determines which interfaces it implements and which methods can be called using a receiver of that type.

| Type | Method Set |
|------|-----------|
| `T` | Methods declared with receiver `T` |
| `*T` | Methods declared with receiver `T` or `*T` |
| Interface `I` | Methods declared in `I` |
| `struct{ T }` | Promoted methods of `T` (if T is non-pointer embedded) |
| `struct{ *T }` | Promoted methods of `T` and `*T` |

### Critical Rule: Value vs Pointer

A value of type `T` only has value-receiver methods. A pointer `*T` has both value-receiver and pointer-receiver methods. This means:

```go
type Flusher interface {
    Flush() error
}

type Buffer struct { data []byte }

func (b *Buffer) Flush() error {  // pointer receiver
    b.data = b.data[:0]
    return nil
}

var _ Flusher = (*Buffer)(nil)  // OK: *Buffer has Flush
var _ Flusher = Buffer{}        // COMPILE ERROR: Buffer does not have Flush
```

**Automatic address-taking:** For an addressable variable, the compiler automatically takes the address when calling a pointer-receiver method:

```go
b := Buffer{}
b.Flush()    // compiles: Go does (&b).Flush() — b is addressable
```

This automatic address-taking does **not** apply to interface satisfaction. The method set of `Buffer` does not include `Flush`, regardless of addressability.

**Map elements are not addressable:**

```go
m := map[string]Buffer{"main": {}}
m["main"].Flush()  // COMPILE ERROR: cannot take the address of m["main"]

mp := map[string]*Buffer{"main": {}}
mp["main"].Flush()  // OK: mp["main"] is already *Buffer
```

---

## Interface Satisfaction: Compile-Time Assertions

To verify at compile time that a type implements an interface, use a blank-identifier assignment:

```go
// var _ InterfaceName = (*ConcreteType)(nil)
var _ io.Reader = (*os.File)(nil)
var _ Storer = (*MemoryStore)(nil)
var _ Storer = (*RedisStore)(nil)
```

Convention: place these directly below the type definition or at the top of the file. They produce no runtime overhead — the variable is discarded. If the assertion fails, the compiler reports exactly which methods are missing.

---

## Interface Values

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

An interface value has two components: a **dynamic type** and a **dynamic value**.

```
interface value = (dynamic type, dynamic value)
```

- **Dynamic type**: the concrete type of the stored value (`*MemoryStore`, `*RedisStore`, etc.)
- **Dynamic value**: the actual value of that type

A nil interface value has both dynamic type and dynamic value equal to nil:

```go
var s Storer  // s == nil: both type and value are nil
```

### The Nil Interface Trap

An interface value is `nil` only if **both** its dynamic type and dynamic value are nil.

```go
var fs *FileStore = nil         // nil concrete pointer
var s Storer = fs               // non-nil interface: type=*FileStore, value=nil

fmt.Println(s == nil)           // false — the interface has a type
fmt.Println(fs == nil)          // true — the concrete pointer is nil

s.Get("key")                    // PANIC: nil pointer dereference
```

This is one of the most common surprises for new Go developers. When a function returns an interface, never return a typed nil — return an untyped nil:

```go
// WRONG: returns non-nil interface with nil value
func newStore(useDisk bool) Storer {
    var fs *FileStore
    if useDisk {
        fs = &FileStore{path: "/tmp"}
    }
    return fs  // if useDisk==false: returns (*FileStore, nil) interface — not nil!
}

// CORRECT: returns truly nil interface
func newStore(useDisk bool) Storer {
    if !useDisk {
        return nil  // untyped nil — both type and value are nil
    }
    return &FileStore{path: "/tmp"}
}
```

---

## Type Assertions

Source: [spec#Type_assertions](https://go.dev/ref/spec#Type_assertions)

For an expression `x` of interface type and a type `T`, the type assertion `x.(T)` asserts that `x` is not nil and the value stored in `x` is of type `T`.

### Single-Return (Panics on Failure)

```go
var s Storer = &MemoryStore{}
mem := s.(*MemoryStore)  // panics if s doesn't hold *MemoryStore
```

**Panics** with a runtime error if the dynamic type of `x` is not `T`.

### Two-Return (Safe)

```go
mem, ok := s.(*MemoryStore)
if ok {
    // mem is *MemoryStore; safe to use
}
// if !ok, mem is nil (zero value of *MemoryStore)
```

The `ok` form never panics. If the assertion fails, `ok` is `false` and the asserted value is the zero value of `T`.

### Asserting to Interface

You can also assert to an interface type — checking whether the concrete value satisfies a more specific interface:

```go
type BulkStorer interface {
    Storer
    SetBulk(pairs map[string][]byte) error
}

if bulk, ok := s.(BulkStorer); ok {
    // s also satisfies BulkStorer — use the extended capability
    _ = bulk.SetBulk(data)
}
```

---

## Type Switches

Source: [spec#Type_switches](https://go.dev/ref/spec#Type_switches)

A type switch compares the dynamic type of an interface value against a list of types.

### Syntax

```go
switch v := x.(type) {
case T1:
    // v is of type T1
case T2, T3:
    // v is of type x's interface type (since multiple types)
case nil:
    // x was nil
default:
    // v is the same type as x (the interface type)
}
```

The expression `x.(type)` is only valid in a type switch — not elsewhere.

### Binding

If the TypeSwitchGuard includes a binding (`v :=`), `v` is bound to the concrete type in each case arm:

```go
switch v := s.(type) {
case *MemoryStore:
    fmt.Println(v.Len())   // v is *MemoryStore here
case *RedisStore:
    fmt.Println(v.Addr())  // v is *RedisStore here
default:
    fmt.Printf("unknown: %T\n", v)  // v is Storer (interface type)
}
```

If a case lists multiple types (`case T1, T2:`), `v` retains the interface type (since it could be either).

If no binding is used (`switch x.(type)`), the switch tests the type but binds nothing:

```go
switch x.(type) {
case *MemoryStore:
    log.Println("in-memory")
}
```

---

## The Empty Interface

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

The interface type with no methods is the empty interface:

```go
interface{}  // pre-Go1.18
any          // type alias for interface{}, added in Go 1.18 — preferred
```

Every type implements the empty interface. `any` is the type of values with unknown type at compile time.

```go
var v any = 42
v = "hello"
v = []byte{1, 2, 3}

// Retrieve the value:
s, ok := v.(string)
```

**Standard library uses:** `fmt.Println(a ...any)`, `json.Marshal(v any)`, `context.WithValue(parent Context, key, val any)`.

**Go 1.18+:** Prefer type parameters over `any` for new generic code:

```go
// Before Go 1.18 — uses any
func Keys(m map[string]any) []string { ... }

// Go 1.18+ — type-safe
func Keys[V any](m map[string]V) []string { ... }
```

---

## Interface Embedding (Composition)

Source: [spec#Interface_types](https://go.dev/ref/spec#Interface_types)

An interface may embed other interface types. The resulting interface has the union of all method sets.

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
type Writer interface {
    Write(p []byte) (n int, err error)
}
type Closer interface {
    Close() error
}

// Composed interfaces
type ReadWriter interface {
    Reader
    Writer
}
type ReadWriteCloser interface {
    Reader
    Writer
    Closer
}
```

The embedded interface names appear as unqualified identifiers (within the same package) or qualified identifiers (`io.Reader`):

```go
type ReadWriteCloser interface {
    io.Reader   // qualified — from package io
    io.Writer
    io.Closer
}
```

An interface cannot embed itself (cyclic embedding is a compile error).

---

## Standard Library Interfaces

### `io.Reader`

Source: [pkg.go.dev/io#Reader](https://pkg.go.dev/io#Reader)

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}
```

`Read` reads up to `len(p)` bytes into `p`. Returns `(0, io.EOF)` at end of stream. Implemented by: `*os.File`, `*bytes.Buffer`, `*strings.Reader`, `net.Conn`, `http.Response.Body`, compression readers, crypto readers.

### `io.Writer`

Source: [pkg.go.dev/io#Writer](https://pkg.go.dev/io#Writer)

```go
type Writer interface {
    Write(p []byte) (n int, err error)
}
```

`Write` writes `len(p)` bytes from `p`. Returns error if `n < len(p)`. Implemented by: `*os.File`, `*bytes.Buffer`, `net.Conn`, `http.ResponseWriter`, compression writers.

### `fmt.Stringer`

Source: [pkg.go.dev/fmt#Stringer](https://pkg.go.dev/fmt#Stringer)

```go
type Stringer interface {
    String() string
}
```

Types implementing `Stringer` are formatted using `String()` by `%v`, `%s`, and `Println`. The standard library checks for `Stringer` before falling back to reflection-based formatting.

### `error`

Source: [spec#Errors](https://go.dev/ref/spec#Errors)

```go
type error interface {
    Error() string
}
```

The built-in `error` interface. Any type with an `Error() string` method implements `error`. Nil `error` values signal success. The predeclared identifier `error` is the only built-in interface type.

### `sort.Interface`

Source: [pkg.go.dev/sort#Interface](https://pkg.go.dev/sort#Interface)

```go
type Interface interface {
    Len() int
    Less(i, j int) bool
    Swap(i, j int)
}
```

Implement on any collection type to use `sort.Sort`. `sort.Reverse` wraps it with an inverted `Less`. Since Go 1.21, `slices.SortFunc` is the preferred alternative for slices.

### `http.Handler`

Source: [pkg.go.dev/net/http#Handler](https://pkg.go.dev/net/http#Handler)

```go
type Handler interface {
    ServeHTTP(ResponseWriter, *Request)
}
```

Implemented by any HTTP endpoint. The `http.HandlerFunc` adapter converts a plain function to a `Handler`. All of Go's HTTP middleware chains are built on this one-method interface.

---

## Comparable Interfaces

Source: [spec#Comparison_operators](https://go.dev/ref/spec#Comparison_operators)

Interface values are comparable with `==` and `!=`. Two interface values are equal if both have identical dynamic types and equal dynamic values (or both are nil).

```go
var a, b Storer
a = &MemoryStore{id: "a"}
b = &MemoryStore{id: "a"}

fmt.Println(a == b)  // false — different pointer values
fmt.Println(a == a)  // true — same pointer

a = nil
b = nil
fmt.Println(a == b)  // true — both nil
```

**Panic risk:** If the dynamic type is not comparable (e.g., contains a slice or map), a runtime panic occurs when comparing:

```go
type NonComparable struct{ Tags []string }
var x any = NonComparable{Tags: []string{"a"}}
var y any = NonComparable{Tags: []string{"a"}}
_ = x == y  // PANIC: runtime error: comparing uncomparable type
```

Use `reflect.DeepEqual` or define `Equal` methods on types you need to compare by value.
