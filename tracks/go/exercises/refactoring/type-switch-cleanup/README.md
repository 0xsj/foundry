# Refactoring Exercise — Type Switch Cleanup

## Context

You've inherited a webhook handler that processes different event types from a third-party API. The code works but has several maintainability issues.

## Current State

The code in `before.go` handles three webhook event types:
- `user.created` → Send welcome email
- `payment.succeeded` → Log transaction
- `subscription.cancelled` → Send retention offer

All tests pass. However, the code has grown organically and now has several code smells.

## Your Task

Identify code smells and refactor the code to improve:
- **Readability**: Easier to understand
- **Maintainability**: Easier to add new event types
- **Testability**: Easier to test handlers in isolation
- **Type Safety**: Use Go's type system better

## Constraints

- All existing tests must continue to pass
- Handler registration should be easy to extend
- No dependencies on external packages (use stdlib only)

## Code Smells to Look For

- [ ] Long functions (> 30 lines)
- [ ] Deeply nested conditionals
- [ ] Duplicated logic (parsing, error handling)
- [ ] Type assertions without error checking
- [ ] Poor error messages
- [ ] Mixing parsing, validation, and business logic
- [ ] Hard to add new event types

## Approach

1. **Read** `before.go` and identify smells
2. **Document** what smells you found (use checklist above)
3. **Refactor** incrementally, running tests after each change
4. **Compare** with `after.go` to see the reference refactoring

## Your Analysis

### Smells Identified

1.
2.
3.

### Refactoring Plan

1.
2.
3.

### Patterns Applied

- Pattern:
- Why:

### Trade-offs

What got better:
-

What got more complex (if anything):
-

---

See `analysis.md` for detailed explanation of the reference refactoring.
