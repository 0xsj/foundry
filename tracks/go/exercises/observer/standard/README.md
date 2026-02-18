# Health Check Monitor

## Scenario

Your team operates a microservice mesh with 20+ services. Each service exposes a `/healthz` endpoint. You need a centralized health aggregator that monitors these services and notifies multiple downstream systems when a service's status changes -- an alerting system (PagerDuty), a metrics collector (Prometheus), and a real-time dashboard (WebSocket push). The monitor should be safe for concurrent use because health checks run in parallel goroutines.

## Brief

Implement a `HealthMonitor` that uses the Observer pattern to decouple health check polling from downstream notification. Services transition between `Healthy`, `Degraded`, and `Unhealthy` states. Observers are notified only when a service's state *changes* (not on every poll).

## Acceptance Criteria

- [ ] `HealthMonitor` tracks the status of named services
- [ ] `StatusChange` events include: service name, previous status, new status, timestamp, and optional metadata (e.g., error message, response time)
- [ ] Observers implement a `HealthObserver` interface with `OnStatusChange(StatusChange) error`
- [ ] `HealthMonitor.Register(observer)` adds an observer; `Unregister(observer)` removes it
- [ ] `HealthMonitor.ReportStatus(service, status, metadata)` records the current status and notifies observers only if the status changed from the previous report
- [ ] All operations are safe for concurrent use (multiple goroutines calling `ReportStatus` simultaneously)
- [ ] Observers are notified outside of any lock (copy-on-read pattern to prevent deadlocks)
- [ ] If an observer returns an error, the monitor continues notifying remaining observers and collects all errors
- [ ] `HealthMonitor.CurrentStatus(service)` returns the last known status for a service
- [ ] `HealthMonitor.AllStatuses()` returns a snapshot of all service statuses

## Constraints

- Services can have statuses: `Healthy`, `Degraded`, `Unhealthy`, `Unknown`
- The initial status for any service is `Unknown` until the first report
- Multiple goroutines will call `ReportStatus` concurrently -- use the race detector to verify
- Observers must not block each other -- if one observer is slow, others still get notified promptly
- The monitor should handle the case where an observer panics (recover and continue)

## Files

- `starter/main.go` -- Scaffold with types and interfaces defined, implementation left as TODOs
- `starter/main_test.go` -- Full test suite (run with `go test -race`)

## Getting Started

```bash
cd starter
go test -race  # Should fail initially -- all TODOs need implementation
```

## Hints

<details>
<summary>Hint 1: Data structure for observer list</summary>

Use `sync.RWMutex` to protect the observer slice. `RLock` when reading (during notification snapshot), `Lock` when modifying (register/unregister). Store observers in a `[]HealthObserver` slice.

</details>

<details>
<summary>Hint 2: Avoiding deadlocks during notification</summary>

Copy the observer slice under the read lock, then release the lock before calling any observer methods. This prevents deadlocks if an observer tries to register or unregister during notification.

```go
b.mu.RLock()
snapshot := make([]HealthObserver, len(b.observers))
copy(snapshot, b.observers)
b.mu.RUnlock()

for _, obs := range snapshot {
    obs.OnStatusChange(change)
}
```

</details>

<details>
<summary>Hint 3: Handling observer panics</summary>

Wrap each observer call in a function with `defer recover()`:

```go
func safeNotify(obs HealthObserver, change StatusChange) error {
    defer func() {
        if r := recover(); r != nil {
            // log the panic, return error
        }
    }()
    return obs.OnStatusChange(change)
}
```

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
