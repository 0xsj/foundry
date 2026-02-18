# Exercises: Builder Pattern

Practice the Builder pattern through different exercise types. Each reinforces the same concepts from a different angle: building, fixing, and reading.

## Exercise Types

| Type | Status | Description |
|------|--------|-------------|
| [Standard](#standard) | Available | Build an HTTP request builder for an API client SDK |
| [Debugging](#debugging) | Available | Fix 4 builder pattern bugs (shared state, value receivers, missing validation, race condition) |
| [Code Review](#code-review) | Available | Review a PR that implements a connection pool builder |
| Refactoring | N/A | Builder pattern is already the refactored form of complex constructors |
| API Design | Not yet generated | Design a configuration DSL using functional options (ask to generate) |

## Standard

**Location:** `standard/`

Build an HTTP request builder for an API client SDK. Support both functional options (for the client) and fluent builder (for requests). The builder constructs requests with method, URL, headers, query params, body, authentication, timeout, and retry config.

**Skills practiced:**
- Functional options pattern (client configuration)
- Fluent builder pattern (request construction)
- Validation at Build() time
- Composable option presets
- Error handling in builders

## Debugging

**Location:** `debugging/`

Fix 4 bugs in builder pattern implementations. Each bug maps to a common real-world mistake:
1. Functional option that mutates shared default config
2. Builder method with value receiver (chaining silently broken)
3. Build() that doesn't validate required fields
4. Race condition in concurrent builder usage

**Skills practiced:**
- Understanding pointer vs value semantics in builders
- Detecting shared mutable state
- Recognizing missing validation
- Identifying concurrency bugs

## Code Review

**Location:** `code-review/`

Review a PR that implements a connection pool builder. The code works for basic cases but has several design and correctness issues to identify.

**Skills practiced:**
- Spotting encapsulation violations (exposed internal state)
- Identifying missing required vs optional parameter distinction
- Recognizing silent zero-value defaults
- Finding side effects in option functions
- Evaluating builder API design
