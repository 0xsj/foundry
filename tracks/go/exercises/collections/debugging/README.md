# Debugging Exercise: Audit Log Processor

## Scenario

You've inherited an audit log processor that aggregates user actions across services. It reads batches of log entries, builds a summary by user, and produces per-service breakdowns. Several engineers have reported data corruption and crashes in production. The bugs were filed as:

- "Sometimes the system crashes on startup with a nil map panic"
- "Filtering logs by service gives wrong results — some entries appear in multiple services"
- "The per-user summary occasionally shows wrong action counts"
- "Deduplicating IDs doesn't seem to work — we're still seeing duplicates"

The tests expose these failures. Your job is to find and fix all the bugs without looking at `solution.md` first.

## Setup

```bash
cd tracks/go/exercises/collections/debugging
go test -v ./...
```

All tests fail. Fix the code in `buggy.go` until they all pass.

## What You Should Not Change

- Function signatures
- Test file (`buggy_test.go`)
- The overall algorithmic approach (fix the bugs, don't rewrite)

## Hints (try without these first)

<details>
<summary>Hint — general areas to look</summary>

The four bugs map directly to concepts from the collections lesson:
1. A write to a nil map
2. Shared backing array corruption when splitting a slice
3. range loop value copy semantics (struct mutation doesn't stick)
4. append returning a new slice header that the caller doesn't see
</details>
