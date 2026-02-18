# Pointers & Smart Pointers — Go

## What a Pointer Is

A pointer is a variable that holds a memory address. Instead of storing a value directly, it stores the location of a value. You dereference the pointer to read or write the value at that address.

In Go, every pointer has a type: `*T` is a pointer to a value of type `T`. There are no untyped raw pointers in normal Go code (we'll briefly cover `unsafe.Pointer` at the end).

```go
x := 42          // x holds the value 42, stored on the stack
p := &x          // p holds the address of x — type is *int
fmt.Println(p)   // prints something like 0xc0000b4008 (the address)
fmt.Println(*p)  // prints 42 — dereferencing: read the value at that address
*p = 100         // write through the pointer — changes x
fmt.Println(x)   // prints 100
```

The `&` operator takes the address of a variable. The `*` operator dereferences a pointer (reads or writes the value at that address). Used in a type, `*T` means "pointer to T". Used on a value, `*p` means "the value pointed to by p".

This dual meaning of `*` confuses beginners. Context determines which:
- In a type: `var p *int` — p is a pointer to int
- On a variable: `*p = 5` — write 5 through pointer p
- On a variable: `fmt.Println(*p)` — read the value through pointer p

### Under the Hood

On a 64-bit system, all pointers are 8 bytes — the size of a memory address. A `*Config` to a 500-byte struct is still just 8 bytes. The struct lives somewhere in memory; the pointer is just the address of its first byte.

```
Stack:                    Heap (or stack, wherever x was allocated):
┌─────────────┐           ┌────┐
│  p = 0x1000 │──────────▶│ 42 │  (x)
└─────────────┘           └────┘
  (8 bytes)                 address 0x1000
```

Go's garbage collector manages heap allocations. When no pointers to a value remain, the GC can collect it. Stack allocations are cleaned up when the function returns — but if you take the address of a local variable and that pointer escapes (is returned, stored in a heap struct, etc.), Go automatically allocates the variable on the heap instead. This is called **escape analysis**.

### Your notes

---

## nil: The Zero Value of Pointers

The zero value of any pointer type is `nil`. A nil pointer holds no address. Dereferencing it panics immediately.

```go
var p *int
fmt.Println(p)   // prints <nil>
fmt.Println(*p)  // PANIC: runtime error: invalid memory address or nil pointer dereference
```

Always check before dereferencing a pointer you don't control:

```go
func printValue(p *int) {
    if p == nil {
        fmt.Println("no value")
        return
    }
    fmt.Println(*p)
}
```

`nil` panics are Go's most common runtime crash. The fix is always the same: check for nil before dereferencing, or restructure the code so nil is impossible (return a non-pointer, use a constructor that guarantees initialization).

**Coming from JavaScript/TypeScript:** JS objects are reference types — `const obj = {}` gives you a reference automatically. You don't opt in to references. In Go, the default is a value. You explicitly opt in to a pointer with `&`. Nil in JS is semantically similar — you get a null-reference-equivalent panic when you try to use a nil pointer. But in Go it crashes immediately and loudly, with a clear stack trace.

### Your notes

---

## Two Ways to Allocate

Go has two ways to create a pointer to a new value:

### `&T{...}` — Composite literal address

```go
type Config struct {
    Host    string
    Port    int
    Timeout int
}

cfg := &Config{
    Host:    "localhost",
    Port:    5432,
    Timeout: 30,
}
// cfg is *Config — points to a newly allocated Config on the heap
```

This is idiomatic Go for structs. You get a pointer to a zero-initialized (or specified) struct. Used everywhere in constructors.

### `new(T)` — Built-in allocator

```go
p := new(int)       // allocates an int, zero-initializes it, returns *int
fmt.Println(*p)     // 0

cfg := new(Config)  // allocates a Config, all fields zeroed, returns *Config
fmt.Println(cfg.Port)  // 0
```

`new(T)` allocates space for one value of type `T`, zero-initializes it, and returns a pointer to it. It's equivalent to `&T{}`.

**In practice:** `&T{...}` wins for structs because you can specify field values inline. `new(T)` shows up occasionally for primitive types (`new(int)`, `new(bool)`) and when you want to emphasize that you're just getting a pointer to a zero value.

```go
// These are equivalent:
p1 := new(int)
var zero int
p2 := &zero

// These are equivalent:
cfg1 := new(Config)
cfg2 := &Config{}
```

### Your notes

---

## Pass by Value vs Pass by Pointer

**Go is always pass-by-value.** Every function argument is a copy. Every time. No exceptions.

But a copy of a pointer is still a pointer to the same underlying data. This is how Go simulates pass-by-reference.

```go
func doubleValue(n int) {
    n *= 2  // modifies local copy — caller's n is unchanged
}

func doublePointer(n *int) {
    *n *= 2  // dereferences the pointer, modifies the value at that address
}

x := 5
doubleValue(x)
fmt.Println(x)  // 5 — unchanged

doublePointer(&x)
fmt.Println(x)  // 10 — changed via pointer
```

When you pass a pointer, you're copying 8 bytes (the address). The function receives its own copy of the pointer, but both the caller's pointer and the function's pointer copy point to the same underlying data. Writing through the pointer (`*n = ...`) changes the shared data.

### Modifying Struct Fields

This is the most common reason to use pointers:

```go
type Counter struct {
    count int
}

// Does NOT modify the caller's Counter — works on a copy
func incrementBroken(c Counter) {
    c.count++
}

// DOES modify the caller's Counter — works through pointer
func increment(c *Counter) {
    c.count++
}

c := Counter{count: 0}
incrementBroken(c)
fmt.Println(c.count)  // 0 — unchanged

increment(&c)
fmt.Println(c.count)  // 1 — changed
```

Go auto-dereferences struct field access through pointers — `c.count` when `c` is `*Counter` is automatically `(*c).count`. You don't need to write `(*c).count` (though you can).

### When to Use Pointers

Use a pointer when you need one of:

1. **Mutation** — the function needs to modify the caller's data
2. **Large struct** — copying is expensive (benchmark first; Go is fast at copying)
3. **Optional value** — `*T` can be nil, modeling "field may not be present"
4. **Shared state** — multiple parts of the code need to see the same instance
5. **Interface satisfaction** — your method set requires pointer receivers

Don't use a pointer when:
- The value is small and immutable (strings, ints, small structs)
- You want callers to have independent copies
- The nil case would be a bug, not a valid state

### Large Struct Optimization

```go
type DeviceMetrics struct {
    DeviceID    string
    Readings    [1000]float64  // 8KB of data
    Metadata    [50]string     // additional fields
    // ... more fields
}

// Passes 8KB+ on every call — slow
func processBroken(m DeviceMetrics) {
    // ...
}

// Passes 8 bytes (the pointer) — fast
func process(m *DeviceMetrics) {
    // ...
}
```

For small structs (a handful of fields, total size under ~64 bytes), copying is fine — often faster because it avoids a heap allocation and keeps data in cache. Profile before optimizing.

### Your notes

---

## Pointer Receivers: A Deeper Look

We covered value vs pointer receivers in the structs module. Here's the deeper angle through a pointer lens.

A method with a pointer receiver `(c *Connection)` requires that the receiver be addressable or already a pointer. Go provides automatic address-taking for addressable values:

```go
type Connection struct {
    host   string
    active bool
}

func (c *Connection) Connect() {
    c.active = true
}

conn := Connection{host: "db.example.com"}
conn.Connect()   // Go automatically does (&conn).Connect() — conn is addressable (a variable)
```

But this automatic address-taking does **not** apply to:
- Map values: `myMap["key"].Connect()` — map elements are not addressable
- Values returned from functions: `getConnection().Connect()` — temporary values are not addressable
- Interface values: you can't take the address through an interface

This is why you often see `map[string]*Connection` instead of `map[string]Connection`.

### Method Sets and Interface Satisfaction

The rule:
- `T` can only call methods with value receivers
- `*T` can call methods with both value and pointer receivers

This matters enormously for interfaces:

```go
type Connector interface {
    Connect() error
    Close() error
}

type DBConnection struct { /* ... */ }

func (c *DBConnection) Connect() error { /* ... */ return nil }
func (c *DBConnection) Close() error   { /* ... */ return nil }

// *DBConnection implements Connector ✓
// DBConnection does NOT implement Connector ✗ (Connect and Close have pointer receivers)

var conn Connector = &DBConnection{}   // correct
var conn2 Connector = DBConnection{}   // compile error: DBConnection does not implement Connector
```

The fix is always the same: if any method needs a pointer receiver, use pointer receivers for all methods, and always pass `*T` where the interface is expected.

### When All Methods Should Use Pointer Receivers

If any method on a type uses a pointer receiver, all methods should use pointer receivers. Mixing creates confusion about which method set satisfies which interface, and makes the type harder to reason about.

There's one deliberate exception: a `String()` method for `fmt.Stringer` is often given a value receiver so that both `T` and `*T` automatically implement `Stringer`. If you give it a pointer receiver, only `*T` implements `Stringer`, and `fmt.Println(myValue)` won't call it.

### Your notes

---

## Pointers to Reference Types

Go's built-in composite types — slices, maps, and channels — are already reference types. They contain an internal pointer to their underlying data.

```
Slice header (24 bytes):          Underlying array (on heap):
┌──────────┬──────┬──────┐        ┌───┬───┬───┬───┬───┐
│ ptr      │ len  │ cap  │───────▶│ 1 │ 2 │ 3 │ 4 │ 5 │
└──────────┴──────┴──────┘        └───┴───┴───┴───┴───┘
```

When you pass a slice to a function, the function gets a copy of the header (ptr, len, cap). The ptr still points to the same underlying array. Modifications to elements (`s[i] = x`) are visible to the caller. But operations that change the header (`append`, `s = s[:n]`) only affect the local copy — the caller's slice is unchanged.

```go
func modifyElement(s []int) {
    s[0] = 999  // visible to caller — same underlying array
}

func appendToSlice(s []int) {
    s = append(s, 42)  // caller's slice unchanged — header copy was modified
}

nums := []int{1, 2, 3}
modifyElement(nums)
fmt.Println(nums[0])  // 999 — element change visible

appendToSlice(nums)
fmt.Println(len(nums))  // 3 — append invisible to caller
```

### When You Do Need a Pointer to a Slice

If a function needs to modify the slice header (grow it with append, reslice it), pass `*[]T`:

```go
func appendToSlicePtr(s *[]int, val int) {
    *s = append(*s, val)  // modifies caller's slice header
}

nums := []int{1, 2, 3}
appendToSlicePtr(&nums, 42)
fmt.Println(nums)  // [1 2 3 42] — append visible
```

In practice, this pattern appears less often than you'd think. The idiomatic Go solution is usually to return the modified slice: `func addItem(s []int, val int) []int { return append(s, val) }`. Returning the slice is cleaner and avoids the `*[]T` awkwardness.

Maps are similar. A map variable is already a pointer to the underlying hash table. Modifications through a copy are visible everywhere. The one thing you can't do with a map copy is replace the entire map (`m = make(map[string]int)`) and have that be visible to the caller — for that you'd need `*map[string]int`.

### Your notes

---

## Patterns: Constructors, Optional Fields, Pointer Pitfalls

### Constructor Returns *T

The standard Go constructor pattern returns `*T`:

```go
type Server struct {
    host    string
    port    int
    timeout time.Duration
    logger  *log.Logger  // optional — nil means "no logger"
}

func NewServer(host string, port int) (*Server, error) {
    if host == "" {
        return nil, fmt.Errorf("host is required")
    }
    if port < 1 || port > 65535 {
        return nil, fmt.Errorf("port %d out of range", port)
    }
    return &Server{
        host:    host,
        port:    port,
        timeout: 30 * time.Second,
    }, nil
}
```

Why `*Server` and not `Server`? Three reasons:
1. All mutating methods will need pointer receivers — you want callers to already have `*Server`
2. You want callers sharing the same instance (shared state, connection pools, caches)
3. The zero value of `Server` is probably not meaningful — it requires a constructor

When *should* you return a value instead of a pointer? When the type is immutable or small, when zero value is valid, or when you specifically want each caller to get their own independent copy (config value objects, for instance).

### Optional Fields with *T

`*T` naturally models "this field may not be set":

```go
type WebhookConfig struct {
    URL        string
    Secret     string
    MaxRetries *int    // nil means "use the default"
    Timeout    *time.Duration  // nil means "no custom timeout"
    Tags       *[]string       // nil means "no tags"
}

func processWebhook(cfg WebhookConfig) {
    retries := 3  // default
    if cfg.MaxRetries != nil {
        retries = *cfg.MaxRetries
    }
    // ...
}
```

This is common in API config structs where most fields are optional. The alternative is using sentinel zero values (`MaxRetries == 0` means "use default") but that's less clear — what if the caller genuinely wants 0 retries?

A helper function makes this pattern less noisy:

```go
func intPtr(n int) *int { return &n }
func durationPtr(d time.Duration) *time.Duration { return &d }

cfg := WebhookConfig{
    URL:        "https://example.com/hook",
    MaxRetries: intPtr(5),
}
```

### Pointer to Loop Variable — Classic Go Gotcha

Before Go 1.22, loop variables were shared across iterations. Capturing a pointer to a loop variable would give you a pointer that always points to the same memory — which holds the last iteration's value after the loop:

```go
// Go < 1.22 behavior — all pointers point to same address
addrs := make([]*int, 3)
for i := 0; i < 3; i++ {
    addrs[i] = &i  // all three capture the same 'i' variable
}
// After loop, i == 3
fmt.Println(*addrs[0], *addrs[1], *addrs[2])  // 3 3 3 — not 0 1 2!
```

The fix (still necessary for range loop variables and goroutines in pre-1.22 Go, and good practice everywhere):

```go
addrs := make([]*int, 3)
for i := 0; i < 3; i++ {
    i := i  // shadow: create a new 'i' scoped to this iteration
    addrs[i] = &i
}
fmt.Println(*addrs[0], *addrs[1], *addrs[2])  // 0 1 2
```

As of Go 1.22, loop variables are per-iteration by default. But this pattern appears in codebases targeting older Go versions and shows up in goroutine captures.

### Pointer to Interface — Rarely What You Want

One of the most confusing pointer patterns in Go: you almost never need a pointer to an interface (`*SomeInterface`).

An interface value already contains a pointer to the underlying concrete value (when that concrete type uses pointer receivers). Wrapping an interface in another pointer gives you an extra level of indirection with no benefit.

More dangerously: a `*SomeInterface` where the interface holds a nil value is not nil itself:

```go
type Handler interface {
    Handle(r Request) error
}

var h *MyHandler = nil    // typed nil pointer
var iface Handler = h     // non-nil interface wrapping a nil pointer

if iface != nil {         // this is TRUE — iface is non-nil (it has type info)
    iface.Handle(req)     // PANIC — the underlying pointer is nil
}
```

The rule: accept interfaces, return concrete types. Don't store `*Interface` unless you have a very specific reason (like using sync.Mutex on an interface value).

### Your notes

---

## unsafe.Pointer: What It Is and Why You Skip It

`unsafe.Pointer` is a special pointer type that can hold any pointer value, bypassing Go's type system. It's the escape hatch for:

- Talking to C code via `cgo`
- Memory layout tricks (reinterpreting a `[]byte` as a struct)
- Implementing lock-free data structures (with `atomic.Pointer`)
- Operating on memory at a lower level than Go normally allows

```go
import "unsafe"

x := int32(42)
p := unsafe.Pointer(&x)  // *int32 → unsafe.Pointer (allowed)
q := (*float32)(p)        // unsafe.Pointer → *float32 (allowed — but dangerous!)
fmt.Println(*q)           // reinterprets the bits of 42 as a float32
```

The compiler treats `unsafe.Pointer` specially — it can convert to/from any pointer type. This bypasses type safety completely. The garbage collector also won't track pointers inside values that have been cast away from their typed pointer.

**In production code:** `unsafe` is almost always a sign something has gone wrong architecturally. The standard library uses it in a few carefully audited hot paths (`strings`, `reflect`, `sync`). Application code should not. If you find yourself reaching for it, the better question is: what design problem am I trying to avoid?

The only area where `unsafe.Pointer` is common for application developers: working with `atomic.Pointer[T]` for lock-free concurrent data structures — but that's the `sync/atomic` package wrapping `unsafe` safely, not raw `unsafe.Pointer` usage.

### Your notes

---

## sync.Pool: Pointer Reuse

`sync.Pool` is a concurrent-safe pool of objects. The use case: you're allocating many short-lived objects of the same type (e.g., per-request byte buffers, per-operation structs), and you want to reuse allocations instead of constantly hitting the GC.

```go
var bufPool = sync.Pool{
    New: func() any {
        return &bytes.Buffer{}
    },
}

func processRequest(data []byte) string {
    buf := bufPool.Get().(*bytes.Buffer)
    defer func() {
        buf.Reset()
        bufPool.Put(buf)  // return to pool for reuse
    }()

    buf.Write(data)
    buf.WriteString(" processed")
    return buf.String()
}
```

Key rules for `sync.Pool`:
1. **Always reset before returning to pool.** The next `Get` will receive a dirty object otherwise.
2. **Never store state you care about.** The GC can drop pool contents at any time (between GC cycles). Don't put anything in a pool that needs to persist.
3. **Use `New` to provide a factory.** If the pool is empty and `New` is nil, `Get` returns nil.
4. **Profile first.** `sync.Pool` solves a specific GC pressure problem. If you're not seeing GC pressure in profiles, it adds complexity for no benefit.

`sync.Pool` is an advanced tool for specific performance scenarios (parsing, HTTP servers, serializers). It's worth knowing about but not worth reaching for prematurely.

### Your notes

---

## TypeScript / JavaScript Comparison

| Concept | Go | TypeScript / JavaScript |
|---------|-----|-------------------------|
| Reference types | Explicit: `*T` (pointer) | Implicit: objects, arrays are references by default |
| Value types | Default: structs, scalars | Primitives (number, string, bool) are by value |
| Null/nil pointer | `var p *T` → nil, panics on dereference | `null` / `undefined`, throws TypeError |
| Optional field | `*T` (nil = absent) | `T \| undefined` or `T?` |
| Pass by reference | Pass `*T`, write `*p = ...` | Automatic for objects — you mutate properties |
| "Smart pointers" | `sync.Pool` for reuse; `atomic.Pointer` for concurrent | No direct equivalent — GC handles everything |
| Null check | `if p != nil { ... }` | `if (x !== null && x !== undefined) { ... }` or optional chaining `x?.field` |
| Memory management | Automatic (GC + stack allocation) | Automatic (GC) |

**The biggest mental shift:** In TypeScript, when you write `const cfg = { host: "localhost" }`, you automatically have a reference — cfg is a reference to the object on the heap. You didn't opt in. In Go, `cfg := Config{Host: "localhost"}` gives you a value — the struct is stored directly wherever cfg is (often the stack). To get reference semantics, you explicitly write `cfg := &Config{Host: "localhost"}`.

This means Go requires you to be intentional about "should this be shared or copied?" — which catches a whole class of bugs at design time rather than runtime.

## Rust Comparison

| Concept | Go | Rust |
|---------|-----|------|
| Pointer to T | `*T` | `&T` (shared ref), `&mut T` (exclusive ref) |
| Heap allocation | `&T{}` or `new(T)` | `Box::new(T)` |
| Null pointer | `nil` (*T can be nil) | Not possible — references are never null |
| Optional pointer | `*T` (nil = absent) | `Option<Box<T>>` or `Option<&T>` |
| Reference counted | `sync.Mutex` + pointer (manual) | `Rc<T>` (single-threaded), `Arc<T>` (multi-threaded) |
| Concurrent ref | `*T` + synchronization | `Arc<Mutex<T>>` |
| GC | Yes — GC manages all heap objects | No — borrow checker + RAII, no GC |
| Safety guarantees | Runtime panics on nil dereference | Compile-time: no null references, no dangling pointers |

**Key difference:** Rust's borrow checker prevents dangling pointers, data races, and use-after-free at compile time. Go's GC prevents dangling pointers but allows nil panics and data races at runtime. Go trades Rust's compile-time safety for simplicity — no lifetime annotations, no ownership system to learn.

Go has no direct equivalents to Rust's `Rc`, `Arc`, or `Box`. In Go, all heap objects are managed by the GC. If you need shared ownership, you just pass pointers and let the GC handle the lifetime. If you need concurrent access, you use `sync.Mutex` or `sync/atomic`.
