# Solution — Observer Pattern Debugging Exercise

## Bug 1: Borrow Checker Conflict

### Root Cause

The `notify_with_count` method takes `&mut self`. In the original code, the caller tries to access `subject.count()` (which needs `&self`) while also needing `&mut self` for `notify_with_count`. This is Rust preventing simultaneous mutable and immutable borrows of the same value.

The bug is subtle because `notify_with_count` and `count()` are separate methods — you might think the mutable borrow ends after `notify_with_count` returns. It does. But the original code structured the calls in a way that created overlapping borrows (e.g., passing `subject.count()` as an argument while also calling `&mut self` methods, or storing a reference to the count across the mutable call).

### The Fix

Separate the borrows into distinct statements. Read `count()` first, store the result, then call the mutable method:

```rust
// BEFORE (conflict):
// Depending on how the code is structured, something like:
// println!("Notifying {} observers...", subject.count());
// subject.notify_with_count(&event);  // If these were combined in one expression

// AFTER (fixed):
let count = subject.count();  // Immutable borrow starts and ends here
println!("About to notify {} observers", count);
subject.notify_with_count(&event);  // Mutable borrow starts here, no conflict
```

Alternatively, restructure `notify_with_count` to not require `&mut self` if observers only need `&self` (use `Fn` instead of `FnMut`). But if observers need mutable state, the `&mut self` is necessary, and the caller must sequence borrows.

### Prevention

- Never hold an immutable reference to a struct while calling a `&mut self` method on it
- Store derived values in local variables before mutable operations
- Consider whether methods truly need `&mut self` — use `Fn` closures instead of `FnMut` when possible

---

## Bug 2: Deadlock with Shared Logger

### Root Cause

`LoggingSubject::notify()` acquires a lock on `self.logger`, then calls each observer's closure. The observer closures also try to lock `self.logger`. Since `std::sync::Mutex` is **not reentrant** in Rust, the same thread trying to acquire a lock it already holds causes a **deadlock** (the thread blocks forever waiting for a lock it will never release).

This is a classic re-entrant locking problem, disguised by the observer pattern. The subject locks a resource during notification, and observers try to use the same resource.

### The Fix

Release the logger lock before calling observers:

```rust
fn notify(&self, event: &DeployEvent) {
    // Lock logger, log, then RELEASE before calling observers
    {
        let mut logger = self.logger.lock().unwrap();
        logger.log(format!("Notifying {} observers for {:?}", self.observers.len(), event));
    } // Logger lock released here

    // Now observers can safely lock the logger
    for observer in &self.observers {
        let mut obs = observer.lock().unwrap();
        obs(event);
    }
}
```

Alternative approaches:
- Use `parking_lot::ReentrantMutex` if you genuinely need re-entrant locking (rare, and usually a design smell)
- Pass a mutable reference to the logger through the observer callback instead of having observers lock it independently
- Use channels to decouple logging from notification entirely

### Prevention

- Never hold a lock while calling user-provided callbacks (closures, trait method implementations)
- Minimize lock scope — acquire, do the minimum work, release
- If a resource is shared between the subject and observers, either pass it through the callback or use a channel
- Draw the lock acquisition graph: if there is a cycle, there is a potential deadlock

### Related Pitfalls

- [[rust-mutex-deadlock]] — General Mutex deadlock patterns in Rust

---

## Bug 3: Moved Value — Forgot to Clone Arc

### Root Cause

`MultiSubject::subscribe()` takes `Arc<Mutex<dyn EventHandler>>` **by value** (move semantics). When the caller passes the `notifier` Arc to `subject_a.subscribe(notifier)`, ownership of the Arc moves into the method. The subsequent call `subject_b.subscribe(notifier)` fails because `notifier` has already been moved.

This is a fundamental Rust ownership error: `Arc` is specifically designed for shared ownership via cloning, but you must **actually clone it** before passing it to multiple owners.

### The Fix

Clone the Arc before passing it to each subject:

```rust
let notifier = Arc::new(Mutex::new(SlackNotifier::new("deployments".into())));

// Clone the Arc — this increments the reference count, not deep copy
subject_a.subscribe(notifier.clone());
subject_b.subscribe(notifier.clone());

// We can still use `notifier` here because we only passed clones
let n = notifier.lock().unwrap();
println!("Message count: {}", n.message_count);
```

Alternatively, change the `subscribe` method signature to accept a reference and clone internally:

```rust
fn subscribe(&mut self, handler: &Arc<Mutex<dyn EventHandler>>) {
    self.handlers.push(handler.clone());
}
```

This is more ergonomic for callers — they cannot accidentally move the Arc.

### Prevention

- When a value needs to be shared, clone `Arc` before passing it
- Consider taking `&Arc<T>` in method signatures if the method will clone anyway — this prevents accidental moves
- Remember: `Arc::clone()` is cheap (atomic increment), not a deep copy

---

## Bug 4: Channel Receiver Dropped Early — Silent Data Loss

### Root Cause

The code subscribes two receivers but only spawns a thread for the first one. The second receiver (`_rx2`) is created and immediately dropped. When the publisher sends events, the send to the second subscriber's sender fails (the receiver is gone), causing an error that was originally silently ignored.

In the provided buggy code, this manifests as the publisher printing send failures for the second subscriber. The first subscriber correctly receives events because its receiver was moved into the spawned thread. But the developer intended both subscribers to receive events.

The deeper issue: `mpsc::Sender::send()` returns `Err` when the receiver is dropped, but the original code (before the error printing was added) used `let _ = sender.send(...)`, discarding the error entirely. This is **silent data loss** — events are sent but nobody receives them, and no error is visible.

### The Fix

Either:
1. Spawn a thread for every subscriber (so every receiver is actively consumed):

```rust
let rx1 = publisher.subscribe();
let rx2 = publisher.subscribe();

let handle1 = thread::spawn(move || {
    while let Ok(event) = rx1.recv() {
        println!("[Sub 1] {}", event.service);
    }
});

let handle2 = thread::spawn(move || {
    while let Ok(event) = rx2.recv() {
        println!("[Sub 2] {}", event.service);
    }
});
```

2. Or prune dead senders before/during publish:

```rust
fn publish(&mut self, event: DeployEvent) {
    self.senders.retain(|sender| sender.send(event.clone()).is_ok());
}
```

3. Or do not create subscriptions you will not use.

### Prevention

- Every `subscribe()` call creates a channel — the receiver MUST be consumed
- Use `retain` with `send().is_ok()` to automatically prune dead senders
- Never use `let _ = sender.send(...)` in production — always handle the error or at least log it
- Consider a `SubscriptionGuard` pattern: a struct that holds the receiver and unsubscribes (removes the sender) on drop

### Related Pitfalls

- [[rust-channel-silent-loss]] — Silent data loss in channel-based patterns
