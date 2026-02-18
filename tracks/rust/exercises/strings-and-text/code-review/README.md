# Code Review Exercise: String Utilities

## Scenario

A teammate has opened a PR adding a shared `string_utils` module to your internal
tooling library. The module provides functions for normalizing, extracting, and
summarizing text fields from API responses and log pipelines. The code compiles, all
tests pass, and it works correctly. You have been asked to review it before merging.

## Your Task

1. Read `proposed.rs` carefully
2. Identify all issues: unnecessary allocations, idiomatic problems, API ergonomics,
   correctness concerns, and design decisions you would push back on
3. Fill in `my-review.md` using the standard review structure:
   - Critical Issues (bugs, correctness)
   - Major Concerns (performance, design)
   - Minor Suggestions (style, idioms)
   - Positive Feedback
4. Then compare your review to `expert-review.md`

## What to Look For

Focus on how strings are being handled. Ask:
- Is `.clone()` being called where borrowing would work?
- Are there indexing assumptions that could panic on non-ASCII input?
- Is `Cow<str>` missing where it would avoid an allocation?
- Are `Display` and `Debug` being used for the right purposes?
- Are function signatures taking `String` ownership when `&str` would suffice?

## Context

- This is a library module — it will be called from many places across the codebase
- Some functions are in hot paths (called per-log-line in a streaming pipeline)
- The library supports arbitrary UTF-8 text including content from external systems
- The test suite passes as written
