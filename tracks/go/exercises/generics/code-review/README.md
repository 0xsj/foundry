# Code Review: Generic Utility Functions

## Scenario

A colleague has submitted a PR to the internal platform team's shared utility library. The library provides helper functions used across 15 microservices. The PR description reads:

> "Refactored the utils package to use generics. Removed type-specific variants (mapInts, filterStrings, etc.) and replaced with generic versions. Also added some new generic helpers for working with configs and validation pipelines."

The PR touches 4 functions. Your job is to review `proposed.go` and identify:

1. Places where the code is over-engineered with generics (a regular function or interface would be clearer)
2. Constraint mistakes (too broad or too narrow)
3. Readability issues from excessive type parameters

## What to look for

Focus on the generics-specific issues. Assume the business logic is correct — don't comment on algorithm choices unless they directly relate to a type parameter decision.

Use `my-review.md` to write your review before reading `expert-review.md`.

## Context

- This is a shared library — changes affect all 15 consumers
- The functions in this PR are all new additions (no backward compatibility concern)
- The team's coding standard: "prefer simple over clever; types should communicate intent"
