# Code Review Exercise: Rate Limiter Test Suite PR

## Scenario

A PR has been opened adding a test suite to the production rate limiter. The author is
solid on Rust syntax but newer to testing discipline. The tests compile and pass — but
passing tests are not the same as good tests.

Your job: review `proposed.rs` as if this were a real PR. Identify issues across all
severity levels — things that would cause a regression to silently pass, things that
make the tests fragile, and things that are just noise.

## PR Context

- **PR title:** "Add test coverage for RateLimiter"
- **Author:** junior engineer, 3 months in
- **CI:** passing (all tests pass, no clippy errors)
- **Reviewer:** you

The rate limiter public API:
- `RateLimiter::new(capacity: u32, window_millis: u64, clock: C) -> Self`
- `limiter.check() -> bool`
- `limiter.available_tokens() -> usize`
- `limiter.capacity() -> u32`
- `limiter.window_millis() -> u64`
- `Clock` trait: `fn now_millis(&self) -> u64`

## What to Review

Look for:
1. Issues that would let real bugs through undetected
2. Tests that test the wrong layer (implementation vs behavior)
3. Missing test scenarios that the suite should cover
4. Documentation that's absent or could help downstream users
5. Structural issues that make tests brittle or hard to maintain

Fill in `my-review.md` before reading `expert-review.md`.

## Reviewing Process

1. Read `proposed.rs` fully before writing any comments
2. Note the line numbers of issues as you find them
3. Categorize: Critical / Major / Minor
4. Draft your review in `my-review.md`
5. Compare to `expert-review.md`
