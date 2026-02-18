# Pub/Sub Pattern -- Go

## The Problem: Event Producers That Know Too Much

The Observer pattern (which we covered [[patterns/observer|previously]]) solves the coupling problem between event producers and consumers. But Observer has a limitation: the subject maintains a direct reference to every observer. If you have 50 microservices that care about "user.created" events, the user service needs to know about all 50 of them.

Pub/Sub takes decoupling one step further by introducing a **broker** (or **message bus**) between publishers and subscribers. Publishers send messages to a topic on the broker. Subscribers register interest in topics with the broker. Neither side knows the other exists.

```
Observer:    Producer ----> Observer1
                      \---> Observer2
                      \---> Observer3

Pub/Sub:     Publisher ----> [Broker] ----> Subscriber1
             Publisher ----> [Broker] ----> Subscriber2
                             [Broker] ----> Subscriber3
```

This distinction matters in practice:

1. **Observer**: The subject holds references to observers. Adding a new observer means touching the subject's registration. Common within a single service.
2. **Pub/Sub**: The broker holds all routing logic. Publishers and subscribers connect to the broker independently. Common across service boundaries and within event-driven architectures.

If you've used Redis Pub/Sub, NATS, Kafka, RabbitMQ, or even Node.js EventEmitter with named events -- you've used Pub/Sub. Go's channels make it particularly natural to implement because channels *are* the communication primitive between goroutines.

### Your notes
<!-- -->

---

## Pub/Sub vs Observer: When to Use Which

| Dimension | Observer | Pub/Sub |
|-----------|----------|---------|
| Coupling | Subject knows about observers (interface) | Publisher and subscriber don't know each other |
| Mediator | None -- direct notification | Broker / message bus |
| Communication | Typically synchronous callbacks | Typically asynchronous (channels, queues) |
| Filtering | Observer decides what to ignore | Broker routes by topic; subscribers only get what they asked for |
| Scaling | Hard to distribute across processes | Natural fit for distributed systems |
| Complexity | Simpler to implement | More moving parts, but more flexible |
| Go idiom | Interface slice + mutex | Channels + goroutines + select |

**Rule of thumb**: If the producer and consumers live in the same struct or package and you have fewer than ~5 consumers, Observer is fine. Once you need topic routing, async delivery, or the system might grow to many consumers, reach for Pub/Sub.

### Your notes
<!-- -->

---

## Core Concepts

### Topics

A topic is a named channel of communication. Publishers send messages to a topic, and subscribers register to receive messages from specific topics.

```go
// Topics are typically strings that follow a hierarchical naming convention
const (
    TopicOrderCreated   = "order.created"
    TopicOrderShipped   = "order.shipped"
    TopicPaymentFailed  = "payment.failed"
    TopicUserSignedUp   = "user.signed_up"
)
```

Some systems support **wildcard topics**: `order.*` matches `order.created`, `order.shipped`, etc. This is common in AMQP, NATS, and MQTT. We'll implement this.

### Messages

A message is the payload sent through the broker. It typically includes metadata (topic, timestamp, ID) alongside the actual data.

```go
type Message struct {
    ID        string
    Topic     string
    Payload   []byte      // or any -- serialized data
    Timestamp time.Time
    Metadata  map[string]string
}
```

Using `[]byte` for the payload is idiomatic in Go for broker implementations because it mirrors what you'd get from a real message queue (Kafka, NATS, Redis). The subscriber is responsible for deserialization.

### Subscribers

A subscriber expresses interest in one or more topics and provides a channel (or callback) to receive messages.

```go
type Subscriber struct {
    ID     string
    Topics []string
    Ch     chan Message
}
```

Using channels instead of callbacks is the Go-idiomatic approach. It lets subscribers process messages at their own pace, enables `select`-based multiplexing, and naturally handles backpressure through channel buffering.

### Your notes
<!-- -->

---

## In-Process Pub/Sub with Channels

The simplest Pub/Sub implementation uses a map of topic to subscriber channels, protected by a mutex for concurrent access.

```go
type Broker struct {
    mu          sync.RWMutex
    subscribers map[string][]chan Message  // topic -> subscriber channels
    closed      bool
}

func NewBroker() *Broker {
    return &Broker{
        subscribers: make(map[string][]chan Message),
    }
}

func (b *Broker) Subscribe(topic string, bufSize int) <-chan Message {
    b.mu.Lock()
    defer b.mu.Unlock()

    ch := make(chan Message, bufSize)
    b.subscribers[topic] = append(b.subscribers[topic], ch)
    return ch
}

func (b *Broker) Publish(topic string, msg Message) {
    b.mu.RLock()
    defer b.mu.RUnlock()

    msg.Topic = topic
    msg.Timestamp = time.Now()

    for _, ch := range b.subscribers[topic] {
        // Non-blocking send: drop message if subscriber is full
        select {
        case ch <- msg:
        default:
            // subscriber channel full -- message dropped
            // In production, you'd log this or send to a dead letter queue
        }
    }
}
```

**Key decisions in this implementation:**

1. **Buffered channels for subscribers.** The `bufSize` parameter controls backpressure. A subscriber with `bufSize=0` gets synchronous delivery (publisher blocks until subscriber reads). A subscriber with `bufSize=100` can absorb bursts.

2. **Non-blocking send with `select/default`.** If a subscriber's channel is full, the message is dropped rather than blocking the publisher. This is an **at-most-once** delivery semantic. The alternative (blocking send) risks deadlocking the publisher if any subscriber is slow.

3. **RWMutex for concurrent reads.** Publishing only needs a read lock because it doesn't modify the subscriber map. Only Subscribe/Unsubscribe need a write lock.

> **Common Pitfall**: Sending on a closed channel panics in Go. If you close a subscriber's channel while the publisher is iterating, you get a runtime panic. The broker must coordinate shutdown carefully. See [[pitfalls/go-send-on-closed-channel]].

### Your notes
<!-- -->

---

## Topic-Based Routing and Wildcards

Real message brokers support wildcard subscriptions. A subscriber to `order.*` receives messages published to `order.created`, `order.shipped`, and `order.cancelled`. This is extremely useful for monitoring, logging, and audit systems that need visibility into an entire domain.

```go
// matchTopic checks if a subscription pattern matches a published topic.
// Supports:
//   - Exact match:   "order.created" matches "order.created"
//   - Single wildcard: "order.*" matches "order.created" but not "order.us.created"
//   - Multi wildcard: "order.>" matches "order.created" and "order.us.created"
func matchTopic(pattern, topic string) bool {
    if pattern == topic {
        return true
    }

    patternParts := strings.Split(pattern, ".")
    topicParts := strings.Split(topic, ".")

    for i, part := range patternParts {
        if part == ">" {
            return true // matches everything from here
        }
        if i >= len(topicParts) {
            return false
        }
        if part == "*" {
            continue // matches any single segment
        }
        if part != topicParts[i] {
            return false
        }
    }

    return len(patternParts) == len(topicParts)
}
```

This follows NATS-style topic matching:
- `order.created` -- exact match only
- `order.*` -- matches `order.created`, `order.shipped` (one level)
- `order.>` -- matches `order.created`, `order.us.east.created` (any depth)

The publish method needs to check every subscription pattern against the published topic:

```go
func (b *Broker) Publish(topic string, msg Message) {
    b.mu.RLock()
    defer b.mu.RUnlock()

    msg.Topic = topic
    msg.Timestamp = time.Now()

    for pattern, subs := range b.subscribers {
        if matchTopic(pattern, topic) {
            for _, ch := range subs {
                select {
                case ch <- msg:
                default:
                    // dropped
                }
            }
        }
    }
}
```

**Performance note:** Iterating all patterns on every publish is O(patterns * subscribers). For a handful of patterns, this is fine. NATS and Kafka use trie-based routing for thousands of patterns. For an in-process broker, the simple approach works until you measure otherwise.

### Your notes
<!-- -->

---

## Fan-Out and Fan-In

Two fundamental patterns emerge from Pub/Sub:

### Fan-Out: One Publisher, Many Subscribers

A single event goes to multiple consumers. Each consumer processes independently. This is the natural behavior of Pub/Sub -- every subscriber to a topic gets every message.

```go
// Fan-out: one publisher, three subscribers
ch1 := broker.Subscribe("events", 10)
ch2 := broker.Subscribe("events", 10)
ch3 := broker.Subscribe("events", 10)

broker.Publish("events", Message{Payload: []byte("hello")})
// All three channels receive the message
```

**Use cases**: Audit logging alongside business logic, sending notifications to multiple channels (email + Slack + PagerDuty), replicating data to multiple downstream services.

### Fan-In: Many Publishers, One Subscriber

Multiple producers funnel into a single processing pipeline. In Go, this is typically done by having multiple goroutines publish to the same topic, with a single subscriber consuming.

```go
// Fan-in: multiple publishers, one subscriber
results := broker.Subscribe("results", 100)

// Launch workers that all publish to "results"
for i := 0; i < 10; i++ {
    go func(workerID int) {
        // ... do work ...
        broker.Publish("results", Message{
            Payload: []byte(fmt.Sprintf("result from worker %d", workerID)),
        })
    }(i)
}

// Single consumer aggregates all results
for msg := range results {
    fmt.Println(string(msg.Payload))
}
```

**Use cases**: Aggregating results from parallel workers, collecting metrics from multiple sources, merging log streams.

### Your notes
<!-- -->

---

## Backpressure and Flow Control

Backpressure is what happens when a subscriber can't keep up with the publisher's rate. Go's buffered channels give you several strategies:

### Strategy 1: Block the Publisher (Unbuffered/Full Buffer)

```go
ch := make(chan Message)      // unbuffered: publisher blocks every send
ch := make(chan Message, 10)  // buffered: publisher blocks when buffer is full
```

**Tradeoff**: Guarantees delivery but the slowest subscriber controls the publishing rate. One slow subscriber blocks all publishers.

### Strategy 2: Drop Messages (Non-Blocking Send)

```go
select {
case ch <- msg:
    // delivered
default:
    // dropped -- subscriber too slow
    droppedCount.Add(1)
}
```

**Tradeoff**: Publishers never block, but subscribers miss messages. Good for metrics, monitoring, and situations where the latest value matters more than every value.

### Strategy 3: Drop Oldest (Ring Buffer)

```go
select {
case ch <- msg:
default:
    // Channel full: drain one, then send
    <-ch       // discard oldest
    ch <- msg  // send newest
}
```

**Tradeoff**: Subscribers always get the most recent messages. Useful for real-time dashboards where stale data is worse than gaps.

### Strategy 4: Timeout

```go
select {
case ch <- msg:
    // delivered
case <-time.After(100 * time.Millisecond):
    // subscriber didn't read in time
    log.Printf("delivery timeout for subscriber")
}
```

**Tradeoff**: Bounded latency but messages can still be lost. Good for latency-sensitive systems.

> **Key Insight**: There is no "right" backpressure strategy. It depends on your delivery semantics. Audit logs need at-least-once (block or retry). Metrics dashboards can tolerate drops. Real-time feeds want the latest data. Choose based on what your subscribers need, not what's easiest to implement.

### Your notes
<!-- -->

---

## Delivery Semantics

Understanding delivery guarantees is critical for choosing the right approach:

| Semantic | Guarantee | Go Implementation | When to Use |
|----------|-----------|-------------------|-------------|
| **At-most-once** | Message delivered 0 or 1 times | Non-blocking `select` with `default` | Metrics, monitoring, non-critical notifications |
| **At-least-once** | Message delivered 1 or more times | Blocking send + retry on failure | Payment processing, audit logs, order fulfillment |
| **Exactly-once** | Message delivered exactly 1 time | Requires idempotency keys + deduplication | Financial transactions, inventory updates |

In-process Pub/Sub with channels typically provides **at-most-once** (with non-blocking send) or **at-least-once** (with blocking send + acknowledgment). True exactly-once requires infrastructure-level support (Kafka transactions, etc.) and is beyond what an in-process broker can provide.

For at-least-once delivery, subscribers need to **acknowledge** messages:

```go
type AckMessage struct {
    Message
    ack chan struct{}
}

func (m *AckMessage) Ack() {
    close(m.ack) // signal to broker that message was processed
}

// Broker waits for ack or redelivers
func (b *Broker) publishWithAck(topic string, msg Message, timeout time.Duration) error {
    ack := make(chan struct{})
    amsg := AckMessage{Message: msg, ack: ack}

    // Send to subscriber
    ch <- amsg

    // Wait for ack
    select {
    case <-ack:
        return nil
    case <-time.After(timeout):
        return fmt.Errorf("delivery not acknowledged within %v", timeout)
    }
}
```

### Your notes
<!-- -->

---

## Go Channel Patterns for Pub/Sub

### Multiplexing with Select

The `select` statement is how subscribers listen to multiple topics or handle shutdown signals:

```go
func processEvents(
    orders <-chan Message,
    payments <-chan Message,
    done <-chan struct{},
) {
    for {
        select {
        case msg := <-orders:
            handleOrder(msg)
        case msg := <-payments:
            handlePayment(msg)
        case <-done:
            fmt.Println("shutting down")
            return
        }
    }
}
```

### Context Cancellation for Graceful Shutdown

Use `context.Context` to propagate cancellation through the entire Pub/Sub system:

```go
func (b *Broker) Start(ctx context.Context) {
    go func() {
        <-ctx.Done()
        b.Close() // close all subscriber channels
    }()
}

func subscriber(ctx context.Context, ch <-chan Message) {
    for {
        select {
        case msg, ok := <-ch:
            if !ok {
                return // channel closed, broker shut down
            }
            process(msg)
        case <-ctx.Done():
            return // context cancelled
        }
    }
}
```

### Done Channels for Signaling

A `done` channel is a simple coordination primitive. Closing it broadcasts to all listeners:

```go
done := make(chan struct{})

// Multiple goroutines wait on done
for i := 0; i < 5; i++ {
    go func() {
        <-done  // blocks until done is closed
        fmt.Println("received shutdown signal")
    }()
}

// Signal all goroutines to stop
close(done) // closing a channel unblocks ALL receivers
```

This is how you implement graceful shutdown: close the `done` channel, wait for all goroutines to finish (via `sync.WaitGroup`), then clean up resources.

### Your notes
<!-- -->

---

## Cross-Language Comparison

### TypeScript: EventEmitter and RxJS

TypeScript's EventEmitter is essentially an in-process Pub/Sub:

```typescript
import { EventEmitter } from 'events';

const bus = new EventEmitter();

// Subscribe
bus.on('order.created', (data) => {
    console.log('New order:', data);
});

// Publish
bus.emit('order.created', { orderId: '123' });
```

RxJS provides a more sophisticated approach with `Subject` (multicast) and operators for backpressure:

```typescript
import { Subject } from 'rxjs';
import { filter, bufferTime } from 'rxjs/operators';

const events$ = new Subject<Event>();

// Subscribe with filtering and batching
events$.pipe(
    filter(e => e.type === 'order.created'),
    bufferTime(1000), // batch events over 1 second
).subscribe(batch => processBatch(batch));

// Publish
events$.next({ type: 'order.created', data: { orderId: '123' } });
```

**Key difference**: Node.js EventEmitter is synchronous (callbacks run in the same tick). Go channels are inherently concurrent. RxJS adds async operators on top of the synchronous Subject.

### Rust: tokio::sync::broadcast and crossbeam

Rust's `tokio::sync::broadcast` is the closest analog to Go's channel-based Pub/Sub:

```rust
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel(100); // buffer size 100
let mut rx2 = tx.subscribe(); // second subscriber

// Publish
tx.send("order.created").unwrap();

// Subscribe
let msg = rx1.recv().await.unwrap(); // "order.created"
let msg = rx2.recv().await.unwrap(); // "order.created"
```

For synchronous (non-async) Pub/Sub, `crossbeam::channel` provides similar semantics to Go channels with `select!` macro.

**Key difference**: Rust requires choosing between `async` (tokio) and synchronous (crossbeam) channels. Go channels work in both synchronous and concurrent contexts without different types.

### Your notes
<!-- -->

---

## Real-World Systems

### Redis Pub/Sub

Redis provides a lightweight Pub/Sub where publishers and subscribers connect to a Redis server:

```
SUBSCRIBE order.created
PUBLISH order.created '{"order_id": "123"}'
```

**Limitation**: Fire-and-forget. If a subscriber is disconnected when a message is published, it misses that message. No persistence, no replay. Good for real-time notifications; bad for critical business events.

### NATS

NATS is a Go-native messaging system with subject-based routing:

```go
nc, _ := nats.Connect("nats://localhost:4222")

// Subscribe with wildcard
nc.Subscribe("order.*", func(m *nats.Msg) {
    fmt.Println("Received:", string(m.Data))
})

// Publish
nc.Publish("order.created", []byte(`{"order_id": "123"}`))
```

NATS supports at-most-once (core NATS) and at-least-once (JetStream). It uses the same wildcard syntax we implemented (`*` for single segment, `>` for multi-segment).

### Kafka

Kafka is a distributed commit log with consumer groups. Unlike Pub/Sub where messages are pushed to subscribers, Kafka subscribers *pull* from a topic at their own pace. Messages are persisted and can be replayed.

**Conceptual connection**: Our in-process broker is to Kafka what a linked list is to a database. Same conceptual model (topics, publish, subscribe), vastly different operational characteristics. Building the in-process version teaches you the mental model that transfers to any message broker.

### Your notes
<!-- -->

---

## Anti-Patterns

### 1. Unbounded Channels

```go
// WRONG: Unbounded channel can grow forever
ch := make(chan Message, 1_000_000) // "big buffer = no problems" is not a strategy
```

A large buffer just delays the inevitable. If your subscriber is slower than your publisher, the buffer fills up eventually. Choose a backpressure strategy explicitly.

### 2. No Error Handling on Publish

```go
// WRONG: Ignoring publish failures silently
func (b *Broker) Publish(topic string, msg Message) {
    for _, ch := range b.subscribers[topic] {
        ch <- msg // blocks forever if subscriber is stuck
    }
}
```

At minimum, log dropped messages. Better: return delivery results so the caller can decide what to do.

### 3. Zombie Subscribers

```go
// WRONG: Subscriber crashes but channel is never cleaned up
go func() {
    for msg := range ch {
        if err := process(msg); err != nil {
            return // goroutine exits, but channel still registered with broker
        }
    }
}()
```

The channel stays in the broker's subscriber map, consuming buffer space and blocking publishes. Always unsubscribe on exit, using `defer`.

### 4. Publishing Inside a Subscribe Handler on the Same Topic

```go
// WRONG: If the broker uses a non-reentrant mutex, this deadlocks
broker.Subscribe("events", func(msg Message) {
    // This calls Publish, which tries to acquire the same lock
    broker.Publish("events", transformedMsg)
})
```

This is a classic deadlock: the subscribe handler holds a lock, and Publish tries to acquire the same lock. Solutions: use `RWMutex` (read lock for publish, write lock for subscribe), use channels instead of callbacks, or make publish asynchronous.

### 5. No Graceful Shutdown

```go
// WRONG: Just stopping the publisher leaves subscribers hanging
func (b *Broker) Close() {
    b.closed = true
    // What about messages in flight?
    // What about subscribers blocked on channel reads?
}
```

Proper shutdown: stop accepting new publishes, drain in-flight messages, close all subscriber channels (which unblocks `range` loops), wait for subscriber goroutines to exit.

### Your notes
<!-- -->

---

## Summary

| Concept | Key Takeaway |
|---------|-------------|
| Pub/Sub vs Observer | Pub/Sub adds a broker for deeper decoupling; Observer is direct |
| Topics | Named channels for routing messages to interested subscribers |
| Wildcards | `*` (single segment) and `>` (multi-segment) for flexible subscriptions |
| Fan-out | One message to many subscribers (the default behavior) |
| Fan-in | Many publishers into one subscriber (aggregate results) |
| Backpressure | Block, drop, ring buffer, or timeout -- choose based on delivery needs |
| Delivery semantics | At-most-once (drop), at-least-once (retry), exactly-once (dedup) |
| Go channels | Natural fit for Pub/Sub: buffering, select, context cancellation |
| Graceful shutdown | Close done channels, drain in-flight messages, close subscriber channels |

**Next**: Work through the exercises to build a production-grade event bus, debug common channel pitfalls, and review a Pub/Sub implementation.

### Your notes
<!-- -->
