# Collections — Go

## The Three Collection Types

Go gives you three built-in collection types: **arrays**, **slices**, and **maps**. Each solves a different problem. Understanding when to reach for each — and knowing their internal mechanics — is where a lot of Go bugs either get introduced or avoided.

Quick orientation:

| Type | Fixed size? | Value or reference? | Zero value | Use case |
|---|---|---|---|---|
| `[N]T` array | Yes | **Value** (copied whole) | All elements zeroed | Checksums, fixed-size buffers, crypto keys |
| `[]T` slice | No | **Reference** (header + backing array) | `nil` | Almost everything |
| `map[K]V` | No | **Reference** | `nil` (but panics on write!) | Lookups, deduplication, grouping |

In practice, you'll use slices 80% of the time, maps 18%, and arrays rarely.

---

## Arrays

Arrays are the foundation that slices build on. In Go, an array's size is part of its type.

```go
var a [3]int           // zero value: [0, 0, 0]
b := [3]int{1, 2, 3}  // literal
c := [...]int{4, 5, 6} // ... lets the compiler count
```

`[3]int` and `[5]int` are **different types**. You cannot pass one where the other is expected. This is unlike C, where `int[]` erases the size.

### Arrays Are Value Types

This is the biggest thing to internalize. When you assign or pass an array, you copy the entire thing.

```go
a := [3]int{1, 2, 3}
b := a          // b is a full copy — not a reference
b[0] = 99
fmt.Println(a)  // [1 2 3] — a is unchanged
fmt.Println(b)  // [99 2 3]
```

**JS comparison:** In JavaScript, arrays are objects and are always passed by reference. Assigning `const b = a` just gives you another pointer to the same array. This is the opposite of Go's behavior.

### When to Use Arrays

Arrays are rare in Go. Use them when:
- The size is fixed at compile time and meaningful (e.g., `[32]byte` for a SHA-256 hash, `[4]byte` for an IPv4 address)
- You explicitly want value semantics (copies, no aliasing)
- You're working with `unsafe` or interfacing with C code

For everything else, use slices.

### Your notes
<!-- -->

---

## Slices

Slices are the workhorse of Go collections. Under the hood, a slice is a three-field struct:

```
type slice struct {
    array unsafe.Pointer  // pointer to the backing array
    len   int             // number of elements accessible
    cap   int             // total size of the backing array from this point
}
```

When you write `s := []int{1, 2, 3}`, Go allocates an array on the heap and gives you a slice header on the stack that points into it.

### Visual: The Slice Header

```
Stack                    Heap (backing array)
┌─────────────┐         ┌───┬───┬───┬───┬───┐
│ ptr ────────┼────────▶│ 1 │ 2 │ 3 │   │   │
│ len = 3     │         └───┴───┴───┴───┴───┘
│ cap = 5     │
└─────────────┘
```

The slice header is 24 bytes (3 × 8-byte fields on a 64-bit system). It's small enough to copy cheaply. The actual data lives in the heap.

### Length vs Capacity

- `len(s)` — how many elements you can currently access
- `cap(s)` — how many elements the backing array can hold from the start of this slice

```go
s := make([]int, 3, 5)  // len=3, cap=5
fmt.Println(len(s), cap(s))  // 3 5
s = append(s, 99)
fmt.Println(len(s), cap(s))  // 4 5 — no reallocation
```

### Creating Slices

Three ways:

```go
// 1. Slice literal (common for small, known data)
names := []string{"alice", "bob", "carol"}

// 2. make (common for pre-sized allocations)
events := make([]Event, 0, 100)  // len=0, cap=100 — preallocate for 100 events

// 3. From an array or existing slice
arr := [5]int{1, 2, 3, 4, 5}
s := arr[1:3]  // [2, 3], shares arr's memory
```

**When to use `make`:** When you know the approximate final size and want to avoid repeated reallocations. `make([]T, 0, n)` gives you an empty slice that won't reallocate until you've appended n elements.

### append: Growth Mechanics

`append` is the idiomatic way to grow a slice. It returns a new slice header — the original is unchanged.

```go
s := []int{1, 2, 3}
s = append(s, 4)      // must reassign — append returns a new header
s = append(s, 5, 6)   // append multiple at once
other := []int{7, 8}
s = append(s, other...) // append a whole slice with ...
```

When `len == cap`, `append` allocates a new, larger backing array, copies the existing elements, and returns a new header pointing at the new array. The growth factor is roughly 2× for small slices, gradually decreasing as slices get larger (Go 1.18+ uses a more nuanced formula).

**Key point:** After a reallocation, the old backing array is no longer referenced by the slice. This is both a safety feature and a common source of bugs.

### Backing Array Sharing — The Main Gotcha

Two slices can share the same backing array. Writes through one are visible through the other.

```go
a := []int{1, 2, 3, 4, 5}
b := a[1:3]  // [2, 3], shares a's backing array

b[0] = 99
fmt.Println(a)  // [1 99 3 4 5] — a is modified!
fmt.Println(b)  // [99 3]
```

Visual:

```
a:  ptr─┐  len=5  cap=5
        ▼
       ┌───┬────┬───┬───┬───┐
       │ 1 │ 99 │ 3 │ 4 │ 5 │
       └───┴────┴───┴───┴───┘
b:         ptr─┘  len=2  cap=4
```

`b` starts at the second element of `a`'s array. They overlap. Mutating `b[0]` mutates `a[1]`.

This is usually fine inside a single function. It becomes a bug when you pass a sub-slice somewhere that doesn't expect to share memory.

### append on Sub-Slices: The Subtle Bug

```go
a := []int{1, 2, 3, 4, 5}
b := a[0:2]  // len=2, cap=5

// b still has cap 5 — appending doesn't immediately reallocate!
b = append(b, 99)
fmt.Println(a)   // [1 2 99 4 5] — a[2] was silently overwritten!
```

When you take a sub-slice, it inherits the remaining capacity of the backing array. Appending within that capacity overwrites the tail of the original slice — silently, with no error.

This is one of the most common Go slice bugs in code review.

### Slice Expressions: Two Forms

**Simple form:** `s[low:high]`
- `len = high - low`
- `cap = cap(s) - low` — inherits full remaining capacity

**Full form (three-index):** `s[low:high:max]`
- `len = high - low`
- `cap = max - low` — you control the cap

The full form lets you prevent the "append overwrites original" bug:

```go
a := []int{1, 2, 3, 4, 5}
// Three-index slice: cap limited to 2 — append will always reallocate
b := a[0:2:2]  // len=2, cap=2
b = append(b, 99)  // reallocates — a is safe
fmt.Println(a)     // [1 2 3 4 5] — unchanged
```

Use the three-index form when handing a sub-slice to code that will append to it.

### copy(): Preventing Aliasing

`copy` copies elements between slices without sharing memory:

```go
src := []int{1, 2, 3, 4, 5}
dst := make([]int, len(src))
n := copy(dst, src)  // returns number of elements copied
dst[0] = 99
fmt.Println(src)  // [1 2 3 4 5] — unchanged
fmt.Println(dst)  // [99 2 3 4 5]
```

`copy` copies `min(len(dst), len(src))` elements. The destination must already be sized — `copy` won't grow it.

**Common use:** Clone a slice before passing it somewhere that might mutate it:

```go
func processEvents(events []Event) []Event {
    // Work on a private copy — don't mutate the caller's slice
    local := make([]Event, len(events))
    copy(local, events)
    // ... process local ...
    return local
}
```

### nil vs empty slice

This distinction trips up Go beginners (and sometimes interviewers):

```go
var a []int        // nil slice:   a == nil, len=0, cap=0
b := []int{}       // empty slice: b != nil, len=0, cap=0
c := make([]int, 0) // empty slice: c != nil, len=0, cap=0
```

**Behavior:**
- Both can be ranged over (zero iterations)
- Both can be appended to
- Both have len=0
- `nil` slice marshals to `null` in JSON; empty slice marshals to `[]`
- `reflect.DeepEqual(nil, []int{})` returns `false`

**Rule of thumb:**
- Return `nil` from functions to signal "nothing", `[]T{}` to signal "empty collection"
- Use `var s []T` to declare — don't allocate until you need to append

### Your notes
- The slice header is 24 bytes on stack (pointer 8 + len 8 + cap 8). Passing a slice to a function copies the header, not the data. The function sees the same backing array. This is why slices "feel" like references.
- `append` always returns a new header. Forgetting to reassign is a common mistake: `append(s, x)` (wrong) vs `s = append(s, x)` (right).
- Three-index slices (`a[0:2:2]`) are underused and often exactly the right tool when writing library code.

---

## Maps

### Internal Structure

Go maps are hash tables implemented as arrays of "buckets". Each bucket holds up to 8 key-value pairs. When a bucket fills, Go chains overflow buckets and rehashes when the load factor exceeds ~6.5 items per bucket.

You don't need to understand the internal implementation to use maps correctly, but knowing that:
- Key lookup is O(1) average, O(n) worst case
- Iteration order is **intentionally randomized** (since Go 1.0)
- Maps are **not safe for concurrent use** (use `sync.Map` or a mutex if needed)

...helps you make the right decisions.

### Creating Maps

```go
// Literal — when you know the initial data
config := map[string]string{
    "host": "localhost",
    "port": "8080",
}

// make — for empty maps
cache := make(map[string][]byte)
cache := make(map[string][]byte, 1000)  // hint: expect ~1000 entries
```

Never use `var m map[K]V` if you're going to write to it. The zero value of a map is `nil`, which panics on write. Always `make()` or use a literal.

```go
var m map[string]int  // m is nil
m["key"] = 1          // PANIC: assignment to entry in nil map

// Fix:
m := make(map[string]int)
m["key"] = 1  // fine
```

### Reading from Maps: The Comma-Ok Pattern

Map reads always succeed — missing keys return the zero value. Use the comma-ok pattern to distinguish "key has value zero" from "key is missing":

```go
counts := map[string]int{"errors": 0}

// Single-value form — returns zero for missing keys
n := counts["requests"]  // 0 — but is it missing, or zero?

// Comma-ok form — unambiguous
n, ok := counts["requests"]  // n=0, ok=false (missing)
n, ok := counts["errors"]    // n=0, ok=true (present, value is 0)
```

In production code, reach for comma-ok whenever the absence of a key has different meaning than a zero value.

### Deleting and Checking

```go
m := map[string]int{"a": 1, "b": 2}

delete(m, "a")           // delete key "a" — no-op if missing
delete(m, "nonexistent") // safe — doesn't panic

if _, ok := m["a"]; !ok {
    // key is gone
}
```

### Iteration Order is Randomized

Go explicitly randomizes map iteration order on every run to prevent programs from accidentally depending on it:

```go
m := map[string]int{"c": 3, "a": 1, "b": 2}
for k, v := range m {
    fmt.Printf("%s: %d\n", k, v)
}
// Output order varies every run — never rely on it
```

If you need consistent ordering, sort the keys:

```go
keys := make([]string, 0, len(m))
for k := range m {
    keys = append(keys, k)
}
sort.Strings(keys)
for _, k := range keys {
    fmt.Printf("%s: %d\n", k, m[k])
}
```

### Sets via map[T]struct{}

Go has no built-in set type. The idiomatic pattern is a map with `struct{}` as the value:

```go
seen := make(map[string]struct{})

// Add
seen["user-123"] = struct{}{}

// Check membership
if _, ok := seen["user-123"]; ok {
    // already seen
}

// Delete
delete(seen, "user-123")
```

Why `struct{}`? It's a zero-size type — the map consumes no memory for the values, only for the keys. Using `map[T]bool` is also common and slightly more readable:

```go
seen := make(map[string]bool)
seen["user-123"] = true
if seen["user-123"] {
    // already seen — no comma-ok needed because missing gives false
}
```

The `struct{}` version uses slightly less memory (no bool per key). For large sets the difference matters; for small sets, use whichever reads better.

### Map Gotchas

**Maps are not comparable with `==`.** You can compare a map to `nil`, but not to another map:

```go
var a, b map[string]int
fmt.Println(a == nil)  // true — fine
// fmt.Println(a == b) // compile error: maps cannot be compared
```

Use `reflect.DeepEqual` or write your own comparison.

**Struct values in maps require a read-modify-write cycle:**

```go
type Stats struct{ Count int }
m := map[string]Stats{"key": {Count: 0}}

// m["key"].Count++ // compile error: cannot assign to map index expression

// Fix: read-modify-write
s := m["key"]
s.Count++
m["key"] = s
```

The reason: `m["key"]` isn't addressable. You can't take the address of a map element. Use pointers as values if you want in-place mutation:

```go
m := map[string]*Stats{"key": {Count: 0}}
m["key"].Count++  // fine — m["key"] is a pointer, we're dereferencing it
```

### Your notes
- The nil map gotcha is covered in [[pitfalls/go-nil-map-panic]]. Summary: readable nil maps return zero values; writable nil maps panic. Always `make()`.
- The "can't assign to map index expression" error is because map values aren't addressable — Go can't give you a pointer to the slot in the hash table (it might move during rehashing). Read-modify-write or use pointer values.
- Map randomization was introduced to break programs that accidentally relied on iteration order. If you see code sorting map keys before iteration, that's why.

---

## range

The `range` keyword iterates over arrays, slices, maps, strings, and channels.

```go
nums := []int{10, 20, 30}

// Index + value
for i, v := range nums {
    fmt.Printf("[%d] = %d\n", i, v)
}

// Index only
for i := range nums {
    fmt.Printf("index %d\n", i)
}

// Value only (blank identifier for index)
for _, v := range nums {
    fmt.Printf("value %d\n", v)
}
```

### range Copies Values

The loop variable `v` is a **copy** of the element. Mutating `v` does not mutate the slice:

```go
events := []Event{{Type: "click"}, {Type: "submit"}}
for _, e := range events {
    e.Type = "processed"  // modifies local copy — events is unchanged
}
// events[0].Type is still "click"
```

To mutate in place, use the index:

```go
for i := range events {
    events[i].Type = "processed"  // modifies the original
}
```

Or use a pointer-element slice: `[]*Event`.

### range on Maps

```go
m := map[string]int{"a": 1, "b": 2}
for key, val := range m {
    fmt.Printf("%s: %d\n", key, val)
}

// Key only
for key := range m {
    fmt.Println(key)
}
```

Iteration order is random on every run.

### range on Strings

Iterating a string with `range` decodes UTF-8 runes, not bytes:

```go
for i, r := range "café" {
    fmt.Printf("byte offset %d: %c\n", i, r)
}
// byte offset 0: c
// byte offset 1: a
// byte offset 2: f
// byte offset 3: é   (index jumps 2 — é is 2 bytes)
```

To iterate bytes: `for i := 0; i < len(s); i++ { ... }`.

### Your notes
<!-- -->

---

## Sorting

### sort package

```go
import "sort"

nums := []int{5, 2, 8, 1, 9}
sort.Ints(nums)    // in place: [1 2 5 8 9]

strs := []string{"banana", "apple", "cherry"}
sort.Strings(strs) // in place: [apple banana cherry]
```

### sort.Slice — Ad-Hoc Comparators

For custom types or custom orderings, `sort.Slice` takes a less function:

```go
type Event struct {
    Timestamp int64
    Priority  int
    Type      string
}

events := []Event{
    {Timestamp: 1000, Priority: 2, Type: "click"},
    {Timestamp: 500,  Priority: 1, Type: "hover"},
    {Timestamp: 750,  Priority: 3, Type: "submit"},
}

// Sort by timestamp ascending
sort.Slice(events, func(i, j int) bool {
    return events[i].Timestamp < events[j].Timestamp
})

// Sort by priority descending, then timestamp ascending for ties
sort.SliceStable(events, func(i, j int) bool {
    if events[i].Priority != events[j].Priority {
        return events[i].Priority > events[j].Priority
    }
    return events[i].Timestamp < events[j].Timestamp
})
```

Use `sort.SliceStable` when equal elements must keep their original relative order (stable sort).

### sort.Interface — Implementing Sort for Your Type

For types you sort repeatedly or want to make sortable in a reusable way:

```go
type ByTimestamp []Event

func (s ByTimestamp) Len() int           { return len(s) }
func (s ByTimestamp) Less(i, j int) bool { return s[i].Timestamp < s[j].Timestamp }
func (s ByTimestamp) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }

sort.Sort(ByTimestamp(events))
```

### slices package (Go 1.21+)

The newer `slices` package provides generic versions of common operations:

```go
import "slices"

nums := []int{5, 2, 8, 1}
slices.Sort(nums)                          // sorts in place

idx, found := slices.BinarySearch(nums, 5) // binary search on sorted slice

slices.Reverse(nums)                       // reverses in place

// Custom sort with generics
slices.SortFunc(events, func(a, b Event) int {
    return cmp.Compare(a.Timestamp, b.Timestamp)
})
```

The `slices` package is preferred in new code for its generic, type-safe API.

### Your notes
<!-- -->

---

## Common Patterns

### Filter

Pre-generics idiom (still common in codebases targeting <1.18):

```go
func filter(events []Event, predicate func(Event) bool) []Event {
    result := make([]Event, 0)  // or: result := events[:0] to reuse backing array
    for _, e := range events {
        if predicate(e) {
            result = append(result, e)
        }
    }
    return result
}

errors := filter(events, func(e Event) bool {
    return e.Type == "error"
})
```

With generics (Go 1.18+):

```go
func Filter[T any](s []T, f func(T) bool) []T {
    result := make([]T, 0, len(s))
    for _, v := range s {
        if f(v) {
            result = append(result, v)
        }
    }
    return result
}
```

### Map (Transform)

```go
func Map[T, U any](s []T, f func(T) U) []U {
    result := make([]U, len(s))
    for i, v := range s {
        result[i] = f(v)
    }
    return result
}

types := Map(events, func(e Event) string { return e.Type })
```

### Reduce

```go
func Reduce[T, U any](s []T, initial U, f func(U, T) U) U {
    result := initial
    for _, v := range s {
        result = f(result, v)
    }
    return result
}

total := Reduce(events, 0, func(acc int, e Event) int {
    return acc + e.PayloadSize
})
```

**Note:** The standard library doesn't ship map/filter/reduce — you write them or pull in `golang.org/x/exp/slices` (experimental). The pattern is idiomatic Go but the implementations are per-project.

### Grouping with Maps

```go
// Group events by type
byType := make(map[string][]Event)
for _, e := range events {
    byType[e.Type] = append(byType[e.Type], e)
}
// No need to initialize the inner slices — nil slice + append works
```

### Deduplicate with a Set

```go
func deduplicate(ids []string) []string {
    seen := make(map[string]struct{}, len(ids))
    result := make([]string, 0, len(ids))
    for _, id := range ids {
        if _, ok := seen[id]; !ok {
            seen[id] = struct{}{}
            result = append(result, id)
        }
    }
    return result
}
```

---

## Comparison: JavaScript vs Go Collections

| Operation | JavaScript | Go |
|---|---|---|
| Array/slice literal | `[1, 2, 3]` | `[]int{1, 2, 3}` |
| Length | `arr.length` | `len(s)` |
| Append | `arr.push(x)` | `s = append(s, x)` |
| Slice | `arr.slice(1, 3)` | `s[1:3]` |
| Copy | `[...arr]` or `arr.slice()` | `copy(dst, src)` |
| Find index | `arr.findIndex(f)` | manual loop (or `slices.IndexFunc`) |
| Filter | `arr.filter(f)` | manual loop (or generic `Filter`) |
| Map | `arr.map(f)` | manual loop (or generic `Map`) |
| Object/map literal | `{ key: val }` | `map[string]T{"key": val}` |
| Map lookup | `obj.key` or `obj["key"]` | `m["key"]` (zero if missing) |
| Map lookup (safe) | `"key" in obj` | `val, ok := m["key"]` |
| Delete from map | `delete obj.key` | `delete(m, "key")` |
| Set | `new Set()` | `map[T]struct{}{}` |

**The most important differences:**

1. **JS arrays are always references.** Assigning `b = a` means both point to the same array. Go slices feel like references (because the header is small and the backing array is shared) but the header itself is copied. Append does not affect the caller's slice unless the backing array is shared AND you didn't reallocate.

2. **JS objects can always be written to.** Go nil maps panic on write. You must always `make()` before writing.

3. **JS has first-class map/filter/reduce.** Go does not (yet) — you write loops or add generics. The `slices` and `maps` standard packages (Go 1.21+) cover common operations generically.

4. **JS Map preserves insertion order.** Go maps do not — iteration order is randomized every run.

---

## Performance Notes

### Pre-Allocate When Size Is Known

```go
// Bad: repeated reallocations as the slice grows
var events []Event
for i := 0; i < 10000; i++ {
    events = append(events, Event{...})
}

// Good: one allocation
events := make([]Event, 0, 10000)
for i := 0; i < 10000; i++ {
    events = append(events, Event{...})
}
```

For maps:

```go
// Hint to the runtime how many entries to expect
m := make(map[string]int, 1000)
```

The capacity hint prevents rehashing as the map grows.

### Slice of Pointers vs Slice of Values

- `[]Event` — values are stored inline (better cache locality, fewer allocations)
- `[]*Event` — each element is a heap pointer (allows independent mutation, but more GC pressure)

Prefer `[]Event` (values) for small structs and read-heavy workloads. Use `[]*Event` when you need independent mutation or nil-ability per element.

### Your notes
<!-- -->
