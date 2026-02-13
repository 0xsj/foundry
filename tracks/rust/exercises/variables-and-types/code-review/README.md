# Code Review Exercise: In-Memory Cache

## PR Context

A teammate submitted a pull request for an in-memory key-value cache with TTL
(time-to-live) expiration support. The cache maps string keys to string values,
each entry has an expiration timestamp, and expired entries are cleaned up on access.

The code compiles (with one intentional workaround noted in a comment) and basic
operations work. But there are issues ranging from crash-on-bad-input to
unnecessary allocations to broken encapsulation.

## Your Task

Review `proposed.rs` as if this were a real PR. Identify issues at three levels:

1. **Critical** -- Bugs, panics, or correctness problems
2. **Major** -- Design issues, unnecessary allocations, wrong abstractions
3. **Minor** -- Style, idiom, readability

Write your review in `my-review.md`, then compare with `expert-review.md`.

## How to Approach

1. Read through `proposed.rs` carefully -- do not run it first
2. For each issue you find, note:
   - The line(s) involved
   - What the problem is
   - What the fix should be
   - What Rust concept is relevant
3. Also note what was done well

## Concepts Tested

- `unwrap()` safety and error handling with `Result`
- Ownership: `String` vs `&str` in function parameters
- Lifetimes: returning references to local data
- `Clone` trait: derived vs manual implementations
- Pattern matching: `match` vs `if let`
- Encapsulation: `pub` visibility
