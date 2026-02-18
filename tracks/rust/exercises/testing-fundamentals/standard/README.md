# Standard Exercise: Test Suite for a Token Bucket Rate Limiter

## Scenario

Your team has shipped a token bucket rate limiter used by the API gateway to enforce per-client
request quotas. The implementation is done and working — but it has no tests. Before the next
feature sprint begins, you've been asked to write a comprehensive test suite. The goal is not
coverage theater: tests should catch real regressions, document behavior precisely, and make the
clock-dependent logic testable without sleeping.

## Brief

The rate limiter is provided in `starter/rate_limiter.rs`. It uses a `Clock` trait so you can
inject a fake clock in tests. Write the full test suite in `starter/tests.rs`. The reference
implementation's tests are in `solutions/solution_tests.rs`.

## Acceptance Criteria

### 1. Unit tests for core behavior (`#[cfg(test)] mod tests`)

Write tests covering:

- **New limiter**: starts with full token count
- **Allow**: `check()` returns `true` when tokens are available, decrements count
- **Block**: `check()` returns `false` when tokens are exhausted
- **Window reset**: tokens refill after the window elapses
- **Partial refill behavior**: if a window elapses mid-use, the full capacity is restored
- **Edge case**: requesting immediately after the boundary (off-by-one on window math)

### 2. Trait-based clock mock

- Implement `FakeClock` in the test module using `std::cell::Cell<u64>` for interior mutability
- `FakeClock::advance(&self, millis: u64)` must advance time through `&self` (not `&mut self`)
- All time-dependent tests must use `FakeClock` — no sleeping, no `Instant::now()`

### 3. `#[should_panic]` tests with `expected =`

Write `#[should_panic(expected = "...")]` tests for:

- Zero capacity (`RateLimiter::new(0, 1000, clock)`)
- Zero window (`RateLimiter::new(5, 0, clock)`)

Both tests must use the `expected` attribute — bare `#[should_panic]` without a message is
not accepted.

### 4. Result-returning test

Write at least one test that returns `Result<(), Box<dyn std::error::Error>>` and uses `?`
to propagate errors. Scenario: parse a rate limit config string and build a limiter from it.

### 5. Doc test

Add a doc comment to the `RateLimiter` struct in `starter/rate_limiter.rs` that:
- Shows basic usage: construct, call `check()`, assert the return value
- Compiles correctly (no undefined names, no missing imports)
- Uses `# ` to hide boilerplate setup lines from rendered docs

### 6. `#[ignore]` test

Write one test marked `#[ignore]` that is too slow for a normal run. Scenario: simulate
sustained load over many windows and assert total allowed vs blocked counts fall within
expected ratios.

## Constraints

- No external crates beyond `std`
- All non-ignored tests must pass with `rustc --test starter/tests.rs` or `cargo test`
- `FakeClock` must use `Cell<u64>` — not `RefCell`, not `Mutex`, not `AtomicU64`
- `#[should_panic]` tests must have the `expected =` attribute set to a non-empty string
- The doc test in `rate_limiter.rs` must compile and pass (`cargo test --doc` or `rustdoc --test`)

## Hints

<details>
<summary>Hint 1: FakeClock and interior mutability</summary>

`Clock::now_millis` takes `&self`, not `&mut self`. That means your fake can't directly
mutate a `u64` field through `&self`. Use `Cell<u64>`:

```rust
use std::cell::Cell;

struct FakeClock {
    now: Cell<u64>,
}

impl FakeClock {
    fn new(start: u64) -> Self { FakeClock { now: Cell::new(start) } }
    fn advance(&self, millis: u64) { self.now.set(self.now.get() + millis); }
}

impl Clock for FakeClock {
    fn now_millis(&self) -> u64 { self.now.get() }
}
```

</details>

<details>
<summary>Hint 2: Testing window reset</summary>

The FakeClock is moved into the limiter. You can still call `limiter.clock.advance()` after
construction — as long as `Clock::now_millis` takes `&self` (which it does), the advance
method on FakeClock also takes `&self`. The borrow checker is happy because you're not taking
a mutable reference to the limiter itself.

```rust
let mut limiter = RateLimiter::new(2, 1000, FakeClock::new(0), ...);
limiter.check("k"); // t=0
limiter.check("k"); // t=0 — exhausted
assert!(!limiter.check("k")); // still blocked

limiter.clock.advance(1001); // past the window
assert!(limiter.check("k")); // now allowed
```

</details>

<details>
<summary>Hint 3: Doc test with hidden lines</summary>

```rust
/// A token bucket rate limiter.
///
/// ```
/// # use rate_limiter::{RateLimiter, FakeClock};
/// let clock = FakeClock::new(0);
/// let mut limiter = RateLimiter::new(5, 60_000, clock);
/// assert!(limiter.check());   // allowed: 4 tokens remain
/// assert!(limiter.check());   // allowed: 3 tokens remain
/// ```
```

Lines starting with `# ` are compiled but hidden from rendered docs.

</details>

<details>
<summary>Hint 4: Result-returning test pattern</summary>

```rust
#[test]
fn build_from_config_string() -> Result<(), Box<dyn std::error::Error>> {
    let (capacity, window_ms) = parse_rate_config("10/30s")?;
    let mut limiter = RateLimiter::new(capacity, window_ms, FakeClock::new(0));
    assert!(limiter.check());
    Ok(())
}
```

`Box<dyn std::error::Error>` accepts any error type that implements `Error`. If
`parse_rate_config` returns a custom error type, you may need to implement `std::error::Error`
for it (or derive it, or convert with `.map_err(|e| format!("{}", e))`).

</details>
