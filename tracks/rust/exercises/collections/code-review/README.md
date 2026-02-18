# Code Review Exercise: Word Frequency Counter

## Scenario

A teammate has opened a PR adding a text statistics tool to your analytics pipeline.
The tool reads a block of text and computes word frequencies, top words, and a few
aggregate statistics. The code is functional and the tests pass, but you've been asked
to review it before merging.

## Your Task

1. Read `proposed.rs` carefully
2. Identify all issues: correctness bugs, performance problems, idiomatic style, and
   design decisions you'd push back on
3. Fill in `my-review.md` using the standard review structure:
   - Critical Issues (bugs, correctness)
   - Major Concerns (performance, design)
   - Minor Suggestions (style, idioms)
   - Positive Feedback
4. Then compare your review to `expert-review.md`

## What to Look For

Focus on how collections are being used. Ask:
- Is the right collection being chosen for each job?
- Is the entry API being used correctly?
- Are there unnecessary allocations or redundant passes over data?
- Are there places where the code could panic?
- Is the API ergonomic for callers?

## Context

- This is a library module — it will be called from multiple places
- Performance matters: the tool may process multi-megabyte documents
- The test suite passes as written
