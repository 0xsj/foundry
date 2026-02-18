# Code Review Exercise: Retry/Backoff Utility

## PR Context

A teammate has opened a PR adding a generic retry-with-backoff utility to the platform
shared library. The utility is meant to be used across multiple services to wrap fallible
operations (HTTP calls, database queries, file I/O) with automatic retry logic.

The code compiles and the tests pass. But there are design and performance issues that
a senior Rust engineer would catch in review. Your job is to identify them.

## What to Review

Read `proposed.rs` carefully. Focus on:

1. **Closure trait bounds** — are the bounds (`Fn`, `FnMut`, `FnOnce`) correct and minimal?
2. **Unnecessary allocations** — are there `Box<dyn Fn>` usages where `impl Fn` would suffice?
3. **Move vs borrow** — are captures appropriate? Is anything moved unnecessarily?
4. **Return types** — are closures returned with the right type?
5. **API ergonomics** — is the API easy to use correctly? Hard to misuse?

## Instructions

1. Read `proposed.rs` from top to bottom
2. Fill in `my-review.md` with your findings (critical issues first, then major, then minor)
3. Check your review against `expert-review.md`

## What Not To Do

- Don't just list style nits
- Don't rewrite the whole thing — identify specific issues with concrete explanations
- Focus on Rust-specific concerns (this is not a generic code quality review)
