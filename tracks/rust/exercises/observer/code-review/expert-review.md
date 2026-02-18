# Expert Review — EventBus PR

## Summary

This PR has **4 critical issues**, **3 major concerns**, and several minor suggestions. The closure-based `on()` method works for the demo but the overall design has fundamental thread-safety and soundness problems that must be resolved before merging.

---

## Critical Issues

### 1. `Rc<RefCell<>>` is not `Send + Sync` — EventBus cannot be shared across threads

**Location:** `EventBus::shared_registry` field

```rust
shared_registry: Rc<RefCell<Vec<Box<dyn Any>>>>,
```

The PR description states the bus must be `Send + Sync` for use across request-handling threads. `Rc` is explicitly `!Send` and `!Sync`. Wrapping the bus in `Arc` will fail to compile because `EventBus` contains `Rc`.

**Fix:** Replace `Rc<RefCell<>>` with `Arc<Mutex<>>` or `Arc<RwLock<>>`:

```rust
shared_registry: Arc<Mutex<Vec<Box<dyn Any + Send + Sync>>>>,
```

Similarly, `MetricsCollector` uses `RefCell` for interior mutability. In a multi-threaded context, this must be `Mutex` or `RwLock`:

```rust
struct MetricsCollector {
    totals: Mutex<HashMap<String, u64>>,  // NOT RefCell
}
```

**Severity:** Blocker. The stated requirement of thread-safety is violated.

---

### 2. `EventHandler` trait is NOT object-safe

**Location:** `EventHandler<E>` trait definition

```rust
trait EventHandler<E> {
    fn handle(&self, event: &E);
    fn clone_handler(&self) -> Self where Self: Sized;
}
```

The `clone_handler(&self) -> Self` method returns `Self`, which makes the trait not object-safe when `Self: Sized` is not a bound on the trait itself. While the `where Self: Sized` bound on the method excludes it from the vtable (making `dyn EventHandler<E>` *technically* object-safe), this means `clone_handler` **cannot be called on trait objects**. This defeats the purpose of including it in the trait.

More importantly, the `subscribe` method tries to use `&dyn EventHandler<E>`:

```rust
fn subscribe<E: 'static>(&mut self, handler: &dyn EventHandler<E>) {
```

This signature borrows the handler. The reference is ephemeral — it does not outlive the function call. The handler is not actually stored; instead, a dangling raw pointer is created (see Critical Issue #3).

**Fix:** Either:
- Remove `clone_handler` from the trait and use a separate `Clone` bound
- Change the trait to not return `Self`
- Store `Box<dyn EventHandler<E>>` instead of references

---

### 3. `subscribe` method stores a raw pointer — Undefined Behavior

**Location:** `EventBus::subscribe()`

```rust
fn subscribe<E: 'static>(&mut self, handler: &dyn EventHandler<E>) {
    let handler_ref: &dyn Fn(&E) = &|_event| { /* placeholder */ };
    self.handlers
        .entry(type_id)
        .or_default()
        .push(Box::new(handler_ref as *const dyn Fn(&E)));
}
```

This code:
1. Creates a temporary closure that captures nothing (ignoring the actual handler entirely)
2. Takes a reference to this temporary
3. Casts it to a raw pointer
4. Stores the raw pointer as `Box<dyn Any>`

The temporary closure is dropped at the end of the function. The stored pointer is **dangling**. Dereferencing it is undefined behavior.

Even if the intent was to store the handler, storing `&dyn EventHandler<E>` requires a lifetime — the handler must outlive the bus. Raw pointers bypass the borrow checker but do not extend lifetimes.

**Fix:** Use owned values. Accept `Box<dyn EventHandler<E>>` or `impl EventHandler<E> + 'static`:

```rust
fn subscribe<E: 'static>(&mut self, handler: Box<dyn Fn(&E)>) {
    let type_id = TypeId::of::<E>();
    self.handlers
        .entry(type_id)
        .or_default()
        .push(Box::new(handler));
}
```

Or just use the `on()` method, which works correctly.

**Severity:** Blocker. Undefined behavior if `subscribe` is ever called and the handler is dispatched.

---

### 4. `MetricsCollector` uses `RefCell` — panics under concurrent access

**Location:** `MetricsCollector::totals` field

```rust
struct MetricsCollector {
    totals: RefCell<HashMap<String, u64>>,
}
```

`RefCell` provides runtime borrow checking for single-threaded use. If two threads call `handle()` simultaneously, `borrow_mut()` will panic (RefCell detects the concurrent mutable borrow at runtime).

**Fix:** Use `Mutex<HashMap<String, u64>>` for thread-safe interior mutability:

```rust
struct MetricsCollector {
    totals: Mutex<HashMap<String, u64>>,
}

impl EventHandler<OrderCreated> for MetricsCollector {
    fn handle(&self, event: &OrderCreated) {
        let mut totals = self.totals.lock().unwrap();
        *totals.entry(event.customer_id.clone()).or_insert(0) += event.total_cents;
    }
}
```

---

## Major Concerns

### 5. No unsubscribe mechanism — memory leak

There is no way to remove a handler from the bus. In a long-running service, handlers from previous request contexts or short-lived services will accumulate forever.

**Recommendation:** Add ID-based unsubscription:

```rust
fn on<E: 'static>(&mut self, callback: impl Fn(&E) + Send + Sync + 'static) -> SubscriptionId {
    let id = self.next_id;
    self.next_id += 1;
    // Store (id, callback) pair
    id
}

fn unsubscribe(&mut self, id: SubscriptionId) -> bool { /* ... */ }
```

Or use `Weak` references if handler lifetimes are managed externally.

---

### 6. No `Send + Sync` bounds on stored closures

**Location:** `EventBus::on()`

```rust
fn on<E: 'static, F: Fn(&E) + 'static>(&mut self, callback: F) {
```

The closure bound is `Fn(&E) + 'static` with no `Send` or `Sync`. This means:
- Closures can capture non-`Send` types (e.g., `Rc`, raw pointers)
- The resulting `EventBus` cannot be `Send` or `Sync`

**Fix:** Add bounds:

```rust
fn on<E: 'static, F: Fn(&E) + Send + Sync + 'static>(&mut self, callback: F) {
```

---

### 7. Type erasure via `dyn Any` loses compile-time safety

The `handlers` map stores `Box<dyn Any>`, requiring runtime downcasting during `emit()`. If the downcast fails (due to a bug in type registration), the handler is silently skipped — no error, no log, no compile-time check.

**Recommendation:** At minimum, add a debug assertion or log when downcast fails:

```rust
if let Some(callback) = handler.downcast_ref::<Box<dyn Fn(&E)>>() {
    callback(event);
} else {
    debug_assert!(false, "Handler type mismatch for event {:?}", TypeId::of::<E>());
}
```

Alternatively, consider a design where each event type has its own typed handler list (avoids type erasure entirely):

```rust
struct TypedHandlers<E> {
    handlers: Vec<Box<dyn Fn(&E) + Send + Sync>>,
}
```

This can be achieved with a trait-based approach or a macro that generates typed registries.

---

## Minor Suggestions

### 8. `clone_handler` is unused

The `clone_handler` method is defined on `EventHandler` and implemented on all concrete types, but it is never called anywhere. Remove dead code or add a comment explaining the intended use case.

### 9. Inconsistent handler registration

The `subscribe` method (trait-based) and `on` method (closure-based) coexist but have very different implementations and correctness properties. The `subscribe` method is broken (Critical Issue #3) while `on` works. Pick one approach and remove the other, or clearly document that `subscribe` is experimental.

### 10. Missing `Debug` implementations

Event types derive `Debug` and `Clone` but the `EventBus` itself and the handlers do not implement `Debug`. For a framework-level component, `Debug` output is essential for troubleshooting.

### 11. No error handling strategy

What happens if a handler panics during `emit()`? Currently, the panic propagates and prevents remaining handlers from being notified. Consider `std::panic::catch_unwind`:

```rust
fn emit<E: 'static>(&self, event: &E) {
    if let Some(handlers) = self.handlers.get(&TypeId::of::<E>()) {
        for handler in handlers {
            if let Some(callback) = handler.downcast_ref::<Box<dyn Fn(&E)>>() {
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback(event);
                })) {
                    eprintln!("Handler panicked: {:?}", e);
                }
            }
        }
    }
}
```

---

## Positive Feedback

1. **Typed events via generics + TypeId**: The approach of using `TypeId` to route events to the correct handlers is sound and allows strong typing at the registration site.

2. **Clean separation of domain events**: The `OrderCreated`, `PaymentProcessed`, and `InventoryReserved` events are well-defined with clear fields. Good domain modeling.

3. **Closure-based `on()` method**: The API design of `bus.on::<OrderCreated, _>(|event| { ... })` is ergonomic and familiar. The closure approach is idiomatic Rust.

4. **Handler count introspection**: `handler_count::<E>()` is a useful diagnostic method.

5. **Benchmark-driven motivation**: Starting from a performance observation (3x improvement) and designing a solution around it is the right approach.

---

## Recommended Path Forward

1. **Remove `subscribe`** entirely — it is broken and the `on` method covers the same use case
2. **Replace `Rc<RefCell<>>` with `Arc<Mutex<>>`** everywhere
3. **Add `Send + Sync` bounds** to stored closures
4. **Add unsubscribe mechanism** with ID-based removal
5. **Replace `RefCell` with `Mutex`** in `MetricsCollector`
6. **Add panic handling** in `emit()`
7. **Consider `Weak` references** for observer lifecycle management
8. **Add integration tests** with multi-threaded dispatch

After these fixes, the core design (TypeId-based routing with closure handlers) is solid and the performance improvement justifies the PR.
