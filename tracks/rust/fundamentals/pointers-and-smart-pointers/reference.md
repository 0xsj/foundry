# Rust Reference — Pointers & Smart Pointers

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/),
> [The Rust Book](https://doc.rust-lang.org/book/), and
> [The Rustonomicon](https://doc.rust-lang.org/nomicon/) for the
> `pointers-and-smart-pointers` module.

---

## References

Source: [reference/types/pointer.html](https://doc.rust-lang.org/reference/types/pointer.html)

### Shared References (`&T`)

A **shared reference** (`&T`) is a non-owning pointer to a value of type `T`. Multiple shared references to the same value may coexist simultaneously.

- Allows read-only access to the referent
- Covariant over its lifetime parameter
- Implements `Copy`: sharing a reference is cheap
- `&T` is `Send` if `T: Sync`; `&T` is `Sync` if `T: Sync`

```rust
let x: i32 = 42;
let r1: &i32 = &x;
let r2: &i32 = &x;  // multiple shared references — fine
```

### Mutable References (`&mut T`)

A **mutable reference** (`&mut T`) is a non-owning pointer that allows read/write access.

- At most one `&mut T` to a given location at any time
- No other references (shared or mutable) may coexist with an active `&mut T` to the same location
- Invariant over its lifetime parameter
- `&mut T` is `Send` if `T: Send`; `&mut T` is `Sync` if `T: Sync`

### The Borrow Rules (Compile-Time)

At any point in a program, for any value `T`, either:
1. Zero or more shared references (`&T`) exist, OR
2. Exactly one mutable reference (`&mut T`) exists

But never both simultaneously. This is enforced statically by the borrow checker.

### Lifetime Elision

Source: [reference/lifetime-elision.html](https://doc.rust-lang.org/reference/lifetime-elision.html)

Explicit lifetimes are often elided when unambiguous:

```rust
// Elided — compiler infers 'a
fn first(v: &[i32]) -> &i32 { &v[0] }

// Explicit equivalent
fn first<'a>(v: &'a [i32]) -> &'a i32 { &v[0] }
```

Elision rules:
1. Each parameter with a reference gets its own lifetime parameter
2. If there is exactly one input lifetime, it is assigned to all output lifetimes
3. If one of the inputs is `&self` or `&mut self`, its lifetime is assigned to all outputs

---

## Box\<T\>

Source: [std::boxed](https://doc.rust-lang.org/std/boxed/index.html)

`Box<T>` is the simplest heap-allocating type. It allocates `T` on the heap, stores a pointer, and frees the memory when it goes out of scope via its `Drop` implementation.

### Key Properties

- **Single owner** — `Box<T>` implements `Deref<Target = T>` and `DerefMut`
- **`Send` if `T: Send`** — safe to move across threads
- **`Sync` if `T: Sync`** — safe to share references across threads
- **Zero runtime overhead** — in optimized builds, a `Box<T>` is equivalent to a raw pointer

### Creation and Access

```rust
let b: Box<i32> = Box::new(5);
let val: i32 = *b;          // dereference to get owned value (i32 is Copy)
let r: &i32 = &b;           // &Box<T> derefs to &T via Deref coercion
```

### Trait Objects

`Box<dyn Trait>` is the canonical form of a heap-allocated trait object:

```rust
let obj: Box<dyn std::fmt::Debug> = Box::new(42);
println!("{:?}", obj);
```

The fat pointer for `Box<dyn Trait>` contains a data pointer and a vtable pointer. Size is 2 machine words (16 bytes on 64-bit).

### Recursive Types

```rust
// Without Box: compile error "recursive type has infinite size"
enum List {
    Cons(i32, Box<List>),  // Box provides fixed-size indirection
    Nil,
}
```

---

## Rc\<T\>

Source: [std::rc](https://doc.rust-lang.org/std/rc/index.html)

`Rc<T>` ("Reference Counted") enables multiple ownership of heap-allocated data within a single thread.

### Key Properties

- **Reference counted** — allocation freed when `strong_count` reaches 0
- **NOT `Send`** — cannot be sent across thread boundaries
- **NOT `Sync`** — cannot be shared across threads via `&Rc<T>`
- **Interior immutability** — access through `Rc` is always `&T`; combine with `RefCell<T>` for mutation

### API

```rust
use std::rc::Rc;

let a = Rc::new(5);
let b = Rc::clone(&a);          // increments strong_count; same allocation
let c = a.clone();              // equivalent to Rc::clone(&a)

assert_eq!(Rc::strong_count(&a), 3);
assert!(Rc::ptr_eq(&a, &b));    // same allocation

drop(b);
assert_eq!(Rc::strong_count(&a), 2);

// Get mutable reference if no other Rcs exist:
let mut x = Rc::new(5);
if let Some(v) = Rc::get_mut(&mut x) {
    *v = 10;
}
```

### Weak\<T\>

`Weak<T>` is a non-owning reference that does not contribute to the strong count.

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(5);
let weak: Weak<i32> = Rc::downgrade(&strong);

assert_eq!(Rc::strong_count(&strong), 1);
assert_eq!(Rc::weak_count(&strong), 1);

// Upgrade to Rc (returns None if allocation was freed):
match weak.upgrade() {
    Some(val) => println!("{}", val),
    None => println!("already freed"),
}
```

**Purpose:** Break reference cycles. A `Weak<T>` does not keep the allocation alive.

---

## RefCell\<T\>

Source: [std::cell](https://doc.rust-lang.org/std/cell/index.html)

`RefCell<T>` provides **interior mutability** — the ability to mutate `T` through a shared (`&`) reference, with borrow rules checked dynamically at runtime.

### Key Properties

- **NOT `Sync`** — single-threaded only
- **Runtime panics** on borrow violations (instead of compile errors)
- Implements `Deref<Target = T>` indirectly via `Ref<T>` and `RefMut<T>`

### API

```rust
use std::cell::RefCell;

let rc: RefCell<Vec<i32>> = RefCell::new(vec![]);

// Immutable borrow — returns Ref<Vec<i32>> (implements Deref<Target = Vec<i32>>)
let r: std::cell::Ref<Vec<i32>> = rc.borrow();
println!("{:?}", *r);
drop(r);  // release borrow before taking mutable borrow

// Mutable borrow — returns RefMut<Vec<i32>>
rc.borrow_mut().push(1);

// try_borrow / try_borrow_mut — non-panicking variants returning Result
match rc.try_borrow_mut() {
    Ok(mut v) => v.push(2),
    Err(e) => eprintln!("borrow failed: {}", e),
}
```

### Panics

Calling `borrow()` when a mutable borrow is active, or calling `borrow_mut()` when any borrow is active, **panics at runtime** with: `already borrowed: BorrowMutError`.

---

## Cell\<T\>

Source: [std::cell::Cell](https://doc.rust-lang.org/std/cell/struct.Cell.html)

`Cell<T>` provides interior mutability for `Copy` types by replacing the value rather than lending a reference.

### Key Properties

- **Requires `T: Copy`** for `get()`; `set()` and `replace()` work for any `T`
- **NOT `Sync`** — single-threaded only
- **Zero runtime overhead** — no borrow tracking, no reference counting
- More restricted than `RefCell<T>`: cannot get a reference to the inner value

### API

```rust
use std::cell::Cell;

let c = Cell::new(0u64);
c.set(42);
assert_eq!(c.get(), 42);  // T must be Copy

// replace: set new value, return old value
let old = c.replace(100);
assert_eq!(old, 42);
assert_eq!(c.get(), 100);
```

---

## Arc\<T\>

Source: [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)

`Arc<T>` ("Atomically Reference Counted") is the thread-safe version of `Rc<T>`. The reference count uses atomic operations.

### Key Properties

- **`Send` if `T: Send + Sync`**
- **`Sync` if `T: Send + Sync`**
- **Atomic reference count** — slightly slower than `Rc` (memory barrier on increment/decrement)
- Same API as `Rc<T>` for cloning, downgrading, and counting

### Comparison to Rc

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Thread-safe | No | Yes |
| Reference count | `usize` (non-atomic) | `AtomicUsize` |
| `Send` | No | Yes (if `T: Send + Sync`) |
| Overhead | None beyond ref count | Atomic operations |

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);
let data2 = Arc::clone(&data);

thread::spawn(move || {
    println!("{:?}", data2);
}).join().unwrap();
```

---

## Cow\<'a, B\>

Source: [std::borrow::Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html)

`Cow<'a, B>` is a clone-on-write smart pointer. It can hold either a borrowed reference (`&'a B`) or an owned value (`B::Owned`).

### Definition

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

Common specializations:
- `Cow<'a, str>` — borrowed `&str` or owned `String`
- `Cow<'a, [T]>` — borrowed `&[T]` or owned `Vec<T>`
- `Cow<'a, Path>` — borrowed `&Path` or owned `PathBuf`

### API

```rust
use std::borrow::Cow;

fn ensure_lowercase(s: &str) -> Cow<str> {
    if s.chars().all(|c| c.is_lowercase() || !c.is_alphabetic()) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_lowercase())
    }
}

let a = ensure_lowercase("hello");  // Cow::Borrowed
let b = ensure_lowercase("HELLO");  // Cow::Owned

// into_owned() — always produces an owned value, cloning if necessary
let owned: String = a.into_owned();
```

### `to_owned()` vs `into_owned()`

- `to_owned()` — always clones, returns `B::Owned`
- `into_owned()` — returns `B::Owned` by moving if already `Owned`, or cloning if `Borrowed`

---

## The Deref Trait

Source: [std::ops::Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html)

```rust
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

### Deref Coercions

When `T: Deref<Target = U>`, Rust automatically coerces:
- `&T` to `&U`
- `&mut T` to `&U` (if `T: Deref<Target = U>`)
- `&mut T` to `&mut U` (if `T: DerefMut<Target = U>`)

Coercions chain: if `T: Deref<Target = U>` and `U: Deref<Target = V>`, then `&T` coerces to `&V`.

Standard library deref chains:
- `Box<T>` → `T`
- `Rc<T>` → `T`
- `Arc<T>` → `T`
- `String` → `str`
- `Vec<T>` → `[T]`
- `RefMut<'_, T>` → `T`

### Implementation Guideline

Source: [API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html)

> Implement `Deref` only for newtype wrappers that behave like pointers. Do not use `Deref` to provide method forwarding from a wrapper type to the wrapped type.

---

## The Drop Trait

Source: [std::ops::Drop](https://doc.rust-lang.org/std/ops/trait.Drop.html)

```rust
pub trait Drop {
    fn drop(&mut self);
}
```

`drop` is called automatically when a value goes out of scope (LIFO order for local variables). Manual invocation via `std::mem::drop(value)` is the only way to force early cleanup.

### Drop and Copy

A type cannot implement both `Copy` and `Drop`. `Copy` types are bitwise-copyable, which precludes any custom cleanup. The compiler enforces this:

```rust
// compile error: the trait `Copy` may not be implemented for this type;
// the type has a destructor
#[derive(Copy)]
struct Handle { fd: i32 }
impl Drop for Handle {
    fn drop(&mut self) { /* close fd */ }
}
```

### Drop Order

- Local variables: dropped in reverse declaration order (LIFO)
- Struct fields: dropped in declaration order
- Enum variants: dropped when the enum is dropped (only the active variant's fields)
- `Box<T>`: `T` dropped first, then heap memory freed

---

## Smart Pointer Summary

| Type | Owners | Threads | Mutation | Overhead |
|------|--------|---------|----------|----------|
| `&T` | N/A (borrow) | Yes (`T: Sync`) | No | None |
| `&mut T` | N/A (borrow) | No (exclusive) | Yes | None |
| `Box<T>` | 1 | Yes (`T: Send`) | Via `&mut` | Allocation only |
| `Rc<T>` | Many | No | No | `usize` ref count |
| `Rc<RefCell<T>>` | Many | No | Yes (runtime check) | ref count + borrow flag |
| `Arc<T>` | Many | Yes | No | atomic ref count |
| `Arc<Mutex<T>>` | Many | Yes | Yes (blocks) | atomic + OS mutex |
| `Cell<T>` | 1 (interior) | No | Yes (`Copy` only) | None |
| `RefCell<T>` | 1 (interior) | No | Yes (runtime check) | borrow flag |
| `Weak<T>` | 0 (non-owning) | Mirrors Rc/Arc | No | weak count |
| `Cow<'a, B>` | 0 or 1 | — | No (until modified) | None if borrowed |

---

## References

- [The Rust Book, Chapter 15 — Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [The Rust Reference — Pointer Types](https://doc.rust-lang.org/reference/types/pointer.html)
- [std::boxed](https://doc.rust-lang.org/std/boxed/index.html)
- [std::rc](https://doc.rust-lang.org/std/rc/index.html)
- [std::cell](https://doc.rust-lang.org/std/cell/index.html)
- [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)
- [std::borrow::Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [std::ops::Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html)
- [std::ops::Drop](https://doc.rust-lang.org/std/ops/trait.Drop.html)
- [The Rustonomicon — Leaking](https://doc.rust-lang.org/nomicon/leaking.html)
