# Debugging: Delivery Channel Manager

## Symptoms

A teammate wrote a delivery channel manager that tracks active channels, updates their configurations at runtime, and dispatches test pings. QA reports three separate issues:

1. **Channel status update has no effect** — after calling `Disable("email")`, the email channel is still enabled in subsequent reads. The method runs without error but the change doesn't persist.

2. **Application panics when pinging the fallback channel** — the fallback channel is set at startup, but calling `Ping()` on it crashes with a nil pointer dereference. No one touched the fallback after initialization.

3. **The "last updated" channel is always the first channel registered, not the most recently updated one** — `LastUpdated()` always returns the same channel name regardless of which channel was modified.

There are 3 bugs total — one causing each symptom above.

## Your Task

1. Read `buggy.go` and `buggy_test.go`
2. Run the tests: `go test -v ./...`
3. Identify and fix all 3 bugs without looking at `solution.md`
4. For each bug, write down: what it was, why it happened, and how to prevent it

## Difficulty

**Intermediate** — each bug maps directly to concepts from the structs-methods-and-enums lesson. If you have the lesson in front of you, you've seen the underlying cause of all three.

## Hints

<details>
<summary>Hint 1: The update that doesn't stick</summary>

When you call a method, what does the receiver get — the original struct or a copy? Does it depend on `*T` vs `T`?
</details>

<details>
<summary>Hint 2: The nil pointer panic</summary>

The `ChannelManager` embeds a pointer to a struct. What is the zero value of a pointer? What happens when you call a method through a nil pointer?
</details>

<details>
<summary>Hint 3: The wrong last-updated channel</summary>

When you read a struct from a map, what do you get? If you modify that value and store it back, which map entry does the modification go to?
</details>
