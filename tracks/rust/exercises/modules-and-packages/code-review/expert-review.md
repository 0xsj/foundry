# Expert Review: Webhook Relay Module Restructure

## Critical Issues

### 1. `Route::secret` is `pub` — HMAC signing secret is exposed

**Location:** `src/routing.rs`, `Route` struct

```rust
pub struct Route {
    pub kind: EventKind,
    pub target_url: String,
    pub secret: String,   // signing secret for HMAC — should not be pub
}
```

**Problem:** Signing secrets used for HMAC authentication must not be directly accessible to callers. External code that can read `route.secret` can forge signatures. Even if the current callers don't do this, a future developer reading `Route` will see `pub secret` and think it's safe to log, serialize, or pass to user-facing code.

This is a defense-in-depth issue: the type system should make it impossible to accidentally expose secrets, not just unlikely.

**Fix:** Make `secret` private, add a method that uses it without revealing it:

```rust
pub struct Route {
    pub kind: EventKind,
    pub target_url: String,
    secret: String,   // private — use sign() to generate signatures
}

impl Route {
    pub fn new(kind: EventKind, target_url: &str, secret: &str) -> Route {
        Route {
            kind,
            target_url: target_url.to_string(),
            secret: secret.to_string(),
        }
    }

    /// Signs a payload using this route's secret. Returns the HMAC signature.
    /// The secret itself is never exposed.
    pub(crate) fn sign(&self, payload: &[u8]) -> String {
        // HMAC implementation here
        format!("hmac-sha256={:?}", payload) // stub
    }
}
```

**Concept:** Private fields prevent accidental exposure of sensitive data. This is exactly the kind of invariant the type system can enforce — no runtime checks needed.

---

## Major Concerns

### 2. No `pub use` re-exports in `lib.rs` — callers use awkward paths

**Location:** `src/lib.rs` (implicit from PR description)

```rust
// lib.rs as written:
pub mod events;
pub mod delivery;
pub mod routing;
pub mod store;
// No pub use
```

**Problem:** Callers must import from the internal module paths:

```rust
use webhook_relay::events::{Event, EventKind};
use webhook_relay::store::EventStore;
use webhook_relay::routing::{Router, Route};
use webhook_relay::delivery::DeliveryWorker;
```

This leaks the internal module structure as the public API. If the team later decides to merge `events` and `routing` into a single `model` module, every caller breaks. The module structure is an implementation detail that should be hidden behind a stable facade.

**Fix:** Add re-exports to `lib.rs`:

```rust
pub mod events;
pub mod delivery;
pub mod routing;
pub mod store;

// Stable public API — independent of internal module structure
pub use events::{Event, EventKind};
pub use routing::{Router, Route};
pub use store::EventStore;
pub use delivery::DeliveryWorker;
```

Now callers write:
```rust
use webhook_relay::{Event, EventKind, Router, Route, EventStore, DeliveryWorker};
```

Refactoring the module structure no longer breaks callers.

**Concept:** `pub use` is the primary mechanism for building stable public APIs in Rust. The internal module tree is for the library author's organization. The `pub use` surface is the contract with users.

---

### 3. `EventMetadata` is `pub` — internal implementation detail exposed

**Location:** `src/events.rs`

```rust
pub struct EventMetadata {
    pub received_at: u64,
    pub attempt_count: u32,
    pub source_ip: String,
}

pub struct Event {
    pub metadata: EventMetadata,  // full public access
}
```

**Problem:** `EventMetadata` is described in a comment as "not intended for external callers — used only within the relay crate." Yet it's fully `pub`. Any external user of this library can read and write `event.metadata.attempt_count` directly, bypassing `increment_attempts()`.

This creates two problems:
1. External code can set `attempt_count` to any value, breaking retry logic
2. If `EventMetadata` changes shape (rename fields, add fields), all callers break

**Fix:** Make `EventMetadata` `pub(crate)` and keep `metadata` private. Expose what callers actually need through specific methods:

```rust
pub(crate) struct EventMetadata {
    pub(crate) received_at: u64,
    pub(crate) attempt_count: u32,
    pub(crate) source_ip: String,
}

pub struct Event {
    pub id: String,
    pub kind: EventKind,
    pub payload: String,
    metadata: EventMetadata,  // private
}

impl Event {
    /// Returns when this event was received (Unix timestamp, milliseconds).
    pub fn received_at(&self) -> u64 {
        self.metadata.received_at
    }

    /// Returns how many delivery attempts have been made.
    pub fn attempt_count(&self) -> u32 {
        self.metadata.attempt_count
    }

    // `source_ip` and `increment_attempts` might be pub(crate) only,
    // since they're internal delivery concerns.
}
```

**Concept:** `pub(crate)` is the right visibility for types shared across modules within your crate but not exposed to external users. It's the most underused visibility modifier.

---

### 4. `EventStore::queue` and `EventStore::dead_letter` are `pub` fields

**Location:** `src/store.rs`

```rust
pub struct EventStore {
    pub queue: VecDeque<Event>,
    pub dead_letter: Vec<Event>,
    pub max_retries: u32,
}
```

**Problem:** Exposing the queue as a `pub VecDeque` means:
- External code can push to the front, pop from the back, or clear the queue entirely, bypassing the FIFO ordering that `enqueue`/`dequeue` enforces
- External code can directly manipulate the dead letter queue, which should only be populated via `move_to_dead_letter`
- The data structure is locked in — switching from `VecDeque` to a `BinaryHeap` (for priority delivery) would break all callers

In `DeliveryWorker`, the comment says: "pub — but EventStore.queue is also pub, so callers can bypass the worker entirely." That's not a justification for the design — it's an acknowledgment that the design is broken.

**Fix:** Make fields private, expose only what's necessary:

```rust
pub struct EventStore {
    queue: VecDeque<Event>,        // private
    dead_letter: Vec<Event>,       // private
    pub(crate) max_retries: u32,   // crate-internal (delivery uses this)
}

impl EventStore {
    pub fn enqueue(&mut self, event: Event) { ... }
    pub fn dequeue(&mut self) -> Option<Event> { ... }
    pub(crate) fn move_to_dead_letter(&mut self, event: Event) { ... }
    pub fn queue_depth(&self) -> usize { ... }
    pub fn dead_letter_count(&self) -> usize { self.dead_letter.len() }
}
```

---

## Minor Suggestions

### 5. Glob imports in `delivery.rs` — opaque provenance

**Location:** `src/delivery.rs`

```rust
use crate::events::*;
use crate::routing::*;
use crate::store::*;
```

**Problem:** When reading the code, it's impossible to know where `Event`, `Route`, or `EventStore` come from without cross-referencing the modules. With a large codebase, this becomes a real friction point. Clippy warns about this in non-test code (`clippy::wildcard_imports`).

**Fix:**

```rust
use crate::events::{Event, EventKind};
use crate::routing::{Route, Router, match_routes};
use crate::store::EventStore;
```

Note: glob imports in `#[cfg(test)]` modules (`use super::*;`) are idiomatic and fine — the author correctly uses them in the test module.

---

### 6. `match_routes` is `pub` but is an internal implementation detail

**Location:** `src/routing.rs`

```rust
/// Matches an event to the routes that should receive it.
/// This is an internal function used only by the delivery module.
/// It should not be part of the external API.
pub fn match_routes<'a>(event: &Event, routes: &'a [Route]) -> Vec<&'a Route> { ... }
```

The comment literally says "should not be part of the external API" — but then it's `pub`.

**Fix:** Change to `pub(crate)`:

```rust
pub(crate) fn match_routes<'a>(event: &Event, routes: &'a [Route]) -> Vec<&'a Route> { ... }
```

Or remove it entirely — `Router::routes_for` already encapsulates this call. `match_routes` as a free function only exists because the author wasn't sure whether to put logic in the method or the free function. The method is cleaner.

---

### 7. Silent drop when no routes match

**Location:** `src/delivery.rs`, `DeliveryWorker::process_next`

```rust
let routes = self.router.routes_for(&event);
if routes.is_empty() {
    return true;  // no routes — silently drop
}
```

Not a visibility bug, but worth flagging in a code review: silently dropping events with no matching routes is almost certainly wrong in a production webhook relay. At minimum, these should go to the dead letter queue. This should be a conscious decision, not an implicit default.

---

## Positive Feedback

1. **Good separation of concerns** — splitting events, routing, delivery, and storage into separate modules is the right call. The concerns are genuinely distinct. The module boundaries make sense; the visibility within them just needs work.

2. **`Router::routes_for` method encapsulates `match_routes`** — building the method on top of the helper function is good layering. Callers use `router.routes_for(&event)`, not `match_routes(&event, &router.routes)`.

3. **`DeliveryWorker` owns its dependencies** — passing `EventStore` and `Router` by value into `DeliveryWorker::new` is good ownership design. The worker is the single owner, which simplifies lifetime reasoning.

4. **`increment_attempts` as a method** — rather than exposing `metadata.attempt_count` as mutable, the PR provides `increment_attempts()`. The right instinct, undermined only by the public `metadata` field that bypasses it.

5. **Tests are well-structured** — covering basic routing, delivery, retry with dead-lettering, and exponential backoff. The `make_event` and `make_route` helpers reduce test boilerplate.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `Route::secret` is `pub` | Private fields for sensitive data |
| 2 | Major | No `pub use` re-exports in `lib.rs` | Stable public API via re-exports |
| 3 | Major | `EventMetadata` and `Event::metadata` are `pub` | `pub(crate)` for internal types |
| 4 | Major | `EventStore::queue` and `dead_letter` are `pub` | Encapsulation via private fields |
| 5 | Minor | Glob imports in `delivery.rs` | Explicit imports for readability |
| 6 | Minor | `match_routes` is `pub` despite being internal | `pub(crate)` vs `pub` |
| 7 | Minor | Silent drop when no routes match | Explicit error handling in relay logic |

## Related Concepts

- [[fundamentals/rust/modules-and-packages]] — visibility modifiers in depth
- [[pitfalls/rust-pub-everything]] — the cost of over-exposing internal types
- [[pitfalls/rust-missing-pub-use]] — when to add re-exports
