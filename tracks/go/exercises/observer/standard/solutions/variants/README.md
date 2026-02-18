# Solution Variants -- Health Check Monitor

## Approach Comparison

| Variant | Notification Model | Pros | Cons | When to Use |
|---------|-------------------|------|------|-------------|
| **Interface-based** (solution.go) | Synchronous, sequential | Simple, predictable ordering, easy to test | Slow observer blocks all subsequent observers | Default choice; when observers are fast |
| **Channel-based** | Async per-observer channels | Non-blocking publish, natural backpressure | More complex lifecycle management, potential goroutine leaks | When observers have varying processing speeds |
| **Callback with goroutines** | Fire-and-forget concurrent | Maximum throughput, no observer blocks publisher | Hard to collect errors, ordering is non-deterministic | Fire-and-forget scenarios (metrics, analytics) |

## Interface-Based (Reference Solution)

The reference solution (`solution.go`) uses the interface-based approach:

- Observers implement `HealthObserver` interface
- Notification is synchronous and sequential
- Copy-on-read pattern prevents deadlocks
- Panic recovery protects the notification loop
- Error collection allows callers to inspect failures

This is the recommended starting point because it's the simplest to reason about and test. You can always wrap individual observers in async wrappers later if needed.

## Channel-Based Variant

A channel-based approach would replace the observer interface with subscriber channels:

```go
type HealthSubscriber struct {
    Changes <-chan StatusChange
    changes chan StatusChange
    quit    chan struct{}
}

func (m *HealthMonitor) Subscribe(bufferSize int) *HealthSubscriber {
    ch := make(chan StatusChange, bufferSize)
    sub := &HealthSubscriber{
        Changes: ch,
        changes: ch,
        quit:    make(chan struct{}),
    }
    m.mu.Lock()
    m.subscribers = append(m.subscribers, sub)
    m.mu.Unlock()
    return sub
}
```

**Tradeoffs vs interface-based:**
- Publish never blocks (if buffer has space) -- better for hot paths
- Subscribers process at their own pace -- natural backpressure
- But: buffer full means dropped events (or blocking, depending on design)
- Lifecycle management is more complex (must close channels, avoid goroutine leaks)
- Testing requires goroutines to consume from channels

**When to choose this:** When `ReportStatus` is called from a latency-sensitive hot path (e.g., inside a request handler) and you cannot afford to wait for slow observers.

## Callback with Concurrent Goroutines

A concurrent notification variant:

```go
func (m *HealthMonitor) ReportStatus(service string, status ServiceStatus, metadata map[string]string) {
    // ... (same status change detection) ...

    var wg sync.WaitGroup
    for _, obs := range snapshot {
        wg.Add(1)
        go func(o HealthObserver) {
            defer wg.Done()
            defer func() { recover() }()
            o.OnStatusChange(change)
        }(obs)
    }
    wg.Wait() // or don't wait for fire-and-forget
}
```

**Tradeoffs:**
- All observers are notified concurrently -- total notification time = slowest observer
- But: notification ordering is non-deterministic
- Error collection requires a channel or mutex-protected slice
- More goroutine overhead (one per observer per event)

**When to choose this:** When you have many observers and total notification latency matters more than ordering.

## Performance Notes

For a health check monitor with ~5 observers checking ~20 services every 10 seconds:

- **Interface-based** is the clear winner. The overhead of sequential calls to 5 observers is negligible compared to the 10-second polling interval.
- **Channel-based** adds unnecessary complexity at this scale.
- **Concurrent goroutines** add overhead without meaningful benefit.

The channel-based and concurrent approaches become relevant when:
- You have 100+ observers
- Publish frequency is very high (1000+ events/sec)
- Individual observers may be slow (network calls, disk I/O)

Start simple. Measure. Optimize only when you have evidence.
