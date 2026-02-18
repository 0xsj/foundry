# Debugging Solution: Broken Test Suite

## Bug 1: `assert_eq!` with the same expression on both sides

**Location:** `consuming_tokens_decrements_available_count`

**The bug:**
```rust
assert_eq!(l.available_tokens(), l.available_tokens());
```

Both arguments are identical calls to `l.available_tokens()`. This always passes — you're
asserting that a value equals itself. The test name says "consuming tokens decrements
available count" but the test verifies nothing about the expected token count.

**The fix:**
```rust
assert_eq!(l.available_tokens(), 3, "after 2 checks from capacity 5, 3 tokens should remain");
```

**Why this happens:** Copy-paste error, or the author was confused about which value to
compare against. The correct `assert_eq!` is `assert_eq!(actual, expected)`. One argument
should be the thing you computed; the other should be the constant you expect.

**How to catch it:** Any time both `assert_eq!` arguments are the same expression, a linter
should flag it (Clippy's `eq_op` lint). In code review, if both arguments look identical,
ask "what's the expected value?"

---

## Bug 2: Empty test body

**Location:** `requests_beyond_capacity_are_blocked`

**The bug:**
```rust
#[test]
fn requests_beyond_capacity_are_blocked() {
    // body is empty
}
```

An empty test always passes. The test name describes behavior (blocking beyond capacity)
that is never verified. This is a hollow test — it gives a false sense of coverage.

**The fix:**
```rust
#[test]
fn requests_beyond_capacity_are_blocked() {
    let mut l = limiter(2, 60_000);
    l.check(); // 1
    l.check(); // 2 — capacity exhausted
    assert!(!l.check(), "3rd request should be blocked (capacity=2)");
}
```

**Why this happens:** Usually a test was scaffolded with the intent to fill it in later,
then forgotten. Or the test body was accidentally deleted.

**How to catch it:** Static analysis or a linter that detects tests without assertions.
Clippy does not catch this by default, but `#[deny(clippy::assertions_on_constants)]`
catches trivially-true assertions. Empty test bodies are harder to detect automatically —
code review is the main defense.

---

## Bug 3: `#[should_panic]` without `expected`

**Location:** `zero_capacity_panics_with_correct_message`

**The bug:**
```rust
#[test]
#[should_panic]
fn zero_capacity_panics_with_correct_message() {
    RateLimiter::new(0, 1_000, FakeClock::new(0));
}
```

The test name says "with correct message" but the bare `#[should_panic]` attribute accepts
any panic — from any cause. If the implementation changed to panic with "invalid argument"
or panicked due to an unrelated bug earlier in the function, this test would still pass.

**The fix:**
```rust
#[test]
#[should_panic(expected = "capacity must be > 0")]
fn zero_capacity_panics_with_correct_message() {
    RateLimiter::new(0, 1_000, FakeClock::new(0));
}
```

`expected = "..."` is a substring match. The test passes only if the panic message
contains exactly "capacity must be > 0". Any other panic fails the test.

**Rule of thumb:** Bare `#[should_panic]` is almost never the right choice. Always pin the
test to the expected message. The only exception is when you explicitly don't care about the
message content — but if that's the case, make it clear in a comment.

---

## Bug 4: Shared mutable `static mut` between parallel tests

**Location:** `check_returns_true_for_first_request`

**The bug:**
```rust
static mut CALL_COUNT: u32 = 0;

#[test]
fn check_returns_true_for_first_request() {
    unsafe { CALL_COUNT += 1; }
    let mut l = limiter(5, 60_000);
    let result = l.check();
    assert!(result, "first check() should return true");
    unsafe { assert!(CALL_COUNT > 0, "call count should be positive"); }
}
```

`CALL_COUNT` is a `static mut` — shared across all threads. Rust's test runner runs tests
in parallel by default (multiple threads). Multiple tests mutating `CALL_COUNT` simultaneously
is a data race: undefined behavior in Rust (hence `unsafe`). The second assertion is also
wrong: `CALL_COUNT > 0` is vacuously true after any test that increments it.

**The fix:** Remove the shared state entirely. Each test should be self-contained.

```rust
#[test]
fn check_returns_true_for_first_request() {
    let mut l = limiter(5, 60_000);
    assert!(l.check(), "first check() should return true");
}
```

The original `CALL_COUNT` assertion ("call count should be positive") doesn't test the
rate limiter at all — it tests the test infrastructure. That's the deeper problem: the
test was doing two things at once. The fix collapses it to the one thing that matters.

**How to avoid:** Never use `static mut` in tests. If you need state shared across multiple
tests (rare), use `std::sync::OnceLock` or `std::sync::LazyLock` for initialization, or
run tests serially with `-- --test-threads=1`.

---

## Summary

| Bug | Type | Consequence | Detection |
|-----|------|-------------|-----------|
| `assert_eq!(x, x)` | Hollow assertion | Always passes even with regression | Clippy `eq_op`, code review |
| Empty test body | Hollow test | Always passes, zero coverage | Code review, mutation testing |
| `#[should_panic]` without `expected` | Too permissive | Wrong panics pass | Code review, style guide |
| `static mut` shared across tests | Data race | Intermittent failures, UB | Rust compiler warning, TSAN |

## Related Pitfalls

- [[pitfalls/rust-should-panic-without-expected]] — Why bare should_panic is dangerous
- [[pitfalls/rust-shared-state-in-parallel-tests]] — Data races in test suites
- [[pitfalls/assert-eq-same-expression]] — Hollow assertions
