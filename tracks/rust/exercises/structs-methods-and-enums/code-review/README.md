# Code Review Exercise: Typed Event Bus

## Context

A teammate opened a PR adding a typed event bus to your platform's plugin system.
The event bus lets plugins publish and subscribe to strongly-typed events. It uses
enums to represent event types and structs to hold subscribers and their state.

The code compiles and the tests pass, but there are design issues visible in the
implementation.

## Your Task

Review `proposed.rs` as if it were a real PR. Look for:

1. **Pub field exposure** — fields that should be private with accessor methods
2. **Missing `Default` implementation** — where a sensible zero-value could be derived
3. **Newtype opportunities** — bare primitives that would benefit from wrapper types
4. **`match` vs `if let`** — verbose match expressions with an empty arm
5. **Unnecessary `.clone()` calls** — copying data that could be borrowed

Record your findings in `my-review.md` using the Critical / Major / Minor structure.

## Review Checklist

Work through the code in order:

- [ ] Struct definitions — visibility, field types
- [ ] Constructors — `new()`, missing `Default`
- [ ] Methods — correct receiver, efficient borrows
- [ ] `match` expressions — are they the right construct?
- [ ] Derive macros — anything missing or unnecessary?

## How to Run the Code

```
rustc proposed.rs && ./proposed
```
