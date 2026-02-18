# Solution — Health Check Monitor

## Approach

The solution uses `Arc<Mutex<dyn Observer>>` as the core observer storage mechanism. This allows:
- The `HealthMonitor` (subject) and the caller to both hold references to each observer
- Observers to mutate their own state during notification (`&mut self` via `Mutex`)
- Thread-safe sharing via `Arc` and `Mutex`

## Key Decisions

1. **`Arc<Mutex<dyn Observer>>` over `Box<dyn Observer>`**: The caller needs to inspect observer state after notification (e.g., check `alerts_fired()`). With `Box`, ownership moves to the monitor and the caller loses access. `Arc` provides shared ownership.

2. **Unsubscribe by name with index collection**: We collect indices of matching observers first (locking each briefly), then remove them in reverse order. This avoids holding locks during mutation of the observer list. An alternative is `retain()` with a lock inside the closure — also valid but slightly less explicit about lock scope.

3. **Status as string key for metrics**: Using `"healthy"`, `"degraded"`, `"unhealthy"` as `HashMap` keys is simpler than making `HealthStatus` hashable. For a production system, you would `#[derive(Hash, Eq)]` on the enum.

4. **Consecutive failure tracking with reset on healthy**: The `AlertObserver` resets its counter when a service reports healthy. This prevents stale state from triggering false alerts after recovery.

## Complexity Analysis

- **Time**: O(n) per notification where n = number of observers. Each observer processes the event in O(1) or O(1) amortized (HashMap operations).
- **Space**: O(n * s) where n = observers and s = number of services. The `MetricsObserver` stores all latency samples, so it grows linearly with events — in production, you would use a rolling window or exponential moving average.

## Variants

| Approach | Pros | Cons | When to Use |
|----------|------|------|-------------|
| `Arc<Mutex<dyn Observer>>` (this solution) | Shared access, mutable observers, named trait | Mutex lock per observer per event, potential deadlock on re-entrant notify | Need to inspect observer state externally, want trait-based polymorphism |
| `Box<dyn Fn>` closures | Simpler, no trait boilerplate, captures environment | Cannot inspect observers, opaque callbacks | Fire-and-forget events, no external state access needed |
| `Weak<Mutex<dyn Observer>>` | Auto-cleanup of dropped observers | Slightly more complex subscription, upgrade overhead | Observer lifetimes are unpredictable or managed elsewhere |
| `mpsc` channels | Natural concurrency, no shared state | Events must be Clone, no synchronous processing guarantee | Each observer processes independently on its own thread |

## Implementation Notes

- **Lock scope matters**: In `unsubscribe`, we lock each observer briefly to check its name, then release the lock before modifying the vector. Holding the lock while removing elements would not cause a deadlock in this implementation, but it is good practice to minimize lock scope.
- **No panic on missing unsubscribe**: `unsubscribe("NonExistent")` returns `false` without error. Defensive coding — callers should not need to track exact subscription state.
- **`Send` bound on `Observer`**: The trait requires `Send` so that `Arc<Mutex<dyn Observer>>` can be moved across threads. Even though this exercise is single-threaded, the bound ensures the design is thread-safe by construction.

## Further Reading

- [[observer]] — Cross-language Observer pattern comparison
- [[interfaces-and-traits]] — Rust trait object fundamentals
- [[pointers-and-smart-pointers]] — Arc, Mutex, Weak in depth
