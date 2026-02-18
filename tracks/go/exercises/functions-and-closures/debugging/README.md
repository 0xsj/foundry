# Debugging: Retry Wrapper

## Symptoms

A teammate wrote a retry utility for an HTTP client. It wraps any operation that might fail transiently and retries it up to N times with configurable delays. QA and a code review have surfaced three issues:

1. **Wrong number of retries** — the retry wrapper claims to retry 3 times, but tests show it runs the operation 4 times on consistent failure. The attempt count displayed in logs is also off.

2. **Cleanup runs with the wrong value** — the wrapper logs "cleaning up after attempt N" at the end, but the logged attempt number is always the total attempt count, not the attempt number that failed. This makes tracing failures in logs confusing.

3. **Resource leak under load** — when retrying many operations concurrently, the system runs out of file descriptors. Review shows each retry loop registers multiple timer resources that aren't released until the outer function returns, not at the end of each retry.

There are exactly 3 bugs. The tests in `buggy_test.go` fail due to these bugs. Find and fix all three without looking at `solution.md`.

## Your Task

1. Read `buggy.go` and `buggy_test.go`
2. Run the tests: `go test -v ./...`
3. Find all 3 bugs without looking at `solution.md`
4. Fix each bug and get all tests passing
5. For each bug, write down: what it was, why it happened, and how to prevent it

## Difficulty

**Intermediate** — all three bugs stem directly from the functions-and-closures lesson. If you covered closures, defer in loops, and named return + defer interaction, you've seen the mechanics behind each one.

## Hints

<details>
<summary>Hint 1: Wrong retry count</summary>

Look carefully at the loop variable used to build the retry closures. Are all the attempt functions capturing the same variable? What value does that variable have when they execute?
</details>

<details>
<summary>Hint 2: Cleanup value</summary>

The cleanup function is registered with `defer`. When are defer arguments evaluated — at the time the defer statement runs, or when the deferred call executes?
</details>

<details>
<summary>Hint 3: Resource leak</summary>

`defer` inside a loop doesn't run at the end of each iteration. When does it run? What does that mean for resources registered inside a retry loop?
</details>
