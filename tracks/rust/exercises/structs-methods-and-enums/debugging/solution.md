# Solution: Event Processor Bugs

## Bug 1: Missing `#[derive(Clone)]` on `EventRecord`

### Root Cause

`EventRecord` does not derive `Clone`. The test `test_is_retriable_fresh_record` calls
`record.is_retriable()` and then accesses `record.retries`. If `is_retriable` takes `self`
(Bug 2), the record is moved and `record.retries` is invalid. Even after fixing Bug 2 to
use `&self`, the `Clone` derive is needed whenever the caller wants to explicitly duplicate
a record.

More broadly: any struct that may need to be duplicated (for retry logic, for storing a
copy before mutation, for passing to multiple consumers) should derive `Clone` unless
there's a specific reason not to.

### Fix

```rust
#[derive(Debug, Clone)]  // add Clone
struct EventRecord {
    id: u64,
    kind: String,
    payload: String,
    retries: u32,
}
```

### Lesson

- If all fields implement `Clone` (which `String` and `u32` do), you can always `#[derive(Clone)]`.
- Omitting `Clone` is an intentional design choice meaning "this value should not be copied."
  For domain objects like event records that you'll want to retry, clone, or store — derive it.
- `Copy` requires `Clone` plus all fields being `Copy`. `String` is not `Copy`, so `EventRecord`
  can never be `Copy`. `Clone` (explicit duplication) is the appropriate choice.

---

## Bug 2: `is_retriable` takes `self` instead of `&self`

### Root Cause

```rust
// BUG: takes ownership
fn is_retriable(self) -> bool {
    self.retries < 3
}
```

A method that only reads a field should borrow `self` (`&self`), not consume it. With `self`,
calling `record.is_retriable()` moves the record into the method. The record is dropped at the
end of `is_retriable`, and the caller can no longer use it.

This is why the test fails to compile:

```
assert!(record.is_retriable());
assert_eq!(record.retries, 0);  // compile error: use of moved value: `record`
```

### Fix

```rust
fn is_retriable(&self) -> bool {  // borrow, not consume
    self.retries < 3
}
```

### When to Use Each Receiver

| Receiver | Use when |
|---|---|
| `&self` | Reading fields, computing derived values, checking state |
| `&mut self` | Modifying fields |
| `self` | Converting to another type, terminal operations (the record should be "used up") |

Checking a condition (`retries < 3`) is clearly a read — `&self` is correct.

### Lesson

- A method that does not modify the struct and does not convert it to another type should
  almost always take `&self`.
- If you see a method signature like `fn name(self)` returning a bool or a simple derived
  value, that's a red flag. The consuming receiver makes the function one-shot: you call it
  once and lose the value.
- In Go, methods on pointer receivers (`func (r *Record) IsRetriable() bool`) are the norm
  for structs. In Rust, the equivalent is `&self` (shared borrow) or `&mut self` (mutable borrow).

---

## Bug 3: Wildcard arm shadows a specific arm in `classify`

### Root Cause

In a `match` expression, arms are checked in declaration order. The first arm that matches
wins. When `_ => "unknown"` appears before `"health.ping" => "system"`, the wildcard matches
everything — including `"health.ping"`. The `"health.ping"` arm is unreachable.

```rust
fn classify(&self) -> &str {
    match self.kind.as_str() {
        "user.created" => "identity",
        "user.deleted" => "identity",
        "order.placed" => "commerce",
        "order.cancelled" => "commerce",
        "payment.failed" => "commerce",
        _ => "unknown",         // BUG: this matches everything below
        "health.ping" => "system",  // never reached
    }
}
```

The compiler emits a warning for this (`unreachable_patterns`) but does not treat it as an
error. The code compiles and runs — it just silently returns `"unknown"` for `"health.ping"`.

### Fix

Move the wildcard arm to the end:

```rust
fn classify(&self) -> &str {
    match self.kind.as_str() {
        "user.created" | "user.deleted" => "identity",
        "order.placed" | "order.cancelled" | "payment.failed" => "commerce",
        "health.ping" => "system",
        _ => "unknown",  // wildcard must be last
    }
}
```

Or-patterns (`|`) are also a nice cleanup here.

### Lesson

- In `match`, **more specific patterns must come before wildcard patterns**.
- The compiler warns (`unreachable_patterns`) — enable `#[deny(unreachable_patterns)]` or run
  Clippy to catch this as an error in CI.
- This is easy to introduce when refactoring: you add a new case but forget to put it before
  the existing wildcard. Treating the warning as an error prevents this.
- This cannot happen in TypeScript's `switch` because TypeScript doesn't enforce exhaustiveness
  unless you use a union type with a never check. Rust makes exhaustiveness the default.

---

## Bug 4: Missing `PoisonPill` variant in `EventOutcome`

### Root Cause

The test `test_process_poison_pill` expects `EventOutcome::PoisonPill { record_id: 99 }`,
but `EventOutcome` has no such variant. This is a compile error (the pattern doesn't exist).

Additionally, `process_one` returns `EventOutcome::Rejected` for poison pills rather than the
new variant. And `outcome_record_id` and `partition_outcomes` have non-exhaustive match
expressions — once the new variant is added, the compiler will flag them.

### Fix

Add the variant to the enum:

```rust
#[derive(Debug, Clone, PartialEq)]
enum EventOutcome {
    Accepted { record_id: u64 },
    Rejected { record_id: u64, reason: String },
    Deferred { record_id: u64, delay_ms: u64 },
    PoisonPill { record_id: u64 },  // new
}
```

Update `process_one` to return the new variant:

```rust
if record.payload == "POISON" {
    return EventOutcome::PoisonPill { record_id: record.id };
}
```

Update `outcome_record_id` to handle it:

```rust
fn outcome_record_id(outcome: &EventOutcome) -> u64 {
    match outcome {
        EventOutcome::Accepted { record_id } => *record_id,
        EventOutcome::Rejected { record_id, .. } => *record_id,
        EventOutcome::Deferred { record_id, .. } => *record_id,
        EventOutcome::PoisonPill { record_id } => *record_id,
    }
}
```

Update `partition_outcomes` (poison pills go in rejected, or add a fourth bucket):

```rust
EventOutcome::PoisonPill { record_id } => rejected.push(record_id),
```

### Why This Is the Most Valuable Bug

Non-exhaustive matches are the most common source of correctness bugs when extending enums.
The workflow is:
1. Add a new variant
2. Recompile
3. The compiler tells you every match expression that doesn't handle it
4. Fix them all

This is Rust's exhaustiveness checking working as designed. In Go, the equivalent with
interface type switches gives no compile-time guarantee. In TypeScript with discriminated
unions, you get a compile error only if you use the `never` trick. In Rust, you always get it.

### Lesson

- Adding a variant to an enum forces you to update every `match` — the compiler won't let
  you forget. This is the primary reason to prefer enums over strings or integers for
  categorical data.
- Design your enums to be as specific as possible. `PoisonPill` as a distinct variant (rather
  than `Rejected { reason: "poison pill" }`) makes the type system enforce that callers handle
  it as a distinct case — they can't accidentally treat it like a normal rejection.

---

## Summary

| Bug | Category | Receiver / Concept | Prevention |
|-----|----------|--------------------|------------|
| Missing `Clone` derive | Missing trait | `Clone` vs `Copy` | Derive `Clone` for any struct that may need duplication |
| `is_retriable(self)` consuming | Wrong receiver | `self` vs `&self` | Read-only methods always take `&self` |
| Wildcard before specific arm | Unreachable pattern | match arm ordering | Enable `#[deny(unreachable_patterns)]` |
| Missing enum variant | Non-exhaustive match | Exhaustiveness | Add variants deliberately; let the compiler guide updates |

## Related Pitfalls

- [[rust-match-arm-ordering]] — wildcard placement in match expressions
- [[rust-self-receiver-semantics]] — when to use self vs &self vs &mut self
- [[rust-derive-missing-clone]] — when Clone is required
- [[rust-enum-exhaustiveness]] — using exhaustive match to prevent silent misses
