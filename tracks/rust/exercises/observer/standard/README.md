# Health Check Monitor — Observer Pattern

## Scenario

Your team runs a microservice platform. A central health check service polls each service every 30 seconds and publishes health events. Multiple consumers need to react to these events: an alerting system, a dashboard updater, an auto-scaler, and an audit logger. These consumers are developed by different teams and deployed independently — the health checker should not know about any of them directly.

## Brief

Implement a `HealthMonitor` that uses the Observer pattern to fan out health check results to multiple subscribers. Observers subscribe via a trait and receive events when service health changes.

## Acceptance Criteria

- [ ] Define a `HealthEvent` struct with fields: `service_name: String`, `status: HealthStatus`, `latency_ms: u64`, `timestamp: u64`, `message: Option<String>`
- [ ] Define a `HealthStatus` enum with variants: `Healthy`, `Degraded`, `Unhealthy`
- [ ] Define an `Observer` trait with `fn on_health_event(&mut self, event: &HealthEvent)` and `fn name(&self) -> &str`
- [ ] Implement `HealthMonitor` that stores observers as `Arc<Mutex<dyn Observer>>` and supports `subscribe`, `unsubscribe` (by name), and `notify_all`
- [ ] Implement `AlertObserver` that tracks consecutive unhealthy checks per service and prints an alert when the count reaches a configurable threshold
- [ ] Implement `DashboardObserver` that maintains the latest status for each service (a `HashMap<String, HealthStatus>`)
- [ ] Implement `MetricsObserver` that counts total events by status and tracks average latency per service
- [ ] All observers must be `Send` (safe to move across threads)
- [ ] Dead observer cleanup: if an observer is removed, subsequent `notify_all` calls should not panic

## Constraints

- Do NOT use channels — this exercise practices the `Arc<Mutex<dyn Observer>>` approach
- Observers must support mutable state updates during notification
- The `HealthMonitor` should work correctly if `unsubscribe` is called for an observer that does not exist
- Use `#[derive(Clone, Debug)]` on event types for testability

## Files

- `starter/main.rs` — Scaffold code with type definitions and TODOs
- `solutions/solution.rs` — Reference implementation
- `my-solution/` — Your implementation

## Getting Started

```bash
cd starter
rustc main.rs && ./main
# Should compile but print TODO messages initially
```

## Hints

<details>
<summary>Hint 1: Observer storage</summary>

Store observers as `Vec<Arc<Mutex<dyn Observer>>>`. When subscribing, the caller passes an `Arc<Mutex<dyn Observer>>` and retains a clone. The monitor clones the Arc to store it.

</details>

<details>
<summary>Hint 2: Unsubscribe by name</summary>

To unsubscribe by name, iterate the vector and lock each observer to check its `name()`. Use `retain` to keep only non-matching observers. Be careful with the lock scope — lock, check, release, then decide whether to retain.

</details>

<details>
<summary>Hint 3: Consecutive unhealthy tracking</summary>

`AlertObserver` needs a `HashMap<String, u32>` to track consecutive unhealthy counts. When a `Healthy` event arrives, reset the count to 0. When `Unhealthy` arrives, increment. When the count crosses the threshold, print the alert and optionally reset.

</details>

## Solution

After completing your implementation, compare with `solutions/` directory.
