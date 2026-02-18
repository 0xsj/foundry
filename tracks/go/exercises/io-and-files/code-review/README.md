# Code Review: File-Based Response Cache

## Scenario

A backend team is adding a simple file-based cache to avoid redundant API calls for expensive, infrequently-changing data. A junior engineer has submitted a PR with an initial implementation. The PR has been passed to you for review before merge.

Review `proposed.go` as if you were doing a real code review. Write your findings in `my-review.md`, then compare against `expert-review.md`.

## PR Description (from the author)

> **feat: add file-based response cache**
>
> Adds a `FileCache` that stores API responses on disk so we don't hammer the upstream service for the same queries. Cache entries expire after a configurable TTL. If the cached file is missing or expired, the caller fetches fresh data and stores it.
>
> Usage:
> ```go
> cache := NewFileCache("/var/cache/api", 1*time.Hour)
> data, err := cache.Get("user:8821")
> if err != nil {
>     data, err = fetchFromAPI("user:8821")
>     cache.Set("user:8821", data)
> }
> ```

## What to Review

Consider all dimensions of correctness:
- Correctness: does it actually work?
- Resource management: are files/handles properly cleaned up?
- Error handling: are errors surfaced and wrapped correctly?
- Portability: any platform assumptions?
- Concurrency: any race conditions if two goroutines access the same key?
- I/O patterns: are reads and writes efficient and safe?

## Files

- `proposed.go` — the PR code to review
- `my-review.md` — template for your review (fill this in first)
- `expert-review.md` — reference expert review (read after writing yours)

## Format

Use this structure for your review:

1. **Critical Issues** — bugs, data loss, resource leaks
2. **Major Concerns** — design problems, safety issues
3. **Minor Suggestions** — style, names, small improvements
4. **Positive Feedback** — what's done well
