# Solution: Notification Pipeline Bugs

## Bug 1: Non-Object-Safe Trait (`AlertFormatter`)

### Root Cause

`AlertFormatter` has a generic method:

```rust
fn format_any<T: fmt::Display>(&self, label: &str, value: T) -> String {
    format!("{}: {}", label, value)
}
```

A generic method cannot appear in a vtable because the vtable would need infinitely many
entries — one for every type `T` you might ever call it with. The compiler rejects this
as soon as you try to use `Box<dyn AlertFormatter>`.

```
error[E0038]: the trait `AlertFormatter` cannot be made into an object
note: method `format_any` has generic type parameters
```

The confusing part is that the error often points to `Vec<&dyn AlertFormatter>` or
`Box<dyn AlertFormatter>` rather than the method definition. The *root cause* is the
generic method, even though the error appears elsewhere.

### Fix

Remove the generic method from the trait. Since `format_str` and `format_num` already
cover the concrete types needed (`&str` and `i64`), the generic method was redundant.

Alternatively, exclude it from the vtable with `where Self: Sized`:

```rust
fn format_any<T: fmt::Display>(&self, label: &str, value: T) -> String
where
    Self: Sized,  // excluded from dyn vtable
{
    format!("{}: {}", label, value)
}
```

With `where Self: Sized`, the trait becomes object-safe. The generic method cannot be
called on `dyn AlertFormatter`, but concrete types can still use it.

The simplest fix for this exercise is just removing `format_any` from the trait.

### Lesson

- A trait is object-safe when all methods satisfy: no generic parameters, no `Self`
  in return position (except behind a pointer), no `where Self: Sized` on the method itself.
- The error will often point to `Box<dyn Trait>` usage rather than the problematic method.
  Always look at what trait methods exist, not just where the error appears.
- `where Self: Sized` is the escape hatch for mixing object-safe and non-object-safe methods.

---

## Bug 2: Missing `Display` Implementation for `PagerDutySender`

### Root Cause

Inside `dispatch_notification`:

```rust
let formatted = format!("tags=[{}] parts={}", self.tags, parts);
```

Wait — this line is Bug 3's symptom. But the test explicitly calls `format!("{}", sender)`,
and `PagerDutySender` does not implement `Display`.

```
error[E0277]: `PagerDutySender` doesn't implement `std::fmt::Display`
```

The error points to the test or the format call, not to `PagerDutySender`'s definition.
This is the classic "confusing downstream error" pattern: the root cause is a missing impl,
but the compiler flags the usage site.

### Fix

Implement `Display` for `PagerDutySender`:

```rust
impl fmt::Display for PagerDutySender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PagerDuty({})", self.service_key)
    }
}
```

The test checks that `format!("{}", sender)` contains the service key:

```rust
assert!(display.contains("svc-key-abc123"), ...);
```

### Lesson

- When you see "type X doesn't implement Display," the fix is always on the type definition,
  not at the format! call site. The error points to *where* Display is needed, not *why* it's missing.
- Get in the habit of implementing `Display` for any type that might appear in error messages,
  logs, or user-facing output. `Debug` is for developers; `Display` is for everyone.
- In Go, all types implement `String() string` implicitly if they have the method. In Rust,
  `Display` is an explicit choice — which is why missing it causes hard errors.

---

## Bug 3: Orphan Rule Violation (`impl Display for Vec<String>`)

### Root Cause

```rust
impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.join(", "))
    }
}
```

`fmt::Display` is defined in `std`. `Vec<String>` is defined in `std`. Neither is in your
crate. The orphan rule forbids implementing an external trait for an external type.

```
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

### Fix

Remove the blanket impl. In `dispatch_notification`, replace the usage:

```rust
// Before (using the illegal impl):
let formatted = format!("tags=[{}] parts={}", self.tags, parts);

// After (call .join() directly — no impl needed):
let formatted = format!("tags=[{}] parts=[{}]", self.tags.join(", "), parts.join(", "));
```

Or, if you need a reusable formatting helper for `Vec<String>`, create a newtype:

```rust
struct StringList(Vec<String>);

impl fmt::Display for StringList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(", "))
    }
}
```

This works because `StringList` is your type.

### Lesson

- The orphan rule: you can implement a trait for a type only if you own the trait OR the type.
- The most common violation: trying to add standard library traits (Display, Iterator, From)
  to standard library types (Vec, String, Option). Always use a newtype instead.
- The orphan rule exists to prevent ecosystem-wide conflicts. Without it, two crates could
  both implement `Display for Vec<String>` with contradictory output. The compiler would have
  no way to choose. The orphan rule guarantees uniqueness globally.

---

## Bug 4: `&dyn AlertFormatter` Instead of `Box<dyn AlertFormatter>`

### Root Cause

```rust
struct AlertRegistry {
    formatters: Vec<&dyn AlertFormatter>,  // BUG: borrowed reference
    ...
}

fn add_formatter(&mut self, formatter: &dyn AlertFormatter) {
    self.formatters.push(formatter);
}
```

`&dyn AlertFormatter` is a borrow — it doesn't own the formatter. Storing it in a struct
means the registry can only live as long as every borrowed formatter exists. The borrow
checker requires a lifetime parameter on the struct:

```
error[E0106]: missing lifetime specifier
```

And even if you added a lifetime (`struct AlertRegistry<'a> { formatters: Vec<&'a dyn AlertFormatter> }`),
it makes the registry impractical — you could never return it from a function or store it
in another struct without threading lifetimes through everything.

The real issue is ownership: the registry should *own* its formatters, not borrow them.

### Fix

Change `&dyn AlertFormatter` to `Box<dyn AlertFormatter>` throughout:

```rust
struct AlertRegistry {
    formatters: Vec<Box<dyn AlertFormatter>>,
    ...
}

fn add_formatter(&mut self, formatter: Box<dyn AlertFormatter>) {
    self.formatters.push(formatter);
}
```

Callers wrap concrete types in `Box::new(...)`:

```rust
registry.add_formatter(Box::new(PlainFormatter));
registry.add_formatter(Box::new(BracketFormatter));
```

`Box<dyn AlertFormatter>` is an owned fat pointer. It allocates the formatter on the heap and
the registry holds full ownership. When the registry drops, the formatters drop too.

### When to Use `&dyn` vs `Box<dyn>`

| | `&dyn Trait` | `Box<dyn Trait>` |
|--|---|---|
| Ownership | Borrows — caller owns | Takes ownership |
| Lifetime | Tied to caller's scope | Independent |
| Storing in structs | Requires lifetime parameters | Clean, no lifetime needed |
| Returning from functions | Requires named lifetime | Fine — heap-allocated |
| Use when | Passing to a function temporarily | Storing, returning, owning |

### Lesson

- `&dyn Trait` is for temporary, short-lived borrows: "I'll call a method on this thing,
  then give it back." You cannot store it without lifetime parameters.
- `Box<dyn Trait>` is for ownership: "I'm taking this thing and will manage its lifetime."
  This is almost always what you want when storing trait objects.
- The borrow checker's error for missing lifetimes on `&dyn` in a struct often feels
  cryptic — but the fix is almost always to switch to `Box<dyn>`.

---

## Summary

| Bug | Category | Error Symptom | Fix |
|-----|----------|--------------|-----|
| Generic method in trait | Object safety | `cannot be made into an object` | Remove or mark `where Self: Sized` |
| Missing Display on PagerDutySender | Missing trait impl | `doesn't implement Display` | `impl fmt::Display for PagerDutySender` |
| `impl Display for Vec<String>` | Orphan rule | `only traits defined in current crate...` | Remove impl; use `.join()` directly |
| `&dyn` in struct instead of `Box<dyn>` | Ownership/lifetimes | `missing lifetime specifier` | Change to `Box<dyn AlertFormatter>` |

## Related Pitfalls

- [[rust-object-safety]] — when dyn Trait requires removing or marking methods
- [[rust-orphan-rule]] — implementing external traits on external types
- [[rust-dyn-vs-ref-dyn]] — when to use Box<dyn> vs &dyn in structs
- [[rust-missing-display]] — confusing downstream errors from missing Display impls
