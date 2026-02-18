# Exercises: Dependency Injection (Go)

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a notification service with injectable dependencies (template renderer, delivery backend, audit logger, rate limiter) |
| Debugging | Available | Four DI-related bugs: circular dependency, typed nil interface, shared mutable state, broken mock |
| Code Review | Available | Review a proposed service with service locator anti-pattern, concrete dependencies, God struct, and missing nil checks |
| Refactoring | N/A | Covered implicitly by code review exercise (refactoring from anti-patterns to proper DI) |
| API Design | N/A | Interface design is covered thoroughly in the standard exercise |
| Codebase Navigation | N/A | Not enough completed modules for codebase nav yet |

## Recommended Order

1. **Standard** -- Build DI from scratch to internalize constructor injection and interface design
2. **Debugging** -- Encounter and fix the subtle bugs that DI introduces (typed nils, circular deps)
3. **Code Review** -- Read existing code critically, spot anti-patterns, and suggest improvements

## Related Concepts

- [[strategy]] -- DI is how strategies are injected into consumers
- [[factory]] -- Factories are often injected dependencies themselves
- [[repository]] -- The canonical DI example in Go
- [[decorator-middleware]] -- Decorators are injected in place of the original dependency
- [[testing-fundamentals]] -- DI is the foundation for isolated unit testing
