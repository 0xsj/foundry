# Code Review: Priority Notification Queue

## PR Context

A junior developer on your team is adding a priority-based notification queue to the dispatch system. The feature needs to:

- Accept notifications with priority levels (critical, high, normal, low)
- Process higher-priority notifications first
- Track queue statistics (enqueue count, dequeue count, peak size)
- Support pausing and resuming processing per priority level

The PR has been submitted for review before going to staging. The developer's local tests pass, but you want a thorough review before it ships.

## Your Task

1. Read `proposed.go` — the code being submitted
2. Write your review in `my-review.md` using the template provided
3. Look for issues related to:
   - Struct design (exported fields, constructor validation, zero value traps)
   - Method receivers (value vs pointer, consistency)
   - Embedding (correct use of promoted methods, nil pointer risk)
   - Interface design and implementation
   - Enum patterns and type safety
4. After writing your review, compare against `expert-review.md`

## Review Checklist

For each issue you find, categorize it:

- **Critical** — Will cause a bug, crash, or data corruption
- **Major** — Design problem that will create issues as the code grows
- **Minor** — Style, naming, or small improvements

## Hints

<details>
<summary>Hint 1: How many issues?</summary>

There are 6 issues: 2 critical, 2 major, 2 minor.
</details>

<details>
<summary>Hint 2: Focus areas</summary>

Look at: (1) which fields are exported vs unexported and why that matters, (2) where a nil pointer might hide, (3) which methods mutate state and whether their receivers are correct, (4) the enum design.
</details>

<details>
<summary>Hint 3: Value receiver + mutation</summary>

Check every method that changes any field. Does it use a pointer receiver? If not — is the mutation preserved or silently discarded?
</details>
