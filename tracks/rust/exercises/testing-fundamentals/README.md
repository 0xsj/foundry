# Exercises: Testing Fundamentals (Rust)

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Write comprehensive tests for a working token bucket rate limiter: unit tests, trait-based clock mock, doc tests, should_panic tests for invalid config |
| Debugging | Available | Four bugs in test code: reversed assert_eq! arguments, hollow test body, should_panic without expected message, shared mutable state between tests |
| Code Review | Available | PR adding tests to a rate limiter: tests private internals, no doc tests, hardcoded magic values, overly-coupled mock that breaks on refactor |
| Refactoring | N/A | Testing fundamentals is about writing tests, not refactoring production code patterns |
| API Design | N/A | Too early — API design for test infrastructure belongs in the patterns track |
| Codebase Navigation | N/A | Not enough codebase surface area yet; revisit after multiple modules |
