# Exercises: Strategy Pattern

## Overview

These exercises reinforce the Strategy pattern in Rust from three angles: building from scratch, fixing broken implementations, and reviewing proposed code.

All exercises use realistic production scenarios: API rate limiting, payment processing, and async task execution.

## Exercise Index

| Type | Status | Description |
|------|--------|-------------|
| [Standard](./standard/) | Available | Build an API rate limiter with pluggable strategies (token bucket, sliding window, fixed window) |
| [Debugging](./debugging/) | Available | Fix four bugs in a strategy-based task executor: object safety violation, lifetime errors, missing bounds, wrong dispatch |
| [Code Review](./code-review/) | Available | Review a payment processing PR with trait design issues, unnecessary boxing, and anti-patterns |
| Refactoring | N/A | Better suited for later modules with established codebases to refactor |
| API Design | Not yet generated | Could be generated for designing a plugin system API — ask to generate |
| Codebase Navigation | N/A | Requires a larger codebase context |

## Recommended Order

1. **Standard** — Build a rate limiter using trait objects, then optionally refactor to generics
2. **Debugging** — Fix subtle strategy pattern bugs (object safety, lifetimes, dispatch)
3. **Code Review** — Read and critique a payment processing implementation

## Related

- Lesson: `tracks/rust/patterns/strategy/lesson.md`
- Reference: `tracks/rust/patterns/strategy/reference.md`
- Code examples: `notification.rs`, `retry.rs`, `compression.rs`
- Vault: `vault/patterns/strategy.md` (cross-language comparison)
