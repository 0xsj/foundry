# Code Review: Inventory Management System

## PR Context

A teammate has submitted a PR for a small inventory management module. It tracks products with prices and quantities, and supports operations like adding stock, looking up products, and calculating total inventory value.

The code compiles and "seems to work" in manual GHCi testing, but you're reviewing it before it goes into the shared codebase.

## Your Task

1. Read `Proposed.hs`
2. Write your review in `my-review.md` using the template
3. Look for issues related to:
   - Type safety (could newtypes prevent bugs?)
   - Partial functions (crash risks)
   - Maybe handling (silent failures)
   - Numeric types (precision, overflow)
   - List performance characteristics
4. After writing your review, compare against `expert-review.md`

## Review Checklist

For each issue, categorize it:

- **Critical** — Will crash or produce wrong results
- **Major** — Design problem, type safety gap, or silent failure
- **Minor** — Style, naming, or could-be-better

## Hints

<details>
<summary>Hint 1: How many issues?</summary>

There are 6 issues: 2 critical, 2 major, 2 minor.
</details>

<details>
<summary>Hint 2: Focus areas</summary>

Look at: what happens with empty lists, bare Int/Double vs newtype, fromJust usage, String vs newtype for IDs, numeric precision for money, and repeated list traversals.
</details>
