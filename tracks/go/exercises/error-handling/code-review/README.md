# Code Review: HTTP Client Error Handling

## Scenario

A backend engineer has submitted a PR adding error handling to an internal HTTP client used by multiple platform services. The client wraps `net/http`, adding retries, auth headers, and structured error responses. The PR is ready for review — it compiles and the author says manual testing works.

Your job: review the error handling specifically. You're not reviewing HTTP logic or performance. Focus on whether errors are constructed, propagated, and exposed correctly.

## What to Review

The PR is in `proposed.go`. It adds:
- A custom error type for HTTP errors
- Retry logic with error accumulation
- Error wrapping at the boundary between internal and external errors
- A panic for a specific failure case

## PR Description

> This PR adds proper error handling to the platform HTTP client. Errors from the underlying `net/http` calls are now wrapped with context, retries accumulate errors, and non-2xx responses return a structured error type. Also added a panic guard for nil config — we should never call Do() without a config.

## Your Task

1. Read `proposed.go`
2. Write your review in `my-review.md` using the structure provided
3. Compare your review to `expert-review.md`

**Focus areas:**
- Error type design: is it exported correctly? Can callers use `errors.Is`/`errors.As`?
- Error wrapping: is the chain preserved at every layer?
- Panic use: is the panic justified or should it be an error?
- Consistency: are errors handled the same way everywhere?
- Error strings: do they follow Go conventions?

Run: no compilation needed — this is a reading exercise.
