# Variables and Types — Go

## How Variables Work Under the Hood

### Declaration and Memory

When you write `var x int = 42`, here's what actually happens:

1. The compiler allocates space on the **stack** (or heap — more on that below)
2. That space is sized for the type: `int` is 8 bytes on 64-bit systems
3. The value `42` is written into that memory
4. The name `x` is a compile-time label — it doesn't exist at runtime. It's just an offset.

```go
var x int = 42
// x is not a "box" — it's a label the compiler maps to a memory address
// At runtime, there's just a value at some address on the stack
fmt.Printf("value: %d, address: %p, size: %d bytes\n", x, &x, unsafe.Sizeof(x))
```

### Stack vs Heap

Go decides where to allocate a variable through **escape analysis** at compile time.

- **Stack**: fast, automatically cleaned up when the function returns. Default for local variables.
- **Heap**: slower (requires garbage collection), but the value survives after the function returns.

A variable "escapes to the heap" when:
- You return a pointer to a local variable
- You pass it to a function that stores it beyond the current scope
- The compiler can't prove it stays local

```go
func stackAlloc() int {
    x := 42       // stays on the stack — returned by value (copy)
    return x
}

func heapAlloc() *int {
    x := 42       // escapes to the heap — returned by pointer
    return &x     // Go allows this (unlike C). The GC keeps x alive.
}
```

You can see escape analysis decisions with: `go build -gcflags="-m" .`

### Your notes
- Escape analysis is the compiler deciding stack vs heap at build time. It asks: "does this variable outlive the function?" If yes → heap. If no → stack.
- Returning a pointer to a local variable forces it to the heap. Returning by value keeps it on the stack (it's copied out).
- `fmt.Printf` with `%p` or interface args can cause escapes too — the value gets boxed into `interface{}`. So print calls themselves can trigger heap allocation.
- Run `go build -gcflags="-m" .` to see the compiler's decisions.


---

## Type System

### Static, Structural, Strong

Go's type system is:
- **Static**: types are checked at compile time
- **Structural** (for interfaces): a type implements an interface if it has the right methods — no `implements` keyword
- **Strong**: no implicit conversions. `int` and `int64` are different types. You must convert explicitly.

```go
var a int = 42
var b int64 = int64(a)    // explicit conversion required
// var c int64 = a         // compile error — Go won't do this for you
```

### Primitive Types and Their Sizes

| Type | Size | Zero Value | Notes |
|---|---|---|---|
| `bool` | 1 byte | `false` | |
| `int` | 8 bytes (64-bit) | `0` | Platform-dependent size |
| `int8` | 1 byte | `0` | -128 to 127 |
| `int16` | 2 bytes | `0` | |
| `int32` | 4 bytes | `0` | Alias: `rune` |
| `int64` | 8 bytes | `0` | |
| `uint` | 8 bytes (64-bit) | `0` | Platform-dependent |
| `float32` | 4 bytes | `0.0` | |
| `float64` | 8 bytes | `0.0` | Default for float literals |
| `string` | 16 bytes | `""` | Header: pointer (8) + length (8) |
| `byte` | 1 byte | `0` | Alias for `uint8` |

### Strings Are Immutable Headers

A Go `string` is a struct under the hood:

```
type stringHeader struct {
    Data uintptr  // pointer to the byte array
    Len  int      // length in bytes
}
```

The string variable itself is 16 bytes on the stack (pointer + length). The actual character data lives elsewhere (usually read-only memory for literals, heap for constructed strings). Strings are **immutable** — every modification creates a new string.

### Your notes
- String is a struct under the hood: pointer (8 bytes) + length (8 bytes) = 16 bytes on the stack. The actual character data lives elsewhere.
- `len(s)` is O(1) — it just reads the Len field from the header. No iteration. Compare with C strings which are null-terminated and require O(n) to get length.
- `len()` returns **bytes, not characters**. This matters for anything outside ASCII:
```go
s := "café"
fmt.Println(len(s))                    // 5 — the é is 2 bytes in UTF-8
fmt.Println(utf8.RuneCountInString(s)) // 4 — actual characters
```
- In JS, `.length` gives UTF-16 code units (aligns with characters for most text, breaks on emoji). Go gives raw bytes. Watch out for validation like "max 255 characters" — use `utf8.RuneCountInString`, not `len()`.

---

## Zero Values

Every type in Go has a zero value. This is not `nil` or `null` — it's a real, usable value.

| Type | Zero Value |
|---|---|
| Numeric types | `0` |
| `bool` | `false` |
| `string` | `""` (empty string) |
| Pointers | `nil` |
| Slices | `nil` (but len=0, cap=0 — usable without make) |
| Maps | `nil` (**not** usable — writing panics) |
| Channels | `nil` |
| Interfaces | `nil` |
| Structs | All fields set to their zero values |

This is a design choice: Go wants every variable to be usable at declaration. No "uninitialized memory" footguns.

But watch out: **a nil map looks valid but panics on write**. A nil slice is fine.

```go
var s []int
s = append(s, 1)  // works — append handles nil slices

var m map[string]int
// m["key"] = 1    // PANIC — must use make(map[string]int) first
```

### Your notes
<!-- -->


---

## Pointers

Go has pointers but no pointer arithmetic (unlike C/Rust's unsafe).

```go
x := 42
p := &x          // p is *int, holds the memory address of x
fmt.Println(*p)  // 42 — dereference to get the value
*p = 100         // changes x through the pointer
fmt.Println(x)   // 100
```

Key mental model:
- `&x` = "give me the address of x"
- `*p` = "give me the value at this address"
- `*int` = "this is a type: pointer to int"

Pointers matter for:
- Avoiding copies of large structs
- Allowing functions to modify the caller's data
- Signaling "this value might not exist" (nil pointer)

### Value Semantics vs Pointer Semantics

```go
type Config struct {
    Port int
}

func updateByValue(c Config) {
    c.Port = 9090   // modifies a COPY — caller's Config unchanged
}

func updateByPointer(c *Config) {
    c.Port = 9090   // modifies the ORIGINAL — caller sees the change
}
```

Rule of thumb: use pointer receivers for structs you want to mutate. Use value receivers for small, immutable types.

### Your notes
- **Key difference from JS:** In JS, objects are always passed by reference. `const b = {}` — `b` holds a reference, not the object itself. Functions can mutate the original. The spread/copy pattern (`{ ...b }`) is a convention to avoid mutation, not a language requirement.
- **Go is the opposite:** structs are value types. Passing a struct copies the whole thing. Pointers let you opt into what JS gives you by default.
- Mental model:

| | JS | Go |
|---|---|---|
| Primitives | copied | copied |
| Objects/Structs | reference (always) | copied (default), reference (with `*`) |

- Go makes you **choose explicitly** per call site. JS chose for you.
- Two reasons to use pointers: (1) you need to mutate the original, (2) the struct is large and copying is wasteful.


---

## Constants and iota

```go
const maxRetries = 3         // untyped constant — adapts to context
const port int = 8080        // typed constant — locked to int

const (
    StatusPending = iota     // 0
    StatusActive             // 1
    StatusClosed             // 2
)
```

Untyped constants in Go are special: they have higher precision than any concrete type and adapt to their usage context. `const x = 1` can be used as `int`, `float64`, `int32`, etc. without conversion.

### Your notes
<!-- -->


---

## Composite Types (Preview)

These get their own lesson, but a quick map:

| Type | Description | Zero Value |
|---|---|---|
| `[3]int` | Array — fixed size, value type | `[0, 0, 0]` |
| `[]int` | Slice — dynamic, reference to underlying array | `nil` |
| `map[K]V` | Hash map | `nil` |
| `struct{}` | Struct — composite value type | All fields zeroed |

### Your notes
<!-- -->
