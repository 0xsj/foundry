# Interfaces — Go

## What an Interface Actually Is

An interface in Go is a set of method signatures. Any type that has all of those methods automatically satisfies the interface — no declaration, no `implements` keyword, no registration. This is called **structural typing** or **duck typing with compile-time verification**.

```go
type Storer interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
}
```

That's it. If a type has `Get`, `Set`, and `Delete` with those exact signatures, it satisfies `Storer`. The type doesn't know `Storer` exists. The author of `Storer` didn't need to know about all the types that would satisfy it.

This is the fundamental difference from TypeScript and Java. In TypeScript, interfaces are also structural, but the convention is to declare `implements`. In Java, you must explicitly write `implements Storer`. In Go, the connection between type and interface is entirely implicit — verified by the compiler, invisible in the source.

### Under the Hood: How Interfaces Are Stored

An interface value is a two-word structure in memory:

```
+-----------+-----------+
|   type    |   data    |
|  pointer  |  pointer  |
+-----------+-----------+
```

The **type pointer** points to runtime type information (the concrete type's method table). The **data pointer** points to the concrete value (or, if the value fits in a pointer, sometimes the value itself).

When you assign a concrete value to an interface:

```go
var s Storer = &RedisStore{addr: "localhost:6379"}
```

Go stores:
1. A pointer to `*RedisStore`'s type descriptor (which includes the method table for `Storer`)
2. A pointer to the `RedisStore` value

This is important for one of the subtlest bugs in Go — the nil interface gotcha, which we'll cover later.

### Your notes

---

## Defining and Satisfying Interfaces

### Defining

Interface declarations go anywhere a type declaration goes — at package level, inside a function, or inline. Package-level is most common.

```go
// Package-level interface — exported, used across packages
type Storer interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
    List(prefix string) ([]string, error)
}
```

Naming convention: interfaces with one method are named after the method + "er". `Read` → `Reader`. `Write` → `Writer`. `Store` → `Storer`. `Format` → `Formatter`. Multi-method interfaces describe the role: `http.Handler`, `sort.Interface`, `io.ReadWriter`.

### Satisfying

A type satisfies an interface by implementing all its methods with the matching signatures:

```go
// MemoryStore satisfies Storer — no declaration needed
type MemoryStore struct {
    mu   sync.RWMutex
    data map[string][]byte
}

func (m *MemoryStore) Get(key string) ([]byte, error) {
    m.mu.RLock()
    defer m.mu.RUnlock()
    v, ok := m.data[key]
    if !ok {
        return nil, fmt.Errorf("key %q not found", key)
    }
    return v, nil
}

func (m *MemoryStore) Set(key string, value []byte) error {
    m.mu.Lock()
    defer m.mu.Unlock()
    m.data[key] = value
    return nil
}

func (m *MemoryStore) Delete(key string) error {
    m.mu.Lock()
    defer m.mu.Unlock()
    delete(m.data, key)
    return nil
}

func (m *MemoryStore) List(prefix string) ([]string, error) {
    m.mu.RLock()
    defer m.mu.RUnlock()
    var keys []string
    for k := range m.data {
        if strings.HasPrefix(k, prefix) {
            keys = append(keys, k)
        }
    }
    return keys, nil
}
```

`*MemoryStore` satisfies `Storer`. Anywhere the code expects a `Storer`, you can pass `&MemoryStore{...}`.

### Polymorphism Through Interfaces

The power: write code against the interface, swap implementations without changing the caller.

```go
// This function doesn't know or care whether it's talking to
// Redis, Postgres, S3, or a test double.
func cacheResponse(store Storer, key string, body []byte, ttl time.Duration) error {
    if err := store.Set(key, body); err != nil {
        return fmt.Errorf("caching response for %q: %w", key, err)
    }
    return nil
}
```

In tests, pass a `MemoryStore`. In production, pass a `RedisStore`. Same function, zero changes.

### Interface Guards: Compile-Time Satisfaction Checks

Go checks interface satisfaction lazily — only when you assign to an interface variable. This means you can write a whole type and only discover it doesn't satisfy an interface when you try to use it.

To catch this at the declaration site, use an **interface guard**:

```go
// Compile-time assertion: *RedisStore must satisfy Storer.
// If it doesn't, this line produces a compile error.
var _ Storer = (*RedisStore)(nil)
```

This is idiomatic Go. The `var _ Storer` declares a blank variable of type `Storer` (blank identifier means "I don't need this value"). The `(*RedisStore)(nil)` creates a nil pointer of type `*RedisStore` and tries to assign it. No runtime cost — the compiler checks it and discards the variable. If `*RedisStore` is missing any `Storer` method, you get an error at the line of the guard, not deep in some call site.

You'll see this pattern throughout major Go codebases. Put interface guards directly below the type they verify.

### Your notes

---

## The Method Set Rule: Value vs Pointer Receivers

This is the rule that trips up almost every Go developer at least once.

**The rule:**
- A value `T` satisfies an interface only using methods with value receiver `T`
- A pointer `*T` satisfies an interface using methods with either value receiver `T` or pointer receiver `*T`

In plain English: pointer types get all methods. Value types only get value-receiver methods.

```go
type Flusher interface {
    Flush() error
}

type BufferedWriter struct {
    buf []byte
}

// Value receiver — works on both BufferedWriter and *BufferedWriter
func (b BufferedWriter) Len() int { return len(b.buf) }

// Pointer receiver — works on *BufferedWriter only
func (b *BufferedWriter) Flush() error {
    fmt.Println("flushing", len(b.buf), "bytes")
    b.buf = b.buf[:0]
    return nil
}

var _ Flusher = (*BufferedWriter)(nil) // OK — *BufferedWriter has Flush
var _ Flusher = BufferedWriter{}       // COMPILE ERROR — BufferedWriter lacks Flush
```

**Why this rule exists:** Go's type system needs to be able to take the address of a value to call a pointer-receiver method. For a variable, Go does this automatically:

```go
b := BufferedWriter{}
b.Flush() // Go implicitly does (&b).Flush() — works because b is addressable
```

But an interface value stores a copy of the concrete value. Go cannot take the address of a value stored inside an interface — the interface holds it by value and calling a pointer-receiver method on a non-addressable copy would mutate the copy, not the original. So Go simply disallows it at the type level.

**Practical rule:** If any method modifies the receiver, use pointer receivers on all methods for that type. Store the pointer in the interface:

```go
store := &MemoryStore{data: make(map[string][]byte)}
var s Storer = store // correct — *MemoryStore satisfies Storer
```

### Your notes

---

## Interface Composition

### Embedding Interfaces

Interfaces can embed other interfaces. The composed interface includes all the methods of the embedded interfaces:

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

// ReadWriter is the union of Reader and Writer
type ReadWriter interface {
    Reader  // embedded
    Writer  // embedded
}
```

This is how `io.ReadWriter` is defined in the standard library. `io.ReadWriteCloser` adds `Closer`. You compose capabilities rather than building a monolithic interface with all methods.

### Real-World Composition

In a storage system, you might break the interface along capability lines:

```go
type Reader interface {
    Get(key string) ([]byte, error)
    List(prefix string) ([]string, error)
}

type Writer interface {
    Set(key string, value []byte) error
    Delete(key string) error
}

type ReadWriter interface {
    Reader
    Writer
}

type Transactional interface {
    ReadWriter
    Begin() (Transaction, error)
}
```

Now functions can express exactly what they need. A function that only reads can accept `Reader`. A function that reads and writes accepts `ReadWriter`. A function that needs transactions accepts `Transactional`. Each constraint is precisely stated.

This also makes testing easier — implementing a `Reader` test double requires only two methods, not the full `Transactional` interface.

### Interface Upgrades

A common pattern is checking whether a value satisfies a more capable interface at runtime. This lets you write general code against a small interface, but take advantage of optional capabilities when available:

```go
type Storer interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
}

type BulkStorer interface {
    Storer
    SetBulk(pairs map[string][]byte) error
}

func populateCache(store Storer, data map[string][]byte) error {
    // Check if the store supports bulk writes for efficiency
    if bulk, ok := store.(BulkStorer); ok {
        return bulk.SetBulk(data)
    }
    // Fall back to individual writes
    for k, v := range data {
        if err := store.Set(k, v); err != nil {
            return err
        }
    }
    return nil
}
```

The function accepts `Storer`, but silently upgrades to `BulkStorer` if available. Redis might support bulk writes; a simple in-memory store might not. The caller doesn't know or care.

### Your notes

---

## The Empty Interface: `any`

The empty interface has no methods:

```go
interface{}  // the old way
any          // alias added in Go 1.18, prefer this
```

Every type satisfies the empty interface — an `any` can hold any value. This is Go's escape hatch for when you genuinely don't know or can't express the type at compile time.

```go
// Encoding a JSON value — could be string, number, bool, object, array, null
func Marshal(v any) ([]byte, error) { ... }

// Generic cache that stores anything
type Cache struct {
    data map[string]any
}
```

**When `any` is appropriate:**
- Serialization/deserialization (JSON, YAML, protobuf generic decoding)
- Generic containers before Go 1.18 (now use type parameters instead)
- Plugin systems where types are unknown at compile time
- `fmt.Println(args ...any)` — formatting arbitrary values

**When `any` is a code smell:**
- As a shortcut to avoid thinking about types
- When you end up type-asserting immediately after receiving the `any`
- When a smaller, well-defined interface would be clearer

If you find yourself writing:

```go
func process(v any) {
    switch val := v.(type) {
    case string:
        // ...
    case int:
        // ...
    }
}
```

Consider whether a proper interface or generic function would be clearer.

### Your notes

---

## Type Assertions and Type Switches

### Type Assertions

A type assertion extracts the concrete value from an interface:

```go
var s Storer = &MemoryStore{...}

// Single-return form: panics if s doesn't hold *MemoryStore
mem := s.(*MemoryStore)

// Two-return form: safe — never panics
mem, ok := s.(*MemoryStore)
if ok {
    fmt.Println("in-memory store with", len(mem.data), "keys")
}
```

Use the two-return form whenever there's any possibility the assertion could fail. The single-return form is only safe when you have a compile-time guarantee (e.g., you just stored the value yourself and there's no way it changed).

### Type Switches

When you need to handle multiple possible concrete types, use a type switch:

```go
func describeStore(s Storer) string {
    switch st := s.(type) {
    case *MemoryStore:
        return fmt.Sprintf("in-memory store (%d keys)", len(st.data))
    case *RedisStore:
        return fmt.Sprintf("redis store at %s", st.addr)
    case *S3Store:
        return fmt.Sprintf("s3 store in bucket %s", st.bucket)
    default:
        return fmt.Sprintf("unknown store: %T", st)
    }
}
```

The variable `st` in each case arm is typed as the concrete type — `*MemoryStore`, `*RedisStore`, etc. — so you can access type-specific fields. In the `default` case, `st` has the interface type.

`s.(type)` is only valid inside a `switch` statement. Don't try to use it elsewhere.

### The nil Interface Trap

This is one of the most surprising behaviors in Go. An interface value is nil only if both its type and value are nil. A non-nil interface can hold a nil concrete value — and that's a non-nil interface.

```go
func newStore(useDisk bool) Storer {
    var fs *FileStore // nil pointer to FileStore

    if useDisk {
        fs = &FileStore{path: "/tmp/store"}
    }
    return fs // DANGER: returning a non-nil interface with a nil value
}

s := newStore(false)
if s == nil {              // FALSE — s is not nil!
    fmt.Println("no store")
}
s.Get("key")               // PANIC: nil pointer dereference
```

What happened? The function returns `Storer` (interface). When it returns `fs` (which is `nil *FileStore`), Go wraps it: type = `*FileStore`, value = `nil`. The interface value is not nil — it has a type. The comparison `s == nil` checks both type and value. The type is `*FileStore`, so the interface is not nil, even though the concrete value is.

**The fix:** Return a nil interface, not a nil concrete type.

```go
func newStore(useDisk bool) Storer {
    if useDisk {
        return &FileStore{path: "/tmp/store"}
    }
    return nil  // returns a truly nil interface — type and value both nil
}
```

Or use an explicit nil check:

```go
func newStore(useDisk bool) Storer {
    var fs *FileStore
    if useDisk {
        fs = &FileStore{path: "/tmp/store"}
    }
    if fs == nil {
        return nil  // return nil interface, not nil *FileStore
    }
    return fs
}
```

**The mental model to remember:** An interface is like an envelope. A nil interface is an empty envelope. Returning a `nil *FileStore` is like putting an empty sheet of paper in an envelope — the envelope is not empty. The comparison `== nil` checks the envelope, not the paper.

> **Common Pitfall:** See [[go-nil-interface-trap]] for the full mechanics of why this happens and how to avoid it systematically.

### Your notes

---

## Common Stdlib Interfaces

Go's standard library defines a small, stable set of interfaces that appear everywhere. Learning them is learning Go's vocabulary.

### `io.Reader` and `io.Writer`

```go
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}
```

These are the most used interfaces in the entire language. Files, network connections, HTTP bodies, buffers, compressors, encryptors — they all implement `Reader` and/or `Writer`. Functions that accept `io.Reader` can be used with any of them.

```go
// This works with a file, a TCP connection, an HTTP body, a bytes.Buffer...
func computeHash(r io.Reader) (string, error) {
    h := sha256.New()
    if _, err := io.Copy(h, r); err != nil {
        return "", err
    }
    return hex.EncodeToString(h.Sum(nil)), nil
}
```

### `fmt.Stringer`

```go
type Stringer interface {
    String() string
}
```

When you implement `String() string`, `fmt.Println`, `fmt.Sprintf("%v", ...)` and friends automatically call it. You've already used this for enums — it applies everywhere.

### `error`

```go
type error interface {
    Error() string
}
```

The built-in `error` type is an interface. Any type with an `Error() string` method is an `error`. This is why custom error types work:

```go
type NotFoundError struct {
    Key string
}

func (e *NotFoundError) Error() string {
    return fmt.Sprintf("key %q not found", e.Key)
}

// *NotFoundError satisfies error — no declaration
func (m *MemoryStore) Get(key string) ([]byte, error) {
    v, ok := m.data[key]
    if !ok {
        return nil, &NotFoundError{Key: key}
    }
    return v, nil
}
```

### `sort.Interface`

```go
type Interface interface {
    Len() int
    Less(i, j int) bool
    Swap(i, j int)
}
```

Implement this on any slice type and you can use `sort.Sort`. This is how the standard library lets you sort arbitrary types without generics (before Go 1.18) or alongside `slices.SortFunc`.

### Your notes

---

## Interface Design Principles

### Accept Interfaces, Return Structs

The most-quoted Go interface principle. Functions should accept interfaces (accepting the minimum they need) and return concrete types (giving callers the full thing).

```go
// GOOD: accepts the minimum it needs
func backup(r io.Reader, dst string) error { ... }

// BAD: accepts a specific type, limiting callers
func backup(f *os.File, dst string) error { ... }

// GOOD: returns the concrete type
func NewMemoryStore() *MemoryStore { ... }

// BAD: returns the interface, hiding the concrete type from callers
func NewMemoryStore() Storer { ... }
```

Why return concrete types? Because the caller often needs methods that aren't in the interface. If you return `Storer`, the caller can't access `MemoryStore.Reset()` without a type assertion. Return `*MemoryStore` and everything is available. The caller can always assign it to a `Storer` variable if that's what they need.

The exception: factory functions that are *meant* to be polymorphic — where the caller genuinely should not know which concrete type they got.

### Keep Interfaces Small

The ideal interface has one or two methods. The standard library's most-used interfaces are all tiny: `io.Reader` (1 method), `io.Writer` (1 method), `fmt.Stringer` (1 method), `error` (1 method).

Why small? Because:
- Small interfaces are easier to implement — more types can satisfy them
- Small interfaces are easier to mock in tests
- Small interfaces compose into larger ones when needed

```go
// Too broad — many types can't do all of this
type StorageService interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
    Delete(key string) error
    List(prefix string) ([]string, error)
    Backup(dst io.Writer) error
    Restore(src io.Reader) error
    Stats() StorageStats
    Close() error
}

// Better — split by what callers actually need
type Getter interface { Get(key string) ([]byte, error) }
type Setter interface { Set(key string, value []byte) error }
type ReadWriter interface { Getter; Setter }
```

### Define Interfaces at the Point of Use

Go's approach: define the interface where you need it, not where you implement it. The package that needs to accept multiple backends defines the interface. The implementing packages don't reference it.

```go
// In package cache — defines what it needs
type Store interface {
    Get(key string) ([]byte, error)
    Set(key string, value []byte) error
}

// In package redis — knows nothing about cache.Store
type Client struct { ... }
func (c *Client) Get(key string) ([]byte, error) { ... }
func (c *Client) Set(key string, value []byte) error { ... }
// *redis.Client satisfies cache.Store — verified by the compiler if you use it that way
```

This is the opposite of Java/C#, where you define the interface in the same package as the implementation and "implement" it everywhere that uses it.

### When to Use a Function Type Instead of an Interface

Single-method interfaces that represent callbacks or transformations can often be replaced with function types:

```go
// Interface with one method
type Transformer interface {
    Transform(data []byte) ([]byte, error)
}

// A function type that does the same thing
type TransformFunc func(data []byte) ([]byte, error)

// Usage: a function type is often cleaner for simple callbacks
pipeline := []TransformFunc{
    compress,
    encrypt,
    sign,
}
```

Use function types when:
- The interface has exactly one method
- The implementations are short, inline functions
- You don't need to group state with the behavior

Use interfaces when:
- The type has state
- There are multiple related methods
- You're modeling a swappable component with complex behavior

### Your notes

---

## TypeScript and Rust Comparison

### TypeScript: Structural but Declared

TypeScript interfaces are also structural — a type doesn't need to declare `implements`. But the convention in TypeScript is often to declare it:

```typescript
interface Storer {
    get(key: string): Promise<Uint8Array | null>;
    set(key: string, value: Uint8Array): Promise<void>;
}

class MemoryStore implements Storer {  // explicitly declared, even though optional
    // ...
}
```

The key difference: TypeScript's structural typing is a type-check-only feature. At runtime, there are no interfaces — JavaScript doesn't have them. Go's interfaces are a runtime mechanism (the two-word type+data pair), which enables dynamic dispatch, type assertions, and reflection.

TypeScript also has **type unions** and **discriminated unions** as alternatives to interface polymorphism:

```typescript
type Store = MemoryStore | RedisStore | S3Store;
```

Go doesn't have union types (outside of `any` + type assertions). Interfaces are the primary polymorphism mechanism.

### Rust: Traits (Explicit but Powerful)

Rust's equivalent is **traits** — but they're explicitly implemented:

```rust
trait Storer {
    fn get(&self, key: &str) -> Result<Vec<u8>, Error>;
    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), Error>;
}

// Explicit implementation — must say "impl Storer for MemoryStore"
impl Storer for MemoryStore {
    fn get(&self, key: &str) -> Result<Vec<u8>, Error> { ... }
    fn set(&mut self, key: &str, value: &[u8]) -> Result<(), Error> { ... }
}
```

Rust traits are more powerful than Go interfaces — they support associated types, default implementations, trait bounds, and compile-time (static) dispatch via generics. Go interfaces are runtime (dynamic) dispatch always.

| Feature | Go Interface | TypeScript Interface | Rust Trait |
|---|---|---|---|
| Satisfaction declaration | Implicit (structural) | Implicit (but conventionally declared) | Explicit (`impl Trait for Type`) |
| Runtime dispatch | Yes (always) | N/A (compile-time only) | Optional (`dyn Trait` = dynamic, generics = static) |
| Composition | Interface embedding | Interface extension | Trait bounds |
| Default methods | No | No | Yes (`default` implementations) |
| Type assertions | Yes | Type guards (`instanceof`, type predicates) | `dyn Any` downcast |
| nil/null interface | Yes (nil interface trap) | `null` / `undefined` | No nil (Option<Box<dyn Trait>>) |

The biggest practical difference from Rust: Go interfaces always use dynamic dispatch (a pointer to a vtable). Rust traits with generics use static dispatch (monomorphized at compile time — one copy of the function per type). Go chose simplicity over performance here.

### Your notes
