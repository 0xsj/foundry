# Go Reference -- Observer Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), and
> [Standard Library Documentation](https://pkg.go.dev/std)
> for the `observer` module. Covers: channels, sync primitives, context cancellation, standard library observer patterns.

---

## Channels

Source: [Go Spec -- Channel types](https://go.dev/ref/spec#Channel_types)

A channel provides a mechanism for concurrently executing functions to communicate by sending and receiving values of a specified element type.

```go
chan T          // can both send and receive values of type T
chan<- T        // can only send to the channel
<-chan T        // can only receive from the channel
```

### Buffered vs Unbuffered Channels

| Property | Unbuffered (`make(chan T)`) | Buffered (`make(chan T, n)`) |
|----------|---------------------------|----------------------------|
| Send blocks when | No receiver is ready | Buffer is full |
| Receive blocks when | No sender is ready | Buffer is empty |
| Synchronization | Synchronous handoff | Asynchronous up to buffer size |
| Observer use case | Tight coupling, guaranteed delivery | Decoupled, tolerates burst |
| Backpressure | Immediate (blocks sender) | Deferred (blocks when buffer full) |

### Channel Operations

```go
ch <- v      // send v to channel ch
v := <-ch    // receive from ch, assign to v
v, ok := <-ch // receive with closed check (ok=false if closed and empty)
close(ch)    // close channel (signals no more values will be sent)
```

### Select Statement

Source: [Go Spec -- Select statements](https://go.dev/ref/spec#Select_statements)

A `select` statement lets a goroutine wait on multiple communication operations.

```go
select {
case msg := <-ch1:
    // received from ch1
case ch2 <- val:
    // sent val to ch2
case <-ctx.Done():
    // context cancelled
default:
    // no channel operation ready (non-blocking)
}
```

**Observer pattern usage:**
- `select` with `default` for non-blocking publish (drop event if buffer full)
- `select` with `ctx.Done()` for observer lifecycle management
- `select` with multiple event channels for multiplexing

### Closing Channels

Source: [Effective Go -- Channels](https://go.dev/doc/effective_go#channels)

Only the sender should close a channel. Closing signals that no more values will be sent.

```go
// Range over channel -- exits when channel is closed
for event := range eventChan {
    process(event)
}
```

**Rules for observer implementations:**
- The publisher (or bus) owns and closes subscriber channels
- Never close a channel from the receiver side
- Closing an already-closed channel panics
- Sending on a closed channel panics

---

## sync.RWMutex

Source: [sync package -- RWMutex](https://pkg.go.dev/sync#RWMutex)

A `RWMutex` is a reader/writer mutual exclusion lock. The lock can be held by an arbitrary number of readers or a single writer.

```go
type RWMutex struct {
    // contains filtered or unexported fields
}

func (rw *RWMutex) Lock()      // acquire write lock
func (rw *RWMutex) Unlock()    // release write lock
func (rw *RWMutex) RLock()     // acquire read lock
func (rw *RWMutex) RUnlock()   // release read lock
```

### Observer Pattern Usage

| Operation | Lock Type | Reason |
|-----------|-----------|--------|
| Subscribe (add observer) | `Lock()` (write) | Mutates the observer list |
| Unsubscribe (remove observer) | `Lock()` (write) | Mutates the observer list |
| Publish (read observer list) | `RLock()` (read) | Only reads the list; multiple publishers can proceed concurrently |
| Notify (call observers) | None | Call observers *after* releasing the lock to prevent deadlock |

### Critical Pattern: Copy-on-Read

```go
func (b *Bus) Publish(event Event) {
    b.mu.RLock()
    snapshot := make([]Handler, len(b.handlers))
    copy(snapshot, b.handlers)
    b.mu.RUnlock()

    // Iterate snapshot without holding lock
    for _, h := range snapshot {
        h(event)
    }
}
```

This prevents deadlocks where a handler calls `Subscribe` or `Unsubscribe` during notification.

---

## sync.Once

Source: [sync package -- Once](https://pkg.go.dev/sync#Once)

Useful for one-time initialization of observer infrastructure:

```go
var (
    globalBus  *EventBus
    busOnce    sync.Once
)

func GetEventBus() *EventBus {
    busOnce.Do(func() {
        globalBus = NewEventBus()
    })
    return globalBus
}
```

---

## Context for Observer Lifecycle

Source: [context package](https://pkg.go.dev/context)

Context provides cancellation signals and deadlines. For observer patterns, context controls observer lifetime.

```go
func (o *Observer) Start(ctx context.Context) {
    go func() {
        for {
            select {
            case event := <-o.events:
                o.handle(event)
            case <-ctx.Done():
                // Clean shutdown: drain remaining events
                for {
                    select {
                    case event := <-o.events:
                        o.handle(event)
                    default:
                        return
                    }
                }
            }
        }
    }()
}
```

### Context Propagation in Observers

```go
// Pass context through Notify so observers can respect cancellation
func (s *Subject) Notify(ctx context.Context, event Event) error {
    for _, obs := range s.observers {
        if err := ctx.Err(); err != nil {
            return err // stop notifying if context cancelled
        }
        if err := obs.OnEvent(ctx, event); err != nil {
            return err
        }
    }
    return nil
}
```

### Timeouts for Observer Notification

```go
// Give each observer a deadline
func (s *Subject) NotifyWithTimeout(event Event, timeout time.Duration) {
    for _, obs := range s.observers {
        ctx, cancel := context.WithTimeout(context.Background(), timeout)
        obs.OnEvent(ctx, event)
        cancel()
    }
}
```

---

## Standard Library Observer Patterns

### os/signal.Notify

Source: [signal package](https://pkg.go.dev/os/signal)

```go
func Notify(c chan<- os.Signal, sig ...os.Signal)
func Stop(c chan<- os.Signal)
func Reset(sig ...os.Signal)
```

Channel-based observer for OS signals.

```go
c := make(chan os.Signal, 1)
signal.Notify(c, os.Interrupt, syscall.SIGTERM)
defer signal.Stop(c)

sig := <-c
fmt.Println("Got signal:", sig)
```

**Design notes:**
- Buffer size of 1 recommended (signals arrive asynchronously)
- `signal.Stop` unsubscribes the channel
- Multiple channels can subscribe to the same signal
- If the channel is not ready (buffer full, no receiver), the signal is dropped

### http.Server.RegisterOnShutdown

Source: [net/http package](https://pkg.go.dev/net/http#Server.RegisterOnShutdown)

```go
func (srv *Server) RegisterOnShutdown(f func())
```

Callback-based observer. Registered functions are called when `Shutdown` is called.

```go
srv.RegisterOnShutdown(func() {
    log.Println("closing database connections")
    db.Close()
})
```

**Design notes:**
- No unsubscribe mechanism (shutdown is a one-time event)
- Callbacks run in their own goroutines
- Called during `Shutdown`, not during `Close`

### database/sql.DB Connection Pool Events

Source: [database/sql package](https://pkg.go.dev/database/sql#DB.SetConnMaxLifetime)

The connection pool internally observes connection lifecycle events (creation, idle, max lifetime) to manage pool health. While not a public observer API, the internal pattern is instructive.

### sync.Cond

Source: [sync package -- Cond](https://pkg.go.dev/sync#Cond)

A lower-level primitive that implements wait/signal semantics:

```go
type Cond struct {
    L Locker
}

func NewCond(l Locker) *Cond
func (c *Cond) Wait()       // release lock, wait for signal, reacquire lock
func (c *Cond) Signal()     // wake one waiting goroutine
func (c *Cond) Broadcast()  // wake all waiting goroutines
```

`sync.Cond` is the most primitive observer mechanism in Go. `Broadcast()` notifies all waiters (observers). In practice, channels are preferred because `Cond` is error-prone (must hold the lock when calling `Wait`, easy to miss signals). But it's worth knowing this is the lowest-level building block.

---

## Comparison: Callback vs Channel vs Interface

| Feature | Callback (`func`) | Channel (`chan Event`) | Interface (`Observer`) |
|---------|-------------------|----------------------|----------------------|
| Registration | `bus.Subscribe(fn)` | `sub := bus.Subscribe()` | `bus.Register(obs)` |
| Notification | `fn(event)` | `ch <- event` | `obs.OnEvent(event)` |
| Async | Manual (`go fn(e)`) | Built-in (goroutine reads channel) | Manual (`go obs.OnEvent(e)`) |
| Unsubscribe | Need ID or wrapper | Close channel or signal | Remove from slice |
| Backpressure | None | Buffered channel | None |
| Error handling | Return error from callback | Separate error channel | Return error from method |
| State | Closure captures state | Goroutine owns state | Struct fields |
| Multiple methods | One function per event type | One channel per subscriber | One interface, multiple event types |
| Testing | Pass test function | Read from channel in test | Pass mock struct |
| Composition | Higher-order functions | `select` multiplexing | Interface embedding |
| Lifecycle | Manual cleanup | Channel close + `select` | `Deregister` + context |
| Memory overhead | Slice of function pointers | Channel buffer + goroutine stack | Slice of interface values |

---

## sync/atomic for Lock-Free Observer Patterns

Source: [sync/atomic package](https://pkg.go.dev/sync/atomic)

For high-throughput scenarios, atomic operations can replace mutexes for the observer list:

```go
type AtomicBus struct {
    handlers atomic.Value // stores []HandlerFunc
}

func NewAtomicBus() *AtomicBus {
    b := &AtomicBus{}
    b.handlers.Store([]HandlerFunc{})
    return b
}

func (b *AtomicBus) Subscribe(h HandlerFunc) {
    for {
        old := b.handlers.Load().([]HandlerFunc)
        new := make([]HandlerFunc, len(old)+1)
        copy(new, old)
        new[len(old)] = h
        // Note: this is NOT a CAS -- atomic.Value.Store replaces unconditionally.
        // For true CAS, you'd need atomic.CompareAndSwapPointer or a mutex for writes.
        b.handlers.Store(new)
        return
    }
}

func (b *AtomicBus) Publish(event Event) {
    handlers := b.handlers.Load().([]HandlerFunc)
    for _, h := range handlers {
        h(event)
    }
}
```

**Tradeoff:** `atomic.Value` provides lock-free reads (great for frequent publishes with rare subscribes) but write contention requires additional synchronization. This pattern is used in performance-critical paths where publish frequency vastly exceeds subscribe frequency.

---

## Error Handling in Observer Notifications

### Strategies

| Strategy | Implementation | When to Use |
|----------|---------------|-------------|
| Ignore errors | `_ = obs.OnEvent(e)` | Fire-and-forget events (metrics, logging) |
| Collect all errors | Append to `[]error`, return | Need to report but not stop |
| Fail fast | Return on first error | Critical events where all observers must succeed |
| Error channel | Observer sends errors to separate channel | Async observers |
| Circuit breaker | Track failures per observer, disable after threshold | Production systems with unreliable observers |

### errors.Join (Go 1.20+)

```go
func (s *Subject) Notify(ctx context.Context, event Event) error {
    var errs []error
    for _, obs := range s.observers {
        if err := obs.OnEvent(ctx, event); err != nil {
            errs = append(errs, fmt.Errorf("observer %T: %w", obs, err))
        }
    }
    return errors.Join(errs...)
}
```

---

## Race Detector

Source: [Data Race Detector](https://go.dev/doc/articles/race_detector)

Always test observer implementations with the race detector:

```bash
go test -race ./...
```

The race detector instruments memory accesses and reports concurrent unsynchronized access. It has no false positives -- every report is a real bug. It adds ~2-10x runtime overhead, so use it in tests, not production.

Common races in observer implementations:
- Concurrent read/write on the observer slice
- Concurrent map access (if using `map[EventType][]Observer`)
- Shared state in observer callbacks without synchronization
