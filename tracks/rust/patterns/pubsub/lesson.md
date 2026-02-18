# Publish/Subscribe Pattern — Rust

## The Problem: Decoupled Communication at Scale

The Observer pattern gives you one-to-many notification: a subject directly notifies all its observers. But what happens when publishers should not know about subscribers at all? When you need topic-based routing? When publishers and subscribers have completely independent lifecycles?

Consider a distributed metrics pipeline. Multiple services emit metric events — counters, gauges, histograms. Multiple consumers process those metrics differently:
- A dashboard aggregator collects gauge values
- An alerting system watches counter thresholds
- An audit logger writes every event to disk
- A billing system tracks API usage counters

The Observer pattern would require each service to maintain a list of subscribers. Pub/Sub introduces a **broker** — an intermediary that routes messages by topic. Publishers emit to topics. Subscribers subscribe to topics. Neither side knows the other exists.

In JavaScript, this distinction barely matters — `EventEmitter` blurs Observer and Pub/Sub because events *are* topics and `.on('event-name', callback)` is topic-based subscription. In Go, you typically build a broker with a `map[string][]chan Event` — topic to channel slice. In Rust, the ownership system shapes the design in specific ways that reveal the mechanics of message passing.

### Your notes
<!-- -->


---

## Pub/Sub vs Observer: The Distinction That Matters

The difference is architectural, not semantic:

| Aspect | Observer | Pub/Sub |
|--------|----------|---------|
| **Coupling** | Subject knows it has observers (holds references) | Publisher has no knowledge of subscribers |
| **Intermediary** | None — direct notification | Broker routes messages |
| **Addressing** | Object identity (subscribe to *this* subject) | Topic/channel (subscribe to *"metrics.cpu"*) |
| **Lifecycle** | Observer lifetime tied to subject | Publisher and subscriber lifetimes independent |
| **Fan-out** | One subject to many observers | Many publishers to many subscribers via broker |
| **Filtering** | Observer receives all events from subject | Subscriber receives only events from subscribed topics |

In Rust's Observer lesson, we built the `EventBus<E>` with closures and the channel-based `MetricPublisher`. Those are already close to Pub/Sub — particularly the channel approach. The formal Pub/Sub pattern adds:

1. **A standalone broker** that both publishers and subscribers interact with (neither interacts with the other)
2. **Topic-based routing** via `HashMap<Topic, Vec<Sender<Message>>>`
3. **Thread-safe shared state** via `Arc<Mutex<BrokerState>>` so the broker can be used from multiple threads
4. **Lifecycle independence** — publishers can come and go without affecting subscribers, and vice versa

The broker is the key abstraction. Everything else flows from it.

### Your notes
<!-- -->


---

## Channel-Based Pub/Sub with `std::sync::mpsc`

The most idiomatic Rust Pub/Sub uses `mpsc` channels. Each subscriber gets its own `Receiver<Message>`. The broker maintains a `HashMap<Topic, Vec<Sender<Message>>>` and fans out messages to the appropriate channels.

```rust
use std::collections::HashMap;
use std::sync::mpsc;

type Topic = String;

#[derive(Clone, Debug)]
struct Message {
    topic: Topic,
    payload: String,
}

type SubscriptionId = u64;

struct Broker {
    next_id: SubscriptionId,
    // topic -> list of (id, sender) pairs
    subscribers: HashMap<Topic, Vec<(SubscriptionId, mpsc::Sender<Message>)>>,
}

impl Broker {
    fn new() -> Self {
        Broker {
            next_id: 0,
            subscribers: HashMap::new(),
        }
    }

    fn subscribe(&mut self, topic: &str) -> (SubscriptionId, mpsc::Receiver<Message>) {
        let (tx, rx) = mpsc::channel();
        let id = self.next_id;
        self.next_id += 1;

        self.subscribers
            .entry(topic.to_string())
            .or_default()
            .push((id, tx));

        (id, rx)
    }

    fn publish(&mut self, topic: &str, payload: String) {
        let msg = Message {
            topic: topic.to_string(),
            payload,
        };

        if let Some(subs) = self.subscribers.get_mut(topic) {
            // retain only subscribers whose channels are still open
            subs.retain(|(_, tx)| tx.send(msg.clone()).is_ok());
        }
    }

    fn unsubscribe(&mut self, topic: &str, id: SubscriptionId) {
        if let Some(subs) = self.subscribers.get_mut(topic) {
            subs.retain(|(sub_id, _)| *sub_id != id);
        }
    }
}
```

This is structurally identical to Go's `map[string][]chan Event` approach. The key difference: in Rust, sending to a dropped receiver returns `Err`, which we use in `retain` to prune dead subscribers. In Go, sending to a closed channel panics unless you use a select with a done channel.

**Ownership flow:**
- The broker owns the `Sender` ends
- Each subscriber owns its `Receiver` end
- When a subscriber drops its `Receiver`, the next `publish` call detects the broken channel and removes that subscriber
- Messages are `Clone`d for each subscriber — we will address this cost later

### Your notes
<!-- -->


---

## Making the Broker Thread-Safe with `Arc<Mutex<>>`

A single-threaded broker is useful for learning, but production Pub/Sub involves multiple threads publishing and subscribing concurrently. Wrap the broker in `Arc<Mutex<>>`:

```rust
use std::sync::{Arc, Mutex};

let broker = Arc::new(Mutex::new(Broker::new()));

// Publisher thread
let broker_pub = broker.clone();
std::thread::spawn(move || {
    let mut b = broker_pub.lock().unwrap();
    b.publish("metrics.cpu", "usage=78.5".to_string());
});

// Subscriber thread — subscribe first, then process
let broker_sub = broker.clone();
let rx = {
    let mut b = broker_sub.lock().unwrap();
    let (_, rx) = b.subscribe("metrics.cpu");
    rx
};

std::thread::spawn(move || {
    while let Ok(msg) = rx.recv() {
        println!("Received: {:?}", msg);
    }
});
```

The critical insight: **lock the broker briefly to subscribe or publish, then release it**. The subscriber's `Receiver` works independently of the broker lock. You never hold the broker lock while waiting for messages — that would deadlock.

This is where Rust's ownership model shines. The `Receiver` is *moved* into the subscriber thread. There is no possibility of two threads sharing it (unlike Go, where you could accidentally share a channel receiver across goroutines without explicit synchronization).

### Your notes
<!-- -->


---

## Ownership Challenges: Cloning vs `Arc<Message>`

When publishing to N subscribers, the naive approach clones the message N times:

```rust
fn publish(&mut self, msg: Message) {
    for sub in &self.subscribers {
        sub.send(msg.clone()).ok(); // N clones for N subscribers
    }
}
```

For small messages (a few strings, some numbers), cloning is fine. For large messages (a serialized JSON payload, a large buffer, an image thumbnail), cloning is wasteful.

**Solution: wrap the message in `Arc`:**

```rust
use std::sync::Arc;

#[derive(Debug)]
struct LargeMessage {
    topic: String,
    payload: Vec<u8>, // Could be megabytes
}

type SharedMessage = Arc<LargeMessage>;

struct Broker {
    subscribers: HashMap<String, Vec<mpsc::Sender<SharedMessage>>>,
}

impl Broker {
    fn publish(&mut self, msg: LargeMessage) {
        let shared = Arc::new(msg); // One allocation
        if let Some(subs) = self.subscribers.get_mut(&shared.topic) {
            subs.retain(|tx| tx.send(shared.clone()).is_ok()); // Arc::clone is cheap
        }
    }
}
```

`Arc::clone` increments a reference count — it does not copy the data. This is the Rust equivalent of Go passing a pointer through a channel, but with guaranteed memory safety: the message is deallocated when the last subscriber finishes processing it.

**When to use which:**

| Message Size | Strategy | Why |
|-------------|----------|-----|
| Small (< 1KB, simple structs) | `Clone` | Clone is cheap, simpler code |
| Medium (1KB-100KB) | `Arc<Message>` | Avoids redundant allocation |
| Large (> 100KB) | `Arc<Message>` | Mandatory — cloning would dominate CPU time |
| Message needs mutation by subscriber | `Clone` | Each subscriber needs its own copy |

In TypeScript, this is never a concern — objects are always passed by reference. In Go, passing a pointer through a channel is the default. In Rust, you must make the choice explicit, which prevents accidental data races but adds design friction.

### Your notes
<!-- -->


---

## Type-Safe Topics with Enums

Using `String` for topics is flexible but error-prone. A typo in a topic name silently drops messages. Rust's enum system gives you compile-time topic verification:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum MetricTopic {
    CpuUsage,
    MemoryUsage,
    DiskIO,
    NetworkLatency,
    RequestCount,
}

#[derive(Clone, Debug)]
enum MetricEvent {
    Counter { name: String, value: u64 },
    Gauge { name: String, value: f64 },
    Histogram { name: String, values: Vec<f64> },
}

struct TypedBroker {
    subscribers: HashMap<MetricTopic, Vec<mpsc::Sender<MetricEvent>>>,
}
```

Now `broker.subscribe(MetricTopic::CpuUsage)` is compile-time checked. You cannot subscribe to a topic that does not exist. If you add a new topic variant, `match` statements that handle topics will produce compiler warnings if they are not exhaustive.

Subscribers can pattern-match on the event type:

```rust
while let Ok(event) = rx.recv() {
    match event {
        MetricEvent::Counter { name, value } => {
            println!("Counter {}: {}", name, value);
        }
        MetricEvent::Gauge { name, value } => {
            if value > 90.0 {
                alert(&name, value);
            }
        }
        MetricEvent::Histogram { name, values } => {
            let p99 = percentile(&values, 99);
            report_latency(&name, p99);
        }
    }
}
```

This is something neither Go nor JavaScript can do at the type level. Go's channel-based pub/sub uses `interface{}` or generics (since 1.18). JavaScript's `EventEmitter` uses strings. Rust's enum-based approach catches routing errors at compile time.

### Your notes
<!-- -->


---

## Backpressure with Bounded Channels

`mpsc::channel()` creates an unbounded channel. A slow subscriber causes the channel to grow without limit, consuming memory until the process crashes. This is a real production issue — the publisher has no way to know the subscriber is falling behind.

`mpsc::sync_channel(bound)` creates a bounded channel. When the buffer is full, `send()` **blocks** until the subscriber consumes a message:

```rust
use std::sync::mpsc;

// Bounded channel: max 100 messages buffered
let (tx, rx) = mpsc::sync_channel::<Message>(100);

// When the buffer is full, this blocks
tx.send(msg).unwrap(); // Blocks if 100 messages are buffered

// Non-blocking alternative: try_send
match tx.try_send(msg) {
    Ok(()) => { /* sent */ }
    Err(mpsc::TrySendError::Full(returned_msg)) => {
        // Buffer full — decide: drop, retry, or buffer elsewhere
        eprintln!("Subscriber lagging, dropping message");
    }
    Err(mpsc::TrySendError::Disconnected(_)) => {
        // Subscriber gone
    }
}
```

**Backpressure strategies:**

| Strategy | Implementation | When to Use |
|----------|---------------|-------------|
| **Block** | `sync_channel(n)` + `send()` | Publisher can afford to wait |
| **Drop newest** | `try_send()`, discard on `Full` | Latest data matters most (metrics) |
| **Drop oldest** | Custom ring buffer or `try_send` + re-receive | Dashboard/display updates |
| **Overflow to disk** | `try_send()`, write to file on `Full` | Audit logs, compliance data |
| **Notify publisher** | Return `Err` to publisher | Publisher needs to adapt behavior |

In Go, buffered channels (`make(chan Event, 100)`) provide the same mechanism. The difference: Go's `select` with `default` gives you non-blocking send, while Rust uses `try_send()`. Both are explicit about what happens when the buffer is full.

In JavaScript, there is no built-in backpressure for `EventEmitter` — it is entirely push-based. Node.js streams have backpressure, but the event system does not. This is one of the recurring pain points in high-throughput Node.js systems.

### Your notes
<!-- -->


---

## Graceful Shutdown

Shutdown is the hardest part of any concurrent Pub/Sub system. You need to:
1. Stop accepting new messages
2. Drain in-flight messages
3. Wait for all subscribers to finish processing
4. Clean up resources

Rust's ownership model actually helps here: when you drop all `Sender` ends, every `Receiver::recv()` returns `Err(RecvError)`, signaling that the channel is closed. This is the Rust equivalent of Go's `close(ch)`.

```rust
use std::sync::mpsc;
use std::thread;

struct ShutdownBroker {
    subscribers: Vec<mpsc::Sender<String>>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl ShutdownBroker {
    fn subscribe<F>(&mut self, handler: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);

        let handle = thread::spawn(move || {
            let mut handler = handler;
            while let Ok(msg) = rx.recv() {
                handler(msg);
            }
            // recv() returned Err — channel closed, subscriber exits
        });

        self.handles.push(handle);
    }

    fn publish(&self, msg: String) {
        for tx in &self.subscribers {
            let _ = tx.send(msg.clone());
        }
    }

    fn shutdown(self) {
        // Step 1: Drop all senders — this signals subscribers to stop
        drop(self.subscribers);

        // Step 2: Wait for all subscriber threads to finish processing
        for handle in self.handles {
            handle.join().expect("subscriber thread panicked");
        }
    }
}
```

The `shutdown` method consumes `self` (takes ownership), which means you cannot accidentally use the broker after shutting it down. The compiler enforces it. Compare this to Go, where you might accidentally send on a closed channel and panic at runtime.

The `Drop` trait can automate this:

```rust
impl Drop for ShutdownBroker {
    fn drop(&mut self) {
        // Senders are dropped automatically when ShutdownBroker is dropped.
        // But we cannot join handles from Drop because join() takes ownership
        // of the JoinHandle, and Drop only has &mut self.
        //
        // Solution: use Option<JoinHandle> and take() from the Option.
    }
}
```

This is a real design tension in Rust: `Drop` gives you `&mut self`, but `JoinHandle::join()` takes ownership. The standard workaround is storing `Option<JoinHandle<()>>` and calling `.take()` in the drop implementation.

### Your notes
<!-- -->


---

## Comparison: Rust vs Go vs TypeScript

| Aspect | Rust | Go | TypeScript |
|--------|------|----|-----------|
| **Broker state** | `Arc<Mutex<HashMap<Topic, Vec<Sender>>>>` | `sync.RWMutex` + `map[string][]chan Event` | `Map<string, Set<callback>>` |
| **Fan-out** | Clone message or `Arc` per subscriber | Send pointer through channel | Direct callback invocation |
| **Backpressure** | `sync_channel(n)` / `try_send` | Buffered channel / `select default` | None built-in |
| **Topic type safety** | Enums with exhaustive matching | Strings or `iota` constants | Strings or union types |
| **Dead subscriber** | `send()` returns `Err` | Recover from panic on closed channel | Manual `removeListener` |
| **Shutdown** | Drop senders, join handles | Close channels, `WaitGroup` | Clear listeners, resolve promises |
| **Thread safety** | Compiler-enforced (`Send`/`Sync`) | Runtime (race detector) | Single-threaded event loop |
| **Message ownership** | Explicit (clone vs Arc vs move) | Implicit (pointer sharing) | Implicit (reference sharing) |

Go's pub/sub is more concise because channels and goroutines are language primitives. Rust's is more explicit — you decide clone vs Arc, bounded vs unbounded, which thread owns what. TypeScript's is simplest because everything is single-threaded and garbage-collected, but it lacks backpressure and thread isolation.

The fundamental insight: **Pub/Sub is a shared mutable state problem.** The broker is shared state. Publishing mutates subscriber queues. Subscribing mutates the broker's registry. Rust forces you to handle every one of these mutation points explicitly. Go trusts you to use channels correctly. TypeScript avoids the problem by being single-threaded.

### Your notes
<!-- -->


---

## Connection to Other Patterns

- **Observer** is the foundation. Pub/Sub adds a broker and topic-based routing. If you only have one publisher and direct subscriber references, you have Observer. Once you add an intermediary, you have Pub/Sub.

- **Mediator** centralizes all communication logic. Pub/Sub's broker is a specific kind of mediator — it routes messages by topic rather than encoding arbitrary coordination logic.

- **Event Sourcing** pairs naturally with Pub/Sub. Instead of just delivering events, you persist them. Subscribers can replay the event log to reconstruct state. The broker becomes an event store.

- **CQRS (Command Query Responsibility Segregation)** uses Pub/Sub to propagate state changes from the write side to the read side. Commands produce events; read models subscribe to those events.

- **Message Queue** (RabbitMQ, Kafka, NATS) is Pub/Sub at the distributed system level. The in-process broker we built here is the single-process equivalent. The patterns are identical; the transport layer changes.

### Your notes
<!-- -->
