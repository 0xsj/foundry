# Rust Reference — Observer Pattern

> Extracted from [The Rust Standard Library](https://doc.rust-lang.org/std/),
> [The Rust Reference](https://doc.rust-lang.org/reference/), and
> [Tokio documentation](https://docs.rs/tokio/latest/tokio/).
> Covers: closures as callbacks, channels (mpsc), smart pointers (Arc, Weak, Mutex), trait objects, and async broadcast.

---

## Closures as Callbacks

Source: [The Rust Reference - Closure Types](https://doc.rust-lang.org/reference/types/closure.html)

Closures in Rust implement one or more of the `Fn` traits depending on how they capture their environment:

| Trait | Captures | Can be called | Use in Observer |
|-------|----------|---------------|-----------------|
| `Fn(&self)` | By shared reference | Multiple times, concurrently safe | Read-only callbacks |
| `FnMut(&mut self)` | By mutable reference | Multiple times, not concurrently | Stateful callbacks |
| `FnOnce(self)` | By move | Exactly once | One-shot handlers |

For observer patterns, `Fn` is the typical bound because callbacks are invoked repeatedly:

```rust
// Storing a callback that borrows shared references from its environment
let callbacks: Vec<Box<dyn Fn(&str)>> = Vec::new();
```

For mutable state in callbacks, the closure must own the state or use interior mutability:

```rust
use std::sync::{Arc, Mutex};

let count = Arc::new(Mutex::new(0));
let count_clone = count.clone();

// The closure captures the Arc by move, then uses Mutex for interior mutability
let callback: Box<dyn Fn(&str)> = Box::new(move |event| {
    let mut c = count_clone.lock().unwrap();
    *c += 1;
});
```

### Thread Safety Bounds

For multi-threaded observer patterns, closures must be `Send` and/or `Sync`:

```rust
// Single-threaded: no bounds needed
type Callback<E> = Box<dyn Fn(&E)>;

// Multi-threaded (send across threads): requires Send
type SendCallback<E> = Box<dyn Fn(&E) + Send>;

// Multi-threaded (shared across threads): requires Send + Sync
type SharedCallback<E> = Box<dyn Fn(&E) + Send + Sync>;
```

A closure is `Send` if all captured values are `Send`. A closure is `Sync` if all captured values are `Sync`.

---

## `std::sync::mpsc` — Multi-Producer, Single-Consumer Channels

Source: [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/index.html)

### Channel Types

| Function | Type | Buffer | Behavior |
|----------|------|--------|----------|
| `mpsc::channel()` | Asynchronous | Unbounded | `send()` never blocks, memory grows |
| `mpsc::sync_channel(n)` | Synchronous | Bounded to `n` | `send()` blocks when buffer full |

### Core API

```rust
use std::sync::mpsc;

// Create an unbounded channel
let (tx, rx) = mpsc::channel::<String>();

// Clone the sender for fan-in (multiple producers)
let tx2 = tx.clone();

// Send (returns Err if all receivers dropped)
tx.send("event".to_string()).unwrap();

// Receive (blocks until message available)
let msg = rx.recv().unwrap();

// Non-blocking receive
match rx.try_recv() {
    Ok(msg) => { /* process */ }
    Err(mpsc::TryRecvError::Empty) => { /* no message */ }
    Err(mpsc::TryRecvError::Disconnected) => { /* sender dropped */ }
}

// Receive with timeout
use std::time::Duration;
match rx.recv_timeout(Duration::from_secs(5)) {
    Ok(msg) => { /* process */ }
    Err(mpsc::RecvTimeoutError::Timeout) => { /* timed out */ }
    Err(mpsc::RecvTimeoutError::Disconnected) => { /* sender dropped */ }
}
```

### Fan-Out Pattern (Observer)

`mpsc` is multi-producer, single-consumer. For fan-out (one publisher, many subscribers), maintain a `Vec<Sender<T>>`:

```rust
struct FanOut<T: Clone> {
    senders: Vec<mpsc::Sender<T>>,
}

impl<T: Clone> FanOut<T> {
    fn subscribe(&mut self) -> mpsc::Receiver<T> {
        let (tx, rx) = mpsc::channel();
        self.senders.push(tx);
        rx
    }

    fn publish(&mut self, event: T) {
        self.senders.retain(|tx| tx.send(event.clone()).is_ok());
    }
}
```

The `retain` call automatically removes senders whose receivers have been dropped.

### Important: `Sender` is `Send`, `Receiver` is `Send` but NOT `Sync`

- `Sender<T>` implements `Send` (can be sent across threads) and `Sync` (can be shared)
- `Receiver<T>` implements `Send` (can be moved to another thread) but NOT `Sync` (cannot be shared between threads)
- One receiver per consuming thread — this is by design

---

## `Arc<T>` — Atomic Reference Counting

Source: [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)

Thread-safe reference counting pointer. `Arc<T>` provides shared ownership of a value.

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
let data2 = data.clone(); // Increments reference count, not deep copy

// Both `data` and `data2` point to the same allocation
assert!(Arc::ptr_eq(&data, &data2));

// Reference count
assert_eq!(Arc::strong_count(&data), 2);
```

### Properties

| Property | Value |
|----------|-------|
| Thread-safe | Yes (atomic reference count) |
| Mutable access | Only via interior mutability (`Mutex`, `RwLock`) |
| Cloning cost | Atomic increment (cheap, but not free) |
| Drop behavior | Decrements count; deallocates when count reaches 0 |
| `Send` bound | `Arc<T>: Send` when `T: Send + Sync` |
| `Sync` bound | `Arc<T>: Sync` when `T: Send + Sync` |

### `Arc<Mutex<T>>` for Shared Mutable State

```rust
use std::sync::{Arc, Mutex};

let shared = Arc::new(Mutex::new(0));

// Clone Arc, move clone into thread
let shared_clone = shared.clone();
std::thread::spawn(move || {
    let mut val = shared_clone.lock().unwrap();
    *val += 1;
});
```

---

## `Weak<T>` — Non-Owning Reference

Source: [std::sync::Weak](https://doc.rust-lang.org/std/sync/struct.Weak.html)

A `Weak` reference does not prevent the value from being dropped. Created via `Arc::downgrade()`.

```rust
use std::sync::{Arc, Weak};

let strong = Arc::new("data".to_string());
let weak: Weak<String> = Arc::downgrade(&strong);

// Upgrade: returns Some if the value still exists
assert!(weak.upgrade().is_some());

drop(strong);

// After all strong references are gone, upgrade returns None
assert!(weak.upgrade().is_none());
```

### Use in Observer Pattern

```rust
use std::sync::{Arc, Mutex, Weak};

struct Subject {
    observers: Vec<Weak<Mutex<dyn Observer>>>,
}

impl Subject {
    fn notify(&mut self, event: &str) {
        self.observers.retain(|weak| {
            match weak.upgrade() {
                Some(strong) => {
                    strong.lock().unwrap().on_event(event);
                    true  // Keep — still alive
                }
                None => false, // Remove — observer was dropped
            }
        });
    }
}
```

### Weak vs Strong Reference Counts

```rust
let strong = Arc::new(42);
assert_eq!(Arc::strong_count(&strong), 1);
assert_eq!(Arc::weak_count(&strong), 0);

let weak = Arc::downgrade(&strong);
assert_eq!(Arc::strong_count(&strong), 1);
assert_eq!(Arc::weak_count(&strong), 1);

// Value is dropped when strong_count reaches 0, regardless of weak_count
```

---

## `Mutex<T>` — Mutual Exclusion Lock

Source: [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html)

```rust
use std::sync::Mutex;

let m = Mutex::new(5);

{
    let mut val = m.lock().unwrap(); // Acquire lock, get MutexGuard
    *val = 10;
    // MutexGuard dropped here — lock released
}
```

### Lock Poisoning

If a thread panics while holding the lock, the Mutex becomes "poisoned." Subsequent `lock()` calls return `Err(PoisonError)`:

```rust
// To handle poisoning (recover the data):
let val = m.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
```

### `RwLock<T>` — Reader-Writer Lock

Source: [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)

Multiple concurrent readers or one exclusive writer:

```rust
use std::sync::RwLock;

let lock = RwLock::new(vec![1, 2, 3]);

// Multiple readers
let r1 = lock.read().unwrap();
let r2 = lock.read().unwrap(); // Both hold read locks concurrently

drop(r1);
drop(r2);

// One writer (exclusive)
let mut w = lock.write().unwrap();
w.push(4);
```

For observer patterns, `RwLock` is useful when reads (notifications) vastly outnumber writes (subscribe/unsubscribe).

---

## Trait Object Safety

Source: [The Rust Reference - Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

A trait is "object safe" (can be used as `dyn Trait`) if all its methods satisfy these rules:

| Rule | Explanation |
|------|-------------|
| No `Self` in return position | Cannot return `-> Self` (size unknown at compile time) |
| No generic type parameters | `fn foo<T>()` cannot be dispatched dynamically |
| Receiver must be dispatchable | `self`, `&self`, `&mut self`, `Box<Self>`, `Arc<Self>` are OK |
| No `where Self: Sized` | Excludes trait from dynamic dispatch |

### Observer Trait — Object Safe

```rust
// Object safe: &mut self receiver, no generics, no Self return
trait Observer {
    fn on_event(&mut self, event: &str);
    fn name(&self) -> &str;
}

// Can be used as trait object
let observers: Vec<Box<dyn Observer>> = vec![];
```

### Observer Trait — NOT Object Safe

```rust
// NOT object safe: generic method
trait Observer {
    fn on_event<E>(&mut self, event: E); // Cannot use dyn Observer
}

// NOT object safe: returns Self
trait Observer: Sized {
    fn on_event(&mut self, event: &str);
    fn clone_observer(&self) -> Self; // Cannot use dyn Observer
}
```

**Workaround for generic events:** Use an enum for event types instead of generics:

```rust
#[derive(Clone, Debug)]
enum Event {
    HealthCheck { service: String, healthy: bool },
    Metric { name: String, value: f64 },
    Alert { severity: u8, message: String },
}

trait Observer {
    fn on_event(&mut self, event: &Event); // Object safe — no generics
}
```

---

## Tokio Broadcast Channel

Source: [tokio::sync::broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)

Multi-producer, multi-consumer channel where each receiver gets every message.

```rust
use tokio::sync::broadcast;

// Create with capacity (required)
let (tx, mut rx1) = broadcast::channel::<String>(16);
let mut rx2 = tx.subscribe(); // Additional receiver

tx.send("event".to_string()).unwrap();

// Both receivers get the message
assert_eq!(rx1.recv().await.unwrap(), "event");
assert_eq!(rx2.recv().await.unwrap(), "event");
```

### Key Properties

| Property | Value |
|----------|-------|
| Bounded | Yes (capacity specified at creation) |
| Cloning | Messages must implement `Clone` |
| Lagging | Slow receivers get `RecvError::Lagged(n)` when they fall behind |
| Ordering | Messages delivered in send order |
| Sender clone | `tx.clone()` for multiple producers |
| Receiver create | `tx.subscribe()` for new receiver (starts from next message) |

### Error Handling

```rust
use tokio::sync::broadcast::error::RecvError;

match rx.recv().await {
    Ok(msg) => { /* process message */ }
    Err(RecvError::Closed) => { /* all senders dropped */ }
    Err(RecvError::Lagged(n)) => {
        // Receiver missed `n` messages due to slow processing
        // Channel wraps around, oldest messages are overwritten
        eprintln!("Missed {} messages", n);
    }
}
```

---

## Tokio Watch Channel

Source: [tokio::sync::watch](https://docs.rs/tokio/latest/tokio/sync/watch/index.html)

Single-producer, multi-consumer channel that retains only the **latest** value.

```rust
use tokio::sync::watch;

let (tx, mut rx) = watch::channel("initial".to_string());

// Receiver sees current value
assert_eq!(*rx.borrow(), "initial");

tx.send("updated".to_string()).unwrap();

// Wait for change notification
rx.changed().await.unwrap();
assert_eq!(*rx.borrow(), "updated");
```

### When to Use Watch vs Broadcast

| Use Case | Channel Type |
|----------|-------------|
| Every event matters (audit log, metrics) | `broadcast` |
| Only latest state matters (config, leader, health) | `watch` |
| High-throughput events, some loss acceptable | `broadcast` (with lagging handling) |
| Status page, current state display | `watch` |

---

## `std::any::TypeId` for Typed Event Routing

Source: [std::any::TypeId](https://doc.rust-lang.org/std/any/struct.TypeId.html)

For event buses that route by event type:

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

struct TypedEventBus {
    handlers: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl TypedEventBus {
    fn subscribe<E: 'static>(&mut self, handler: Box<dyn Fn(&E)>) {
        let type_id = TypeId::of::<E>();
        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(handler));
    }

    fn emit<E: 'static>(&self, event: &E) {
        let type_id = TypeId::of::<E>();
        if let Some(handlers) = self.handlers.get(&type_id) {
            for handler in handlers {
                if let Some(f) = handler.downcast_ref::<Box<dyn Fn(&E)>>() {
                    f(event);
                }
            }
        }
    }
}
```

Note: This approach uses type erasure and downcasting. It works but loses compile-time type safety at the dispatch layer. Prefer enum-based event types when the set of events is known at compile time.
