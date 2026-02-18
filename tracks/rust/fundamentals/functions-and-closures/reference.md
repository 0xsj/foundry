# Rust Reference — Functions and Closures

> Extracted from [The Rust Reference](https://doc.rust-lang.org/reference/) and
> [The Rust Programming Language](https://doc.rust-lang.org/book/) for the
> `functions-and-closures` module. Covers: function items, function pointers,
> closure expressions, closure trait implementations, `impl Trait`, and diverging functions.

---

## Function Items

Source: [reference/items/functions.html](https://doc.rust-lang.org/reference/items/functions.html)

A **function item** defines a callable, statically dispatched function. Each function definition
creates a distinct, zero-sized type. Two functions with the same signature have different types.

```rust
fn answer_to_life() -> i32 { 42 }
fn add(a: i32, b: i32) -> i32 { a + b }
```

### Syntax

```
fn IDENTIFIER GenericParams? ( FunctionParameters? ) FunctionReturnType? WhereClause? BlockExpression
```

- **Parameters**: each of the form `pattern: Type`. Patterns may be irrefutable, including
  destructuring patterns. The `_` pattern is allowed to explicitly discard a parameter.
- **Return type**: `-> Type`. If omitted, the return type is `()` (unit).
- **Body**: a block expression. The final expression in the block is the return value.

### Qualifiers

| Qualifier | Meaning |
|---|---|
| `pub` | Visible outside the current module |
| `async` | Function returns `impl Future` |
| `const` | Can be evaluated at compile time |
| `unsafe` | Callers must uphold invariants not checked by the compiler |
| `extern "ABI"` | Uses a specific calling convention |

### Function Parameters

Parameters can use patterns, including destructuring:

```rust
fn first_two(&(a, b): &(i32, i32)) -> i32 { a + b }
fn ignore_first(_: i32, y: i32) -> i32 { y }
fn with_self(&self) { }  // only valid in impl blocks
```

Variadic functions are only supported via `extern "C"`.

---

## Return Values and Expressions

Source: [reference/expressions/block-expr.html](https://doc.rust-lang.org/reference/expressions/block-expr.html)

A block `{ ... }` is an expression. Its value is:
- The value of the final expression (if not followed by `;`)
- `()` if the final statement ends with `;` or the block is empty

```rust
let x: i32 = {
    let a = 1;
    let b = 2;
    a + b       // no semicolon — block evaluates to 3
};
```

### The `return` Expression

Source: [reference/expressions/return-expr.html](https://doc.rust-lang.org/reference/expressions/return-expr.html)

`return` causes the function to exit, returning the given value.

- `return` without a value is equivalent to `return ()`
- `return` is an expression of type `!` (never), so it can appear anywhere an expression is expected

```rust
fn find_first_error(items: &[Result<i32, &str>]) -> Result<(), &str> {
    for item in items {
        if let Err(e) = item {
            return Err(e);  // early exit
        }
    }
    Ok(())
}
```

---

## The Never Type (`!`)

Source: [reference/types/never.html](https://doc.rust-lang.org/reference/types/never.html)

The **never type** `!` is a type with no values. A function returning `!` diverges — it never
returns to the caller.

- Can coerce to **any type** (it is a subtype of every type)
- Used to signal: this code path is unreachable, or execution does not continue

```rust
fn fatal_error(msg: &str) -> ! {
    panic!("{}", msg);
}

fn loop_forever() -> ! {
    loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}
```

**Expressions with type `!`:**
- `panic!("...")` and `todo!()`, `unreachable!()`, `unimplemented!()`
- `std::process::exit(code)`
- `loop {}` without a `break` value
- `return <expr>` — the `return` expression itself is `!`

**Practical use:** In a `match` arm, returning `!` satisfies any required type:

```rust
let value: i32 = match some_option {
    Some(v) => v,
    None => panic!("expected value"),  // type ! coerces to i32
};
```

---

## Function Pointers

Source: [reference/types/function-pointer.html](https://doc.rust-lang.org/reference/types/function-pointer.html)

A **function pointer** is a pointer to code. Written as `fn(Type, ...) -> ReturnType`.

- Has a fixed size (one machine pointer)
- Is `Copy` and `Clone`
- Carries no captured state (unlike closures)
- Implements `Fn`, `FnMut`, `FnOnce`

```rust
fn double(x: i32) -> i32 { x * 2 }

let f: fn(i32) -> i32 = double;  // function pointer type
let g = f;                         // copied
let h: fn(i32) -> i32 = |x| x + 1;  // non-capturing closure coerces to fn ptr
```

### Function Item vs Function Pointer

| | Function Item | Function Pointer |
|---|---|---|
| Type | Zero-sized, unique per function | `fn(T) -> U`, one pointer |
| Copy | Yes | Yes |
| Closures | N/A | Only non-capturing |
| Dispatch | Static (zero cost) | Indirect (one pointer dereference) |

Function items coerce to function pointers in most contexts. The coercion is explicit when you
assign to a `fn(...)` variable.

---

## Closure Expressions

Source: [reference/expressions/closure-expr.html](https://doc.rust-lang.org/reference/expressions/closure-expr.html)

A **closure expression** produces a value of an anonymous closure type.

### Syntax

```
move? |CaptureList| ReturnType? Body
```

- Parameters may omit type annotations (inferred from usage)
- Return type annotation is optional
- Body may be a single expression (no braces) or a block (`{ ... }`)
- `move` keyword forces ownership capture of all used bindings

```rust
let add = |a: i32, b: i32| -> i32 { a + b };  // full syntax
let double = |x| x * 2;                         // inferred types
let greet = |name| {                             // braced body
    format!("Hello, {}!", name)
};
let unit = || ();                                // no params, returns ()
```

### Capture Modes

The compiler determines each capture's mode from the closure body:

| How the variable is used | Capture mode | Requires |
|---|---|---|
| `x` (read-only) | Immutable borrow `&x` | — |
| `x = ...` or `x.mutating_method()` | Mutable borrow `&mut x` | — |
| `drop(x)` or move into sub-expression | Move (by value) | — |
| `move` keyword | Move (all captures) | `move` keyword |

Variables of `Copy` types are always copied (not moved), even with `move`.

```rust
let s = String::from("hello");
let n: i32 = 42;

let f = move || {
    println!("{}", s);  // s is moved into closure (String is not Copy)
    println!("{}", n);  // n is copied into closure (i32 is Copy)
};
// println!("{}", s);  // ERROR: s moved
println!("{}", n);      // OK: n was copied
```

---

## Closure Trait Implementations

Source: [reference/types/closure.html](https://doc.rust-lang.org/reference/types/closure.html)

Every closure type automatically implements one or more of `Fn`, `FnMut`, `FnOnce`:

### `FnOnce`

```rust
pub trait FnOnce<Args: Tuple> {
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
```

- Consumes `self` on call. Can only be called once.
- Every closure implements `FnOnce` (even ones that can be called multiple times).
- A closure implements only `FnOnce` when its body **moves out** a captured value.

### `FnMut`

```rust
pub trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
```

- Borrows `self` mutably on call. Can be called multiple times.
- A closure implements `FnMut` (but not `Fn`) when its body **mutates** a captured value.
- Iterators that accumulate state use `FnMut`.

### `Fn`

```rust
pub trait Fn<Args: Tuple>: FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
```

- Borrows `self` immutably on call. Can be called any number of times, including concurrently.
- A closure implements `Fn` when its body only **reads** captured values (or captures nothing).

### Implementation Rules (Summary)

| Closure body | `FnOnce` | `FnMut` | `Fn` |
|---|---|---|---|
| No captures | Yes | Yes | Yes |
| Reads captures | Yes | Yes | Yes |
| Mutates captures | Yes | Yes | No |
| Moves captures out | Yes | No | No |

### `Send` and `Sync`

Closure types are `Send` if all their captured values are `Send`.
Closure types are `Sync` if all their captured values are `Sync`.
This is why `move` closures are commonly used with `std::thread::spawn`:

```rust
// spawn requires F: Send + 'static
// move ensures the closure owns its data (not a reference with a shorter lifetime)
std::thread::spawn(move || { /* ... */ });
```

---

## `impl Trait`

Source: [reference/types/impl-trait.html](https://doc.rust-lang.org/reference/types/impl-trait.html)

`impl Trait` is syntactic sugar that can appear in argument or return position.

### Argument Position (Anonymous Generic)

```rust
fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 { f(x) }
// Equivalent to:
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
```

- Each `impl Trait` parameter is a distinct generic type parameter.
- Monomorphized at compile time — no runtime overhead.
- Cannot be named; use explicit generics if you need to use the type elsewhere.

### Return Position (Existential Type)

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

- The function returns **some specific concrete type** that implements the trait.
- The caller cannot inspect the concrete type.
- The concrete type must be the **same type in all return paths** — you cannot return
  different closure types from different branches using `impl Trait`.
- Use `Box<dyn Trait>` when the concrete type varies across branches.

### `impl Trait` vs `dyn Trait`

| | `impl Trait` | `dyn Trait` |
|---|---|---|
| Dispatch | Static (monomorphized) | Dynamic (vtable) |
| Size | `Sized` (concrete at compile time) | Not `Sized`; needs `&dyn` or `Box<dyn>` |
| Performance | Zero-cost | Vtable lookup + possible indirection |
| Return types | Single concrete type per path | Any type implementing the trait |
| Can store in `Vec` | No (each would be a different type) | Yes: `Vec<Box<dyn Trait>>` |

---

## Higher-Order Functions

Higher-order functions accept or return functions/closures. The standard library is built on them.

### Common Iterator Methods

| Method | Closure trait | Signature (simplified) |
|---|---|---|
| `map` | `FnMut(A) -> B` | `Iterator<Item=A> -> Iterator<Item=B>` |
| `filter` | `FnMut(&A) -> bool` | `Iterator<Item=A> -> Iterator<Item=A>` |
| `filter_map` | `FnMut(A) -> Option<B>` | `Iterator<Item=A> -> Iterator<Item=B>` |
| `fold` | `FnMut(B, A) -> B` | `Iterator<Item=A>, init: B -> B` |
| `for_each` | `FnMut(A)` | Consumes iterator, applies closure |
| `any` | `FnMut(&A) -> bool` | `Iterator<Item=A> -> bool` |
| `all` | `FnMut(&A) -> bool` | `Iterator<Item=A> -> bool` |
| `find` | `FnMut(&A) -> bool` | `Iterator<Item=A> -> Option<A>` |
| `position` | `FnMut(&A) -> bool` | `Iterator<Item=A> -> Option<usize>` |
| `sort_by` | `FnMut(&A, &A) -> Ordering` | Sorts `Vec<A>` in place |
| `retain` | `FnMut(&A) -> bool` | Removes elements from `Vec<A>` |

### Returning Closures

A function returning a closure must either:

1. **Use `impl Fn`** — when a single concrete closure type is returned:
   ```rust
   fn adder(n: i32) -> impl Fn(i32) -> i32 {
       move |x| x + n   // move required: n must be owned by the closure
   }
   ```

2. **Use `Box<dyn Fn>`** — when different closure types may be returned:
   ```rust
   fn selector(flag: bool) -> Box<dyn Fn(i32) -> i32> {
       if flag { Box::new(|x| x * 2) }
       else    { Box::new(|x| x + 1) }
   }
   ```

---

## Generics and Closures (Brief Reference)

Source: [reference/items/generics.html](https://doc.rust-lang.org/reference/items/generics.html)

Generic functions parameterize over types. Combined with trait bounds, they express "any type
that can do X":

```rust
// T must implement PartialOrd to use > and >=
fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { min }
    else if value > max { max }
    else { value }
}
```

`where` clauses are an alternative syntax for complex bounds:

```rust
fn transform<F, T, U>(items: Vec<T>, f: F) -> Vec<U>
where
    F: Fn(T) -> U,
{
    items.into_iter().map(f).collect()
}
```

**Monomorphization:** The compiler generates a concrete version of the function for each unique
set of type arguments. `apply::<i32>` and `apply::<f64>` are separate functions in the binary.
This is how Rust achieves zero-cost abstractions — generic code compiles to the same machine
code as hand-written specialized code.

---

## Key Standard Library Functions

### `std::mem::drop`

```rust
pub fn drop<T>(_x: T) {}
```

Takes ownership of `_x` and immediately drops it. Used to explicitly end a borrow or release
a resource before the end of its enclosing scope.

### `Option::map`

```rust
pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option<U>
```

Transforms `Option<T>` to `Option<U>` by applying `f` to the contained value.
Uses `FnOnce` — the closure is called at most once (zero times for `None`).

### `Result::map` and `Result::and_then`

```rust
pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result<U, E>
pub fn and_then<U, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<U, E>
```

`map` transforms the success value. `and_then` chains fallible operations.
Both use `FnOnce` — called at most once.

### `Vec::sort_by` and `Vec::sort_by_key`

```rust
pub fn sort_by<F: FnMut(&T, &T) -> Ordering>(&mut self, compare: F)
pub fn sort_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, f: F)
```

Comparison function is `FnMut` because it may be called many times. It should be deterministic
(pure function) for correct sort behavior.
