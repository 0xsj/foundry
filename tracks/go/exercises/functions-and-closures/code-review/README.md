# Code Review: Exponential Backoff with Closures

## PR Context

A backend engineer has submitted a PR adding a generic retry utility with exponential backoff to your platform's internal SDK. The utility is intended to be used across all services for retrying HTTP calls, database queries, and third-party API calls. It's a small but high-leverage piece of infrastructure — a subtle bug here would affect every service that uses it.

The engineer is solid but relatively new to Go. The code passes their local tests and handles the happy path correctly. You're reviewing it before it lands in the shared SDK.

## Your Task

1. Read `proposed.go` — the code being submitted
2. Write your review in `my-review.md` using the template provided
3. After writing your review, compare against `expert-review.md`

Look specifically for issues related to:

- **Closure hygiene** — variable capture, state sharing between calls
- **Defer semantics** — argument evaluation timing, defer inside loops
- **Function signature design** — idiomatic Go vs non-idiomatic patterns
- **Error handling** — wrapping, context, return patterns
- **Correctness** — does it actually do what it claims to?

## Review Checklist

For each issue you find, categorize it:

- **Critical** — Will cause incorrect behavior, crashes, or data corruption
- **Major** — Design problem that will cause issues as usage grows or code evolves
- **Minor** — Style, naming, or small improvements that don't affect correctness

## Hints

<details>
<summary>Hint 1: How many issues are there?</summary>

There are 6 issues total: 2 critical, 2 major, 2 minor.
</details>

<details>
<summary>Hint 2: Focus areas</summary>

Look closely at: the closure inside the retry loop, defer argument evaluation, the `Backoff` function signature (return type), error information preservation, a subtle state-sharing issue when the returned function is called multiple times, and naming/style.
</details>
