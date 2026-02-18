# Solution: Notification Dispatcher Bugs

## Bug 1: Rc Cycle — Dispatcher ↔ DispatcherNode

### Root Cause

`Dispatcher` holds `RefCell<Option<Rc<DispatcherNode>>>` and `DispatcherNode` holds
`RefCell<Option<Rc<Dispatcher>>>`. When `attach()` is called, both strong counts are
incremented. Neither is ever decremented — each prevents the other from reaching zero.

```
Dispatcher::strong_count = 2 (your handle + DispatcherNode's Rc)
DispatcherNode::strong_count = 2 (your handle + Dispatcher's Rc)

drop(your handles) ->
  Dispatcher::strong_count = 1 (still held by DispatcherNode's Rc)
  DispatcherNode::strong_count = 1 (still held by Dispatcher's Rc)

Neither reaches 0. Memory is never freed.
```

This doesn't produce a compile error or runtime panic — it silently leaks memory. The
test `test_no_rc_cycle_dispatcher_freed_on_drop` catches it by checking that `Weak::upgrade()`
returns `None` after dropping the strong handle.

### Fix

The relationship is: the Dispatcher **owns** its receipt subscriber (the node is chosen by the
dispatcher). The node tracks the dispatcher but doesn't own it. So the node's back-reference
should be `Weak<Dispatcher>`:

```rust
struct DispatcherNode {
    id: u32,
    dispatcher: RefCell<Option<Weak<Dispatcher>>>,  // Weak — not owning
    receipts_received: Cell<u32>,
}
```

Update `attach`:
```rust
fn attach(node: &Rc<DispatcherNode>, dispatcher: &Rc<Dispatcher>) {
    *node.dispatcher.borrow_mut() = Some(Rc::downgrade(dispatcher));  // downgrade
    Dispatcher::set_receipt_subscriber(dispatcher, node);
}
```

Update `get_dispatcher_name` to upgrade the Weak:
```rust
fn get_dispatcher_name(&self) -> Option<String> {
    self.dispatcher.borrow().as_ref()?.upgrade().map(|d| d.name.clone())
}
```

### Lesson

- Every time you have two types where each holds an `Rc` to the other, you have a cycle.
- Decide which direction is "owning" (forward) and which is "back-reference" (non-owning).
- Forward: `Rc`. Back-reference: `Weak`.
- The canonical pattern: parent owns children (`Rc`), children point back to parent (`Weak`).
- Tools: if you suspect a leak, `Rc::strong_count` in tests can expose unexpected counts.

---

## Bug 2: RefCell Double Borrow in `get_count`

### Root Cause

`get_count` calls `self.counts.borrow_mut()` when it only needs a read. This is wasteful
and dangerous: `borrow_mut()` acquires an exclusive lock on the RefCell. If anything else
tries to borrow `counts` while `get_count` is running (even another `borrow()`), it panics.

```rust
// BUG: takes mutable borrow for a read-only operation
fn get_count(&self, channel: &str) -> u32 {
    *self.counts.borrow_mut().get(channel).unwrap_or(&0)
}
```

At runtime, if `check_and_record` is called and it holds a `borrow_mut()` on `counts`,
then calling `get_count` inside (or in a re-entrant scenario) would panic:
`already mutably borrowed: BorrowMutError`.

### Fix

Use `borrow()` for read-only access:

```rust
fn get_count(&self, channel: &str) -> u32 {
    *self.counts.borrow().get(channel).unwrap_or(&0)
}
```

`borrow()` returns a shared `Ref<T>`. Multiple immutable borrows coexist. Only `borrow_mut()`
blocks other borrows. Use the least restrictive borrow you need.

### Lesson

- `RefCell::borrow_mut()` is a write lock. Hold it for the minimum time needed.
- `RefCell::borrow()` is a read lock. Multiple can coexist — prefer it for reads.
- If you see `borrow_mut()` used for a read-only operation, that's a code smell.
- Clippy warns about some of these patterns. In practice, test coverage catches runtime panics.
- Use `try_borrow()` and `try_borrow_mut()` when panicking is unacceptable (return `Err` instead).

---

## Bug 3: Rc\<T\> Is Not Send

### Root Cause

`Rc<T>` uses a non-atomic `usize` for its reference count. Atomic operations prevent data
races when multiple threads increment/decrement simultaneously. Without atomics, two threads
modifying the count at the same time would corrupt it.

The Rust type system encodes this restriction: `Rc<T>` does not implement `Send`. The compiler
rejects any attempt to move an `Rc` across a thread boundary.

```rust
// compile error: `Rc<ChannelConfig>` cannot be sent between threads safely
fn spawn_channel_worker(config: Rc<ChannelConfig>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || { ... })
}
```

### Fix

Use `Arc<ChannelConfig>` ("atomically reference counted"):

```rust
fn spawn_channel_worker(config: Arc<ChannelConfig>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        format!("worker for channel: {}", config.name)
    })
}
```

`Arc<T>` is `Send` if `T: Send + Sync`. The reference count uses atomic operations (`AtomicUsize`),
making it safe to clone and drop from multiple threads concurrently.

### When to Use Rc vs Arc

| | `Rc<T>` | `Arc<T>` |
|---|---|---|
| Thread-safe | No | Yes |
| Overhead | None beyond ref count | Atomic operations on every clone/drop |
| Use case | Single-threaded: UI, interpreters, trees | Multi-threaded: shared read-only data |

The compiler enforces the choice: if you accidentally use `Rc` where `Arc` is needed, you get
a clear compile error: `` `Rc<T>` cannot be sent between threads safely ``.

---

## Bug 4: Config Cloned Out of Rc Instead of Stored as Rc

### Root Cause

`NotificationChannel::new` receives `Rc<ChannelConfig>` but immediately dereferences and clones
the inner `ChannelConfig`, discarding the `Rc`:

```rust
fn new(config: Rc<ChannelConfig>) -> NotificationChannel {
    NotificationChannel {
        config: (*config).clone(),  // BUG: clones the data, loses the shared handle
        sent_count: 0,
    }
}
```

Two channels created from the same `Rc` each hold independent copies of the config. They don't
share the same allocation. `channel_a.config_ptr() != channel_b.config_ptr()`.

The design intent: both channels point to the same `ChannelConfig`. If the config is updated
(disabled, rate limit changed), both channels see it immediately.

### Fix

Store the `Rc<ChannelConfig>` directly without cloning the inner value:

```rust
struct NotificationChannel {
    config: Rc<ChannelConfig>,  // shared ownership — store the Rc
    sent_count: u32,
}

impl NotificationChannel {
    fn new(config: Rc<ChannelConfig>) -> NotificationChannel {
        NotificationChannel { config, sent_count: 0 }  // move the Rc in, no clone
    }
}
```

Now `Rc::clone(&shared)` in the caller increments the reference count; both channels point to
the same heap allocation. `channel_a.config_ptr() == channel_b.config_ptr()`.

### Lesson

- `Rc<T>` is a handle to shared data. Cloning the `Rc` shares the data. Cloning through the `Rc`
  (`(*rc).clone()`) copies the data — the sharing is lost.
- The test `config_ptr()` comparison is the right way to check identity: same pointer address means
  same allocation. Structural equality (`==`) would be true even for copies with the same values.
- If configs need to be mutated through any channel after construction (e.g., to disable), wrap:
  `Rc<RefCell<ChannelConfig>>`.

---

## Summary

| Bug | Category | Symptom | Fix |
|-----|----------|---------|-----|
| Rc cycle | Memory leak | Silent — no error, memory never freed | Change one Rc to Weak in the cycle |
| borrow_mut for read | RefCell misuse | Runtime panic if called during active borrow | Use borrow() for reads |
| Rc across thread | Send violation | Compile error: not Send | Change Rc to Arc |
| Clone out of Rc | Lost sharing | Logic error — config not shared | Store Rc<T>, don't clone through it |

## Related Pitfalls

- [[rust-rc-cycle-memory-leak]] — how to detect and prevent Rc reference cycles
- [[rust-refcell-borrow-mut-for-read]] — always use the weakest borrow you need
- [[rust-rc-not-send]] — Rc vs Arc thread-safety rules
- [[rust-box-vs-rc]] — single-owner vs shared-owner smart pointer selection
