# Rust Reference — Variables and Types

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/) for the
> `variables-and-types` module. Covers: variable bindings, primitive types, composite
> types, const/static items, and drop semantics.

---

## Variables

Source: [reference/variables.html](https://doc.rust-lang.org/reference/variables.html)

A **variable** is a component of a stack frame. It can be a named function parameter,
an anonymous temporary, or a named local variable.

- Local variables hold values directly, allocated within the stack frame.
- Local variables are **immutable unless declared with `mut`**.
- Variables are **not initialized when allocated** — the entire frame is allocated on
  entry in an uninitialized state. A variable can only be used after it has been
  initialized through all reachable control flow paths.

```rust
fn example() {
    let init_after_if: ();
    let uninit_after_if: ();

    if random_bool() {
        init_after_if = ();
        uninit_after_if = ();
    } else {
        init_after_if = ();
    }

    init_after_if;       // ok — initialized in both branches
    // uninit_after_if;  // error: use of possibly uninitialized variable
}
```

---

## Let Statements

Source: [reference/statements.html](https://doc.rust-lang.org/reference/statements.html)

A `let` statement introduces new variables given by a pattern, optionally followed by a
type annotation and an initializer expression.

- When no type annotation is given, the compiler infers the type.
- Variables are visible from the point of declaration until the end of the enclosing
  block scope, except when shadowed by another variable declaration.
- Without an `else` block, the pattern must be irrefutable.
- With an `else` block, the pattern may be refutable; the `else` must diverge (never type).

```rust
let (mut v, w) = (vec![1, 2, 3], 42);  // destructuring, mut binding

let Some(t) = v.pop() else {  // refutable pattern with else
    panic!();                   // else must diverge
};

let [u, v] = [v[0], v[1]] else {  // irrefutable — compiler warns
    panic!();
};
```

---

## Type System Overview

Source: [reference/types.html](https://doc.rust-lang.org/reference/types.html)

Every variable, item, and value has a type. The type of a value defines the
interpretation of the memory holding it and the operations that may be performed on it.

### Type Categories

| Category | Types |
|---|---|
| **Primitive** | `bool`, integer/float, `char`, `str`, `!` (never) |
| **Sequence** | Tuple `(T, U, ..)`, Array `[T; N]`, Slice `[T]` |
| **User-defined** | `struct`, `enum`, `union` |
| **Function** | Function items, closures |
| **Pointer** | `&T`, `&mut T`, `*const T`, `*mut T`, `fn()` pointers |
| **Trait** | `dyn Trait` (dynamic dispatch), `impl Trait` (abstract return) |

### Recursive Types

Nominal types (struct, enum, union) may be recursive. Recursive fields must use pointer
indirection to keep the size finite:

```rust
enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),  // Box provides indirection
}
```

---

## Numeric Types

Source: [reference/types/numeric.html](https://doc.rust-lang.org/reference/types/numeric.html)

### Signed Integers (two's complement)

| Type | Size | Min | Max |
|---|---|---|---|
| `i8` | 1 byte | -(2^7) = -128 | 2^7 - 1 = 127 |
| `i16` | 2 bytes | -(2^15) = -32,768 | 2^15 - 1 = 32,767 |
| `i32` | 4 bytes | -(2^31) | 2^31 - 1 |
| `i64` | 8 bytes | -(2^63) | 2^63 - 1 |
| `i128` | 16 bytes | -(2^127) | 2^127 - 1 |

### Unsigned Integers

| Type | Size | Min | Max |
|---|---|---|---|
| `u8` | 1 byte | 0 | 255 |
| `u16` | 2 bytes | 0 | 65,535 |
| `u32` | 4 bytes | 0 | 4,294,967,295 |
| `u64` | 8 bytes | 0 | 2^64 - 1 |
| `u128` | 16 bytes | 0 | 2^128 - 1 |

### Machine-Dependent

| Type | Size | Notes |
|---|---|---|
| `usize` | pointer-width | Used for indexing, memory addresses. Min width: 16 bits. |
| `isize` | pointer-width | Used for pointer arithmetic, object size differences. |

### Floating Point

| Type | Size | Spec |
|---|---|---|
| `f32` | 4 bytes | IEEE 754-2008 binary32 |
| `f64` | 8 bytes | IEEE 754-2008 binary64 |

### Bit Validity

For every numeric type `T`, the bit validity is equivalent to `[u8; size_of::<T>()]`.
An uninitialized byte is **not** a valid `u8`.

---

## Boolean Type

Source: [reference/types/boolean.html](https://doc.rust-lang.org/reference/types/boolean.html)

- Two values: `true` and `false`
- **Size and alignment**: 1 byte each
- **Bit patterns**: `false` = `0x00`, `true` = `0x01`. Any other pattern is undefined behavior.
- Implements `Clone`, `Copy`, `Sized`, `Send`, `Sync`

### Operations

| Op | Symbol | Notes |
|---|---|---|
| NOT | `!b` | Logical negation |
| OR | `a \| b` | Bitwise/logical OR |
| AND | `a & b` | Bitwise/logical AND |
| XOR | `a ^ b` | Exclusive OR |
| Short-circuit OR | `a \|\| b` | Lazy evaluation |
| Short-circuit AND | `a && b` | Lazy evaluation |

Used as the operand type for `if` and `while` condition expressions.

---

## Textual Types

Source: [reference/types/textual.html](https://doc.rust-lang.org/reference/types/textual.html)

### `char`

- A **Unicode scalar value** (code point that is not a surrogate)
- Represented as a 32-bit unsigned word (same layout as `u32`)
- Valid range: `0x0000..=0xD7FF` or `0xE000..=0x10FFFF`
- Creating a `char` outside this range is undefined behavior
- Effectively UTF-32 / UCS-4 of length 1

### `str`

- Represented the same way as `[u8]` — a slice of bytes
- **Must contain valid UTF-8 data** (standard library assumes and enforces this)
- A **dynamically sized type (DST)** — can only be used through a pointer: `&str`
- `&str` has the same layout as `&[u8]` (pointer + length)

---

## Tuple Types

Source: [reference/types/tuple.html](https://doc.rust-lang.org/reference/types/tuple.html)

- Structural types for heterogeneous lists: `(T, U, V)`
- Fields named by position: `.0`, `.1`, `.2`
- 1-ary tuples require a trailing comma: `(i32,)`
- Unit type `()` is the empty tuple — like `void`
- Order matters: `(String, i32)` and `(i32, String)` are different types

---

## Array Types

Source: [reference/types/array.html](https://doc.rust-lang.org/reference/types/array.html)

- Fixed-size sequence: `[T; N]` where `N` is a compile-time constant (`usize`)
- Stack-allocated by default
- All elements always initialized
- Access is always bounds-checked in safe code
- For dynamically-sized sequences, use `Vec<T>`

```rust
let array: [i32; 3] = [1, 2, 3];
let boxed_array: Box<[i32]> = Box::new([1, 2, 3]);  // heap-allocated
```

---

## Slice Types

Source: [reference/types/slice.html](https://doc.rust-lang.org/reference/types/slice.html)

- A dynamically-sized view into a contiguous sequence: `[T]`
- Always used through pointer types:
  - `&[T]` — shared slice (borrows data)
  - `&mut [T]` — mutable slice
  - `Box<[T]>` — boxed (heap-allocated) slice
- All elements always initialized
- Access always bounds-checked in safe code

```rust
let boxed: Box<[i32]> = Box::new([1, 2, 3]);
let slice: &[i32] = &boxed[..];
```

---

## Const Items

Source: [reference/items/constant-items.html](https://doc.rust-lang.org/reference/items/constant-items.html)

```rust
const MAX_RETRIES: i32 = 3;
```

- An optionally named constant value **not associated with a specific memory location**
- **Inlined** wherever used — copied directly into the calling context
- Must be **explicitly typed**
- Must have a `'static` lifetime (references in the initializer must be `'static`)
- Evaluated at **compile time** — always, even in unused functions
- Unnamed constants are allowed: `const _: () = { ... };`

### Const vs Let

| | `const` | `let` |
|---|---|---|
| Memory | No guaranteed address; inlined | Allocated on the stack |
| Type | Must be explicit | Can be inferred |
| Evaluation | Compile-time | Runtime |
| Scope | Module or block | Block |

---

## Static Items

Source: [reference/items/static-items.html](https://doc.rust-lang.org/reference/items/static-items.html)

```rust
static GLOBAL_COUNT: u32 = 0;
static mut MUTABLE_GLOBAL: u32 = 0;  // requires unsafe to access
```

- Represents a **single allocation** in the program — all references point to the same address
- Has the `'static` lifetime (outlives all other lifetimes)
- **Does not call `drop`** at the end of the program
- Immutable statics must implement `Sync` (safe to access from any thread)
- Mutable statics require `unsafe` for any read or write

### When to Use Static vs Const

Use `static` when:
- Large amounts of data are stored (avoids inlining copies)
- A single address is required
- Interior mutability is needed

Otherwise, prefer `const`.

---

## Destructors and Drop Semantics

Source: [reference/destructors.html](https://doc.rust-lang.org/reference/destructors.html)

Destructors run automatically when an initialized variable or temporary goes out of
scope. This is Rust's **RAII** (Resource Acquisition Is Initialization) pattern.

### How Destructors Work

The destructor of a type `T`:
1. Calls `Drop::drop` if the type implements the `Drop` trait
2. Recursively drops all fields:
   - Struct fields: declaration order
   - Enum variants: declaration order of active variant
   - Tuple elements: in order
   - Array/slice elements: first to last
   - Closure captures: unspecified order

### Drop Scopes

Variables are dropped when control flow leaves their drop scope, in **reverse order
of declaration**:

```rust
let first = PrintOnDrop("Dropped last");
{
    let inner = PrintOnDrop("Dropped when inner scope ends");
}
let last = PrintOnDrop("Dropped first");
// Drop order: last, first (inner already dropped)
```

### Assignment Drops the Old Value

```rust
let mut x = PrintOnDrop("first");
x = PrintOnDrop("second");  // "first" is dropped here
// At scope end, "second" is dropped
```

### Preventing Destructors

- `core::mem::forget` — prevent destructor from running
- `core::mem::ManuallyDrop` — wrapper that prevents automatic dropping
- Process termination (`exit`, `abort`, abort-on-panic) doesn't run destructors

It is **safe** to prevent destructors from running. Types should not rely on
destructors for soundness.
