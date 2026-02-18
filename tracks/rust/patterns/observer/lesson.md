# Observer Pattern — Rust

## The Problem: Decoupled Notification

The Observer pattern solves a fundamental coupling problem: **how does one component notify others about state changes without knowing who those others are?**

Consider a monitoring system. When a service health check fails, you need to:
- Send a Slack alert
- Increment a Prometheus counter
- Write to an audit log
- Trigger a PagerDuty escalation

Without Observer, the health checker imports and calls each of these directly. Adding a new notification channel means modifying the health checker. That is tight coupling in its most recognizable form.

The Observer pattern inverts this: the health checker publishes events. Observers subscribe. The health checker never imports Slack, Prometheus, or PagerDuty — it only knows about an abstract "subscriber" interface.

In JavaScript, you know this as `EventEmitter`. In Go, you typically reach for channels. In Rust, the ownership system makes the classic OOP Observer pattern fundamentally difficult to implement — and this difficulty reveals something important about the pattern itself.

### Your notes
<!-- -->


---

## Why Classic OOP Observer Breaks in Rust

In Java or C#, the textbook Observer looks like this:

```java
// Java: The classic approach
interface Observer {
    void update(Event event);
}

class Subject {
    List<Observer> observers = new ArrayList<>();

    void subscribe(Observer o) { observers.add(o); }
    void notify(Event e) {
        for (Observer o : observers) o.update(e);
    }
}
```

Every observer holds a reference to the subject (to unsubscribe), and the subject holds references to all observers. This bidirectional reference graph is trivial in garbage-collected languages.

In Rust, it is a nightmare:

```rust
// DOES NOT COMPILE — illustrating the ownership conflict

trait Observer {
    fn on_event(&self, event: &str);
}

struct Subject {
    observers: Vec<Box<dyn Observer>>,
}

impl Subject {
    fn subscribe(&mut self, observer: Box<dyn Observer>) {
        self.observers.push(observer);
    }

    fn notify(&self, event: &str) {
        for obs in &self.observers {
            obs.on_event(event);
        }
    }
}
```

This *compiles*, but the ownership model creates friction the moment you try to use it:

1. **The subject owns the observers.** Once you pass `Box<dyn Observer>` to `subscribe`, the caller loses access to that observer. You cannot hold a reference to the observer *and* give it to the subject.

2. **Mutable notification problem.** If an observer needs to update its own state when notified (incrementing a counter, writing to a buffer), it needs `&mut self`. But during `notify`, the subject is iterating over `&self.observers` — you cannot hand out `&mut` references from within a shared borrow.

3. **Circular references.** If observers need a reference back to the subject (to unsubscribe or query state), you get circular ownership. Rust has no garbage collector to break the cycle.

4. **Lifetime entanglement.** Storing `&dyn Observer` references means the subject's lifetime is tied to every observer's lifetime. This is almost always unworkable in practice.

These are not limitations of Rust — they are Rust making explicit the problems that garbage-collected languages sweep under the rug. The classic Observer pattern *is* a shared mutable state problem. Rust just forces you to confront it.

### Your notes
<!-- -->


---

## Idiomatic Rust Approaches

Rust offers several approaches to the Observer pattern, each with different tradeoffs. The right choice depends on whether you need synchronous or asynchronous notification, whether observers need mutable state, and whether the system is single-threaded or concurrent.

### Approach 1: Callback Closures with `Box<dyn Fn>`

The simplest approach stores closures instead of trait objects. This sidesteps the "observer needs its own state" problem by letting closures capture their environment.

```rust
use std::collections::HashMap;

type SubscriptionId = u64;

struct EventBus<E> {
    next_id: SubscriptionId,
    listeners: HashMap<SubscriptionId, Box<dyn Fn(&E)>>,
}

impl<E> EventBus<E> {
    fn new() -> Self {
        EventBus {
            next_id: 0,
            listeners: HashMap::new(),
        }
    }

    fn subscribe(&mut self, callback: Box<dyn Fn(&E)>) -> SubscriptionId {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.insert(id, callback);
        id
    }

    fn unsubscribe(&mut self, id: SubscriptionId) {
        self.listeners.remove(&id);
    }

    fn emit(&self, event: &E) {
        for callback in self.listeners.values() {
            callback(event);
        }
    }
}
```

**How closures capture state:**

```rust
use std::sync::{Arc, Mutex};

let alert_count = Arc::new(Mutex::new(0u64));
let counter = alert_count.clone();

bus.subscribe(Box::new(move |event: &HealthEvent| {
    // The closure *owns* the Arc clone. No borrowing conflict.
    let mut count = counter.lock().unwrap();
    *count += 1;
    println!("Alert #{}: {:?}", count, event);
}));

// alert_count is still usable here — we cloned the Arc
println!("Total alerts: {}", alert_count.lock().unwrap());
```

The key insight: the closure *owns* its captured state (or owns an `Arc` clone to shared state). No borrowing conflicts. No lifetime issues.

**Tradeoffs:**
- Simple, no trait boilerplate
- Closures are opaque — you cannot inspect or compare them
- Unsubscription requires keeping the `SubscriptionId`
- Not `Send`/`Sync` unless you use `Box<dyn Fn(&E) + Send + Sync>`

If you are coming from JavaScript, this is the closest to `emitter.on('event', callback)`.

### Your notes
<!-- -->


---

### Approach 2: Channel-Based Observer (`mpsc`)

Channels naturally decouple producers from consumers. The subject sends events; each observer has its own receiving end. This is the most Go-like approach.

```rust
use std::sync::mpsc;
use std::thread;

#[derive(Clone, Debug)]
struct MetricEvent {
    name: String,
    value: f64,
}

struct MetricPublisher {
    subscribers: Vec<mpsc::Sender<MetricEvent>>,
}

impl MetricPublisher {
    fn new() -> Self {
        MetricPublisher {
            subscribers: Vec::new(),
        }
    }

    fn subscribe(&mut self) -> mpsc::Receiver<MetricEvent> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn publish(&self, event: MetricEvent) {
        // Retain only live senders — if a receiver was dropped, send() fails
        self.subscribers.iter().for_each(|tx| {
            let _ = tx.send(event.clone()); // Ignore errors from dropped receivers
        });
    }
}
```

Each subscriber runs on its own thread, processing events from its channel:

```rust
let mut publisher = MetricPublisher::new();

let rx1 = publisher.subscribe();
let rx2 = publisher.subscribe();

// Observer 1: Slack alerter
thread::spawn(move || {
    while let Ok(event) = rx1.recv() {
        println!("[Slack] {} = {}", event.name, event.value);
    }
});

// Observer 2: Metric aggregator
thread::spawn(move || {
    let mut total = 0.0;
    while let Ok(event) = rx2.recv() {
        total += event.value;
        println!("[Aggregator] running total = {}", total);
    }
});

publisher.publish(MetricEvent {
    name: "cpu_usage".into(),
    value: 78.5,
});
```

**Tradeoffs:**
- Natural concurrency — each observer is independent
- No shared mutable state between publisher and observers
- Events must be `Clone` (each subscriber gets its own copy)
- Back-pressure: `mpsc::channel()` is unbounded. Use `sync_channel(bound)` for bounded queues
- Dead subscribers are detected by failed `send()` — you can prune them
- No synchronous "all observers processed this event" guarantee

In Go, you would do this with a slice of `chan Event`. The Rust version is nearly identical, but ownership of the `Receiver` moves to the subscriber thread, which is cleaner than sharing a channel reference.

### Your notes
<!-- -->


---

### Approach 3: `Arc<Mutex<>>` Shared State

When observers need to mutate their own state during notification, and you want trait-based polymorphism, wrap observers in `Arc<Mutex<>>`.

```rust
use std::sync::{Arc, Mutex};

trait Observer: Send {
    fn on_event(&mut self, event: &str);
}

struct Subject {
    observers: Vec<Arc<Mutex<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        Subject { observers: Vec::new() }
    }

    fn subscribe(&mut self, observer: Arc<Mutex<dyn Observer>>) {
        self.observers.push(observer);
    }

    fn notify(&self, event: &str) {
        for obs in &self.observers {
            obs.lock().unwrap().on_event(event);
        }
    }
}
```

The caller retains an `Arc` clone, so both the subject and the caller can access the observer:

```rust
struct AlertCounter {
    count: u64,
}

impl Observer for AlertCounter {
    fn on_event(&mut self, event: &str) {
        self.count += 1;
        println!("Alert #{}: {}", self.count, event);
    }
}

let counter = Arc::new(Mutex::new(AlertCounter { count: 0 }));

let mut subject = Subject::new();
subject.subscribe(counter.clone());

subject.notify("disk_full");
subject.notify("memory_low");

println!("Total alerts: {}", counter.lock().unwrap().count);
// Total alerts: 2
```

**Tradeoffs:**
- Trait-based: observers implement a named trait with a well-defined contract
- Both subject and caller can access observer state
- `Mutex` overhead on every notification (lock/unlock per observer)
- **Deadlock risk**: if an observer's `on_event` tries to call back into the subject (re-entrant notification), you deadlock. This is the same problem as the GC languages, but Rust makes it a deadlock instead of a data race.
- Requires `Send` bound for multi-threaded use

### Your notes
<!-- -->


---

### Approach 4: `Weak<T>` References for Observer Cleanup

A persistent problem with the Observer pattern is lifecycle management: what happens when an observer is destroyed? In GC languages, the subject holds a strong reference, preventing the observer from being garbage collected. This causes memory leaks unless you carefully unsubscribe.

Rust's `Weak<T>` solves this cleanly: the subject holds weak references. When the observer is dropped (all strong references gone), the weak reference returns `None` on upgrade.

```rust
use std::sync::{Arc, Mutex, Weak};

trait Observer: Send {
    fn on_event(&mut self, event: &str);
}

struct WeakSubject {
    observers: Vec<Weak<Mutex<dyn Observer>>>,
}

impl WeakSubject {
    fn new() -> Self {
        WeakSubject { observers: Vec::new() }
    }

    fn subscribe(&mut self, observer: &Arc<Mutex<dyn Observer>>) {
        self.observers.push(Arc::downgrade(observer));
    }

    fn notify(&mut self, event: &str) {
        // Upgrade weak refs — dead observers return None
        self.observers.retain(|weak| {
            if let Some(strong) = weak.upgrade() {
                strong.lock().unwrap().on_event(event);
                true  // keep this observer
            } else {
                false // observer was dropped, remove it
            }
        });
    }
}
```

Usage:

```rust
let mut subject = WeakSubject::new();

{
    let logger = Arc::new(Mutex::new(LogObserver::new()));
    subject.subscribe(&logger);

    subject.notify("event_1"); // LogObserver receives this

    // logger goes out of scope here
}

subject.notify("event_2"); // LogObserver is gone — automatically cleaned up
```

**Tradeoffs:**
- Automatic cleanup of dead observers
- No manual unsubscribe needed (though you can still provide it)
- Subject never prevents observer from being dropped
- Slight overhead from `Weak::upgrade()` on every notification
- Observer must be wrapped in `Arc` by the caller

This is genuinely better than the GC-language approach, where you *must* remember to unsubscribe or leak memory forever. In Rust, the type system enforces cleanup.

### Your notes
<!-- -->


---

## Comparison: Rust vs Go vs JavaScript

| Aspect | Rust | Go | JavaScript |
|--------|------|----|-----------|
| **Typical approach** | Closures, channels, or `Arc<Mutex>` | Channels (`chan Event`) | `EventEmitter` / callbacks |
| **Ownership** | Explicit — must decide who owns observers | GC handles it | GC handles it |
| **Concurrency** | `Send`/`Sync` bounds enforce thread safety | Goroutines + channels | Single-threaded event loop |
| **Unsubscription** | ID-based removal, or `Weak` auto-cleanup | Close the channel | `removeListener` / `off` |
| **Mutable state in observer** | `Arc<Mutex<>>` or closure-captured `Mutex` | Just mutate (GC) | Just mutate (GC) |
| **Back-pressure** | `sync_channel(n)` for bounded | Buffered channels `make(chan, n)` | None (push-based) |
| **Type safety** | Generic event types, trait bounds | `interface{}` or generics (1.18+) | None (any event shape) |
| **Dead observer cleanup** | `Weak<T>` auto-detects | Channel close detection | Manual `removeListener` |

In Go, the idiomatic observer is a slice of channels. Each subscriber gets a channel, listens on it in a goroutine, and the publisher fans out by sending to all channels. Dead subscribers are detected when `send` on a closed channel panics (or you use a select with a done channel).

In JavaScript, `EventEmitter` gives you `on`, `off`, `once`, and `emit`. It is synchronous within a single tick — all listeners fire before `emit` returns. Node.js warns you at 11 listeners (memory leak detection).

Rust has no built-in `EventEmitter`. You build it from primitives (`Fn` closures, channels, `Arc<Mutex<>>`), or use a crate like `event-listener`, `tokio::sync::broadcast`, or `async-broadcast`.

### Your notes
<!-- -->


---

## Real-World Rust: Async Observer with Tokio

In production Rust, you are likely using `tokio`. The `tokio::sync::broadcast` channel is purpose-built for the observer pattern in async code.

```rust
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
struct AuditEvent {
    action: String,
    user_id: String,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    // broadcast channel — multiple receivers, each gets every message
    let (tx, _) = broadcast::channel::<AuditEvent>(100);

    // Subscriber 1: audit log writer
    let mut rx1 = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx1.recv().await {
            println!("[AuditLog] {} by {}", event.action, event.user_id);
        }
    });

    // Subscriber 2: compliance monitor
    let mut rx2 = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx2.recv().await {
            if event.action == "delete_user" {
                println!("[Compliance] ALERT: user deletion by {}", event.user_id);
            }
        }
    });

    // Publish events
    let _ = tx.send(AuditEvent {
        action: "login".into(),
        user_id: "usr_123".into(),
        timestamp: 1700000000,
    });

    let _ = tx.send(AuditEvent {
        action: "delete_user".into(),
        user_id: "admin_1".into(),
        timestamp: 1700000001,
    });

    // Give spawned tasks time to process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
```

`tokio::sync::broadcast` handles:
- Fan-out to multiple receivers (each gets every message)
- Bounded buffer (configurable capacity)
- Lagging receiver detection (if a receiver falls behind, it gets a `RecvError::Lagged` with the count of missed messages)
- Automatic cleanup when receivers are dropped

There is also `tokio::sync::watch` for a different use case: when you only care about the *latest* value, not every intermediate change (think: configuration reload, current leader election state).

```rust
use tokio::sync::watch;

let (tx, rx) = watch::channel("healthy".to_string());

// Observer only sees the latest value
tokio::spawn(async move {
    let mut rx = rx;
    while rx.changed().await.is_ok() {
        println!("Status changed to: {}", *rx.borrow());
    }
});

tx.send("degraded".to_string()).unwrap();
tx.send("unhealthy".to_string()).unwrap();
// Observer might only see "unhealthy" — intermediate values can be skipped
```

### Your notes
<!-- -->


---

## When to Use Which Approach

| Scenario | Approach | Why |
|----------|----------|-----|
| Simple event bus, single-threaded | `Box<dyn Fn>` closures | Minimal boilerplate, no locking |
| Multi-threaded, independent processing | `mpsc` channels | Natural concurrency, no shared state |
| Observers need trait-based polymorphism | `Arc<Mutex<dyn Observer>>` | Named contracts, inspectable observers |
| Observers have unpredictable lifetimes | `Weak<Mutex<dyn Observer>>` | Auto-cleanup, no leaks |
| Async runtime (tokio) | `broadcast::channel` | Built-in fan-out, back-pressure, lagging detection |
| Only care about latest state | `watch::channel` | Skips intermediates, efficient for config/status |

The closure approach is the default starting point. Reach for channels when you need concurrency. Reach for `Arc<Mutex<>>` when you need named observer traits. Reach for `Weak<T>` when observer lifetimes are unpredictable.

### Your notes
<!-- -->


---

## Common Pitfalls

1. **Deadlock during notification.** If an observer's callback calls back into the subject (e.g., to unsubscribe itself), you deadlock if the subject holds a lock during iteration. Solution: collect events, release the lock, then notify. Or use channels to decouple.

2. **Forgetting `Send + Sync` bounds.** `Box<dyn Fn(&E)>` is not `Send` by default. For multi-threaded use, you need `Box<dyn Fn(&E) + Send + Sync>`. The compiler will tell you, but it is easy to forget during design.

3. **Unbounded channels.** `mpsc::channel()` is unbounded. A slow observer causes memory to grow without limit. Use `sync_channel(capacity)` for back-pressure.

4. **Clone requirement for channels.** Events sent through channels must be `Clone` (each receiver gets its own copy). If your events contain large data, consider sending `Arc<Event>` instead.

5. **Notification ordering.** With channels and threads, you have no guarantee about which observer processes an event first. If ordering matters, use synchronous notification (closures or `Arc<Mutex>`).

6. **Observer leaks with strong references.** If the subject holds `Arc<Mutex<dyn Observer>>`, the observer cannot be dropped while the subject exists. Use `Weak<T>` if observer lifetime should be independent.

### Your notes
<!-- -->


---

## Connection to Other Patterns

The Observer pattern is the foundation for several higher-level patterns:

- **Publish/Subscribe (Pub/Sub)**: Observer with a message broker in between. The subject publishes to a topic; subscribers subscribe to topics. This decouples even further — publisher and subscriber never interact directly. In Rust, crates like `tokio::sync::broadcast` blur the line between Observer and Pub/Sub.

- **Event Sourcing**: Instead of notifying observers of the current state, you store every event. Observers can replay the event log to reconstruct state. This pairs with Observer — you can have both live notification and event replay.

- **Mediator**: When multiple objects need to coordinate (not just one-to-many notification), a Mediator centralizes the communication logic. Observer is one-to-many; Mediator is many-to-many with a central coordinator.

- **Reactor Pattern**: The async event loop in tokio is essentially the Reactor pattern — an event demultiplexer that dispatches events to registered handlers. The Observer pattern is a building block for this.

### Your notes
<!-- -->
