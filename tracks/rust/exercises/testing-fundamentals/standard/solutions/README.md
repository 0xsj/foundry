# Solution Notes: Testing Fundamentals Standard Exercise

## Approach

The solution writes tests against the `RateLimiter<C: Clock>` struct, exploiting its
injectable clock to make all time-dependent behavior deterministic. No sleeping. No
`Instant::now()`. Every test is hermetic.

## Key Decisions

### 1. `FakeClock` uses `Cell<u64>` not `RefCell<u64>`

`Cell<T>` requires `T: Copy`. `u64` is `Copy`. `Cell` has zero overhead (no borrow
tracking) and lets us call `advance(&self, ...)` without taking a mutable reference to
the limiter — essential because the clock is owned by the limiter.

`RefCell<u64>` would also work but adds runtime borrow checking overhead and requires
`.borrow_mut()` syntax. `Cell` is simpler for this case.

`AtomicU64` would work for thread-safe scenarios but we don't need it here.

### 2. Fixture helper `fn limiter(capacity, window_millis) -> RateLimiter<FakeClock>`

Eliminates repetition of `RateLimiter::new(capacity, window_millis, FakeClock::new(0))`
across every test. Small investment, large readability gain. Named after the thing it
returns, not after what it does — idiomatic Rust test setup.

### 3. `#[should_panic(expected = "...")]` always uses the `expected` attribute

Bare `#[should_panic]` is a trap: it catches any panic, including bugs that happen to
panic before reaching the line under test. With `expected`, the panic message is
validated. This is the difference between testing a specific invariant and testing
"something panics somewhere."

### 4. Result-returning tests for the config parser

`parse_rate_config` returns `Result`. Using `?` in the test body means: if parsing fails,
the test fails with a clear error message (not a panic-wrapping-an-error). Much more
diagnostic than `.unwrap()`.

### 5. `#[ignore]` for the sustained load test

The load test doesn't simulate real time (the fake clock is instant), so it's not truly
slow. It's marked `#[ignore]` anyway to demonstrate the pattern and to avoid noisy output
in CI where 200 iterations of the inner loop print nothing but do allocate.

## Variants

### Comparison table

| Variant | Trade-off |
|---|---|
| Solution (FakeClock via generics) | Zero-cost, compile-time, verbose struct signature |
| `Box<dyn Clock>` (not shown) | Dynamic dispatch, simpler struct, heap allocation per use |
| Shared `Arc<AtomicU64>` clock | Thread-safe but unnecessary here — adds complexity |

## Performance Notes

None needed here — this is about test structure, not hot paths. The rate limiter itself
uses `VecDeque::retain` (O(n) on entries in window) which is acceptable for the traffic
volumes a single-node limiter would see.

## Related Concepts

- [[fundamentals/rust/testing-fundamentals]] — Rust-specific deep dive
- [[fundamentals/testing-fundamentals]] — Cross-language comparison
- [[pitfalls/rust-should-panic-without-expected]] — Why bare should_panic is dangerous
- [[patterns/dependency-injection]] — The Clock trait injection pattern used here
