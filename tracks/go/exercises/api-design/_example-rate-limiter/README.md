# Example: API Design Exercise — Rate Limiter

This is an **example structure** for API design exercises. Use `vault/templates/api-design-exercise.md` as a template.

## Overview

**Scenario:** You're building a rate limiting library for HTTP servers. It needs to support multiple strategies (token bucket, sliding window, fixed window) and be easy to integrate into existing middleware chains.

**Your Task:** Design the public API/interface for this rate limiter library.

---

## Files in this directory

```
api-design/_example-rate-limiter/
├── README.md              # This file — requirements and constraints
├── my-design/             # Your API design
│   ├── ratelimiter.go     # The public interface
│   ├── examples.go        # Usage examples
│   └── design-notes.md    # Your design decisions
├── designs/               # Alternative valid approaches
│   ├── functional-opts/   # Functional options pattern
│   ├── builder/           # Builder pattern
│   └── fluent/            # Fluent interface
└── evolution.md           # How each design evolves over time
```

---

## Requirements

### Functional
- [ ] Support token bucket, sliding window, fixed window strategies
- [ ] Allow per-IP, per-user, per-endpoint rate limiting
- [ ] Configurable limits and time windows
- [ ] Thread-safe (concurrent requests)
- [ ] Provide clear feedback when limit exceeded

### Non-Functional
- [ ] Ergonomic — easy to integrate into existing middleware
- [ ] Hard to misuse — invalid configs should be caught at compile time
- [ ] Extensible — easy to add new strategies
- [ ] Testable — easy to mock/test without real time delays

### Constraints
- Must work with standard http.Handler
- No global state
- Memory-efficient (don't store unlimited request history)

---

## Creating a New API Design Exercise

1. **Choose a realistic problem** — something you'd actually build at work
2. **Write clear requirements** (functional, non-functional, constraints)
3. **Create 2-3 alternative designs** showing trade-offs
4. **Include usage examples** for common scenarios
5. **Discuss evolution** — how would you add features without breaking changes?

---

## Good API Design Topics

### Library Interfaces
- Rate limiter
- Cache implementation
- Config loader
- Logger
- Retry mechanism
- Circuit breaker
- Connection pool

### Service APIs
- REST API for resource management
- GraphQL schema design
- gRPC service definition
- WebSocket protocol

### Internal APIs
- Plugin system
- Middleware chain
- Event bus
- Job queue

---

## Design Considerations

### Usability
- Can users accomplish common tasks in < 5 lines of code?
- Are the defaults sensible?
- Is it hard to misuse?
- Do function names clearly indicate what they do?

### Clarity
- Are parameter names descriptive?
- Is the structure intuitive?
- Are errors informative?
- Is the responsibility of each type clear?

### Flexibility
- Can the API evolve without breaking changes?
- Can users customize behavior when needed?
- Are extension points obvious?

### Safety
- Are error conditions represented in types?
- Can invalid states be represented?
- Are resources cleaned up properly?
- Is it thread-safe where needed?

---

## Example Design Patterns

### 1. Functional Options (Go)
```go
limiter := ratelimit.New(
    ratelimit.PerIP(),
    ratelimit.TokenBucket(100, time.Minute),
    ratelimit.OnExceeded(func(key string) { log.Warn("exceeded", key) }),
)
```

### 2. Builder
```go
limiter := ratelimit.NewBuilder().
    Strategy(ratelimit.TokenBucket).
    Limit(100).
    Window(time.Minute).
    KeyFunc(extractIP).
    Build()
```

### 3. Struct + Methods
```go
limiter := &ratelimit.Limiter{
    Strategy: ratelimit.TokenBucket,
    Limit:    100,
    Window:   time.Minute,
}
```

---

## Evolution Planning

Good APIs anticipate change. For each design, consider:

1. **Adding a new strategy** — How easy is it to add "leaky bucket"?
2. **Adding configuration** — How do you add "burst size" without breaking existing code?
3. **Adding observability** — How do you add metrics/tracing?
4. **Deprecating features** — How do you phase out the old "fixed window" implementation?

---

## Deliverables

When completing an API design exercise:

1. **Interface definition** — The public API with documentation comments
2. **Usage examples** — Show common use cases (3-5 examples)
3. **Error handling strategy** — How are errors represented?
4. **Alternative approaches** — At least 2 other ways to design it, with pros/cons
5. **Design rationale** — Why did you choose this approach?
6. **Evolution plan** — How would you add features in the future?
