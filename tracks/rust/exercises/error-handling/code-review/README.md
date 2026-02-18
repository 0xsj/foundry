# Code Review Exercise: CSV Data Pipeline

## Scenario

A backend engineer has opened a PR to add error handling to a CSV data pipeline that processes daily sales reports. Previously the pipeline just panicked on any problem. The PR adds `Result` return types throughout and is described as "production-ready error handling." Your job is to review it.

## Proposed Changes

See `proposed.rs` for the full diff.

## What to Review

Write your review in `my-review.md`. Structure it as:

1. **Critical Issues** — bugs or behavior that would fail in production
2. **Major Concerns** — design problems, poor error handling strategy, maintainability
3. **Minor Suggestions** — style, naming, small improvements
4. **Positive Feedback** — what was done well

## Context

- This code runs as a nightly batch job on a server; it does not serve user-facing requests.
- It reads CSV files from a known directory structure.
- Errors should be logged and the job should continue processing other files if possible.
- The pipeline currently handles ~50 files per run.

## Run

```
rustc proposed.rs && ./proposed
```

The code compiles. Your review is about design and correctness, not compilation.
