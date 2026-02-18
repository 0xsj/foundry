package eventstore

// Event represents a structured event emitted by a service.
type Event struct {
	ID        string
	Type      string            // e.g., "error", "purchase", "login"
	Timestamp int64             // Unix milliseconds
	Payload   map[string]string // arbitrary metadata
}

// EventStore is an ordered, indexed, in-memory event store.
//
// Events are stored in insertion order. A type index maps event types to their
// positions in the main events slice for O(1) type-based lookups.
//
// TODO: Add the fields you need.
type EventStore struct {
	// Hint: you need a []Event to store events in order,
	// and a map[string][]int to index positions by type.
}

// NewEventStore creates a ready-to-use EventStore.
func NewEventStore() *EventStore {
	// TODO: Initialize the EventStore with empty (not nil) collections.
	panic("not implemented")
}

// Append adds an event to the store and updates the type index.
func (s *EventStore) Append(e Event) {
	// TODO:
	// 1. Record the current length (this will be the new event's index).
	// 2. Append the event to s.events.
	// 3. Append the recorded index to s.index[e.Type].
	panic("not implemented")
}

// Len returns the total number of events in the store.
func (s *EventStore) Len() int {
	// TODO
	panic("not implemented")
}

// All returns all events in insertion order.
// Returns []Event{} (not nil) if empty.
func (s *EventStore) All() []Event {
	// TODO
	panic("not implemented")
}

// ByType returns all events of the given type, in insertion order.
// Returns []Event{} (not nil) if no events of that type exist.
//
// Hint: use s.index to look up positions, then fetch from s.events.
func (s *EventStore) ByType(t string) []Event {
	// TODO
	panic("not implemented")
}

// ByTimeRange returns events with from <= e.Timestamp <= to, in insertion order.
// Returns []Event{} (not nil) if no events fall in the range.
func (s *EventStore) ByTimeRange(from, to int64) []Event {
	// TODO: Linear scan is fine here.
	panic("not implemented")
}

// Latest returns the n most-recent events, most-recent first.
// If n >= total events, returns all events (most-recent first).
// Returns []Event{} (not nil) if the store is empty.
func (s *EventStore) Latest(n int) []Event {
	// TODO: Do not sort — iterate in reverse.
	panic("not implemented")
}

// Snapshot returns a deep copy of the EventStore.
// Mutations to the snapshot do not affect the original, and vice versa.
//
// Hint: you need to deep-copy:
//   - the events slice (including each Event's Payload map)
//   - the type index (map[string][]int — including the inner []int slices)
func (s *EventStore) Snapshot() EventStore {
	// TODO
	panic("not implemented")
}
