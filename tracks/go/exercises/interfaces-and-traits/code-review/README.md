# Code Review: Logging Middleware

## Scenario

A teammate opened a PR to add request logging to the team's internal API gateway. The gateway routes requests to multiple backend services. The PR adds a `LoggingMiddleware` that wraps a handler and logs each request.

The implementation works — tests pass, production behavior is correct. But the interface design has three distinct problems that will cause pain as the codebase grows.

## Your Task

1. Read `proposed.go` and the surrounding context in `codebase/`
2. Write your review in `my-review.md` (template provided)
3. Compare your review with `expert-review.md` afterward

## What to Look For

This PR is about **interface design**, not correctness. All three issues are design problems that don't cause immediate failures but create friction, coupling, or unnecessary complexity as the codebase evolves.

Think about:
- Are the interfaces as small as they could be?
- Are the function signatures accepting the right types?
- Is an interface the right abstraction here, or would something simpler suffice?

## Difficulty

Medium — requires knowing when interfaces are appropriate and when they're overhead.

## Context

The gateway is a thin HTTP reverse proxy. Handlers receive a `Request`, call a backend, and return a `Response`. Middleware wraps handlers to add cross-cutting concerns (logging, tracing, auth, rate limiting).
