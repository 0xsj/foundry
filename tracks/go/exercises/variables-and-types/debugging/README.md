# Debugging: Webhook Event Tracker

## Symptoms

A teammate wrote a webhook event tracker that counts events by type and records the last payload for each. QA reports:

1. **The service crashes on startup** — panic on the first incoming webhook
2. **Event counts are wrong** — the dashboard shows 0 for event types that have definitely fired
3. **Payload sizes are incorrect** — the reported byte sizes don't match the actual payloads, especially for non-ASCII content (e.g., webhook payloads containing emoji or CJK characters)

The teammate swears the logic is correct. There are 3 bugs total — one causing each symptom.

## Your Task

1. Read `buggy.go` and `buggy_test.go`
2. Run the tests: `go test -v ./...`
3. Find all 3 bugs without looking at `solution.md`
4. Fix each bug and get all tests passing
5. For each bug, write down: what it was, why it happened, and how to prevent it

## Difficulty

**Basic to Intermediate** — Each bug is a classic variables-and-types gotcha. If you completed the lesson, you've seen the concepts behind all three.

## Hints

<details>
<summary>Hint 1: The crash</summary>

What is the zero value of a map? What happens when you write to it?
</details>

<details>
<summary>Hint 2: The wrong counts</summary>

When you assign a struct to a new variable, what happens in Go? Does modifying the copy affect the original?
</details>

<details>
<summary>Hint 3: The payload sizes</summary>

What does `len()` return for a Go string? Is that the same as the number of characters?
</details>
