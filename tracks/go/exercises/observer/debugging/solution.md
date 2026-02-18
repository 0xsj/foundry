# Solution -- Event Bus Debugging Exercise

## Bug 1: Data Race on Handler Slice

### Root Cause

`Subscribe` writes to `b.handlers` without holding the mutex:

```go
func (b *EventBus) Subscribe(eventType EventType, handler HandlerFunc) {
    // Missing lock!
    b.handlers[eventType] = append(b.handlers[eventType], handler)
}
```

When multiple goroutines call `Subscribe` and `Publish` concurrently, they read and write the same map and slice without synchronization. The Go runtime's race detector flags this as a data race. In the worst case, concurrent `append` calls can corrupt the slice header, causing crashes.

### Fix

Add mutex protection:

```go
func (b *EventBus) Subscribe(eventType EventType, handler HandlerFunc) {
    b.mu.Lock()
    defer b.mu.Unlock()
    b.handlers[eventType] = append(b.handlers[eventType], handler)
}
```

### Prevention

- Always run `go test -race` in CI. The race detector has zero false positives -- every report is a real bug.
- When a struct has a mutex field, every method that reads or writes shared fields should use it. If you add a new field, audit all methods.
- Consider using `sync.RWMutex` if reads vastly outnumber writes (like Publish vs Subscribe).

---

## Bug 2: Deadlock from Notifying Inside Lock

### Root Cause

`Publish` calls handlers while holding the mutex:

```go
func (b *EventBus) Publish(event Event) {
    b.mu.Lock()
    defer b.mu.Unlock()

    for _, h := range b.handlers[event.Type] {
        h(event)  // If h() calls Subscribe() -> tries to Lock() -> DEADLOCK
    }
    // ...
}
```

If any handler calls `Subscribe`, `Unsubscribe`, or any method that acquires the mutex, the program deadlocks. With `sync.Mutex`, this is a self-deadlock (same goroutine tries to lock the same mutex twice). Go's `sync.Mutex` is not reentrant -- this is by design.

### Fix

Copy the handler slice under the lock, then release the lock before calling handlers:

```go
func (b *EventBus) Publish(event Event) {
    b.mu.Lock()
    handlers := make([]HandlerFunc, len(b.handlers[event.Type]))
    copy(handlers, b.handlers[event.Type])
    chanSubs := make([]*ChannelSubscriber, len(b.chanSubs[event.Type]))
    copy(chanSubs, b.chanSubs[event.Type])
    b.mu.Unlock()

    for _, h := range handlers {
        if h != nil {
            h(event)
        }
    }

    for _, sub := range chanSubs {
        select {
        case sub.Events <- event:
        default:
            // buffer full, drop or log
        }
    }
}
```

### Prevention

- **Never call external/callback code while holding a lock.** This is the most important concurrency rule for observer patterns.
- The copy-on-read pattern (snapshot + release + iterate) is the standard solution. It uses a bit more memory but eliminates an entire class of deadlocks.
- If using `sync.RWMutex`, use `RLock` for the snapshot (multiple publishers can read concurrently).

---

## Bug 3: Goroutine Leak -- Channel Never Closed

### Root Cause

`UnsubscribeChannel` removes the subscriber from the list but does not close the `Events` channel:

```go
func (b *EventBus) UnsubscribeChannel(eventType EventType, sub *ChannelSubscriber) {
    // ...
    b.chanSubs[eventType] = append(subs[:i], subs[i+1:]...)
    // BUG: Not closing sub.Events!
    return
}
```

Any goroutine doing `for event := range sub.Events` will block forever because `range` only exits when the channel is closed. The goroutine accumulates in memory, and over time this becomes a memory leak.

### Fix

Close the channel when unsubscribing:

```go
func (b *EventBus) UnsubscribeChannel(eventType EventType, sub *ChannelSubscriber) {
    b.mu.Lock()
    defer b.mu.Unlock()

    subs := b.chanSubs[eventType]
    for i, s := range subs {
        if s.ID == sub.ID {
            b.chanSubs[eventType] = append(subs[:i], subs[i+1:]...)
            close(s.Events)  // Signal the consumer goroutine to exit
            return
        }
    }
}
```

### Prevention

- **Every channel should have a clear owner responsible for closing it.** In the observer pattern, the bus (subject) creates and closes subscriber channels.
- Use `runtime.NumGoroutine()` in tests to detect goroutine leaks. Libraries like `go.uber.org/goleak` automate this.
- Consider adding a `Close()` method to subscribers that handles cleanup.

---

## Bug 4: Nil Callback Panic

### Root Cause

`SubscribeWithFilter` creates a nil `wrappedHandler` when `filter` is nil:

```go
func (b *EventBus) SubscribeWithFilter(eventType EventType, filter func(Event) bool, handler HandlerFunc) {
    var wrappedHandler HandlerFunc  // nil by default
    if filter != nil {
        wrappedHandler = func(e Event) {
            if filter(e) {
                handler(e)  // also panics if handler is nil
            }
        }
    }
    // wrappedHandler is nil when filter is nil
    b.handlers[eventType] = append(b.handlers[eventType], wrappedHandler)
}
```

When `Publish` iterates the handlers and calls `wrappedHandler(event)`, calling a nil function panics with `runtime error: invalid memory address or nil pointer dereference`.

Additionally, even when `filter` is not nil, if `handler` is nil, the wrapped function will panic when the filter passes.

### Fix

Validate inputs and handle the nil-filter case:

```go
func (b *EventBus) SubscribeWithFilter(eventType EventType, filter func(Event) bool, handler HandlerFunc) {
    if handler == nil {
        return // or panic("handler must not be nil")
    }

    var wrappedHandler HandlerFunc
    if filter != nil {
        wrappedHandler = func(e Event) {
            if filter(e) {
                handler(e)
            }
        }
    } else {
        // No filter means accept all events
        wrappedHandler = handler
    }

    b.mu.Lock()
    defer b.mu.Unlock()
    b.handlers[eventType] = append(b.handlers[eventType], wrappedHandler)
}
```

### Prevention

- **Validate callback arguments at registration time, not at call time.** Fail early.
- Be explicit about what nil means for each parameter. Document it or reject it.
- Consider using the `if fn == nil { panic(...) }` pattern at the top of registration functions, similar to how `http.HandleFunc` panics on nil handler.

---

## Summary of All Fixes

| Bug | Category | Root Cause | Fix Pattern |
|-----|----------|-----------|-------------|
| Data race | Concurrency | Missing mutex on Subscribe | Add Lock/Unlock |
| Deadlock | Concurrency | Calling handlers while holding lock | Copy-on-read + unlock before calling |
| Goroutine leak | Lifecycle | Channel not closed on unsubscribe | Close channel in Unsubscribe |
| Nil panic | Validation | Nil handler stored in slice | Validate at registration time |

## Related Pitfalls

- [[go-nil-map-panic]] -- Similar nil-value gotcha
- [[go-mutex-deadlock]] -- Detailed mutex deadlock patterns
- [[go-goroutine-leak]] -- Patterns that cause goroutine leaks
