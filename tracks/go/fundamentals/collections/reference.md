# Go Specification Reference — Collections

> Extracted from [The Go Programming Language Specification](https://go.dev/ref/spec)
> and [The Go Standard Library](https://pkg.go.dev/std)
> for the `collections` module. Covers: array types, slice types, map types,
> slice expressions, making slices and maps, append, copy, delete, range, and sort.

---

## Array Types

Source: [spec#Array_types](https://go.dev/ref/spec#Array_types)

An array is a numbered sequence of elements of a single type, called the **element type**.
The number of elements is the **array length** and is never negative.

```go
[32]byte
[2*N] struct{ x, y int32 }
[1000]*float64
[3][5]int         // 3 arrays of 5 ints
[2][2][2]float64  // multi-dimensional
```

### Properties

- The length is **part of the array's type**. `[5]int` and `[10]int` are distinct types.
- The length must be a non-negative constant representable as `int`.
- The length of array `a` is `len(a)`. Length is always known at compile time.
- Array elements are addressed with non-negative integer indices `0` through `len(a)-1`.
- Array types are **comparable** if their element type is comparable. `[3]int` supports `==`.
- Arrays are **value types** — assigning or passing copies the entire array.

### Composite Literal

```go
[3]int{1, 2, 3}        // all elements specified
[10]int{1, 2, 3}       // remaining elements zero-valued
[...]int{1, 2, 3}      // length inferred from initializer: [3]int
[...]int{2: 5, 4: 8}   // index:value syntax: [0 0 5 0 8]
```

---

## Slice Types

Source: [spec#Slice_types](https://go.dev/ref/spec#Slice_types)

A slice is a descriptor for a contiguous segment of an **underlying array**. It provides
access to a numbered sequence of elements from that array. A slice type is written:

```go
[]T
```

The value of an uninitialized slice is `nil`. Unlike arrays, slice types have no
fixed length.

### Slice Internals

A slice value is a struct with three fields (not directly accessible):

| Field | Type | Description |
|---|---|---|
| `array` | `unsafe.Pointer` | Pointer to underlying array element |
| `len` | `int` | Number of elements in the slice |
| `cap` | `int` | Number of elements from `array` to end of underlying array |

### Built-in Functions for Slices

```go
len(s)        // number of elements
cap(s)        // capacity (elements to end of backing array)
append(s, x)  // append element(s); may allocate new backing array
copy(dst, src) // copy elements; returns number copied
```

---

## Slice Expressions

Source: [spec#Slice_expressions](https://go.dev/ref/spec#Slice_expressions)

Slice expressions construct a slice from an array, pointer-to-array, or slice.

### Simple Slice Expression

```go
a[low : high]
```

Constructs a slice with elements from index `low` to `high-1`.

| Property | Value |
|---|---|
| Length | `high - low` |
| Capacity | `cap(a) - low` |

Defaults: `low` defaults to `0`; `high` defaults to `len(a)`.

```go
a := [5]int{1, 2, 3, 4, 5}
s := a[1:4]    // [2, 3, 4], len=3, cap=4
t := a[:]      // same as a[0:5], len=5, cap=5
u := a[2:]     // same as a[2:5], len=3, cap=3
v := a[:3]     // same as a[0:3], len=3, cap=5
```

### Full Slice Expression

```go
a[low : high : max]
```

Constructs a slice with explicitly bounded capacity.

| Property | Value |
|---|---|
| Length | `high - low` |
| Capacity | `max - low` |

Constraint: `0 <= low <= high <= max <= cap(a)`

```go
a := [5]int{1, 2, 3, 4, 5}
s := a[1:3:4]  // [2, 3], len=2, cap=3
```

The full slice expression is only valid for arrays, pointer-to-arrays, or slices.
It is **not** valid for strings.

---

## Making Slices and Maps

Source: [spec#Making_slices_maps_and_channels](https://go.dev/ref/spec#Making_slices_maps_and_channels)

The built-in `make` function allocates and initializes objects of slice, map, or channel types.

### make([]T, n)

```go
make([]T, n)      // len=n, cap=n
make([]T, n, m)   // len=n, cap=m (m >= n required)
```

Creates a slice with the given length and capacity. All elements are initialized to
their zero value.

### make(map[K]V)

```go
make(map[K]V)          // empty map, capacity hint unspecified
make(map[K]V, hint)    // empty map with suggested initial capacity
```

The capacity hint is advisory — the map will grow beyond it as needed.

---

## append

Source: [spec#Appending_and_copying_slices](https://go.dev/ref/spec#Appending_and_copying_slices)

```go
append(s S, x ...T) S
```

Appends elements to the end of slice `s` and returns the result. If capacity is
exceeded, a new underlying array is allocated. The original slice is unmodified.

```go
s0 := []int{0, 0}
s1 := append(s0, 2)          // [0, 0, 2]
s2 := append(s1, 3, 5, 7)   // [0, 0, 2, 3, 5, 7]
s3 := append(s2, s0...)      // [0, 0, 2, 3, 5, 7, 0, 0]
s4 := append(s3[3:6:6], s1[1:3]...) // [3, 5, 7, 0, 2] — full slice limits cap
```

Special case: `append` can append a `string` to a `[]byte`:

```go
b := append([]byte("hello "), "world"...)  // []byte("hello world")
```

### Growth Behavior (Implementation Detail, Not Spec)

When `len == cap`, `append` allocates a new array. The new capacity is at least twice
the old capacity for small slices. For large slices (Go 1.18+), growth uses a smoother
formula to balance memory usage and reallocation frequency.

---

## copy

Source: [spec#Appending_and_copying_slices](https://go.dev/ref/spec#Appending_and_copying_slices)

```go
copy(dst, src []T) int
```

Copies elements from `src` to `dst`. Returns the number of elements copied, which
is `min(len(dst), len(src))`. Source and destination may overlap.

```go
var a = [...]int{0, 1, 2, 3, 4, 5, 6, 7}
var s = make([]int, 6)
n1 := copy(s, a[0:])    // n1 = 6, s = [0, 1, 2, 3, 4, 5]
n2 := copy(s, s[2:])    // n2 = 4, s = [2, 3, 4, 5, 4, 5] — overlapping is valid
```

Special case: copy also copies bytes from a string to a `[]byte`:

```go
b := make([]byte, 5)
n := copy(b, "Hello!")   // n = 5, b = [72 101 108 108 111]
```

---

## Map Types

Source: [spec#Map_types](https://go.dev/ref/spec#Map_types)

An unordered group of elements of one type, indexed by unique keys of another type:

```go
map[KeyType]ElementType
```

### Key Type Constraints

The key type must support equality operators `==` and `!=`. Valid key types include:
- Boolean, integer, float, complex, string
- Pointers, array types (if element type is comparable)
- Struct types (if all fields are comparable)
- Interface types (dynamic type must support `==` at runtime)

Invalid key types: functions, maps, slices.

### Map Properties

- An uninitialized map value is `nil`.
- A nil map **behaves like an empty map when reading** (returns zero value for missing keys).
- A nil map **panics on write**.
- The number of elements is `len(m)`.
- Element type does not need to be comparable.
- Maps are **not safe for concurrent read+write**. Use `sync.RWMutex` or `sync.Map`.

### Map Operations

```go
m := make(map[string]int)

// Set
m["key"] = 42

// Get (zero value if missing)
v := m["missing"]        // v = 0

// Get with existence check
v, ok := m["key"]        // v=42, ok=true
v, ok := m["missing"]    // v=0, ok=false

// Delete (no-op if key missing)
delete(m, "key")

// Length
n := len(m)

// Iteration (order not guaranteed)
for k, v := range m {
    _ = k
    _ = v
}
```

---

## delete

Source: [spec#Deletion_of_map_elements](https://go.dev/ref/spec#Deletion_of_map_elements)

```go
delete(m, k)
```

Removes the element with key `k` from map `m`. If `m` is `nil` or there is no such
element, `delete` is a no-op.

---

## range

Source: [spec#For_range](https://go.dev/ref/spec#For_range)

The `range` expression iterates over a collection. The iteration variables receive
copies of the element values.

### Range Over Slice or Array

```go
for i, v := range s {  // i: index (int), v: copy of s[i]
    ...
}
```

| Form | Variables |
|---|---|
| `for i, v := range s` | index and value |
| `for i := range s` | index only |
| `for _, v := range s` | value only |
| `for range s` | neither (Go 1.22+) |

### Range Over Map

```go
for k, v := range m {  // k: key, v: copy of value
    ...
}
for k := range m {     // key only
    ...
}
```

Iteration order is **not specified** and is not guaranteed to be the same from one
iteration to the next. If a map entry is deleted during iteration, the corresponding
iteration value will not be produced. If a map entry is created during iteration, it
may or may not be produced.

### Range Over String

```go
for i, r := range s {  // i: byte index of rune start, r: rune (int32)
    ...
}
```

Iteration decodes UTF-8. Each `r` is a Unicode code point. If an invalid UTF-8
sequence is encountered, `r` is the replacement character `U+FFFD` and the next
iteration advances by one byte.

### Range Over Integer (Go 1.22+)

```go
for i := range 5 {  // i: 0, 1, 2, 3, 4
    ...
}
```

---

## Sorting — sort package

Source: [pkg.go.dev/sort](https://pkg.go.dev/sort)

The `sort` package provides primitives for sorting slices and user-defined collections.

### Convenience Functions

```go
sort.Ints(s []int)       // sorts in ascending order
sort.Float64s(s []float64)
sort.Strings(s []string)

sort.IntsAreSorted(s []int) bool
sort.StringsAreSorted(s []string) bool
```

### sort.Slice

```go
sort.Slice(s interface{}, less func(i, j int) bool)
sort.SliceStable(s interface{}, less func(i, j int) bool)
```

Sorts the slice `s` using the `less` function to determine order. `SliceStable`
preserves the original order of equal elements.

The `less` function must define a strict weak ordering:
- `less(i, i)` must be `false` (irreflexivity)
- If `less(i, j)` then `!less(j, i)` (asymmetry)
- If `less(i, j)` and `less(j, k)` then `less(i, k)` (transitivity)

### sort.Interface

```go
type Interface interface {
    Len() int
    Less(i, j int) bool
    Swap(i, j int)
}

sort.Sort(data Interface)
sort.Stable(data Interface)
sort.Search(n int, f func(int) bool) int  // binary search
```

`sort.Search` finds the smallest index `i` in `[0, n)` at which `f(i)` is true,
assuming that `f(i) == true` implies `f(i+1) == true`. Returns `n` if no such index exists.

---

## Sorting — slices package (Go 1.21+)

Source: [pkg.go.dev/slices](https://pkg.go.dev/slices)

Generic alternatives to the `sort` package with type safety:

```go
slices.Sort[S ~[]E, E constraints.Ordered](x S)
slices.SortFunc[S ~[]E, E any](x S, cmp func(a, b E) int)
slices.SortStableFunc[S ~[]E, E any](x S, cmp func(a, b E) int)

slices.IsSorted[S ~[]E, E constraints.Ordered](x S) bool
slices.IsSortedFunc[S ~[]E, E any](x S, cmp func(a, b E) int) bool

slices.BinarySearch[S ~[]E, E constraints.Ordered](x S, target E) (int, bool)
slices.BinarySearchFunc[S ~[]E, E, T any](x S, target T, cmp func(E, T) int) (int, bool)

slices.Reverse[S ~[]E, E any](s S)
slices.Contains[S ~[]E, E comparable](s S, v E) bool
slices.Index[S ~[]E, E comparable](s S, v E) int   // -1 if not found
```

The `cmp` function for `SortFunc` must return:
- Negative if `a < b`
- Zero if `a == b`
- Positive if `a > b`

Use `cmp.Compare` from the `cmp` package for ordered types:

```go
import "cmp"
slices.SortFunc(events, func(a, b Event) int {
    return cmp.Compare(a.Timestamp, b.Timestamp)
})
```

---

## maps package (Go 1.21+)

Source: [pkg.go.dev/maps](https://pkg.go.dev/maps)

```go
maps.Keys[Map ~map[K]V, K comparable, V any](m Map) []K
maps.Values[Map ~map[K]V, K comparable, V any](m Map) []V
maps.Equal[Map1, Map2 ~map[K]V, K, V comparable](m1 Map1, m2 Map2) bool
maps.Clone[Map ~map[K]V, K comparable, V any](m Map) Map
maps.Copy[Map1 ~map[K]V, Map2 ~map[K]V, K comparable, V any](dst Map1, src Map2)
maps.Delete[Map ~map[K]V, K comparable](m Map, keys ...K)
```

---

## Capacity Growth Reference

| Operation | When Reallocation Occurs | Notes |
|---|---|---|
| `append` to slice | When `len == cap` | New cap ≥ old cap × 2 (small); smaller factor for large slices |
| Map write | When load factor > ~6.5 | Map doubles bucket count and rehashes |
| `make([]T, n, m)` | Never (pre-allocated) | `m >= n` required |
| `make(map[K]V, hint)` | Rarely initially | Hint sets initial bucket count |

---

## Zero Value Summary for Collections

| Type | Zero Value | Readable? | Writable? |
|---|---|---|---|
| `[N]T` (array) | All elements zeroed | Yes | Yes |
| `[]T` (slice) | `nil` | Yes (len=0, cap=0) | Via `append` only |
| `map[K]V` | `nil` | Yes (returns zero value) | **No — panics** |

---

## References

- [Go Specification — Array types](https://go.dev/ref/spec#Array_types)
- [Go Specification — Slice types](https://go.dev/ref/spec#Slice_types)
- [Go Specification — Slice expressions](https://go.dev/ref/spec#Slice_expressions)
- [Go Specification — Making slices, maps, and channels](https://go.dev/ref/spec#Making_slices_maps_and_channels)
- [Go Specification — Appending and copying slices](https://go.dev/ref/spec#Appending_and_copying_slices)
- [Go Specification — Map types](https://go.dev/ref/spec#Map_types)
- [Go Specification — For statements with range clause](https://go.dev/ref/spec#For_range)
- [Go Blog — Go Slices: usage and internals](https://go.dev/blog/slices-intro)
- [Go Blog — Arrays, slices (and strings): The mechanics of append](https://go.dev/blog/slices)
- [pkg.go.dev/sort](https://pkg.go.dev/sort)
- [pkg.go.dev/slices](https://pkg.go.dev/slices)
- [pkg.go.dev/maps](https://pkg.go.dev/maps)
