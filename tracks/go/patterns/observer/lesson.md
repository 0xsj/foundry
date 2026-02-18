# Observer Pattern -- Go

## The Problem: Coupling Event Producers to Consumers

Every production system has events. A user signs up and you need to send a welcome email, create an audit log entry, update analytics, and provision their workspace. A config file changes and you need to reload TLS certs, update rate limits, and flush caches. A health check fails and you need to fire a PagerDuty alert, update a dashboard, and start draining traffic.

The naive approach is direct calls:

```go
func (s *UserService) Signup(ctx context.Context, req SignupRequest) error {
    user, err := s.repo.Create(ctx, req)
    if err != nil {
        return err
    }
    s.emailer.SendWelcome(user)      // hard dependency
    s.audit.Log("user.signup", user)  // hard dependency
    s.analytics.Track("signup", user) // hard dependency
    s.provisioner.Setup(user)         // hard dependency
    return nil
}
```

This has several problems that compound as a codebase grows:

1. **Tight coupling.** `UserService` knows about emailers, audit loggers, analytics, and provisioners. Every new side effect means editing `UserService`.
2. **Testing pain.** To test signup, you need to mock or stub four unrelated systems.
3. **Fragile ordering.** If analytics tracking is slow, it blocks provisioning. If the email service is down, the whole signup fails (or you need to add error handling for each consumer).
4. **Violation of single responsibility.** Signup logic is tangled with notification logic.

The Observer pattern solves this by inverting the dependency: instead of the producer calling each consumer, consumers *subscribe* to events and the producer *publishes* without knowing who is listening.

If you've used `EventEmitter` in Node.js, `addEventListener` in the browser, or RxJS `Subject.subscribe()`, you already know this pattern. Go's twist is that it has three idiomatic ways to implement it, and the right choice depends on your concurrency requirements.

### Your notes
<!-- -->


---

## Approach 1: Callback Slices

The simplest approach. Store a slice of functions and call them when an event occurs. This is the closest analog to JavaScript's `EventEmitter.on()`.

```go
type EventType string

const (
    UserCreated  EventType = "user.created"
    UserDeleted  EventType = "user.deleted"
    UserUpdated  EventType = "user.updated"
)

type Event struct {
    Type      EventType
    Payload   any
    Timestamp time.Time
}

// HandlerFunc is what observers provide -- a function called on each event.
type HandlerFunc func(Event)

type EventBus struct {
    mu       sync.RWMutex
    handlers map[EventType][]HandlerFunc
}

func NewEventBus() *EventBus {
    return &EventBus{
        handlers: make(map[EventType][]HandlerFunc),
    }
}

func (b *EventBus) Subscribe(eventType EventType, handler HandlerFunc) {
    b.mu.Lock()
    defer b.mu.Unlock()
    b.handlers[eventType] = append(b.handlers[eventType], handler)
}

func (b *EventBus) Publish(event Event) {
    b.mu.RLock()
    handlers := make([]HandlerFunc, len(b.handlers[event.Type]))
    copy(handlers, b.handlers[event.Type])
    b.mu.RUnlock()

    for _, h := range handlers {
        h(event)
    }
}
```

Notice the critical detail in `Publish`: we copy the handler slice under the read lock and then iterate the copy *outside* the lock. This prevents a deadlock where a handler tries to subscribe or unsubscribe during notification. This is a real production bug -- we'll dissect it in the debugging exercise.

**When to use callback slices:**
- Simple in-process event notification
- Handlers are fast and synchronous
- You want the simplest possible implementation
- You don't need backpressure or buffering

**TypeScript parallel:**

```typescript
// Node.js EventEmitter is essentially this pattern
emitter.on('user.created', (event) => {
    auditLog.record(event);
});

emitter.emit('user.created', { userId: '123' });
```

Go doesn't have a built-in `EventEmitter`, so you build it from slices and mutexes. The tradeoff is explicitness -- you see exactly what's happening, at the cost of more boilerplate.

### Your notes
<!-- -->


---

## Approach 2: Channel-Based Fan-Out

Channels are Go's native tool for communication between goroutines. For the observer pattern, each subscriber gets a channel, and the publisher sends events to all subscriber channels. This naturally enables async notification.

```go
type Subscriber struct {
    ID     string
    Events chan Event
    Quit   chan struct{}
}

type ChannelBus struct {
    mu          sync.RWMutex
    subscribers map[EventType][]*Subscriber
}

func NewChannelBus() *ChannelBus {
    return &ChannelBus{
        subscribers: make(map[EventType][]*Subscriber),
    }
}

func (b *ChannelBus) Subscribe(eventType EventType, bufferSize int) *Subscriber {
    sub := &Subscriber{
        ID:     fmt.Sprintf("sub-%d", time.Now().UnixNano()),
        Events: make(chan Event, bufferSize),
        Quit:   make(chan struct{}),
    }
    b.mu.Lock()
    b.subscribers[eventType] = append(b.subscribers[eventType], sub)
    b.mu.Unlock()
    return sub
}

func (b *ChannelBus) Unsubscribe(eventType EventType, sub *Subscriber) {
    b.mu.Lock()
    defer b.mu.Unlock()
    close(sub.Quit)
    subs := b.subscribers[eventType]
    for i, s := range subs {
        if s.ID == sub.ID {
            b.subscribers[eventType] = append(subs[:i], subs[i+1:]...)
            close(s.Events)
            return
        }
    }
}

func (b *ChannelBus) Publish(event Event) {
    b.mu.RLock()
    subs := make([]*Subscriber, len(b.subscribers[event.Type]))
    copy(subs, b.subscribers[event.Type])
    b.mu.RUnlock()

    for _, sub := range subs {
        select {
        case sub.Events <- event:
            // delivered
        default:
            // buffer full -- drop event or log warning
            // this is a design decision: drop, block, or error
        }
    }
}
```

The subscriber runs in its own goroutine, reading from the channel:

```go
sub := bus.Subscribe(UserCreated, 100)
go func() {
    for {
        select {
        case event := <-sub.Events:
            fmt.Printf("received: %s\n", event.Type)
        case <-sub.Quit:
            return
        }
    }
}()
```

**Key design decisions in channel-based observers:**

| Decision | Options | Tradeoff |
|----------|---------|----------|
| Buffer size | 0 (unbuffered), N (buffered) | Unbuffered blocks publisher; buffered adds latency tolerance but can lose events if full |
| Full buffer behavior | Drop event, block, return error | Drop loses data; block can stall entire system; error pushes decision to caller |
| Channel ownership | Bus owns and closes | Prevents goroutine leaks but requires careful lifecycle management |
| Goroutine per subscriber | Yes / shared worker pool | Per-subscriber is simpler; worker pool is more resource-efficient at scale |

**When to use channel-based observers:**
- Subscribers need to process events asynchronously
- You need backpressure (buffered channels naturally provide this)
- Subscriber processing is slow and shouldn't block the publisher
- Events flow between goroutines or across service boundaries

**Compared to JS/TS:**

In Node.js, `EventEmitter` is synchronous -- handlers run in the current tick. Async handling requires the handler itself to spawn work (`setImmediate`, `Promise`, etc.). Go's channel-based approach makes async the default, which aligns with how Go handles concurrency generally -- explicit channels instead of implicit event loops.

### Your notes
<!-- -->


---

## Approach 3: Interface-Based Observers

The classic GoF Observer pattern adapted to Go's interface system. Define an `Observer` interface, let types implement it, and the subject manages a list of observers.

```go
type Observer interface {
    // OnEvent is called when a subscribed event occurs.
    // The observer receives the event and returns an error if processing fails.
    OnEvent(ctx context.Context, event Event) error
}

type Subject struct {
    mu        sync.RWMutex
    observers map[EventType][]Observer
}

func NewSubject() *Subject {
    return &Subject{
        observers: make(map[EventType][]Observer),
    }
}

func (s *Subject) Register(eventType EventType, observer Observer) {
    s.mu.Lock()
    defer s.mu.Unlock()
    s.observers[eventType] = append(s.observers[eventType], observer)
}

func (s *Subject) Deregister(eventType EventType, observer Observer) {
    s.mu.Lock()
    defer s.mu.Unlock()
    obs := s.observers[eventType]
    for i, o := range obs {
        if o == observer {
            s.observers[eventType] = append(obs[:i], obs[i+1:]...)
            return
        }
    }
}

func (s *Subject) Notify(ctx context.Context, event Event) []error {
    s.mu.RLock()
    obs := make([]Observer, len(s.observers[event.Type]))
    copy(obs, s.observers[event.Type])
    s.mu.RUnlock()

    var errs []error
    for _, o := range obs {
        if err := o.OnEvent(ctx, event); err != nil {
            errs = append(errs, err)
        }
    }
    return errs
}
```

Concrete observers implement the interface:

```go
type AuditLogger struct {
    writer io.Writer
}

func (a *AuditLogger) OnEvent(ctx context.Context, event Event) error {
    entry := fmt.Sprintf("[%s] %s: %v\n",
        event.Timestamp.Format(time.RFC3339),
        event.Type,
        event.Payload,
    )
    _, err := a.writer.Write([]byte(entry))
    return err
}

type MetricsCollector struct {
    counters map[EventType]*atomic.Int64
}

func (m *MetricsCollector) OnEvent(ctx context.Context, event Event) error {
    if counter, ok := m.counters[event.Type]; ok {
        counter.Add(1)
    }
    return nil
}
```

**When to use interface-based observers:**
- Observers have complex state (connections, buffers, configuration)
- Observers need to implement multiple methods (not just a single handler)
- You want to leverage Go's interface composition and type assertions
- The observer is a significant component, not just a callback

**Compared to Rust:**

Rust would use a trait for the observer:

```rust
trait Observer {
    fn on_event(&self, event: &Event) -> Result<(), Error>;
}

// Static dispatch via generics -- no vtable overhead
fn notify_all<O: Observer>(observers: &[O], event: &Event) { ... }

// Dynamic dispatch via trait objects -- like Go's interface
fn notify_all(observers: &[Box<dyn Observer>], event: &Event) { ... }
```

Go always uses dynamic dispatch for interfaces. Rust gives you the choice -- static dispatch (monomorphized, zero cost, but one type per call site) or dynamic dispatch (`dyn Trait`, like Go's approach). For observer patterns where you need heterogeneous observer types, both languages end up using dynamic dispatch.

### Your notes
<!-- -->


---

## Choosing the Right Approach

This is the decision matrix you should internalize:

| Concern | Callback Slice | Channel-Based | Interface-Based |
|---------|---------------|---------------|-----------------|
| Simplicity | Highest | Medium | Medium |
| Async notification | Manual (wrap in goroutine) | Built-in | Manual (wrap in goroutine) |
| Backpressure | None | Yes (buffered channels) | None |
| Observer lifecycle | Manual (no cleanup) | Channels close cleanly | Manual (`Deregister`) |
| Observer state | Closures capture state | Goroutine owns state | Struct fields |
| Type safety | Function signature | Channel type | Interface contract |
| Testability | Pass mock functions | Read from channel in test | Pass mock implementing interface |
| Unsubscribe support | Hard (need ID or comparison) | Close subscriber | Remove from slice |
| Production use case | Simple event hooks | Event streaming, pipelines | Plugin architectures |

**In practice, you'll often combine approaches.** A common pattern is interface-based observers that internally use channels for async processing:

```go
type AsyncObserver struct {
    events chan Event
    done   chan struct{}
    handler func(Event) error
}

func NewAsyncObserver(bufSize int, handler func(Event) error) *AsyncObserver {
    o := &AsyncObserver{
        events:  make(chan Event, bufSize),
        done:    make(chan struct{}),
        handler: handler,
    }
    go o.run()
    return o
}

func (o *AsyncObserver) OnEvent(ctx context.Context, event Event) error {
    select {
    case o.events <- event:
        return nil
    case <-ctx.Done():
        return ctx.Err()
    }
}

func (o *AsyncObserver) run() {
    defer close(o.done)
    for event := range o.events {
        if err := o.handler(event); err != nil {
            log.Printf("observer error: %v", err)
        }
    }
}

func (o *AsyncObserver) Close() {
    close(o.events)
    <-o.done // wait for all events to be processed
}
```

This gives you the structured registration of interfaces with the async processing of channels.

### Your notes
<!-- -->


---

## Concurrency Deep Dive

The observer pattern in Go is where concurrency bugs love to hide. Let's look at the three most common issues and their solutions.

### 1. Race Conditions on the Observer List

The classic bug: publishing events while another goroutine is subscribing.

```go
// BROKEN: no synchronization
type UnsafeBus struct {
    handlers []HandlerFunc
}

func (b *UnsafeBus) Subscribe(h HandlerFunc) {
    b.handlers = append(b.handlers, h)  // race: concurrent append
}

func (b *UnsafeBus) Publish(e Event) {
    for _, h := range b.handlers {  // race: reading while another goroutine appends
        h(e)
    }
}
```

Run this with `-race` and you'll see data race warnings. The fix is `sync.RWMutex`:

```go
type SafeBus struct {
    mu       sync.RWMutex
    handlers []HandlerFunc
}

func (b *SafeBus) Subscribe(h HandlerFunc) {
    b.mu.Lock()
    defer b.mu.Unlock()
    b.handlers = append(b.handlers, h)
}

func (b *SafeBus) Publish(e Event) {
    b.mu.RLock()
    snapshot := make([]HandlerFunc, len(b.handlers))
    copy(snapshot, b.handlers)
    b.mu.RUnlock()

    for _, h := range snapshot {
        h(e)
    }
}
```

Why `RWMutex` instead of `Mutex`? Publishing is a read operation (reading the handler list). Multiple goroutines can publish simultaneously -- they just need a consistent snapshot of the handlers. Only subscription mutates the list and needs an exclusive lock.

### 2. Deadlocks from Notifying Under Lock

A subtle but devastating bug: if you call handlers while holding the lock, and a handler tries to subscribe or unsubscribe, you deadlock.

```go
// DEADLOCK-PRONE
func (b *SafeBus) Publish(e Event) {
    b.mu.RLock()
    defer b.mu.RUnlock()
    for _, h := range b.handlers {
        h(e)  // if h() calls Subscribe(), it tries to acquire write lock = deadlock
    }
}
```

The handler acquires a write lock (for `Subscribe`) while the publisher holds a read lock (for iteration). If using a regular `Mutex`, this is a self-deadlock. With `RWMutex`, a write lock waits for all read locks to release, but the read lock won't release until the handler returns -- deadlock.

The fix: copy the slice and release the lock before calling handlers, as shown in the safe version above.

### 3. Goroutine Leaks with Channel Observers

When using channels, forgetting to close them or signal shutdown causes goroutine leaks:

```go
// LEAKY: goroutine runs forever if Unsubscribe is never called
sub := bus.Subscribe(UserCreated, 100)
go func() {
    for event := range sub.Events {
        process(event)
    }
    // This line is never reached if nobody closes sub.Events
}()
```

The fix: always provide a shutdown mechanism. Use a `Quit` channel, context cancellation, or close the events channel on unsubscribe.

```go
sub := bus.Subscribe(UserCreated, 100)
go func() {
    for {
        select {
        case event, ok := <-sub.Events:
            if !ok {
                return // channel closed, clean exit
            }
            process(event)
        case <-sub.Quit:
            return // explicit shutdown signal
        case <-ctx.Done():
            return // context cancelled
        }
    }
}()
```

### Your notes
<!-- -->


---

## Real-World Production Examples

### Standard Library: `signal.Notify`

Go's `os/signal` package uses the channel-based observer pattern. You subscribe to OS signals by providing a channel:

```go
sigChan := make(chan os.Signal, 1)
signal.Notify(sigChan, syscall.SIGTERM, syscall.SIGINT)

go func() {
    sig := <-sigChan
    log.Printf("received signal: %v, shutting down", sig)
    cancel()
}()
```

This is observer pattern: the OS is the subject, your channel is the observer, and `signal.Notify` is the subscription mechanism. The buffer size of 1 is important -- signals can be sent while your goroutine is busy, and a buffered channel prevents the signal from being dropped.

### Standard Library: `http.Server.RegisterOnShutdown`

```go
srv := &http.Server{Addr: ":8080"}
srv.RegisterOnShutdown(func() {
    db.Close()
})
srv.RegisterOnShutdown(func() {
    cache.Flush()
})
```

This is callback-based observer. The HTTP server is the subject. The shutdown callbacks are observers. When `srv.Shutdown(ctx)` is called, all registered callbacks fire. Simple, synchronous, and exactly the right tool for cleanup hooks.

### Kubernetes: Watch API

Kubernetes uses the observer pattern extensively. The Watch API lets clients subscribe to resource changes:

```go
watcher, err := clientset.CoreV1().Pods(namespace).Watch(ctx, metav1.ListOptions{})
if err != nil {
    return err
}

for event := range watcher.ResultChan() {
    pod := event.Object.(*v1.Pod)
    switch event.Type {
    case watch.Added:
        fmt.Printf("pod added: %s\n", pod.Name)
    case watch.Modified:
        fmt.Printf("pod modified: %s\n", pod.Name)
    case watch.Deleted:
        fmt.Printf("pod deleted: %s\n", pod.Name)
    }
}
```

This is channel-based observer with rich event types. The API server is the subject. Each watcher gets a channel of events. The `ResultChan()` method returns a read-only channel. This pattern scales to thousands of watchers across a cluster.

### Prometheus: Metric Collectors

Prometheus uses an interface-based observer pattern for metric collection:

```go
type Collector interface {
    Describe(chan<- *Desc)
    Collect(chan<- Metric)
}

// Register observers (collectors) with the subject (registry)
prometheus.MustRegister(myCollector)
```

Each collector is an observer that the registry notifies when it's time to scrape metrics. The registry iterates all registered collectors and calls `Collect()`. This is exactly the interface-based approach, with channels used for delivering the collected metrics.

### Your notes
<!-- -->


---

## Anti-Patterns and Pitfalls

### 1. Observer Stores Reference to Subject (Circular Dependency)

```go
// BAD: observer knows about the subject
type BadObserver struct {
    subject *Subject  // circular dependency
}

func (o *BadObserver) OnEvent(ctx context.Context, event Event) error {
    // Modifying the subject from inside an observer notification
    // is fragile and can cause infinite recursion
    o.subject.Notify(ctx, Event{Type: "derived.event"})
    return nil
}
```

Observers should be decoupled from the subject. If an observer needs to publish new events, inject a separate publisher interface rather than holding a reference to the subject.

### 2. Synchronous Notification Blocking the Publisher

If you have 10 observers and one takes 5 seconds, the publisher blocks for 5+ seconds. This is acceptable for simple cases but kills throughput in hot paths.

```go
// Publisher blocks until ALL observers complete
for _, o := range observers {
    o.OnEvent(ctx, event)  // slow observer blocks everything after it
}
```

Fix: fire observers concurrently when latency matters:

```go
var wg sync.WaitGroup
for _, o := range observers {
    wg.Add(1)
    go func(obs Observer) {
        defer wg.Done()
        obs.OnEvent(ctx, event)
    }(o)
}
wg.Wait()
```

Or don't wait at all if fire-and-forget is acceptable:

```go
for _, o := range observers {
    go o.OnEvent(ctx, event)  // fire and forget
}
```

### 3. No Unsubscribe Mechanism

Leaking observers means leaking memory and processing time. Every subscription mechanism should have a corresponding unsubscription:

```go
// Good: Subscribe returns a function that unsubscribes
func (b *EventBus) Subscribe(t EventType, h HandlerFunc) (unsubscribe func()) {
    b.mu.Lock()
    id := b.nextID
    b.nextID++
    b.handlers[t] = append(b.handlers[t], namedHandler{id: id, fn: h})
    b.mu.Unlock()

    return func() {
        b.mu.Lock()
        defer b.mu.Unlock()
        handlers := b.handlers[t]
        for i, nh := range handlers {
            if nh.id == id {
                b.handlers[t] = append(handlers[:i], handlers[i+1:]...)
                return
            }
        }
    }
}
```

This pattern -- returning a cleanup function from subscription -- is also how React's `useEffect` works and how Go's `context.WithCancel` provides a cancel function. The cleanup function closes over the subscription ID, making it self-contained.

### 4. Modifying Observer List During Iteration

```go
// BUG: removing an observer while iterating causes index shift
func (s *Subject) Notify(event Event) {
    for i, o := range s.observers {
        if shouldRemove(o) {
            s.observers = append(s.observers[:i], s.observers[i+1:]...)  // corrupts iteration
        }
        o.OnEvent(event)
    }
}
```

Always snapshot the list before iterating, or collect removals and apply them after iteration.

### Your notes
<!-- -->


---

## Comparison Across Languages

| Aspect | Go | TypeScript/Node.js | Rust |
|--------|----|--------------------|------|
| Built-in mechanism | None (build from primitives) | `EventEmitter`, `addEventListener` | None (crates: `tokio::sync::broadcast`, `crossbeam`) |
| Primary tool | Channels + interfaces | `EventEmitter` class | Channels (`mpsc`, `broadcast`) |
| Async model | Goroutines + channels | Event loop + callbacks/promises | `async`/`await` + channels |
| Thread safety | `sync.RWMutex` or channels | Single-threaded (no races) | `Arc<Mutex<>>` or channels |
| Type safety | Interface contracts | Typed events (with generics in TS) | Trait bounds + generics |
| Backpressure | Buffered channels, `select` | Streams (with backpressure in Node Streams) | Bounded channels, `async` streams |
| Common pattern | Copy-on-read for handler list | Direct iteration (single-threaded) | `Arc<RwLock<Vec<Box<dyn Observer>>>>` |

**Key insight for TS developers:** In Node.js, you never worry about race conditions in `EventEmitter` because JavaScript is single-threaded. In Go, *every* observer implementation must handle concurrent access. This is the single biggest difference -- the concurrency safety tax is real but the payoff is true parallelism.

**Key insight for Rust developers:** Rust makes the concurrency safety explicit in the type system (`Send`, `Sync`, `Arc`, `Mutex`). Go makes it your responsibility to use `sync.Mutex` correctly -- the compiler won't catch races (use `go test -race` instead).

### Your notes
<!-- -->
