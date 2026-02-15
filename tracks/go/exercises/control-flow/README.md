# Exercises: Control Flow

Practice control flow concepts through different exercise types. Each reinforces the same concepts from a different angle: building, fixing, and reading.

## Exercise Types

| Type | Status | Description |
|------|--------|-------------|
| [Standard](#standard) | Available | Build a retry mechanism with exponential backoff |
| [Debugging](#debugging) | Available | Fix control flow bugs in a configuration validator |
| [Code Review](#code-review) | Available | Review a PR with control flow anti-patterns |
| Refactoring | N/A | Too early in curriculum (patterns not yet covered) |
| API Design | N/A | Not applicable for fundamental control flow |

## Standard

**Location:** `standard/`

Build a retry mechanism from scratch that uses loops, conditionals, and early returns. Realistic scenario: retrying failed HTTP requests with backoff.

**Skills practiced:**
- For loops with conditions
- Early returns and guard clauses
- Switch statements for state management
- Error handling with control flow

## Debugging

**Location:** `debugging/`

Fix bugs in a configuration validator that has control flow issues. The code compiles but has logic errors.

**Skills practiced:**
- Reading unfamiliar code
- Tracing control flow
- Identifying off-by-one errors
- Understanding switch fallthrough bugs

## Code Review

**Location:** `code-review/`

Review a pull request that implements a task processor with several control flow anti-patterns.

**Skills practiced:**
- Spotting deeply nested conditionals
- Identifying missing guard clauses
- Recognizing inefficient loops
- Understanding idiomatic Go patterns
