# Exercises: Strategy Pattern

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a rate limiter with pluggable strategies (token bucket, sliding window, fixed window) for an API gateway |
| Debugging | Available | Find and fix strategy pattern bugs in a rate limiter: missing interface satisfaction, nil strategy, race condition, wrong strategy selection |
| Code Review | Available | Review a payment processing PR with strategy pattern issues: God interface, concrete types, shared state mutation, nil checks |
| Refactoring | N/A | Strategy is a design-time decision — refactoring would be "extract strategy," which is the standard exercise |
| API Design | Not yet generated | Design a pluggable middleware pipeline using the strategy pattern |
| Codebase Navigation | N/A | Requires a real OSS codebase to explore |

## Prerequisites

- Interfaces module (`tracks/go/fundamentals/interfaces-and-traits/`)
- Functions and closures module (`tracks/go/fundamentals/functions-and-closures/`)
- Testing fundamentals module (`tracks/go/fundamentals/testing-fundamentals/`)

## Related

- Strategy pattern lesson: `tracks/go/patterns/strategy/lesson.md`
- Strategy pattern reference: `tracks/go/patterns/strategy/reference.md`
- Code examples: `tracks/go/patterns/strategy/notification/`, `retry/`, `compression/`
