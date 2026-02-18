// Webhook Relay — Proposed Module Structure (Code Review)
//
// This file represents the combined module structure the PR produces.
// In the actual PR, these would be separate files. They're combined here
// for easier review.
//
// Module structure produced by this PR:
//
//   src/
//   ├── lib.rs          (module declarations only — no pub use re-exports)
//   ├── main.rs         (imports via full paths: webhook_relay::events::Event)
//   ├── events.rs       (Event, EventKind, EventMetadata — all pub)
//   ├── delivery.rs     (DeliveryWorker, deliver, retry_policy — mixed visibility)
//   ├── routing.rs      (Router, Route, match_routes — mixed visibility)
//   └── store.rs        (EventStore, in-memory queue — fields pub)

// ---- Imagine this is src/lib.rs ----
// pub mod events;
// pub mod delivery;
// pub mod routing;
// pub mod store;
// (No pub use re-exports — callers import from full paths)

// ---- Imagine this is src/events.rs ----

/// The kind of event received from upstream.
#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    UserCreated,
    UserDeleted,
    OrderPlaced,
    OrderFulfilled,
    PaymentFailed,
}

/// Internal metadata attached to every event.
/// Not intended for external callers — used only within the relay crate.
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub received_at: u64,
    pub attempt_count: u32,
    pub source_ip: String,
}

/// An incoming webhook event.
#[derive(Debug, Clone)]
pub struct Event {
    pub id: String,
    pub kind: EventKind,
    pub payload: String,
    pub metadata: EventMetadata,  // full public access to internal metadata
}

impl Event {
    pub fn new(id: &str, kind: EventKind, payload: &str, received_at: u64, source_ip: &str) -> Event {
        Event {
            id: id.to_string(),
            kind,
            payload: payload.to_string(),
            metadata: EventMetadata {
                received_at,
                attempt_count: 0,
                source_ip: source_ip.to_string(),
            },
        }
    }

    pub fn increment_attempts(&mut self) {
        self.metadata.attempt_count += 1;
    }
}

// ---- Imagine this is src/store.rs ----

use std::collections::VecDeque;

/// An in-memory queue of events awaiting delivery.
pub struct EventStore {
    pub queue: VecDeque<Event>,   // pub field — anyone can push/pop directly
    pub dead_letter: Vec<Event>,  // pub field — events that failed all retries
    pub max_retries: u32,
}

impl EventStore {
    pub fn new(max_retries: u32) -> EventStore {
        EventStore {
            queue: VecDeque::new(),
            dead_letter: Vec::new(),
            max_retries,
        }
    }

    pub fn enqueue(&mut self, event: Event) {
        self.queue.push_back(event);
    }

    pub fn dequeue(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }

    pub fn move_to_dead_letter(&mut self, event: Event) {
        self.dead_letter.push(event);
    }

    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }
}

// ---- Imagine this is src/routing.rs ----

/// A single routing rule — matches events of a given kind to a target URL.
#[derive(Debug, Clone)]
pub struct Route {
    pub kind: EventKind,
    pub target_url: String,
    pub secret: String,   // signing secret for HMAC — should not be pub
}

/// Matches an event to the routes that should receive it.
/// Returns the matching routes.
///
/// This is an internal function used only by the delivery module.
/// It should not be part of the external API.
pub fn match_routes<'a>(event: &Event, routes: &'a [Route]) -> Vec<&'a Route> {
    routes.iter().filter(|r| r.kind == event.kind).collect()
}

/// The routing table.
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Router {
        Router { routes: Vec::new() }
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn routes_for(&self, event: &Event) -> Vec<&Route> {
        match_routes(event, &self.routes)
    }
}

// ---- Imagine this is src/delivery.rs ----

// Glob import — pulls in everything from events and routing.
// It's not clear which specific items are used.
use crate::events::*;
use crate::routing::*;
use crate::store::*;

/// How long to wait before retrying a failed delivery (milliseconds).
///
/// This is an implementation detail of the delivery algorithm.
/// External callers shouldn't configure this directly; it's part of the
/// delivery policy.
pub fn retry_delay_ms(attempt: u32) -> u64 {
    // Exponential backoff: 1s, 2s, 4s, 8s...
    1000 * 2u64.pow(attempt)
}

/// Attempt to deliver an event to a target URL.
/// Returns true if delivery succeeded.
///
/// In a real implementation, this would make an HTTP POST. For this
/// exercise it's a stub.
pub fn deliver(event: &Event, route: &Route) -> bool {
    // Would POST to route.target_url with HMAC signature
    // using route.secret
    let _ = (event, route);  // suppress unused warnings in stub
    true  // simulate success
}

/// The delivery worker processes events from the store.
pub struct DeliveryWorker {
    pub store: EventStore,    // pub — but EventStore.queue is also pub, so
                              // callers can bypass the worker entirely
    pub router: Router,
}

impl DeliveryWorker {
    pub fn new(store: EventStore, router: Router) -> DeliveryWorker {
        DeliveryWorker { store, router }
    }

    pub fn process_next(&mut self) -> bool {
        let event = match self.store.dequeue() {
            Some(e) => e,
            None => return false,
        };

        let routes = self.router.routes_for(&event);
        if routes.is_empty() {
            return true;  // no routes — silently drop
        }

        let mut success = false;
        for route in routes {
            if deliver(&event, route) {
                success = true;
            }
        }

        if !success {
            let mut failed_event = event;
            failed_event.increment_attempts();
            if failed_event.metadata.attempt_count < self.store.max_retries {
                self.store.enqueue(failed_event);
            } else {
                self.store.move_to_dead_letter(failed_event);
            }
        }

        true
    }
}

// ----- Tests -----

#[cfg(test)]
mod tests {
    // Glob import in tests — idiomatic here
    use super::*;

    fn make_event(kind: EventKind) -> Event {
        Event::new("evt-1", kind, r#"{"data": true}"#, 1000, "10.0.0.1")
    }

    fn make_route(kind: EventKind, url: &str) -> Route {
        Route {
            kind,
            target_url: url.to_string(),
            secret: "s3cr3t".to_string(),
        }
    }

    #[test]
    fn test_event_attempt_tracking() {
        let mut event = make_event(EventKind::OrderPlaced);
        assert_eq!(event.metadata.attempt_count, 0);
        event.increment_attempts();
        assert_eq!(event.metadata.attempt_count, 1);
    }

    #[test]
    fn test_router_matches_by_kind() {
        let mut router = Router::new();
        router.add_route(make_route(EventKind::UserCreated, "https://a.example.com"));
        router.add_route(make_route(EventKind::OrderPlaced, "https://b.example.com"));

        let event = make_event(EventKind::UserCreated);
        let matched = router.routes_for(&event);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].target_url, "https://a.example.com");
    }

    #[test]
    fn test_worker_processes_event() {
        let mut store = EventStore::new(3);
        let mut router = Router::new();
        router.add_route(make_route(EventKind::OrderPlaced, "https://orders.example.com"));

        let event = make_event(EventKind::OrderPlaced);
        store.enqueue(event);

        let mut worker = DeliveryWorker::new(store, router);
        let had_work = worker.process_next();
        assert!(had_work);
        assert_eq!(worker.store.queue_depth(), 0);
    }

    #[test]
    fn test_worker_empty_queue() {
        let store = EventStore::new(3);
        let router = Router::new();
        let mut worker = DeliveryWorker::new(store, router);
        assert!(!worker.process_next());
    }

    #[test]
    fn test_retry_delay_exponential() {
        assert_eq!(retry_delay_ms(0), 1000);
        assert_eq!(retry_delay_ms(1), 2000);
        assert_eq!(retry_delay_ms(2), 4000);
        assert_eq!(retry_delay_ms(3), 8000);
    }
}

fn main() {
    // In the actual PR, main.rs would look like:
    //
    //   use webhook_relay::events::{Event, EventKind};
    //   use webhook_relay::store::EventStore;
    //   use webhook_relay::routing::{Router, Route};
    //   use webhook_relay::delivery::DeliveryWorker;
    //
    // All those paths are long because there are no re-exports in lib.rs.
    // With proper pub use, callers could write:
    //   use webhook_relay::{Event, EventKind, EventStore, Router, Route, DeliveryWorker};

    let mut store = EventStore::new(3);
    let mut router = Router::new();

    router.add_route(Route {
        kind: EventKind::OrderPlaced,
        target_url: "https://orders.example.com/webhooks".to_string(),
        secret: "signing-secret-123".to_string(),
    });

    let event = Event::new(
        "evt-001",
        EventKind::OrderPlaced,
        r#"{"order_id": 42, "total": 99.99}"#,
        1_700_000_000,
        "203.0.113.42",
    );

    store.enqueue(event);

    let mut worker = DeliveryWorker::new(store, router);
    while worker.process_next() {}
    println!("Delivery complete. Dead letter queue: {}", worker.store.dead_letter.len());
}
