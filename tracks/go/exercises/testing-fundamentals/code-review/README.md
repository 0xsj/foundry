# Code Review Exercise: HTTP Handler Tests

## Scenario

A teammate opened a PR adding tests to your team's user-facing HTTP API. The implementation works fine and ships to production. The PR adds a test file that didn't exist before. You've been asked to review the PR before it merges.

The handler itself is in `codebase/handler.go`. The new tests are in `codebase/handler_test.go`.

## What to Review

This is a greenfield test file — there's no existing pattern to compare against. Evaluate it on its own merits:

1. Are the tests verifying **behavior** or **implementation details**?
2. Is the test organization idiomatic Go?
3. Does the mock strategy make the tests brittle?
4. Is there anything missing that would prevent these tests from catching regressions?
5. Are there any correctness issues in the test assertions?

## Process

1. Read `codebase/handler.go` to understand the code under test (5 minutes)
2. Read `codebase/handler_test.go` and take your own notes in `my-review.md`
3. Write your review in `my-review.md` — Critical / Major / Minor / Positive
4. After you've written your review, compare to `expert-review.md`

## Rules

- Write your review before reading `expert-review.md`
- Note at least 2 issues before comparing
- The goal is the thinking process, not matching the expert review exactly
