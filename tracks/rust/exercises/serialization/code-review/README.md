# Code Review Exercise: API Response Types

## PR Context

**PR title:** "Add typed response structs for the metrics API"

**PR description:**
> The metrics endpoint was returning `Value` (untyped JSON) everywhere. This PR
> adds proper typed response structs with serde derives, making the response
> shape explicit and enabling compile-time validation. Also adds some field
> renames so our internal snake_case names match the camelCase JSON the
> frontend expects.

**Your role:** Senior Rust engineer reviewing this PR before it lands on main.

## What to Review

The author has the right idea — typed structs are better than `Value` everywhere —
but the implementation has several problems. Some are serde-specific, some are
general Rust API design issues, and one is a subtle correctness issue.

Look for:
- Serialization attributes that are missing or wrong
- Public API surface that exposes implementation details
- Type choices that are weaker than they could be
- Fields that would benefit from flattening

## Files Changed

- `proposed.rs` — the code under review

## Your Task

1. Read `proposed.rs` carefully
2. Fill in `my-review.md` with your findings (Critical, Major, Minor, Positive)
3. After completing your review, compare with `expert-review.md`

Try to find all issues before looking at the expert review.
