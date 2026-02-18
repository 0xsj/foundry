# Solution Notes: Notification Dispatcher

## Approach

The reference solution builds the dispatcher in layers:

1. **Enums first** — `NotificationType` and `DeliveryStatus` as typed `int` constants with `iota` and `String()` methods. The type safety prevents mixing raw `int` values with notification types.
2. **Shared behaviors as embeddable types** — `RetryPolicy` and `DeliveryLog` are standalone structs with their own methods. Channel types embed them rather than duplicating the logic.
3. **Channel structs** — each embeds both shared types, adds its own config, and implements the concrete `Deliver` method. Pointer receivers throughout because `Deliver` mutates the embedded `DeliveryLog`.
4. **Interface definition** — `Deliverer` captures the minimum required behavior. The dispatcher depends on the interface, not concrete types.
5. **Dispatcher** — holds a `map[NotificationType]Deliverer`, initialized with `make()`. Drives the retry loop by calling `Deliver` and checking `ShouldRetry` on failure.

## Key Decisions

### Embedding by value, not by pointer

`RetryPolicy` and `DeliveryLog` are embedded by value in each channel struct:

```go
type EmailChannel struct {
    RetryPolicy  // by value — zero value is valid
    DeliveryLog  // by value — zero value is valid
    ...
}
```

This works because `RetryPolicy{MaxAttempts: 0}` and `DeliveryLog{}` (empty log) are both valid zero values. If the embedded types required initialization (e.g., a map field inside them), embedding by pointer and initializing in the constructor would be necessary to avoid a nil-dereference panic.

### Pointer receivers on Deliver and DeliveryLog.Append

`Deliver` has a pointer receiver because it calls `d.Append(...)` on the embedded `DeliveryLog`. If `Deliver` had a value receiver, `d.Append` would operate on a copy of the embedded `DeliveryLog`, and the log entries would be lost. The test `TestChannelDeliverAppendsToLog` catches this bug.

### The getLog helper and interface assertion

After dispatch, the report needs to include the channel's delivery log. But the `Deliverer` interface doesn't expose `Entries()` — that's an implementation detail, not part of the contract.

The solution uses a local interface assertion to access `Entries()`:

```go
func getLog(ch Deliverer) []string {
    type logProvider interface {
        Entries() []string
    }
    if lp, ok := ch.(logProvider); ok {
        return lp.Entries()
    }
    return nil
}
```

This is a deliberate design choice: don't widen the `Deliverer` interface just to expose logging internals. The type assertion keeps the interface minimal.

An alternative is to embed log access directly into the dispatch report loop by calling `ch.(interface{ Entries() []string })` inline. Both are valid.

### Retry loop design

```go
for {
    err := ch.Deliver(n)
    attempts++

    if err == nil {
        break // success
    }
    if !ch.(interface{ ShouldRetry(int) bool }).ShouldRetry(attempts) {
        break // exhausted
    }
}
```

The loop calls `ShouldRetry(attempts)` after each failure, where `attempts` is the count of tries completed. This is slightly more natural to read: "should we retry given we've tried N times?"

`ShouldRetry` is accessed via interface assertion because `Deliverer` doesn't include it. Alternatively, `ShouldRetry` could be added to `Deliverer`, but that forces all implementors to provide it — coupling the interface to the retry implementation detail.

## Tradeoffs

| Decision | This Solution | Alternative |
|----------|--------------|-------------|
| Embed by value | Zero value is usable | Embed by pointer — useful if embedded struct needs construction |
| `Deliverer` interface excludes `ShouldRetry` | Minimal interface | Expand interface — simpler dispatch loop |
| `getLog` uses type assertion | Keeps interface minimal | Add `Entries()` to `Deliverer` |
| Retry count in `Deliver` is per-instance | Simple state | Pass attempt count as argument to `Deliver` — more flexible |

## Performance Notes

- Each `DeliveryLog.Append` call does a `time.Now()` and a `fmt.Sprintf` — cheap for notification throughput, but not for tight inner loops.
- `getLog` does a type assertion on every dispatch — O(1) interface type lookup, not meaningful overhead.
- The retry loop is synchronous — the exercise is illustrating struct patterns, not concurrency. Real dispatchers would run delivery in goroutines with context cancellation.
