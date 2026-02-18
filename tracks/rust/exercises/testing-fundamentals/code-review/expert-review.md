# Expert Review: Rate Limiter Test Suite PR

## Critical Issues

### 1. Tests target private implementation, not public behavior

**Location:** `evict_expired_removes_old_entries` — calls `l.evict_expired(1001)` and
`l.entry_count()`, both private methods.

```rust
// Proposed:
l.evict_expired(1001);           // private method — not public API
assert_eq!(l.entry_count(), 0); // private method — not public API
```

**Problem:** Private methods are implementation details. They can be renamed, merged,
split, or eliminated during any refactor — even one that leaves behavior identical.
A test that calls private methods breaks on refactor without the behavior being wrong.
This creates maintenance burden and false failures.

**Deeper problem:** The test was written because the author couldn't easily observe the
eviction behavior through the public API. That's the right instinct — but the solution
is not to call private methods. The solution is to observe the behavior through `check()`
and `available_tokens()`, which reflect eviction indirectly.

**Fix:** Test the same invariant through the public interface:

```rust
#[test]
fn window_reset_evicts_old_entries_and_allows_new_requests() {
    let mut l = RateLimiter::new(2, 1_000, FakeClock::new(0));
    l.check(); // t=0 — entry recorded
    l.check(); // t=0 — entry recorded, capacity exhausted

    l.clock.advance(1_001); // past the window
    // If eviction works, both old entries are gone and we can make 2 new requests
    assert!(l.check(), "1st request after window reset should be allowed");
    assert!(l.check(), "2nd request after window reset should be allowed");
    assert!(!l.check(), "3rd request should be blocked — capacity 2");
}
```

**Principle:** Test behavior visible at the boundary of the unit under test (inputs and
outputs of the public API). Test private methods only when they encapsulate complex logic
that cannot be driven through the public API — and even then, consider if the private
method warrants promotion to a testable standalone function.

---

### 2. `HardcodedEntryCountClock` breaks on any refactor of `now_millis()` call sites

**Location:** All three tests using `HardcodedEntryCountClock`

```rust
struct HardcodedEntryCountClock {
    calls: Cell<usize>,
    timestamps: Vec<u64>,
}

impl Clock for HardcodedEntryCountClock {
    fn now_millis(&self) -> u64 {
        let i = self.calls.get();
        self.calls.set(i + 1);
        *self.timestamps.get(i).unwrap_or(&999_999)
    }
}
```

**Problem:** This clock doesn't represent time advancing — it returns a different value
each time `now_millis()` is called based on call count. The timestamps are carefully
chosen to match the exact number of internal `now_millis()` invocations in the current
implementation.

If the implementation is refactored to call `now_millis()` zero times (caching the value)
or three times (for logging), the clock returns the wrong value for the test's intended
scenario and the test fails — even though the behavior is correct.

**Example of the fragility:** The `window_resets_after_elapsed_time` test:
```rust
let clock = HardcodedEntryCountClock::new(vec![0, 0, 1001]);
```
This assumes exactly 3 calls to `now_millis()`, one per `check()`. If the implementation
is optimized to cache the time within a single call, this breaks.

**Fix:** Use a `FakeClock` with explicit time advancement via `advance()`:

```rust
struct FakeClock { now: Cell<u64> }

impl FakeClock {
    fn new(start: u64) -> Self { FakeClock { now: Cell::new(start) } }
    fn advance(&self, millis: u64) { self.now.set(self.now.get() + millis); }
}

impl Clock for FakeClock {
    fn now_millis(&self) -> u64 { self.now.get() }
}

#[test]
fn window_resets_after_elapsed_time() {
    let mut l = RateLimiter::new(1, 1_000, FakeClock::new(0));
    l.check(); // t=0, fills capacity
    assert!(!l.check()); // t=0, blocked

    l.clock.advance(1_001); // explicitly advance — decoupled from call count
    assert!(l.check()); // t=1001, window reset
}
```

**Principle:** Test doubles should model the abstraction (a clock that returns the current
time), not the implementation's calling pattern (the 3rd call returns 1001).

---

## Major Concerns

### 3. No doc test on `RateLimiter`

**Location:** The `RateLimiter` struct definition (production code section)

The struct has no documentation block with a usage example. A new user looking at this
code has no example to start from. Documentation examples are also tests — they catch API
drift automatically.

**Fix:** Add a doc comment to the struct:

```rust
/// A token bucket rate limiter with an injectable clock for testability.
///
/// # Examples
///
/// ```
/// # use std::cell::Cell;
/// # struct FakeClock(Cell<u64>);
/// # impl FakeClock { fn new(t: u64) -> Self { FakeClock(Cell::new(t)) } }
/// # trait Clock { fn now_millis(&self) -> u64; }
/// # impl Clock for FakeClock { fn now_millis(&self) -> u64 { self.0.get() } }
/// // ... (minimal working example showing check() and return values)
/// ```
pub struct RateLimiter<C: Clock> { ... }
```

**Why this matters:** A doc test is the only test that uses the exact public-facing API
in exactly the way a user would. It's documentation that can't drift, and a regression
test that requires no setup.

---

### 4. No `#[should_panic]` tests for the panic contract

**Location:** Missing — nowhere in the test suite

The production code documents two panics: `capacity == 0` and `window_millis == 0`. Both
are verified only by the documentation comment. No test validates that these panics actually
happen with the expected message.

**Problem:** If someone removes the guard (`if capacity == 0 { panic!(...) }`) during a
"cleanup" refactor, no test fails. The behavior regresses silently.

**Fix:**
```rust
#[test]
#[should_panic(expected = "capacity must be > 0")]
fn zero_capacity_panics() {
    RateLimiter::new(0, 1_000, FakeClock::new(0));
}

#[test]
#[should_panic(expected = "window_millis must be > 0")]
fn zero_window_panics() {
    RateLimiter::new(5, 0, FakeClock::new(0));
}
```

Note: `expected = "..."` is required. Bare `#[should_panic]` is too permissive.

---

### 5. Magic numbers in test setup with no explanation

**Location:** `check_allows_five_requests`

```rust
let mut l = RateLimiter::new(5, 60000, HardcodedEntryCountClock::new(
    vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0] // 10 zeros — why 10?
));
```

- Why 60000? Why not 1000 or 86_400_000?
- Why 10 zeros in the vec? The test only calls `check()` 5 times.
- Why 5 capacity for this test specifically?

Magic numbers make tests hard to maintain. When capacity changes, the reader must figure
out whether the hardcoded numbers need updating. When the number of zeros doesn't match
the number of operations, readers wonder if it's intentional.

**Fix:** Use a simple `FakeClock` (no magic vec), explain constants in comments:

```rust
#[test]
fn check_allows_requests_up_to_capacity() {
    let capacity = 5;
    let mut l = RateLimiter::new(capacity, 60_000, FakeClock::new(0));

    for i in 0..capacity {
        assert!(l.check(), "request {} of {} should be allowed", i + 1, capacity);
    }
}
```

---

## Minor Suggestions

### 6. Missing edge case: window boundary

No test covers the exact boundary condition: entry age == `window_millis`. The
implementation evicts when `age >= window_millis` (exclusive on the new side, inclusive
on the old side). This is a meaningful choice that should be tested and documented.

```rust
#[test]
fn entry_is_evicted_when_age_equals_window() {
    let mut l = RateLimiter::new(1, 1_000, FakeClock::new(0));
    l.check(); // t=0, fills capacity

    l.clock.advance(1_000); // age = exactly window_millis
    // Age 1000 >= window_millis 1000 — should be evicted
    assert!(l.check(), "entry at age == window_millis should be evicted");
}
```

---

### 7. No helper function to reduce setup repetition

Each test constructs the limiter inline. Extracting a helper reduces duplication and
makes tests easier to read:

```rust
fn limiter(capacity: u32, window_millis: u64) -> RateLimiter<FakeClock> {
    RateLimiter::new(capacity, window_millis, FakeClock::new(0))
}
```

---

## Positive Feedback

1. **The `cfg(test)` module structure is correct.** Tests are in a `mod tests` block
   inside `#[cfg(test)]`, `use super::*` is present. The skeleton is right.

2. **The test names are descriptive.** `evict_expired_removes_old_entries`,
   `window_resets_after_elapsed_time` — these read as behavior descriptions, not
   implementation names. Good habit.

3. **The author recognized they needed a controllable time source.** Creating a clock
   fake at all (even an imperfect one) shows the right instinct: time-dependent code
   needs an injectable clock. The issue is the design of the fake, not the idea of it.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | Tests call private methods — breaks on refactor | Test via public API only |
| 2 | Critical | `HardcodedEntryCountClock` is coupled to call count — breaks on refactor | Clock fakes should model time, not call sequences |
| 3 | Major | No doc test on `RateLimiter` | Documentation tests as regression protection |
| 4 | Major | No `#[should_panic]` tests for the panic contract | Testing invariant enforcement |
| 5 | Major | Magic numbers in test setup | Clarity and maintainability |
| 6 | Minor | Missing window boundary edge case | Edge case coverage |
| 7 | Minor | No fixture helper — setup repeated inline | DRY in test code |

## Related Concepts

- [[fundamentals/rust/testing-fundamentals]] — The `FakeClock` pattern with `Cell<T>`
- [[fundamentals/testing-fundamentals]] — Black-box vs white-box testing
- [[pitfalls/rust-testing-private-internals]] — Why testing private methods is fragile
- [[pitfalls/rust-should-panic-without-expected]] — Bare `#[should_panic]` is too permissive
