// Repository Pattern: Event Store
//
// Demonstrates: Append-only repository for event sourcing. Shows how
// ownership works for repositories that only add data and support
// time-range queries. Events are immutable once stored.
//
// Scenario: An audit log / event store for a payment processing system.
// Events are appended and queried by entity ID, time range, or event type.
// The store is append-only -- events are never updated or deleted.
//
// Run: rustc event_store.rs && ./event_store
// Test: rustc --test event_store.rs && ./event_store

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: String,
    pub entity_id: String,
    pub event_type: EventType,
    pub timestamp: u64,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    PaymentInitiated,
    PaymentAuthorized,
    PaymentCaptured,
    PaymentRefunded,
    PaymentFailed,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::PaymentInitiated => write!(f, "payment.initiated"),
            EventType::PaymentAuthorized => write!(f, "payment.authorized"),
            EventType::PaymentCaptured => write!(f, "payment.captured"),
            EventType::PaymentRefunded => write!(f, "payment.refunded"),
            EventType::PaymentFailed => write!(f, "payment.failed"),
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum EventStoreError {
    DuplicateEvent { id: String },
    InvalidTimeRange { from: u64, to: u64 },
    Internal { message: String },
}

impl fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStoreError::DuplicateEvent { id } => {
                write!(f, "duplicate event: {}", id)
            }
            EventStoreError::InvalidTimeRange { from, to } => {
                write!(f, "invalid time range: {} to {}", from, to)
            }
            EventStoreError::Internal { message } => {
                write!(f, "internal error: {}", message)
            }
        }
    }
}

impl std::error::Error for EventStoreError {}

// ---------------------------------------------------------------------------
// Event store trait -- append-only repository
// ---------------------------------------------------------------------------

/// An append-only event store.
///
/// Key differences from a standard CRUD repository:
/// - No `update` or `delete` operations (events are immutable)
/// - Query-oriented: events are retrieved by entity, time range, or type
/// - `append` takes ownership of the Event (the store becomes the owner)
///   vs standard repos where `save` borrows the entity
pub trait EventStore {
    /// Append an event to the store. The store takes ownership.
    /// Returns the event ID on success.
    fn append(&mut self, event: Event) -> Result<String, EventStoreError>;

    /// Retrieve all events for a given entity, ordered by timestamp.
    fn events_for_entity(&self, entity_id: &str) -> Result<Vec<Event>, EventStoreError>;

    /// Query events within a time range (inclusive on both ends).
    fn events_in_range(&self, from: u64, to: u64) -> Result<Vec<Event>, EventStoreError>;

    /// Query events by type.
    fn events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError>;

    /// Get the latest event for an entity (most recent by timestamp).
    fn latest_event(&self, entity_id: &str) -> Result<Option<Event>, EventStoreError>;

    /// Count total events in the store.
    fn count(&self) -> usize;
}

// ---------------------------------------------------------------------------
// In-memory implementation
// ---------------------------------------------------------------------------

pub struct InMemoryEventStore {
    /// All events, keyed by event ID for deduplication
    events: HashMap<String, Event>,
    /// Index: entity_id -> list of event IDs (ordered by insertion)
    entity_index: HashMap<String, Vec<String>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            entity_index: HashMap::new(),
        }
    }
}

impl EventStore for InMemoryEventStore {
    fn append(&mut self, event: Event) -> Result<String, EventStoreError> {
        // Check for duplicate event IDs (idempotency)
        if self.events.contains_key(&event.id) {
            return Err(EventStoreError::DuplicateEvent {
                id: event.id.clone(),
            });
        }

        let event_id = event.id.clone();
        let entity_id = event.entity_id.clone();

        // Update entity index
        self.entity_index
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(event_id.clone());

        // Store the event (ownership transfers to the HashMap)
        self.events.insert(event_id.clone(), event);

        Ok(event_id)
    }

    fn events_for_entity(&self, entity_id: &str) -> Result<Vec<Event>, EventStoreError> {
        let events = self
            .entity_index
            .get(entity_id)
            .map(|ids| {
                let mut events: Vec<Event> = ids
                    .iter()
                    .filter_map(|id| self.events.get(id).cloned())
                    .collect();
                events.sort_by_key(|e| e.timestamp);
                events
            })
            .unwrap_or_default();

        Ok(events)
    }

    fn events_in_range(&self, from: u64, to: u64) -> Result<Vec<Event>, EventStoreError> {
        if from > to {
            return Err(EventStoreError::InvalidTimeRange { from, to });
        }

        let mut events: Vec<Event> = self
            .events
            .values()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .cloned()
            .collect();

        events.sort_by_key(|e| e.timestamp);
        Ok(events)
    }

    fn events_by_type(&self, event_type: &EventType) -> Result<Vec<Event>, EventStoreError> {
        let mut events: Vec<Event> = self
            .events
            .values()
            .filter(|e| &e.event_type == event_type)
            .cloned()
            .collect();

        events.sort_by_key(|e| e.timestamp);
        Ok(events)
    }

    fn latest_event(&self, entity_id: &str) -> Result<Option<Event>, EventStoreError> {
        let latest = self
            .entity_index
            .get(entity_id)
            .and_then(|ids| {
                ids.iter()
                    .filter_map(|id| self.events.get(id))
                    .max_by_key(|e| e.timestamp)
            })
            .cloned();

        Ok(latest)
    }

    fn count(&self) -> usize {
        self.events.len()
    }
}

// ---------------------------------------------------------------------------
// Service layer: payment state reconstruction from events
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum PaymentStatus {
    Initiated,
    Authorized,
    Captured,
    Refunded,
    Failed,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Initiated => write!(f, "initiated"),
            PaymentStatus::Authorized => write!(f, "authorized"),
            PaymentStatus::Captured => write!(f, "captured"),
            PaymentStatus::Refunded => write!(f, "refunded"),
            PaymentStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Reconstructs the current payment status from the event stream.
/// This is the core idea of event sourcing: state is derived from events.
fn reconstruct_payment_status<S: EventStore>(
    store: &S,
    payment_id: &str,
) -> Result<Option<PaymentStatus>, EventStoreError> {
    let latest = store.latest_event(payment_id)?;

    Ok(latest.map(|event| match event.event_type {
        EventType::PaymentInitiated => PaymentStatus::Initiated,
        EventType::PaymentAuthorized => PaymentStatus::Authorized,
        EventType::PaymentCaptured => PaymentStatus::Captured,
        EventType::PaymentRefunded => PaymentStatus::Refunded,
        EventType::PaymentFailed => PaymentStatus::Failed,
    }))
}

/// Builds a timeline summary for a payment.
fn payment_timeline<S: EventStore>(
    store: &S,
    payment_id: &str,
) -> Result<Vec<String>, EventStoreError> {
    let events = store.events_for_entity(payment_id)?;

    let timeline: Vec<String> = events
        .iter()
        .map(|e| format!("[t={}] {} -- {}", e.timestamp, e.event_type, e.payload))
        .collect();

    Ok(timeline)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(id: &str, entity_id: &str, event_type: EventType, ts: u64) -> Event {
        Event {
            id: id.to_string(),
            entity_id: entity_id.to_string(),
            event_type,
            timestamp: ts,
            payload: format!("event {} at t={}", id, ts),
        }
    }

    #[test]
    fn test_append_and_retrieve() {
        let mut store = InMemoryEventStore::new();
        let event = make_event("e1", "pay-001", EventType::PaymentInitiated, 1000);

        store.append(event.clone()).unwrap();

        let events = store.events_for_entity("pay-001").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn test_duplicate_event_rejected() {
        let mut store = InMemoryEventStore::new();
        let event = make_event("e1", "pay-001", EventType::PaymentInitiated, 1000);

        store.append(event.clone()).unwrap();
        let result = store.append(event);
        assert!(matches!(result, Err(EventStoreError::DuplicateEvent { .. })));
    }

    #[test]
    fn test_events_ordered_by_timestamp() {
        let mut store = InMemoryEventStore::new();
        // Insert out of order
        store
            .append(make_event(
                "e3",
                "pay-001",
                EventType::PaymentCaptured,
                3000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e2",
                "pay-001",
                EventType::PaymentAuthorized,
                2000,
            ))
            .unwrap();

        let events = store.events_for_entity("pay-001").unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].timestamp, 1000);
        assert_eq!(events[1].timestamp, 2000);
        assert_eq!(events[2].timestamp, 3000);
    }

    #[test]
    fn test_events_in_range() {
        let mut store = InMemoryEventStore::new();
        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e2",
                "pay-001",
                EventType::PaymentAuthorized,
                2000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e3",
                "pay-002",
                EventType::PaymentInitiated,
                3000,
            ))
            .unwrap();

        let events = store.events_in_range(1500, 2500).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "e2");
    }

    #[test]
    fn test_invalid_time_range() {
        let store = InMemoryEventStore::new();
        let result = store.events_in_range(3000, 1000);
        assert!(matches!(
            result,
            Err(EventStoreError::InvalidTimeRange { .. })
        ));
    }

    #[test]
    fn test_events_by_type() {
        let mut store = InMemoryEventStore::new();
        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e2",
                "pay-002",
                EventType::PaymentInitiated,
                2000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e3",
                "pay-001",
                EventType::PaymentCaptured,
                3000,
            ))
            .unwrap();

        let initiated = store.events_by_type(&EventType::PaymentInitiated).unwrap();
        assert_eq!(initiated.len(), 2);

        let captured = store.events_by_type(&EventType::PaymentCaptured).unwrap();
        assert_eq!(captured.len(), 1);
    }

    #[test]
    fn test_latest_event() {
        let mut store = InMemoryEventStore::new();
        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e2",
                "pay-001",
                EventType::PaymentAuthorized,
                2000,
            ))
            .unwrap();

        let latest = store.latest_event("pay-001").unwrap().unwrap();
        assert_eq!(latest.event_type, EventType::PaymentAuthorized);
        assert_eq!(latest.timestamp, 2000);
    }

    #[test]
    fn test_latest_event_no_events() {
        let store = InMemoryEventStore::new();
        assert!(store.latest_event("pay-999").unwrap().is_none());
    }

    #[test]
    fn test_reconstruct_payment_status() {
        let mut store = InMemoryEventStore::new();
        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        assert_eq!(
            reconstruct_payment_status(&store, "pay-001").unwrap(),
            Some(PaymentStatus::Initiated)
        );

        store
            .append(make_event(
                "e2",
                "pay-001",
                EventType::PaymentAuthorized,
                2000,
            ))
            .unwrap();
        assert_eq!(
            reconstruct_payment_status(&store, "pay-001").unwrap(),
            Some(PaymentStatus::Authorized)
        );

        store
            .append(make_event(
                "e3",
                "pay-001",
                EventType::PaymentCaptured,
                3000,
            ))
            .unwrap();
        assert_eq!(
            reconstruct_payment_status(&store, "pay-001").unwrap(),
            Some(PaymentStatus::Captured)
        );
    }

    #[test]
    fn test_reconstruct_unknown_payment() {
        let store = InMemoryEventStore::new();
        assert_eq!(
            reconstruct_payment_status(&store, "pay-999").unwrap(),
            None
        );
    }

    #[test]
    fn test_count() {
        let mut store = InMemoryEventStore::new();
        assert_eq!(store.count(), 0);

        store
            .append(make_event(
                "e1",
                "pay-001",
                EventType::PaymentInitiated,
                1000,
            ))
            .unwrap();
        store
            .append(make_event(
                "e2",
                "pay-002",
                EventType::PaymentInitiated,
                2000,
            ))
            .unwrap();
        assert_eq!(store.count(), 2);
    }
}

// ---------------------------------------------------------------------------
// Main: demo
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Event Store Repository Demo ===\n");

    let mut store = InMemoryEventStore::new();

    // Simulate a payment lifecycle
    println!("--- Payment pay-001 lifecycle ---");
    store
        .append(Event {
            id: "e1".to_string(),
            entity_id: "pay-001".to_string(),
            event_type: EventType::PaymentInitiated,
            timestamp: 1000,
            payload: r#"{"amount": 99.99, "currency": "USD"}"#.to_string(),
        })
        .unwrap();

    store
        .append(Event {
            id: "e2".to_string(),
            entity_id: "pay-001".to_string(),
            event_type: EventType::PaymentAuthorized,
            timestamp: 1002,
            payload: r#"{"auth_code": "AUTH-4829"}"#.to_string(),
        })
        .unwrap();

    store
        .append(Event {
            id: "e3".to_string(),
            entity_id: "pay-001".to_string(),
            event_type: EventType::PaymentCaptured,
            timestamp: 1005,
            payload: r#"{"capture_id": "CAP-7721"}"#.to_string(),
        })
        .unwrap();

    // A second payment that failed
    println!("--- Payment pay-002 lifecycle ---");
    store
        .append(Event {
            id: "e4".to_string(),
            entity_id: "pay-002".to_string(),
            event_type: EventType::PaymentInitiated,
            timestamp: 2000,
            payload: r#"{"amount": 250.00, "currency": "EUR"}"#.to_string(),
        })
        .unwrap();

    store
        .append(Event {
            id: "e5".to_string(),
            entity_id: "pay-002".to_string(),
            event_type: EventType::PaymentFailed,
            timestamp: 2003,
            payload: r#"{"reason": "insufficient funds"}"#.to_string(),
        })
        .unwrap();

    // Reconstruct state
    println!("\n--- Current payment statuses ---");
    let status1 = reconstruct_payment_status(&store, "pay-001").unwrap().unwrap();
    println!("  pay-001: {}", status1);

    let status2 = reconstruct_payment_status(&store, "pay-002").unwrap().unwrap();
    println!("  pay-002: {}", status2);

    // Timeline
    println!("\n--- Payment pay-001 timeline ---");
    for line in payment_timeline(&store, "pay-001").unwrap() {
        println!("  {}", line);
    }

    // Range query
    println!("\n--- Events between t=1000 and t=1003 ---");
    for event in store.events_in_range(1000, 1003).unwrap() {
        println!("  [t={}] {} ({})", event.timestamp, event.event_type, event.entity_id);
    }

    // Type query
    println!("\n--- All 'initiated' events ---");
    for event in store.events_by_type(&EventType::PaymentInitiated).unwrap() {
        println!("  {} at t={}", event.entity_id, event.timestamp);
    }

    println!("\n--- Store stats ---");
    println!("  Total events: {}", store.count());
}
