# Solution: Job Scheduler Bugs

## Bug 1: Unreachable match arm due to wrong arm order in `route_job`

### Root Cause

The catch-all range `1..=10` is placed before the specific ranges. Since Rust
matches arms top-to-bottom and takes the first match, `1..=10` swallows every
valid priority before `8..=10` or `5..=7` can be tried.

```rust
match job.priority {
    1..=10 => "standard-queue",  // BUG: matches priority 9 before the next arm
    8..=10 => "urgent-queue",    // unreachable — always preempted by 1..=10
    5..=7 => "standard-queue",   // unreachable
    1..=4 => "low-queue",        // unreachable
    _ => "invalid",
}
```

The `#[allow(unreachable_patterns)]` on the function suppressed the compiler
warning. That attribute is a major red flag in code review: the compiler is
trying to tell you something is wrong and it is being silenced.

### Fix

Put the most specific (narrowest) ranges first:

```rust
match job.priority {
    8..=10 => "urgent-queue",
    5..=7 => "standard-queue",
    1..=4 => "low-queue",
    _ => "invalid",
}
```

Now `8..=10` is tested before `1..=10` would have matched.

### Lesson

- Match arms are tried in order. More specific patterns must come before more
  general ones.
- Never use `#[allow(unreachable_patterns)]` on a real match — fix the order
  instead. The warning is the compiler catching a logic bug.
- In Go, `switch` cases are also evaluated in order. In TypeScript, `switch`
  fall-through is the more common pitfall, but case ordering matters in the same
  way if you use value ranges via guards.

---

## Bug 2: Clone escapes the mutation in `schedule_retry`

### Root Cause

The function takes `&mut Job` (mutable borrow), but instead of modifying the job
directly, it clones it into `job_copy`, increments `job_copy.retry_count`, checks
`job_copy`, and then returns. The original `job.retry_count` is never touched.

```rust
fn schedule_retry(job: &mut Job) -> Option<u32> {
    let mut job_copy = job.clone();  // separate copy on the heap
    job_copy.retry_count += 1;       // modifies the copy
    // job.retry_count is still 0   — the original is untouched

    if job_copy.retry_count >= 3 {
        None
    } else {
        Some(job_copy.retry_count)  // reports the copy's count, not job's
    }
}
```

After the function returns, `job_copy` is dropped. The caller's `job` is
unmodified, so `job.retry_count` is always 0 from the caller's perspective.

### Fix

Modify the job through the mutable reference directly:

```rust
fn schedule_retry(job: &mut Job) -> Option<u32> {
    job.retry_count += 1;
    if job.retry_count >= 3 {
        None
    } else {
        Some(job.retry_count)
    }
}
```

No clone needed. `job.retry_count += 1` modifies the caller's `Job` in place
through the mutable reference.

### Lesson

- `clone()` creates an independent copy. Mutations to a clone do not affect the
  original — that's the point of cloning.
- `&mut T` gives you the ability to mutate through the reference directly. If you
  clone instead of mutating, you're defeating the purpose of `&mut`.
- This is a subtler cousin of the Go bug where you modify a copy of a struct
  value: `job := *jobPtr; job.Count++ // original unchanged`.

---

## Bug 3: Infinite loop in `drain_queue`

### Root Cause

The loop reads from the queue without removing items. `queue.last()` returns
a reference to the last element without modifying `queue`. Since the queue is
never shortened, `queue.is_empty()` is always `false` for a non-empty input,
and the loop never terminates.

```rust
while !queue.is_empty() {
    let job = queue.last().unwrap();  // peek only — does not remove
    // ... count job ...
    // queue.len() is unchanged
}
```

### Fix

Use `queue.pop()` to remove the last item with each iteration:

```rust
while let Some(job) = queue.pop() {
    if job.priority == 0 {
        skipped += 1;
    } else {
        processed += 1;
    }
}
```

`pop()` removes and returns the last element, so `queue` shrinks by one per
iteration and the `while let` terminates when it returns `None`.

This also demonstrates idiomatic `while let` — the pattern naturally replaces
the `while !is_empty()` + `unwrap` pattern.

Note: `mut queue` in the parameter is needed because `pop()` mutates the vec.
The compiler warned about `mut queue` being unnecessary — that was a hint that
nothing was actually mutating the queue.

### Lesson

- `last()`, `first()`, `get(i)` are read-only. They return references and do not
  modify the collection.
- `pop()`, `remove(i)`, `drain(..)` modify the collection.
- An "unused mut" warning on a collection is often a sign that you meant to use
  a mutating operation but used a non-mutating one instead.
- `while let Some(x) = collection.pop()` is the idiomatic Rust pattern for
  draining a Vec-as-stack.

---

## Bug 4: Wildcard arm before specific arms in `count_outcomes`

### Root Cause

The `_` wildcard arm is placed before `JobOutcome::Cancelled` and
`JobOutcome::Failed(_)`. Since `_` matches everything, `Cancelled` and `Failed`
are never reached.

```rust
match outcome {
    JobOutcome::Processed => processed += 1,
    JobOutcome::Skipped => skipped += 1,
    _ => processed += 1,              // matches Cancelled and Failed too
    JobOutcome::Cancelled => cancelled += 1,  // unreachable
    JobOutcome::Failed(_) => failed += 1,     // unreachable
}
```

The compiler emits "unreachable pattern" warnings for the `Cancelled` and
`Failed` arms. These warnings were not suppressed but would have been visible
during testing.

### Fix

Put specific arms before the wildcard, or better yet — remove the wildcard and
enumerate all variants explicitly:

```rust
match outcome {
    JobOutcome::Processed => processed += 1,
    JobOutcome::Skipped => skipped += 1,
    JobOutcome::Cancelled => cancelled += 1,
    JobOutcome::Failed(_) => failed += 1,
}
```

With all four variants covered, no `_` is needed and the compiler confirms
exhaustiveness.

### Lesson

- The compiler enforces exhaustive matching on enums. If you add a new variant
  to `JobOutcome`, the compiler will force you to handle it everywhere it's matched.
  This is the power of enum-based error handling.
- When you use `_` in a match on an enum, you lose that compile-time guarantee.
  Prefer explicit variant arms unless you truly want to treat many variants
  identically.
- The "unreachable pattern" warning is the compiler pointing directly at the bug.
  Never suppress it with `#[allow(...)]` — fix the underlying problem.

---

## Summary

| Bug | Category | Rust Concept | Prevention |
|-----|----------|-------------|------------|
| Broad range arm before specific arms | Wrong arm order | Match arm evaluation order | Most specific arms first; never use `#[allow(unreachable_patterns)]` |
| Clone instead of mutate through `&mut` | Logic error | `&mut T` semantics, clone vs mutate | Review all `clone()` calls — are you writing back? |
| `last()` instead of `pop()` in loop | Wrong method | Vec API: read vs mutate | "unused mut" warning on collection = hint you forgot mutation |
| Wildcard before specific arms | Wrong arm order | Match arm evaluation order | Explicit variants preferred over `_` for enum matching |

## Related Pitfalls

- [[rust-match-arm-order]] — specific patterns before general patterns
- [[rust-clone-vs-mutate]] — mutating through `&mut` vs modifying a clone
- [[rust-vec-read-vs-mutate]] — peek methods vs consuming methods
- [[rust-unreachable-patterns-warning]] — never suppress this warning
