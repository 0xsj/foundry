# Debugging: Serialization Bugs

## Context

You've been handed a webhook processing service that ingests events from a payment provider. The service reads events off an HTTP stream, stores them internally, and exports them as JSON for downstream consumers.

Four tests are failing. Each failure has a different root cause — read the test names and failure messages, form a hypothesis, then trace the bug.

## Your Task

1. Run the tests: they fail in specific, informative ways
2. Read `buggy.go` — four functions, four bugs
3. For each failing test, identify the root cause and fix it
4. All tests should pass after your fixes

## Tests

```
go test ./...
```

Expected failures (do not fix the test file — fix `buggy.go`):

- `TestEventRoundTrip` — an unexported field is not appearing in JSON
- `TestZeroScoreOmitted` — a zero value is being silently dropped
- `TestStreamMultipleEvents` — only the first event in a stream is decoded
- `TestTimestampFormat` — an event timestamp is serializing in the wrong format

## Hints

<details>
<summary>Hint 1: How many bugs?</summary>

Four bugs, one per function. Each bug maps to one failing test.
</details>

<details>
<summary>Hint 2: Bug categories</summary>

The four bugs cover: (1) field visibility and JSON encoding, (2) the omitempty zero-value trap, (3) how json.Decoder handles multiple values, (4) time.Time and custom format requirements.
</details>

<details>
<summary>Hint 3: The streaming bug</summary>

A `json.Decoder` is created inside a loop. What happens to the read position on each iteration?
</details>

<details>
<summary>Hint 4: The time format bug</summary>

The struct uses `time.Time` which defaults to RFC3339. The test expects a Unix integer. What type would make a field serialize as an integer?
</details>
