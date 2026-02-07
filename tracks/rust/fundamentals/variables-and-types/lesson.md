# Variables and Types — Rust

## How Variables Work Under the Hood

### Ownership Starts Here

Every variable in Rust **owns** its value. When the variable goes out of scope, the value is dropped (freed). There is no garbage collector. This is the foundation of everything in Rust.

```rust
{
    let s = String::from("hello");  // s owns this heap-allocated string
    // s is valid here
}   // s goes out of scope — Rust calls drop(), memory is freed
```

### Stack vs Heap

Rust gives you direct control:

- **Stack**: fixed-size types. `i32`, `f64`, `bool`, `[i32; 5]`, tuples, `&str` (the pointer, not the data). Fast allocation (just move the stack pointer). Automatically freed when scope ends.
- **Heap**: dynamically-sized types. `String`, `Vec<T>`, `Box<T>`. Allocated via the allocator. Freed when the owner goes out of scope (RAII).

```rust
let x: i32 = 42;                     // 4 bytes on the stack
let s: String = String::from("hello"); // 24 bytes on the stack (ptr + len + capacity)
                                       // + 5 bytes on the heap ("hello")
```

A `String` on the stack looks like:

```
Stack:                     Heap:
┌──────────────┐          ┌───┬───┬───┬───┬───┐
│ ptr ────────────────────→│ h │ e │ l │ l │ o │
│ len: 5       │          └───┴───┴───┴───┴───┘
│ capacity: 5  │
└──────────────┘
```

### Copy vs Move

When you assign one variable to another, Rust either **copies** or **moves**:

```rust
// Copy — stack types that implement the Copy trait
let a: i32 = 42;
let b = a;          // copies the bits. both a and b are valid.
println!("{a} {b}"); // works fine

// Move — heap types transfer ownership
let s1 = String::from("hello");
let s2 = s1;        // s1's ownership MOVES to s2. s1 is now invalid.
// println!("{s1}"); // compile error: value used after move
println!("{s2}");    // works
```

Types that implement `Copy` (all primitives, fixed-size arrays of Copy types, tuples of Copy types) are bitwise copied. Everything else is moved.

This is Rust's killer feature: at compile time, it guarantees exactly one owner for every piece of data. No double-frees, no use-after-free, no data races.

### Your notes
<!-- -->


---

## Type System

### Static, Nominal, Strong, Zero-Cost

Rust's type system is:
- **Static**: all types resolved at compile time
- **Nominal**: a type is defined by its name, not its shape. `struct Meters(f64)` and `struct Seconds(f64)` are different types even though they wrap the same data.
- **Strong**: no implicit conversions. `i32` and `i64` are different. You must use `as` or `from`/`into`.
- **Zero-cost abstractions**: generics are monomorphized — compiled to concrete types with no runtime dispatch.

```rust
let a: i32 = 42;
let b: i64 = a as i64;   // explicit cast required
// let c: i64 = a;        // error: mismatched types
```

### Primitive Types and Their Sizes

| Type | Size | Default | Notes |
|---|---|---|---|
| `i8`/`u8` | 1 byte | — | `u8` is also `byte` |
| `i16`/`u16` | 2 bytes | — | |
| `i32`/`u32` | 4 bytes | — | `i32` is the default integer |
| `i64`/`u64` | 8 bytes | — | |
| `i128`/`u128` | 16 bytes | — | |
| `isize`/`usize` | pointer-sized | — | Used for indexing |
| `f32` | 4 bytes | — | |
| `f64` | 8 bytes | — | Default float |
| `bool` | 1 byte | — | |
| `char` | 4 bytes | — | Unicode scalar value (not a byte!) |
| `()` | 0 bytes | — | Unit type — like void |

**Rust has no zero values.** Every variable must be initialized before use. The compiler enforces this.

### Your notes
<!-- -->


---

## String vs &str

This confuses every Rust beginner. Two string types:

| | `String` | `&str` |
|---|---|---|
| Ownership | Owned | Borrowed |
| Location | Stack header + heap data | Pointer + length (to data elsewhere) |
| Size on stack | 24 bytes (ptr + len + cap) | 16 bytes (ptr + len) |
| Mutable | Yes (if `let mut`) | No |
| Growable | Yes | No |
| Literal | `String::from("hello")` | `"hello"` |

```rust
let literal: &str = "hello";           // points to static memory (baked into binary)
let owned: String = String::from("hello"); // copies to heap, you own it
let borrowed: &str = &owned;           // borrows the String as a &str slice
```

When to use which:
- **`&str`** for function parameters (accept borrows — more flexible)
- **`String`** when you need to own, store, or modify the string
- **`&str`** for string literals

```rust
fn greet(name: &str) {          // accepts both String (via deref) and &str
    println!("hello, {name}");
}

greet("world");                  // &str literal
greet(&String::from("world"));   // borrowed String
```

### Your notes
<!-- -->


---

## Shadowing

Rust lets you redeclare a variable with the same name. This creates a **new variable** — it's not mutation.

```rust
let x = 5;
let x = x + 1;        // new x shadows old x. old x is gone.
let x = x * 2;        // shadows again. x is now 12.

let s = "42";
let s: i32 = s.parse().unwrap();  // shadows with a DIFFERENT TYPE
```

Why this exists:
- Avoids naming pollution (`s_str`, `s_int`, `s_parsed`)
- Keeps variables immutable while still transforming values
- The old value is dropped when shadowed (if nothing else references it)

This is different from `let mut` — mutation changes the value in place, shadowing creates a new binding.

### Your notes
<!-- -->


---

## References and Borrowing (Preview)

This gets its own deep lesson, but the basics:

```rust
let s = String::from("hello");
let r: &String = &s;           // r borrows s (immutable reference)
println!("{r}");                // ok — reading through a borrow
// r is a pointer on the stack, pointing to s's stack data

let mut s = String::from("hello");
let r: &mut String = &mut s;   // mutable borrow — only ONE allowed at a time
r.push_str(" world");
```

Rules:
- You can have **many** immutable references (`&T`) OR **one** mutable reference (`&mut T`), never both
- References must always be valid (no dangling pointers — the borrow checker enforces this)

### Your notes
<!-- -->


---

## Composite Types (Preview)

| Type | Syntax | Stack/Heap | Notes |
|---|---|---|---|
| Tuple | `(i32, f64, bool)` | Stack | Fixed size, mixed types |
| Array | `[i32; 5]` | Stack | Fixed size, same type |
| Vec | `Vec<i32>` | Stack header + heap | Dynamic, same type |
| Struct | `struct Point { x: f64, y: f64 }` | Stack (usually) | Named fields |
| Enum | `enum Option<T> { Some(T), None }` | Stack (usually) | Tagged union — Rust's most powerful type |

### Your notes
<!-- -->
