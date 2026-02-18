# Exercises: Decorator / Middleware Pattern (Go)

| Type | Status | Description |
|------|--------|-------------|
| Standard | Available | Build a composable middleware stack for an API gateway |
| Debugging | Available | Find and fix 4 bugs in middleware implementations |
| Code Review | Available | Review a proposed middleware framework for production issues |
| Refactoring | N/A | Covered implicitly by the standard exercise (building clean middleware from scratch) |
| API Design | Not yet generated | Design a middleware-compatible plugin system — ask to generate |
| Codebase Navigation | N/A | Pattern is well-demonstrated in standard library (`net/http`, `io`) |

## Concepts Exercised

All three exercise types reinforce the same core concepts from different angles:

- `func(http.Handler) http.Handler` middleware signature
- `http.HandlerFunc` adapter pattern
- `ResponseWriter` wrapping and its pitfalls
- Middleware composition and execution ordering
- Request cloning before mutation
- Context value patterns (unexported key types)
- Panic recovery in middleware
- Interface satisfaction and optional interface preservation

## Prerequisites

Before starting these exercises, you should be comfortable with:

- Interfaces and implicit satisfaction (`interfaces-and-traits` module)
- First-class functions and closures (`functions-and-closures` module)
- Error handling patterns (`error-handling` module)
- Basic HTTP handler concepts (`net/http` package)
