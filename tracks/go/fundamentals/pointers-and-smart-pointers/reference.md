# Pointers — Go Language Reference

> Extracted from the [Go Language Specification](https://go.dev/ref/spec) and [Go standard library documentation](https://pkg.go.dev/). Last verified against Go 1.22.

---

## Pointer Types

**Spec:** [Pointer types](https://go.dev/ref/spec#Pointer_types)

A pointer type denotes the set of all pointers to variables of a given type, called the *base type* of the pointer.

```
PointerType = "*" BaseType .
BaseType    = Type .
```

```go
*Point    // pointer to Point
*[4]int   // pointer to array of 4 ints
```

The zero value of a pointer type is `nil`.

Pointers are comparable. Two pointer values are equal if they point to the same variable, or if both are `nil`.

---

## Address Operators

**Spec:** [Address operators](https://go.dev/ref/spec#Address_operators)

For an operand `x` of type `T`, the address operation `&x` generates a pointer of type `*T` to `x`. The operand must be *addressable*: a variable, pointer indirection, or slice indexing operation; or a field selector of an addressable struct operand; or an array indexing operation of an addressable array.

```go
&x        // address of variable x
&a[i]     // address of array element
&s.Field  // address of struct field (if s is addressable)
```

For an operand `p` of pointer type `*T`, the pointer indirection `*p` denotes the variable of type `T` pointed to by `p`. If `p == nil`, attempting `*p` causes a run-time panic.

```go
*p        // value pointed to by p
*p = val  // assignment through pointer
```

**Addressability rules — operands that are NOT addressable:**

- Map elements: `m["key"]` is not addressable
- Values returned directly by function calls
- String bytes: `str[i]` is not addressable
- Interface values

---

## new Built-in Function

**Spec:** [Allocation](https://go.dev/ref/spec#Allocation)

The built-in function `new` takes a type `T`, allocates storage for a variable of that type at run time, and returns a value of type `*T` pointing to it. The variable is initialized to the zero value.

```go
func new(Type) *Type
```

```go
p := new(int)        // *int — points to a zero-initialized int
q := new(SomeStruct) // *SomeStruct — all fields zero-initialized
```

`new(T)` is equivalent to `&T{}` for types with a composite literal syntax.

---

## Variable Declarations and Addressability

**Spec:** [Variables](https://go.dev/ref/spec#Variables)

A variable is a storage location holding a value. Every variable has a type. Variables declared with `var`, short variable declarations `:=`, function parameters, and composite literal components are all addressable.

```go
var x int     // addressable: &x is valid
y := 42       // addressable: &y is valid
a := [3]int{1, 2, 3}  // addressable: &a[0] is valid
```

---

## Method Sets

**Spec:** [Method sets](https://go.dev/ref/spec#Method_sets)

The *method set* of a type determines the interfaces it implements and the methods that can be called using a receiver of that type.

| Type | Method Set |
|------|-----------|
| `T` | Methods declared with receiver type `T` |
| `*T` | Methods declared with receiver type `T` or `*T` |
| Interface type | All methods listed in the interface |

A value of type `T` does not have pointer receiver methods in its method set. A value of type `*T` has both value and pointer receiver methods.

**Automatic address-taking:**

If `x` is addressable and `&x`'s method set contains `m`, `x.m()` is shorthand for `(&x).m()`. The compiler inserts the address-taking automatically.

```go
var c Counter
c.Increment()  // same as (&c).Increment() if Increment has *Counter receiver
```

This automatic promotion does NOT apply to interface method dispatch — the interface stores the value as-is. If you store a `Counter` (not `*Counter`) in an interface that requires `*Counter` methods, it is a compile error.

---

## Composite Literals and Pointer Allocation

**Spec:** [Composite literals](https://go.dev/ref/spec#Composite_literals)

Taking the address of a composite literal allocates a new variable initialized with the literal's value:

```go
p := &Point{1, 2}    // *Point
s := &[]int{1, 2, 3} // *[]int
```

This is the idiomatic way to allocate a struct on the heap with initial values. Equivalent to:

```go
tmp := Point{1, 2}
p := &tmp
```

---

## Pointer Indirection in Selectors

**Spec:** [Selectors](https://go.dev/ref/spec#Selectors)

For a primary expression `x` of type `*T`, `x.f` is shorthand for `(*x).f`. The pointer is automatically dereferenced for field access and method calls:

```go
type Point struct{ X, Y int }
p := &Point{1, 2}
p.X = 10    // same as (*p).X = 10
fmt.Println(p.Y) // same as (*p).Y
```

---

## unsafe Package

**Reference:** [unsafe package](https://pkg.go.dev/unsafe)

The `unsafe` package provides operations that bypass Go's type system. Its use is discouraged in application code.

```go
type Pointer *ArbitraryType
```

`unsafe.Pointer` is a pointer type that can be converted to/from any other pointer type. It allows:

1. A value of any pointer type can be converted to `unsafe.Pointer`
2. An `unsafe.Pointer` value can be converted to any pointer type
3. A `uintptr` can be converted to `unsafe.Pointer`
4. An `unsafe.Pointer` can be converted to `uintptr`

```go
// Size and alignment functions:
unsafe.Sizeof(x)          // size in bytes of variable x
unsafe.Alignof(x)         // alignment requirement
unsafe.Offsetof(s.Field)  // byte offset of a field within a struct
```

**Key constraint:** `unsafe.Pointer` values must follow specific conversion patterns; the GC does not track arbitrary `uintptr` values as roots.

---

## sync.Pool

**Reference:** [sync.Pool](https://pkg.go.dev/sync#Pool)

```go
type Pool struct {
    New func() any
    // contains filtered or unexported fields
}

func (p *Pool) Get() any
func (p *Pool) Put(x any)
```

A `Pool` is a set of temporary objects that may be individually saved and retrieved. The purpose is to cache allocated but unused items for later reuse, relieving pressure on the garbage collector.

```go
var p sync.Pool
p.New = func() any { return &bytes.Buffer{} }

buf := p.Get().(*bytes.Buffer)
// use buf
buf.Reset()
p.Put(buf)
```

**Behavior:**
- `Get` selects an arbitrary item from the pool, removes it, and returns it. If the pool has no items, `Get` calls `New` (if non-nil) or returns `nil`.
- `Put` adds an item to the pool.
- A Pool's contents may be freed automatically at any time without notification — the GC may clear pool contents between GC cycles.
- A Pool is safe for use by multiple goroutines simultaneously.

---

## Pointer Safety and the Garbage Collector

**Reference:** [Go GC guide](https://go.dev/doc/gc-guide)

Go's garbage collector is a concurrent, tricolor mark-and-sweep GC. Key properties relevant to pointers:

- **Safety:** The GC scans all reachable pointers. A value is live as long as at least one pointer to it is reachable from a GC root (globals, stack frames).
- **No dangling pointers:** If you hold a pointer, the GC will not collect the pointed-to value.
- **No weak references** in the standard model (Go 1.23+ adds `weak.Pointer` in `runtime/weak`).
- **Escape analysis:** The compiler determines whether a value escapes to the heap. Local variables that don't escape are stack-allocated (cheaper). Variables whose addresses escape to longer-lived memory are heap-allocated.

To inspect escape analysis decisions:
```
go build -gcflags='-m' ./...
```

---

## Comparison of Pointer Allocation Methods

| Method | Syntax | Use when |
|--------|--------|----------|
| Composite literal | `&T{field: val}` | Struct with initial values (most common) |
| `new` built-in | `new(T)` | Pointer to zero value, single allocation, scalar types |
| Variable address | `x := T{}; &x` | When you need a named variable before taking address |

---

## Related Specifications

- [Spec: Types](https://go.dev/ref/spec#Types)
- [Spec: Expressions](https://go.dev/ref/spec#Expressions)
- [Spec: Statements — short variable declaration](https://go.dev/ref/spec#Short_variable_declarations)
- [Effective Go: Pointers vs. Values](https://go.dev/doc/effective_go#pointers_vs_values)
- [Effective Go: Allocation with new](https://go.dev/doc/effective_go#allocation_new)
- [Go FAQ: When is it appropriate to use a pointer to an interface?](https://go.dev/doc/faq#pointer_to_interface)
- [Go Blog: The Go Memory Model](https://go.dev/ref/mem)
