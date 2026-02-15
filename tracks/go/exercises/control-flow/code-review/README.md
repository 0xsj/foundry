# Code Review Exercise: Task Processor

## Context

A teammate has submitted a PR implementing a background task processor. The code works, but you've been asked to review it for:
- Control flow anti-patterns
- Readability issues
- Idiomatic Go style
- Potential bugs

## Your Task

Review the code in `proposed.go` and write your review in `my-review.md`.

## Review Guidelines

Structure your review with these sections:

### 1. Critical Issues
Bugs, potential panics, or logic errors that would break functionality.

### 2. Major Concerns
Design problems, readability issues, or violations of Go idioms that should be fixed before merging.

### 3. Minor Suggestions
Nitpicks, style improvements, or optional refactors.

### 4. Positive Feedback
What was done well? Good reviews acknowledge good work too.

## What to Look For

As you read the code, ask:
- Are there deeply nested conditionals that could be flattened?
- Are guard clauses used appropriately?
- Is error handling idiomatic?
- Are loops efficient and clear?
- Is the happy path easy to follow?
- Are there off-by-one errors?
- Could switch statements replace if-else chains?

## Instructions

1. Read `proposed.go` carefully
2. Write your review in `my-review.md` (use the template provided)
3. Compare your review to `expert-review.md` when done
4. If you missed issues, understand why they're problematic

## Hints (Progressive)

<details>
<summary>Hint 1: Nesting</summary>

Look at the ProcessTask function. How many levels of nesting are there? Could guard clauses help?
</details>

<details>
<summary>Hint 2: Loop logic</summary>

Check the retry loop. Is the counter incremented correctly? What happens on the last retry?
</details>

<details>
<summary>Hint 3: Error handling</summary>

Look at how errors are handled. Are there cases where errors are silently ignored?
</details>

<details>
<summary>Hint 4: Switch vs If-Else</summary>

Find the status checking logic. Could it be clearer?
</details>
