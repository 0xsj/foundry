# Code Review Exercise: Webhook Validator

## PR Context

A teammate submitted a pull request adding input validation for an incoming
webhook processing service. The validator checks that incoming webhook events
have valid fields before they are handed off to downstream handlers. The code
compiles and the basic tests pass, but there are control flow issues throughout.

## Your Task

Review `proposed.rs` as if this were a real PR. Identify issues at three levels:

1. **Critical** — Bugs, panics, or correctness problems
2. **Major** — Control flow that is more verbose or error-prone than it should be
3. **Minor** — Style, idiom, readability, missed match patterns

Write your review in `my-review.md`, then compare with `expert-review.md`.

## How to Approach

1. Read through `proposed.rs` carefully — do not run it first
2. For each issue you find, note:
   - The line(s) or function involved
   - What the problem is
   - What the idiomatic fix should be
   - Which control flow concept is relevant
3. Also note what was done well

## Concepts Tested

- `if/else` chains vs `match`
- Redundant match arms (patterns that could be consolidated with `|` or `_`)
- `if let` vs `match` for single-arm patterns
- Guards (`if` conditions in match arms)
- Missing exhaustiveness: switch-like logic that forgets cases
- `loop { break }` anti-pattern when a simple expression suffices
