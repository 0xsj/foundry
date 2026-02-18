# Solution: Event Processor Bugs

## Bug 1: FnMut passed where Fn is expected

### Root Cause

`apply_to_all` declares its parameter as `F: Fn(&str) -> String`, which requires
that the closure can be called with a shared (non-exclusive) reference to itself.
The closure passed to it mutates `last_seen`, which requires exclusive access —
making it `FnMut`, not `Fn`.

```rust
// Bug: Fn bound, but closure mutates last_seen
fn apply_to_all<F: Fn(&str) -> String>(items: &[&str], f: F) -> Vec<String> { ... }

let dedup = |event: &str| {
    last_seen = event.to_string();  // mutation — this is FnMut
    ...
};

apply_to_all(events, dedup); // ERROR: Fn required, FnMut provided
```

### Fix

Change the bound on `apply_to_all` from `Fn` to `FnMut`, and make the parameter
`mut f` so Rust knows it can call `f` mutably:

```rust
fn apply_to_all<F: FnMut(&str) -> String>(items: &[&str], mut f: F) -> Vec<String> {
    items.iter().map(|&s| f(s)).collect()
}
```

`Fn: FnMut: FnOnce` — accepting `FnMut` is strictly more permissive than `Fn`.
Any `Fn` closure can be passed where `FnMut` is expected (not the other way around).

### Lesson

- `Fn` = shared reference to closure (`&self` call). The closure cannot mutate captures.
- `FnMut` = exclusive reference to closure (`&mut self` call). The closure can mutate captures.
- When a closure mutates captured variables, it is `FnMut` but not `Fn`.
- Accept `FnMut` as the bound unless you have a specific reason to require `Fn`
  (e.g., you need to call it from multiple threads simultaneously).

---

## Bug 2: move closure borrows after move

### Root Cause

`prefix` is a `&str` — a borrowed reference into some string data owned elsewhere.
`move || format!("{}.{}", prefix, event)` copies the reference into the closure.
The closure is then returned from `make_prefixer`. But `prefix` points to data
owned by the caller of `make_prefixer`. When the caller's scope ends, the data is
gone — the closure's reference would dangle.

```rust
pub fn make_prefixer(prefix: &str) -> impl Fn(&str) -> String {
    move |event| format!("{}.{}", prefix, event)
    // prefix is a &str reference. The closure captures the reference value,
    // but the referenced data is not owned by the closure.
    // ERROR: `prefix` does not live long enough
}
```

The compiler error mentions lifetime issues: `impl Fn(&str) -> String` in return
position has an implicit lifetime bound that the borrowed `prefix` doesn't satisfy.

### Fix

Convert `prefix` to an owned `String` before capturing it:

```rust
pub fn make_prefixer(prefix: &str) -> impl Fn(&str) -> String {
    let prefix = prefix.to_string();  // owned copy — now prefix: String
    move |event| format!("{}.{}", prefix, event)  // prefix (String) is moved into closure
}
```

Now the closure owns `prefix` — it lives as long as the closure does.

### Lesson

- `move` captures variables by value (copies or moves them into the closure).
- If you `move` a `&str`, you move the reference — a pointer + length. The data it
  points to is not owned by the closure and may be freed before the closure runs.
- When returning a closure from a function, it must own all its captured data.
  Convert borrowed data to owned data (`to_string()`, `to_owned()`, `clone()`) before
  the closure captures it.
- In Go and TypeScript, closures always capture by reference (the GC keeps things alive).
  In Rust, you must ensure the closure owns its data if it can outlive the source.

---

## Bug 3: returning a closure without the correct return type

### Root Cause

`fn() -> u32` is the type of a **function pointer** — a raw code address with no
associated state. Function pointers cannot capture variables. The closure `move || { count += 1; count }`
captures and mutates `count`, making it a closure type — not a function pointer.

```rust
pub fn make_counter() -> fn() -> u32 {
    let mut count = 0u32;
    move || { count += 1; count }
    // ERROR: expected fn pointer `fn() -> u32`, found closure
}
```

### Fix

Return `impl FnMut() -> u32` — the closure is `FnMut` because it mutates `count`:

```rust
pub fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0u32;
    move || {
        count += 1;
        count
    }
}
```

The caller must bind the return value as `mut counter` to call it:
```rust
let mut counter = make_counter();
assert_eq!(counter(), 1);
assert_eq!(counter(), 2);
```

### Lesson

- `fn(T) -> U` is a function pointer — no captures, no state, `Copy`.
- `impl Fn(T) -> U` / `impl FnMut(T) -> U` / `impl FnOnce(T) -> U` are closure return types.
- Non-capturing closures (`|| constant_value`) can coerce to `fn()` — but the moment
  a closure captures anything, it cannot be a function pointer.
- Use `impl FnMut` when the returned closure mutates state. Use `impl Fn` when it only reads.

---

## Bug 4: closure captures reference to local variable

### Root Cause

`formatted` is a `Vec<String>` created inside `format_events`. The closure captures a
reference to it (to index into it later). When `format_events` returns, `formatted` is
dropped — the closure's reference dangles.

Additionally, returning `&str` from inside a returned closure is inherently difficult:
`&str` would need to borrow from `formatted`, but `formatted` would be gone. The
lifetime cannot be expressed in this return type signature without owning the data.

```rust
pub fn format_events(events: &[&str], separator: &str) -> impl Fn(usize) -> &str {
    let formatted: Vec<String> = ...;  // local variable
    move |index| formatted[index].as_str()
    // ERROR: `formatted` does not live long enough — it's dropped at end of function
    // Also: can't return &str that borrows from a local
}
```

### Fix

Return `String` instead of `&str` (the closure produces an owned copy rather than a borrow),
and let the closure own `formatted` via `move`:

```rust
pub fn format_events(events: &[&str], separator: &str) -> impl Fn(usize) -> String {
    let formatted: Vec<String> = events
        .iter()
        .map(|&e| e.to_uppercase())
        .collect();

    move |index| formatted[index].clone()  // formatted owned by closure; clone the String
}
```

Update the tests accordingly: `get_event(0)` returns `String`, not `&str`.

```rust
assert_eq!(get_event(0), "LOGIN");  // String == &str via PartialEq — still works
```

### Lesson

- A closure that is returned from a function cannot borrow local variables from that function.
  Once the function returns, locals are dropped.
- To fix: either (a) make the closure own the data via `move` and return `String` instead of `&str`,
  or (b) require the caller to pass in data with a long enough lifetime (but that changes the API).
- This is the same rule that prevents returning a `&str` reference to a local `String` from a function.
  Closures have the same lifetime constraints as any other code.
- In Go and TypeScript, closures keep a reference to closed-over variables, which are kept alive by
  the runtime. In Rust, you opt into heap-based lifetime extension explicitly (by owning data or using `Arc`).

---

## Summary

| Bug | Category | Concept | Fix |
|-----|----------|---------|-----|
| `FnMut` passed where `Fn` expected | Trait bound mismatch | `Fn` vs `FnMut` hierarchy | Change bound to `FnMut`, add `mut f` parameter |
| `move` closure captures dangling reference | Lifetime / ownership | Move captures reference, not data | `.to_string()` before capture to own the data |
| `fn()` return type for capturing closure | Incorrect return type | `fn` pointer vs `impl FnMut` | Return `impl FnMut() -> u32` |
| Closure borrows local variable that is dropped | Dangling reference | Closures and lifetimes | Own the data with `move`, return `String` not `&str` |

## Related Pitfalls

- [[rust-fn-vs-fnmut-bound]] — choosing the right closure trait bound
- [[rust-move-closure-borrows-reference]] — `move` moves the reference, not the referent
- [[rust-fn-pointer-vs-closure]] — `fn()` is not the same as `impl Fn()`
- [[rust-closure-captures-local]] — closures cannot outlive their captured references
