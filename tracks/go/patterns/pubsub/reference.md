# Go Reference -- Pub/Sub Pattern

> Extracted from the [Go Language Specification](https://go.dev/ref/spec),
> [Effective Go](https://go.dev/doc/effective_go), and
> [Standard Library Documentation](https://pkg.go.dev/std)
> for the `pubsub` module. Covers: channels, select, context, sync primitives, and concurrency patterns.

---

## Channels

Source: [Go Spec -- Channel types](https://go.dev/ref/spec#Channel_types)

A channel provides a mechanism for concurrently executing functions to communicate by sending and receiving values of a specified element type. The value of an uninitialized channel is `nil`.

### Channel Types

```go
chan T          // bidirectional: can send and receive values of type T
chan<- T        // send-only: can only send to the channel
<-chan T        // receive-only: can only receive from the channel
```

**Pub/Sub usage**: The broker holds `chan T` internally. It returns `<-chan T` to subscribers (they can only read) and accepts `chan<- T` from publishers (or uses the bidirectional channel internally for publish).

### Creating Channels

```go
ch := make(chan T)      // unbuffered channel
ch := make(chan T, n)   // buffered channel with capacity n
```

| Property | Unbuffered (`make(chan T)`) | Buffered (`make(chan T, n)`) |
|----------|---------------------------|----------------------------|
| Send blocks when | No receiver is ready | Buffer is full |
| Receive blocks when | No sender is ready | Buffer is empty |
| Synchronization | Synchronous handoff | Asynchronous up to buffer size |
| Pub/Sub use case | Tight delivery guarantee | Backpressure tolerance |
| `len(ch)` | Always 0 | Number of elements queued |
| `cap(ch)` | 0 | Buffer size n |

### Channel Operations

```go
ch <- v          // send v to channel ch (blocks if full/unbuffered)
v := <-ch        // receive from ch, assign to v (blocks if empty)
v, ok := <-ch    // receive with closed check (ok=false if closed and empty)
close(ch)        // close channel (signals no more values will be sent)
```

### Channel Axioms

Source: [Effective Go -- Channels](https://go.dev/doc/effective_go#channels)

| Operation | nil channel | Closed channel | Open channel |
|-----------|------------|----------------|--------------|
| Send `ch <- v` | Blocks forever | **PANIC** | Sends or blocks |
| Receive `<-ch` | Blocks forever | Returns zero value (ok=false) | Receives or blocks |
| Close `close(ch)` | **PANIC** | **PANIC** | Closes channel |
| `len(ch)` | 0 | Number of remaining elements | Number of queued elements |

**Critical for Pub/Sub**: Sending on a closed channel panics. The broker must ensure all publishers are done before closing subscriber channels. Use a `sync.WaitGroup` or `context.Context` to coordinate.

### Range Over Channel

```go
for msg := range ch {
    // processes messages until ch is closed
    // equivalent to: for { msg, ok := <-ch; if !ok { break }; ... }
}
```

**Pub/Sub usage**: Subscribers use `range` to continuously read messages. When the broker closes the channel, the loop exits cleanly.

---

## Select Statement

Source: [Go Spec -- Select statements](https://go.dev/ref/spec#Select_statements)

A `select` statement lets a goroutine wait on multiple communication operations. It blocks until one of its cases can proceed, then executes that case.

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

### Select Semantics

1. All channel operands are evaluated (once, in source order).
2. If multiple cases can proceed, one is chosen **uniformly at random**.
3. If no case can proceed and there's a `default`, the `default` executes.
4. If no case can proceed and there's no `default`, select blocks.

### Pub/Sub Select Patterns

**Non-blocking send (at-most-once delivery):**

```go
select {
case ch <- msg:
    // delivered
default:
    // subscriber's buffer full, message dropped
}
```

**Send with timeout:**

```go
select {
case ch <- msg:
    // delivered
case <-time.After(100 * time.Millisecond):
    // delivery timed out
}
```

**Multiplexed receive (subscribe to multiple topics):**

```go
select {
case msg := <-orderCh:
    handleOrder(msg)
case msg := <-paymentCh:
    handlePayment(msg)
case <-ctx.Done():
    return
}
```

**Priority receive (drain one channel before another):**

```go
for {
    // Priority: always drain high-priority first
    select {
    case msg := <-highPriority:
        handleUrgent(msg)
        continue
    default:
    }

    // Then check both
    select {
    case msg := <-highPriority:
        handleUrgent(msg)
    case msg := <-lowPriority:
        handleNormal(msg)
    case <-ctx.Done():
        return
    }
}
```

---

## Context Package

Source: [context package documentation](https://pkg.go.dev/context)

Package `context` defines the `Context` type, which carries deadlines, cancellation signals, and request-scoped values across API boundaries and between goroutines.

### Creating Contexts

```go
ctx := context.Background()                      // root context, never cancelled
ctx := context.TODO()                            // placeholder for undecided context
ctx, cancel := context.WithCancel(parent)         // cancellable context
ctx, cancel := context.WithTimeout(parent, 5*time.Second)   // auto-cancels after timeout
ctx, cancel := context.WithDeadline(parent, time.Now().Add(5*time.Second))
```

### Context for Pub/Sub Shutdown

```go
// Broker accepts a context for lifecycle management
func (b *Broker) Run(ctx context.Context) {
    <-ctx.Done()    // blocks until context is cancelled
    b.shutdown()    // clean up: close channels, drain messages
}

// Subscriber respects context cancellation
func (s *Subscriber) Listen(ctx context.Context, ch <-chan Message) {
    for {
        select {
        case msg, ok := <-ch:
            if !ok {
                return  // channel closed
            }
            s.handle(msg)
        case <-ctx.Done():
            return  // context cancelled
        }
    }
}
```

### Context Cancellation Propagation

When a parent context is cancelled, all derived contexts are also cancelled. This enables cascading shutdown:

```go
rootCtx, rootCancel := context.WithCancel(context.Background())

// Each subscriber gets a derived context
subCtx1, _ := context.WithCancel(rootCtx)
subCtx2, _ := context.WithCancel(rootCtx)

// Cancelling root cancels all subscribers
rootCancel()  // subCtx1.Done() and subCtx2.Done() both fire
```

---

## Sync Package

Source: [sync package documentation](https://pkg.go.dev/sync)

### sync.RWMutex

```go
var mu sync.RWMutex

mu.RLock()    // acquire read lock (multiple readers allowed)
mu.RUnlock()  // release read lock

mu.Lock()     // acquire write lock (exclusive)
mu.Unlock()   // release write lock
```

**Pub/Sub usage**: `RLock` for Publish (reading subscriber map), `Lock` for Subscribe/Unsubscribe (modifying subscriber map). Multiple publishers can publish concurrently since they only need the read lock.

### sync.WaitGroup

```go
var wg sync.WaitGroup

wg.Add(1)   // increment counter
wg.Done()   // decrement counter (call from goroutine via defer)
wg.Wait()   // block until counter reaches 0
```

**Pub/Sub usage**: Track active subscriber goroutines. `Add(1)` when a subscriber starts, `Done()` when it exits, `Wait()` during shutdown.

### sync.Once

```go
var once sync.Once
once.Do(func() {
    // executed exactly once, even across goroutines
})
```

**Pub/Sub usage**: Ensure `Close()` only executes once, even if called from multiple goroutines.

### sync.Map

```go
var m sync.Map

m.Store(key, value)               // set
value, ok := m.Load(key)          // get
m.Delete(key)                     // remove
m.Range(func(key, value any) bool { // iterate
    return true // continue
})
```

**Pub/Sub usage**: Alternative to `map + RWMutex` for the subscriber map. Better when there are many more reads than writes (common in Pub/Sub where publishes vastly outnumber subscribe/unsubscribe operations).

---

## Concurrency Patterns

### Fan-Out Pattern

Source: [Go Blog -- Pipelines and Cancellation](https://go.dev/blog/pipelines)

```go
// Start multiple goroutines reading from the same channel
func fanOut(in <-chan Message, workers int) []<-chan Result {
    outs := make([]<-chan Result, workers)
    for i := 0; i < workers; i++ {
        outs[i] = worker(in)  // each worker reads from the same input
    }
    return outs
}
```

### Fan-In Pattern

```go
// Merge multiple channels into one
func fanIn(channels ...<-chan Message) <-chan Message {
    var wg sync.WaitGroup
    merged := make(chan Message)

    for _, ch := range channels {
        wg.Add(1)
        go func(c <-chan Message) {
            defer wg.Done()
            for msg := range c {
                merged <- msg
            }
        }(ch)
    }

    go func() {
        wg.Wait()
        close(merged)
    }()

    return merged
}
```

### Pipeline Pattern

```go
// Each stage reads from input, processes, and writes to output
func stage(ctx context.Context, in <-chan Message) <-chan Message {
    out := make(chan Message)
    go func() {
        defer close(out)
        for msg := range in {
            select {
            case out <- transform(msg):
            case <-ctx.Done():
                return
            }
        }
    }()
    return out
}

// Chain stages: input -> stage1 -> stage2 -> stage3
output := stage3(ctx, stage2(ctx, stage1(ctx, input)))
```

---

## Goroutine Lifecycle

### Preventing Goroutine Leaks

A goroutine leak occurs when a goroutine blocks forever on a channel operation that will never complete. In Pub/Sub, this happens when:

1. A subscriber goroutine blocks on `<-ch` but the channel is never closed
2. A publisher goroutine blocks on `ch <- msg` but no one reads
3. A goroutine blocks on `<-done` but `done` is never closed

**Prevention**: Always provide an exit path via `context.Context` or `done` channel:

```go
func safeSubscriber(ctx context.Context, ch <-chan Message) {
    for {
        select {
        case msg, ok := <-ch:
            if !ok {
                return // channel closed
            }
            process(msg)
        case <-ctx.Done():
            return // context cancelled -- guaranteed exit
        }
    }
}
```

### Closing Channels Safely

Only the sender should close a channel. In Pub/Sub, the broker creates and owns subscriber channels, so the broker closes them:

```go
func (b *Broker) Close() {
    b.mu.Lock()
    defer b.mu.Unlock()

    if b.closed {
        return
    }
    b.closed = true

    for topic, subs := range b.subscribers {
        for _, ch := range subs {
            close(ch) // safe: broker is the only sender
        }
        delete(b.subscribers, topic)
    }
}
```

---

## References

- [Go Language Specification](https://go.dev/ref/spec)
- [Effective Go](https://go.dev/doc/effective_go)
- [Go Blog: Pipelines and Cancellation](https://go.dev/blog/pipelines)
- [Go Blog: Go Concurrency Patterns](https://go.dev/blog/concurrency-patterns)
- [Go Blog: Advanced Go Concurrency Patterns](https://go.dev/blog/advanced-concurrency)
- [Go Blog: Context](https://go.dev/blog/context)
- [sync package](https://pkg.go.dev/sync)
- [context package](https://pkg.go.dev/context)
