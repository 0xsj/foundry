# Pointers & Smart Pointers — Rust

## The Problem Smart Pointers Solve

In Go, you have one pointer type: `*T`. It points somewhere in memory. You use it when you need indirection, and the garbage collector cleans up the memory eventually. Simple.

Rust has no garbage collector. The ownership system handles memory cleanup. But that single rule — one owner, freed when owner goes out of scope — turns out to be too restrictive for many real programs. What if two parts of the code need to share the same data? What if you need a recursive data structure? What if you need to mutate data through a shared reference?

Smart pointers solve these problems by encoding the ownership and access rules you need directly into the type. The compiler enforces those rules at compile time (or, when impossible, at runtime with a clear panic instead of silent corruption).

This module covers the full smart pointer toolkit. The concepts build on each other, so read in order.

---

## References: &T and &mut T

You've already used these. References are not heap-allocated — they're just pointers with borrow-checker rules attached.

```rust
let config = String::from("host=localhost");

let r: &String = &config;       // shared reference — read only
println!("{}", r);              // config is still valid

let mut s = String::from("hello");
let w: &mut String = &mut s;   // exclusive reference — read/write
w.push_str(" world");
// s is now "hello world"
```

The rules:
- Any number of shared references (`&T`) at the same time, OR
- Exactly one mutable reference (`&mut T`) at the same time
- Never both simultaneously
- References cannot outlive the data they point to

References are the right choice for the vast majority of code. They borrow data without claiming ownership. If `&T` is sufficient, use it.

**When references are not enough:**

1. You need the data to outlive the current scope (heap allocation)
2. Multiple owners need shared, long-lived access (not just temporary borrows)
3. You have a recursive type (would be infinitely sized on the stack)
4. You need interior mutability (mutate data behind a shared reference)

These cases are where smart pointers come in.

### Your notes
<!-- -->


---

## Box\<T\>: Ownership on the Heap

`Box<T>` allocates `T` on the heap and stores a pointer to it. When the `Box` goes out of scope, its `Drop` implementation frees the allocation. Single owner, deterministic cleanup — same rules as any other owned value, just on the heap.

```rust
// Stack allocation — size must be known at compile time
let x: i32 = 42;

// Heap allocation via Box
let y: Box<i32> = Box::new(42);
println!("{}", *y);  // deref to access the value
println!("{}", y);   // or just use y — Deref coercion handles it
```

`Box` is a thin wrapper. In release builds, it compiles to the same machine code as a raw pointer dereference. No runtime overhead beyond the heap allocation itself.

### When to Use Box\<T\>

**1. Recursive data structures**

The most important use case. Without `Box`, recursive types are infinitely sized:

```rust
// This does NOT compile — infinite size
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}
// JsonValue contains Vec<JsonValue> — Vec is fine, it's a heap pointer.
// But what if we had:

enum Tree {
    Leaf(i32),
    Node(i32, Tree, Tree),  // compile error: recursive type has infinite size
}

// Fix: one level of indirection
enum Tree {
    Leaf(i32),
    Node(i32, Box<Tree>, Box<Tree>),  // Box is a fixed-size pointer
}
```

The fix: wrap the recursive variant in `Box`. `Box<Tree>` is a fixed-size pointer (8 bytes on 64-bit). The tree structure lives on the heap.

**2. Trait objects (dynamic dispatch)**

When you need to store different types that implement the same trait, `Box<dyn Trait>` is the owned version:

```rust
trait EventHandler: Send {
    fn handle(&self, payload: &[u8]);
}

struct AuditLogger;
struct MetricsEmitter;
struct WebhookForwarder { url: String }

// These can all live in the same Vec:
let handlers: Vec<Box<dyn EventHandler>> = vec![
    Box::new(AuditLogger),
    Box::new(MetricsEmitter),
    Box::new(WebhookForwarder { url: String::from("https://hooks.example.com") }),
];

for handler in &handlers {
    handler.handle(b"{}");
}
```

We'll cover traits in depth in the traits module. The key point: `Box<dyn Trait>` gives you heap-allocated polymorphism with single ownership.

**3. Large values you want to avoid copying**

Less common, but occasionally useful: if you have a very large struct and want to ensure it's not copied when passed around, wrapping in `Box` makes moves cheap (moving a pointer instead of the whole struct).

### Box and Deref Coercion

`Box<T>` implements `Deref<Target = T>`, so you can use it almost anywhere a `T` would work:

```rust
let s = Box::new(String::from("hello"));

// All of these work via Deref coercion:
println!("{}", s);           // Display is called on &String via deref
println!("{}", s.len());     // calls s.deref().len()
let bytes: &[u8] = &s;       // &Box<String> -> &String -> &[u8] (double deref)
```

### Your notes
<!-- -->


---

## Rc\<T\>: Shared Ownership (Single-Threaded)

`Rc<T>` stands for **reference counted**. It lets multiple parts of the code own the same heap-allocated data, with cleanup deferred until the last owner is dropped.

```rust
use std::rc::Rc;

let config = Rc::new(String::from("timeout=30"));

let owner_a = Rc::clone(&config);  // increment reference count: 2
let owner_b = Rc::clone(&config);  // increment reference count: 3

println!("count: {}", Rc::strong_count(&config));  // 3

drop(owner_a);  // count: 2
drop(owner_b);  // count: 1
// config goes out of scope: count: 0, memory freed
```

`Rc::clone` does not clone the data — it increments the reference count and returns a new handle to the same allocation. Cheap. The data is freed when the count reaches zero.

**The critical constraint: `Rc<T>` is NOT `Send`.** You cannot move an `Rc` across thread boundaries. The reference count is not atomic. Use `Arc<T>` for multi-threaded scenarios (covered below).

**Access through `Rc` is read-only by default.** Since multiple owners exist, Rust's aliasing rules would prohibit mutation through any of them. To mutate, you combine `Rc` with `RefCell` (covered below).

### When to Use Rc\<T\>

The canonical use case: tree or graph structures where nodes need to reference their parents, siblings, or other nodes — and ownership can't be captured in a simple linear chain.

```rust
use std::rc::Rc;

// Component tree for a UI or config hierarchy
#[derive(Debug)]
struct Node {
    name: String,
    children: Vec<Rc<Node>>,
}

let root = Rc::new(Node {
    name: String::from("root"),
    children: vec![],
});

let child = Rc::new(Node {
    name: String::from("child"),
    children: vec![],
});

// Multiple views can hold the same node
let view_a = Rc::clone(&root);
let view_b = Rc::clone(&root);

// They all see the same data
assert!(Rc::ptr_eq(&view_a, &view_b));
```

### Rc\<T\> vs Box\<T\>

| | `Box<T>` | `Rc<T>` |
|---|---|---|
| Owners | 1 | Many |
| Overhead | None beyond allocation | Reference count (usize) |
| Mutation | Via `&mut` to Box | Read-only without RefCell |
| Thread safety | Yes | No (not Send) |
| Use case | Single owner, heap allocation | Shared read-only data |

### Your notes
<!-- -->


---

## Weak\<T\>: Breaking Reference Cycles

`Rc<T>` has a problem: reference cycles cause memory leaks. If `A` holds an `Rc` to `B` and `B` holds an `Rc` to `A`, neither count ever reaches zero.

```rust
use std::rc::Rc;
use std::cell::RefCell;

// If we model parent-child with Rc on both sides:
struct Node {
    children: Vec<Rc<Node>>,
    parent: Option<Rc<Node>>,  // CYCLE: parent holds children, children hold parent
}
// This leaks memory. The parent-child cycle is never broken.
```

The solution: use `Weak<T>` for back-references. A `Weak` reference does not contribute to the reference count. The allocation is freed when strong count hits zero, regardless of existing weak references.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,          // back-reference: weak
    children: RefCell<Vec<Rc<Node>>>,    // forward-reference: strong
}

let root = Rc::new(Node {
    name: String::from("root"),
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![]),
});

let child = Rc::new(Node {
    name: String::from("child"),
    parent: RefCell::new(Rc::downgrade(&root)),  // Weak pointer to parent
    children: RefCell::new(vec![]),
});

root.children.borrow_mut().push(Rc::clone(&child));

// Accessing a Weak: upgrade() returns Option<Rc<T>>
// Returns None if the target has been dropped
if let Some(parent) = child.parent.borrow().upgrade() {
    println!("child's parent: {}", parent.name);
}
```

**The rule of thumb:** in parent-child or owner-member relationships, the "forward" direction (parent → children) uses `Rc`. The "back" direction (child → parent) uses `Weak`. The tree is owned top-down; the upward links are non-owning.

### Your notes
<!-- -->


---

## RefCell\<T\>: Interior Mutability

The borrow checker enforces borrowing rules at compile time. Sometimes that's too restrictive — you know at runtime that no two borrows conflict, but the compiler can't prove it.

`RefCell<T>` moves the borrow check to runtime. It tracks active borrows dynamically and panics if you violate the rules (instead of refusing to compile).

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

// Immutable borrow:
{
    let r = data.borrow();       // returns Ref<Vec<i32>>
    println!("{:?}", *r);
}   // r dropped, borrow released

// Mutable borrow:
{
    let mut w = data.borrow_mut();  // returns RefMut<Vec<i32>>
    w.push(4);
}   // w dropped, borrow released

// This PANICS at runtime (two mutable borrows):
// let _a = data.borrow_mut();
// let _b = data.borrow_mut();  // PANIC: already mutably borrowed
```

`RefCell<T>` is not `Sync` — you cannot share it across threads. It's single-threaded interior mutability. For multi-threaded scenarios, use `Mutex<T>` (covered in the concurrency module).

### When RefCell Is the Right Tool

**1. Implementing a trait that requires `&self` but you need mutation**

```rust
use std::cell::RefCell;

// Imagine a trait from an external library
trait Cache {
    fn get(&self, key: &str) -> Option<String>;
}

struct LruCache {
    store: RefCell<std::collections::HashMap<String, String>>,
    // The HashMap mutation happens through &self — interior mutability required
}

impl Cache for LruCache {
    fn get(&self, key: &str) -> Option<String> {
        // Can mutate store through &self because it's RefCell
        self.store.borrow_mut().get(key).cloned()
    }
}
```

**2. Mock objects in tests**

```rust
use std::cell::RefCell;

struct MockNotifier {
    sent: RefCell<Vec<String>>,
}

impl MockNotifier {
    fn new() -> Self {
        MockNotifier { sent: RefCell::new(vec![]) }
    }

    fn notify(&self, msg: &str) {
        self.sent.borrow_mut().push(msg.to_string());
    }

    fn sent_count(&self) -> usize {
        self.sent.borrow().len()
    }
}
```

### The Rc\<RefCell\<T\>\> Pattern

The combination of `Rc` and `RefCell` gives you multiple-owner, mutable access in a single-threaded context. This is the standard pattern for shared mutable state without threads.

```rust
use std::rc::Rc;
use std::cell::RefCell;

type SharedCounter = Rc<RefCell<u64>>;

fn make_counter() -> SharedCounter {
    Rc::new(RefCell::new(0))
}

fn increment(counter: &SharedCounter) {
    *counter.borrow_mut() += 1;
}

fn get(counter: &SharedCounter) -> u64 {
    *counter.borrow()
}

let c = make_counter();
let c2 = Rc::clone(&c);  // second owner

increment(&c);
increment(&c2);   // both owners modify the same counter
println!("{}", get(&c));   // 2
```

The inner type pattern unlocks for trees and graphs: nodes own their children via `Rc<RefCell<Node>>`, and you can mutate (add/remove children) through any handle.

### Your notes
<!-- -->


---

## Cell\<T\>: Interior Mutability for Copy Types

`Cell<T>` is a simpler version of `RefCell<T>` that works only for `Copy` types. Instead of borrowing the value, it replaces it entirely. Because the value is `Copy`, you can read it by copying it out.

```rust
use std::cell::Cell;

struct RequestStats {
    // These fields can be updated from &self methods
    count: Cell<u64>,
    error_count: Cell<u64>,
}

impl RequestStats {
    fn new() -> Self {
        RequestStats {
            count: Cell::new(0),
            error_count: Cell::new(0),
        }
    }

    fn record_request(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn record_error(&self) {
        self.error_count.set(self.error_count.get() + 1);
    }

    fn success_rate(&self) -> f64 {
        let total = self.count.get() as f64;
        if total == 0.0 { return 1.0; }
        1.0 - (self.error_count.get() as f64 / total)
    }
}
```

`Cell<T>` has no runtime borrow tracking — it's simply a wrapper that bypasses the compile-time mutability rules for `Copy` types. It has zero overhead compared to a raw field, except that it's not `Sync`.

**When to use `Cell` vs `RefCell`:**
- Use `Cell<T>` for simple `Copy` fields (counters, flags, timestamps)
- Use `RefCell<T>` when you need to borrow the inner value (especially non-Copy types like `Vec`, `String`, or custom structs)

### Your notes
<!-- -->


---

## Arc\<T\>: Shared Ownership Across Threads

`Arc<T>` is "atomically reference counted" — it's `Rc<T>` with the reference count using atomic operations. This makes it `Send` and `Sync`, safe to share across threads.

```rust
use std::sync::Arc;
use std::thread;

let config = Arc::new(String::from("max_connections=100"));

let c1 = Arc::clone(&config);
let c2 = Arc::clone(&config);

let t1 = thread::spawn(move || {
    println!("thread 1: {}", c1);
});

let t2 = thread::spawn(move || {
    println!("thread 2: {}", c2);
});

t1.join().unwrap();
t2.join().unwrap();
```

`Arc` is slightly slower than `Rc` due to atomic operations on the reference count. For single-threaded code, prefer `Rc`. Only upgrade to `Arc` when you need threads.

### Arc\<Mutex\<T\>\>: The Multi-Threaded Shared Mutable State Pattern

`Arc` alone gives you read-only shared access across threads. For shared mutable state, combine `Arc` with `Mutex<T>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

// Connection pool with shared idle connection count
let pool_stats = Arc::new(Mutex::new(0u64));

let handles: Vec<_> = (0..4).map(|i| {
    let stats = Arc::clone(&pool_stats);
    thread::spawn(move || {
        let mut count = stats.lock().unwrap();
        *count += 1;
        println!("thread {} incremented, now {}", i, *count);
    })
}).collect();

for h in handles {
    h.join().unwrap();
}

println!("final: {}", pool_stats.lock().unwrap());
```

`Arc<Mutex<T>>` is the multi-threaded equivalent of `Rc<RefCell<T>>`. The pattern is so common it's almost a cliche in Rust code.

| Pattern | Threads | Mutation |
|---|---|---|
| `Rc<T>` | Single | None (or with RefCell) |
| `Rc<RefCell<T>>` | Single | Yes |
| `Arc<T>` | Multi | None |
| `Arc<Mutex<T>>` | Multi | Yes |

### Your notes
<!-- -->


---

## Cow\<T\>: Clone on Write

`Cow<'a, B>` stands for **clone on write**. It represents data that is either borrowed (`&B`) or owned (`B::Owned`). The key feature: allocation only happens when you actually need to modify the data.

```rust
use std::borrow::Cow;

fn normalize_username(s: &str) -> Cow<str> {
    if s.chars().all(|c| c.is_lowercase()) {
        // No transformation needed — borrow the original
        Cow::Borrowed(s)
    } else {
        // Transformation needed — allocate new String
        Cow::Owned(s.to_lowercase())
    }
}

let a = normalize_username("alice");   // Cow::Borrowed — no allocation
let b = normalize_username("Alice");   // Cow::Owned("alice") — allocated

println!("{}", a);  // works the same for both
println!("{}", b);
```

**Why this matters:** Functions that process strings often don't need to modify them. Without `Cow`, you'd either allocate always (`String` return type) or make the API awkward (returning a flag alongside a `&str`). With `Cow`, you get zero-cost borrowing in the common case and allocation only when necessary.

### Common Use Cases

**Normalizing/sanitizing input data:**

```rust
use std::borrow::Cow;

fn sanitize_path(path: &str) -> Cow<str> {
    if !path.contains("..") && !path.starts_with('/') {
        Cow::Borrowed(path)  // safe as-is
    } else {
        // Remove leading slashes, collapse ..
        let cleaned = path.trim_start_matches('/').replace("../", "");
        Cow::Owned(cleaned)
    }
}
```

**Error messages that are usually static but occasionally dynamic:**

```rust
use std::borrow::Cow;

fn validate_port(port: u16) -> Result<(), Cow<'static, str>> {
    if port == 0 {
        return Err(Cow::Borrowed("port cannot be zero"));
    }
    if port < 1024 {
        return Err(Cow::Owned(format!("port {} requires root privileges", port)));
    }
    Ok(())
}
```

`Cow` is also used extensively in the standard library for things like `String::from_utf8_lossy` which returns `Cow<str>` — borrowed if the bytes were valid UTF-8, owned if replacement characters were inserted.

### Your notes
<!-- -->


---

## Deref and DerefMut: How Smart Pointers Feel Transparent

All the smart pointers above feel "transparent" to use — you can call methods on the inner type without explicitly dereferencing. This is `Deref` coercion at work.

`Deref` is a trait:

```rust
pub trait Deref {
    type Target;
    fn deref(&self) -> &Self::Target;
}

pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

When Rust sees `smart_pointer.method()` and `method` doesn't exist on the smart pointer type, it tries `(*smart_pointer).method()`. This happens recursively: `Box<String>` derefs to `String`, `String` derefs to `str`, so you can call `str` methods directly on a `Box<String>`.

```rust
let s: Box<String> = Box::new(String::from("hello"));

// These are all equivalent:
println!("{}", s.len());           // Deref coercion: Box -> String -> len()
println!("{}", (*s).len());        // explicit deref
println!("{}", (**s).len() );      // if needed: String -> str -> len()

// Function parameter coercion:
fn takes_str(s: &str) { println!("{}", s); }

takes_str(&s);   // &Box<String> -> &String -> &str automatically
```

**Implementing Deref for your own type:**

```rust
use std::ops::Deref;

struct Authenticated<T>(T);

impl<T> Deref for Authenticated<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

struct UserProfile { name: String, email: String }

let user = Authenticated(UserProfile {
    name: String::from("alice"),
    email: String::from("alice@example.com"),
});

// Deref coercion: can call UserProfile methods directly
println!("{}", user.name);  // accesses through Deref
```

Only implement `Deref` when your type genuinely represents a "smart pointer" to the inner type — when dereferencing is the primary operation. Don't implement `Deref` just to get method forwarding; use explicit delegation or trait implementations instead. This is one of Rust's guidelines: `Deref` should only be implemented for pointer types.

### Your notes
<!-- -->


---

## Drop: Custom Cleanup

When a value goes out of scope, Rust calls its `Drop` implementation. For smart pointers, `Drop` is what frees the memory or decrements the reference count.

You can implement `Drop` for your own types when cleanup logic is needed beyond field drops:

```rust
use std::sync::Arc;

struct DatabasePool {
    name: String,
    connections: Arc<()>,  // just to illustrate shared tracking
}

impl Drop for DatabasePool {
    fn drop(&mut self) {
        // Called automatically when pool goes out of scope
        println!("[{}] pool shutting down, closing connections", self.name);
        // In real code: close actual connections, flush write-ahead log, etc.
    }
}

fn main() {
    let pool = DatabasePool {
        name: String::from("primary"),
        connections: Arc::new(()),
    };
    println!("pool created");
    // pool dropped here — Drop called automatically
}
// Output:
// pool created
// [primary] pool shutting down, closing connections
```

**Note:** You cannot call `drop` directly — `value.drop()` is a compile error. To force early cleanup, use `std::mem::drop(value)`. This is intentional: the compiler needs to control when destructors run to maintain safety invariants.

```rust
let pool = DatabasePool { ... };
std::mem::drop(pool);  // explicit early cleanup
// pool is no longer usable here
```

`Drop` and `Copy` are mutually exclusive. A `Copy` type can't have a `Drop` implementation, because if copying is implicit, the compiler can't determine when to call cleanup. Types that need cleanup always move.

### Your notes
<!-- -->


---

## When to Use Which: Decision Guide

```
Do you need heap allocation or indirection?
├── No → use &T or &mut T (references, zero cost)
│
└── Yes → Do you need multiple owners?
    ├── No → Box<T>
    │         Use for: recursive types, trait objects (Box<dyn Trait>), large values
    │
    └── Yes → Do you need multiple threads?
        ├── No (single thread) → Rc<T>
        │   └── Do you need mutation through shared refs?
        │       ├── Copy types → Rc<Cell<T>>
        │       └── Non-Copy types → Rc<RefCell<T>>
        │
        └── Yes (multi-thread) → Arc<T>
            └── Do you need mutation?
                └── Arc<Mutex<T>> (or Arc<RwLock<T>> for many readers)

Special cases:
  Back-references in trees/graphs → Weak<T> (with Rc or Arc)
  Avoid allocation for strings/slices → Cow<'a, B>
  Counting events from &self methods → Cell<u64> (Copy types only)
```

### Practical Rules

- **Start with references.** `&T` and `&mut T` solve most problems with zero overhead.
- **Add `Box` when you need ownership on the heap.** Usually for recursive types or `Box<dyn Trait>`.
- **Add `Rc` when ownership gets complicated in single-threaded code.** Tree nodes, callback systems, shared config.
- **Add `Weak` to break cycles** in `Rc` graphs. Parent-child: parent owns children (`Rc`), children point back (`Weak`).
- **Use `Arc` only when you need threads.** It has atomic overhead; don't pay it unnecessarily.
- **Use `RefCell` when `&mut` fights are real but justified.** Mock objects, trait implementations requiring `&self`, interior state.
- **Reach for `Cow` when optimizing string-heavy code** to avoid unnecessary allocations.

### Your notes
<!-- -->


---

## Comparison to Go

In Go, there is one pointer type: `*T`. That's it. Go's GC handles memory management, so there's no need for different ownership strategies.

```go
// Go: one pointer type does everything
config := &Config{Host: "localhost"}   // heap allocation
shared := config                       // both point to the same thing
go func() { fmt.Println(shared.Host) }()  // shared across goroutines, GC handles it
```

Rust's multiple pointer types aren't complexity for its own sake — each type exists because the GC that would make a single `*T` safe doesn't exist. Without a GC, the compiler needs to know _who owns_ the data and _when_ to free it. Different ownership patterns require different types.

| Go | Rust | What it models |
|---|---|---|
| `*T` (single reference) | `Box<T>` | Single owner, heap allocated |
| `*T` (shared across goroutines) | `Arc<T>` | Shared ownership, thread-safe |
| `sync.Mutex` + `*T` | `Arc<Mutex<T>>` | Shared mutable state across threads |
| N/A | `Rc<T>` | Shared ownership, single-threaded |
| N/A | `Rc<RefCell<T>>` | Shared mutable state, single-threaded |
| `*T` (back-pointer in linked list) | `Weak<T>` | Non-owning back reference |

Go's `*T` is flexible because the GC prevents use-after-free. Rust's variety is the tradeoff for compile-time memory safety without a runtime.

The benefit: when you reach for `Rc<RefCell<T>>` in Rust, you've explicitly declared "this data has multiple owners and can be mutated through shared references." That declaration is checked at runtime (`RefCell`'s borrow checking). No silent data races, no memory leaks from unchecked cycles. The type tells you the contract.

### Your notes
<!-- -->
