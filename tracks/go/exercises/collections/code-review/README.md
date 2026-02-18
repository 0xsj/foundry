# Code Review Exercise: Leaderboard System

## Scenario

A teammate has submitted a PR adding a leaderboard system to a gaming analytics platform. The leaderboard tracks player scores across different game modes, supports ranking queries, and provides aggregate statistics.

The PR description says:

> "Added a Leaderboard type to track player scores per game mode. Supports top-N queries, score updates, and bulk imports. Tested locally — works fine on my machine."

Your job is to review the code in `proposed.go` for correctness, performance, and idiomatic Go. Write your review in `my-review.md`, then compare with `expert-review.md`.

## What to Look For

1. **Correctness bugs** — code that will panic or produce wrong results
2. **Data aliasing** — places where shared backing arrays could cause silent corruption
3. **Missing checks** — operations on nil or uninitialized values
4. **Inefficient patterns** — O(n) lookups where a map would give O(1)
5. **Allocation waste** — unnecessary slice copies or reallocations
6. **Idiomatic Go** — conventions that experienced Go developers would flag

## Review Structure

Write your review in `my-review.md` using this format:

**Critical Issues** — Bugs that will crash or produce wrong data
**Major Concerns** — Design problems that will cause pain in production
**Minor Suggestions** — Style, naming, small improvements
**Positive Feedback** — What was done well

Then read `expert-review.md` to compare.
