# Rust Reference — Publish/Subscribe Pattern

> Extracted from [The Rust Standard Library](https://doc.rust-lang.org/std/),
> [The Rust Reference](https://doc.rust-lang.org/reference/), and
> [Rust by Example](https://doc.rust-lang.org/rust-by-example/).
> Covers: `std::sync::mpsc` channels, `Arc` and `Mutex` for shared broker state,
> `sync_channel` for bounded channels, `Drop` trait for cleanup, and `HashMap` for topic routing.

---

## `std::sync::mpsc` — Multi-Producer, Single-Consumer Channels

Source: [std::sync::mpsc](https://doc.rust-lang.org/std/sync/mpsc/index.html)

Rust's standard library provides two channel constructors:

| Function | Buffering | `send()` behavior | Use case |
|----------|-----------|-------------------|----------|
| `mpsc::channel()` | Unbounded | Never blocks, may OOM | Low-throughput or when memory is not a concern |
| `mpsc::sync_channel(n)` | Bounded to `n` | Blocks when buffer full | Production systems requiring backpressure |

### Unbounded Channel

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel::<String>();

// Sender
tx.send("hello".to_string()).unwrap();

// Receiver
let msg = rx.recv().unwrap(); // Blocks until message available
```

### Bounded (Synchronous) Channel

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::sync_channel::<String>(16); // Buffer up to 16 messages

// Blocking send — waits when buffer is full
tx.send("hello".to_string()).unwrap();

// Non-blocking send — returns error if full
match tx.try_send("world".to_string()) {
    Ok(()) => { /* sent */ }
    Err(mpsc::TrySendError::Full(msg)) => {
        // Buffer full; msg is returned to caller
    }
    Err(mpsc::TrySendError::Disconnected(msg)) => {
        // Receiver dropped
    }
}
```

### Sender API

| Method | Behavior | Returns |
|--------|----------|---------|
| `send(value)` | Blocks (sync_channel) or immediate (channel) | `Result<(), SendError<T>>` |
| `try_send(value)` | Non-blocking (sync_channel only) | `Result<(), TrySendError<T>>` |
| `clone()` | Create additional sender | `Sender<T>` |

`Sender<T>` implements `Send` and `Sync` — it can be shared across threads.

`Sender<T>` does **not** implement `Clone` for `SyncSender`. For `mpsc::channel()`, `Sender` does implement `Clone`.

### Receiver API

| Method | Behavior | Returns |
|--------|----------|---------|
| `recv()` | Blocks until message or disconnect | `Result<T, RecvError>` |
| `try_recv()` | Non-blocking | `Result<T, TryRecvError>` |
| `recv_timeout(dur)` | Blocks up to `dur` | `Result<T, RecvTimeoutError>` |
| `iter()` | Blocking iterator over messages | `Iter<T>` |

`Receiver<T>` implements `Send` but **not** `Sync` — it can be moved to another thread but not shared between threads.

### Disconnection Semantics

- When all `Sender`s are dropped, `rx.recv()` returns `Err(RecvError)`
- When the `Receiver` is dropped, `tx.send()` returns `Err(SendError(value))`
- The returned value in `SendError` lets the caller recover the unsent message

```rust
// Detecting disconnection
loop {
    match rx.recv() {
        Ok(msg) => process(msg),
        Err(mpsc::RecvError) => {
            // All senders dropped — channel is closed
            break;
        }
    }
}
```

### Iterator Interface

```rust
// Blocking iteration — loops until all senders are dropped
for msg in rx.iter() {
    process(msg);
}

// Equivalent to:
while let Ok(msg) = rx.recv() {
    process(msg);
}
```

---

## `Arc<T>` — Atomic Reference Counting

Source: [std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html)

Thread-safe shared ownership via atomic reference counting. Used in Pub/Sub to share the broker state across publisher and subscriber threads.

### Core API

```rust
use std::sync::Arc;

let data = Arc::new(42);
let data2 = data.clone();    // Atomic increment, not deep copy

Arc::strong_count(&data);     // Number of Arc pointers
Arc::weak_count(&data);       // Number of Weak pointers
Arc::ptr_eq(&data, &data2);   // Pointer equality check
```

### Properties

| Property | Value |
|----------|-------|
| Thread safety | Yes (atomic reference count operations) |
| Clone cost | Atomic fetch-add (cheap, approximately 1-5 ns) |
| Dereference | Immutable only — `Arc<T>` implements `Deref<Target = T>` |
| Mutable access | Requires interior mutability: `Arc<Mutex<T>>` or `Arc<RwLock<T>>` |
| `Send` | When `T: Send + Sync` |
| `Sync` | When `T: Send + Sync` |
| Drop | Atomic fetch-sub; deallocates when strong count reaches 0 |

### `Arc` for Shared Messages

Instead of cloning large messages per subscriber, wrap in `Arc`:

```rust
use std::sync::Arc;

#[derive(Debug)]
struct LargePayload {
    data: Vec<u8>, // megabytes
}

// One allocation, multiple references
let shared: Arc<LargePayload> = Arc::new(LargePayload {
    data: vec![0u8; 1_000_000],
});

// Each subscriber gets an Arc clone (cheap pointer copy)
let sub1_msg = shared.clone(); // ~5ns
let sub2_msg = shared.clone(); // ~5ns
// vs cloning 1MB of data for each subscriber
```

---

## `Mutex<T>` — Mutual Exclusion Lock

Source: [std::sync::Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html)

Provides exclusive access to the wrapped value. In Pub/Sub, wraps the broker's internal state.

### Core API

```rust
use std::sync::Mutex;

let m = Mutex::new(HashMap::new());

// Acquire lock — blocks until available
let mut guard = m.lock().unwrap();
guard.insert("key", "value");
// Lock released when guard is dropped
```

### Lock Behavior

| Operation | Method | Behavior |
|-----------|--------|----------|
| Blocking lock | `lock()` | Blocks until lock acquired |
| Non-blocking | `try_lock()` | Returns `Err(TryLockError)` if locked |
| Poisoning | `lock().unwrap()` | Panics if previous holder panicked |
| Recover poisoned | `lock().unwrap_or_else(\|e\| e.into_inner())` | Ignores poisoning |

### Lock Poisoning

If a thread panics while holding a `Mutex` lock, the mutex becomes "poisoned":

```rust
use std::sync::Mutex;

let m = Mutex::new(0);

// Thread panics while holding lock
let _ = std::thread::spawn(move || {
    let _guard = m.lock().unwrap();
    panic!("oops");
}).join();

// Subsequent lock attempts get PoisonError
// To recover: m.lock().unwrap_or_else(|e| e.into_inner())
```

### `RwLock<T>` — Reader-Writer Lock

Source: [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html)

When reads vastly outnumber writes (e.g., topic lookups outnumber subscribe/unsubscribe):

```rust
use std::sync::RwLock;

let lock = RwLock::new(HashMap::new());

// Multiple concurrent readers
let r = lock.read().unwrap();

// Exclusive writer
let mut w = lock.write().unwrap();
```

| Operation | Concurrency | Use in Pub/Sub |
|-----------|-------------|----------------|
| `read()` | Multiple concurrent readers | Publishing (looking up topic subscribers) |
| `write()` | Exclusive access | Subscribe/unsubscribe (modifying subscriber list) |

---

## `HashMap<K, V>` for Topic Routing

Source: [std::collections::HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

The broker maps topics to subscriber lists:

```rust
use std::collections::HashMap;
use std::sync::mpsc::Sender;

// String topics
type TopicMap = HashMap<String, Vec<Sender<Message>>>;

// Enum topics (type-safe)
type TypedTopicMap = HashMap<MetricTopic, Vec<Sender<MetricEvent>>>;
```

### Key Operations for Pub/Sub

```rust
use std::collections::HashMap;

let mut map: HashMap<String, Vec<i32>> = HashMap::new();

// Entry API — get or insert default
map.entry("topic".to_string()).or_default().push(42);

// Get mutable reference to subscriber list
if let Some(subs) = map.get_mut("topic") {
    subs.retain(|s| *s != 0); // Remove dead subscribers
}

// Check if topic exists
map.contains_key("topic");

// Remove a topic entirely
map.remove("topic");

// Iterate all topics
for (topic, subs) in &map {
    println!("{}: {} subscribers", topic, subs.len());
}
```

### Requirements for Topic Keys

The topic type must implement `Eq + Hash`:

```rust
// String: works out of the box
HashMap<String, Vec<Sender<Message>>>

// Enum: derive both traits
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Topic {
    Metrics,
    Alerts,
    AuditLog,
}
HashMap<Topic, Vec<Sender<Message>>>
```

---

## `Drop` Trait for Cleanup

Source: [std::ops::Drop](https://doc.rust-lang.org/std/ops/trait.Drop.html)

The `Drop` trait runs cleanup code when a value goes out of scope. For Pub/Sub, this ensures subscriber threads are joined and channels are closed.

### Basic Drop

```rust
struct Broker {
    senders: Vec<mpsc::Sender<String>>,
    handles: Vec<Option<std::thread::JoinHandle<()>>>,
}

impl Drop for Broker {
    fn drop(&mut self) {
        // Step 1: Drop all senders to signal channel closure
        self.senders.clear();

        // Step 2: Join all subscriber threads
        for handle in &mut self.handles {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
    }
}
```

### Drop Ordering Rules

| Rule | Description |
|------|-------------|
| Fields dropped in declaration order | Struct fields are dropped top-to-bottom |
| `Drop::drop` called before fields are dropped | Your custom `drop` runs first, then fields auto-drop |
| Cannot move out of `&mut self` | Use `Option<T>` + `take()` to move owned values |
| Cannot call `drop()` explicitly | Use `std::mem::drop(value)` instead |

### `Option<JoinHandle>` Pattern

Since `Drop::drop` receives `&mut self`, you cannot move `JoinHandle` out directly (`.join()` takes `self`). The workaround:

```rust
struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // take() replaces Option with None and returns the JoinHandle
            handle.join().expect("worker thread panicked");
        }
    }
}
```

---

## `SyncSender<T>` — Bounded Channel Sender

Source: [std::sync::mpsc::SyncSender](https://doc.rust-lang.org/std/sync/mpsc/struct.SyncSender.html)

Returned by `mpsc::sync_channel()`. Unlike `Sender<T>`, `SyncSender<T>` can block on send.

### API

| Method | Behavior |
|--------|----------|
| `send(value)` | Blocks until space available or receiver disconnected |
| `try_send(value)` | Returns immediately with `Ok`, `Err(Full)`, or `Err(Disconnected)` |

### Cloning

`SyncSender<T>` implements `Clone` — multiple producers can share the bounded channel:

```rust
let (tx, rx) = mpsc::sync_channel::<String>(10);
let tx2 = tx.clone(); // Second producer

std::thread::spawn(move || {
    tx.send("from thread 1".into()).unwrap();
});

std::thread::spawn(move || {
    tx2.send("from thread 2".into()).unwrap();
});
```

---

## Thread Spawning and `JoinHandle`

Source: [std::thread::spawn](https://doc.rust-lang.org/std/thread/fn.spawn.html)

### Spawn

```rust
use std::thread;

let handle: thread::JoinHandle<String> = thread::spawn(|| {
    // Closure must be 'static + Send
    "result".to_string()
});

// Wait for thread to finish
let result: String = handle.join().unwrap();
```

### Requirements for Closures Passed to `spawn`

| Bound | Reason |
|-------|--------|
| `'static` | Cannot borrow local variables (thread may outlive caller) |
| `Send` | Closure is sent to another thread |
| `FnOnce` | Closure is called exactly once |

### Scoped Threads (Rust 1.63+)

Source: [std::thread::scope](https://doc.rust-lang.org/std/thread/fn.scope.html)

Scoped threads can borrow local variables because they are guaranteed to finish before the scope exits:

```rust
let data = vec![1, 2, 3];

std::thread::scope(|s| {
    s.spawn(|| {
        // Can borrow &data — guaranteed to finish before scope exits
        println!("{:?}", data);
    });
});
// All scoped threads have finished here
```

This is useful for testing Pub/Sub without `Arc` — scoped threads can borrow the broker directly.

---

## `std::sync::Condvar` — Condition Variable

Source: [std::sync::Condvar](https://doc.rust-lang.org/std/sync/struct.Condvar.html)

Condition variables allow threads to wait for a condition to become true. Useful for custom Pub/Sub signaling beyond basic channel semantics.

```rust
use std::sync::{Arc, Condvar, Mutex};

let pair = Arc::new((Mutex::new(false), Condvar::new()));
let pair2 = pair.clone();

// Waiting thread
std::thread::spawn(move || {
    let (lock, cvar) = &*pair2;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    println!("Condition met!");
});

// Signaling thread
let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();
*started = true;
cvar.notify_one(); // Wake one waiting thread
// cvar.notify_all(); // Wake all waiting threads
```

| Method | Behavior |
|--------|----------|
| `wait(guard)` | Release lock, sleep, re-acquire on wake |
| `wait_timeout(guard, dur)` | Like `wait` but with timeout |
| `notify_one()` | Wake one waiting thread |
| `notify_all()` | Wake all waiting threads |
