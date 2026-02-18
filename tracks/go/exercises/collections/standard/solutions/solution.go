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
// Design decisions:
//   - events []Event stores all events in insertion order. Append-only.
//   - index map[string][]int maps event type → positions in events slice.
//     This gives O(1) type lookup without duplicating Event data.
//   - All queries return []Event{} (empty, not nil) when no results match,
//     so callers can always safely range over results without nil checks.
type EventStore struct {
	events []Event
	index  map[string][]int
}

// NewEventStore creates a ready-to-use EventStore with preallocated storage.
func NewEventStore() *EventStore {
	return &EventStore{
		events: make([]Event, 0),
		index:  make(map[string][]int),
	}
}

// Append adds an event to the store and updates the type index.
//
// The current length before appending is the new event's position index.
// We record that position in the type index before the actual append.
func (s *EventStore) Append(e Event) {
	pos := len(s.events) // position this event will occupy
	s.events = append(s.events, e)
	s.index[e.Type] = append(s.index[e.Type], pos)
	// Note: append to nil slice (first event of a type) works fine —
	// nil + append = new slice. No need to check if key exists first.
}

// Len returns the total number of events in the store.
func (s *EventStore) Len() int {
	return len(s.events)
}

// All returns all events in insertion order.
// Returns an empty slice (not nil) if the store is empty.
func (s *EventStore) All() []Event {
	if len(s.events) == 0 {
		return []Event{}
	}
	// Return a copy so callers can't mutate the store's internal slice.
	// We copy the slice header and the elements — but NOT the Payload maps
	// (shallow copy). Deep copy is reserved for Snapshot().
	result := make([]Event, len(s.events))
	copy(result, s.events)
	return result
}

// ByType returns all events of the given type, in insertion order.
// Returns an empty slice (not nil) if no events of that type exist.
//
// The type index gives us positions without scanning all events.
func (s *EventStore) ByType(t string) []Event {
	positions, ok := s.index[t]
	if !ok || len(positions) == 0 {
		return []Event{}
	}

	result := make([]Event, len(positions))
	for i, pos := range positions {
		result[i] = s.events[pos]
	}
	return result
}

// ByTimeRange returns events with from <= e.Timestamp <= to, in insertion order.
// Returns an empty slice (not nil) if no events fall in the range.
//
// Linear scan is acceptable here — no pre-built time index.
// For high-cardinality time queries, you'd add a sorted time index.
func (s *EventStore) ByTimeRange(from, to int64) []Event {
	result := make([]Event, 0)
	for _, e := range s.events {
		if e.Timestamp >= from && e.Timestamp <= to {
			result = append(result, e)
		}
	}
	return result
}

// Latest returns the n most-recent events, most-recent first.
// If n >= total events, all events are returned (most-recent first).
// Returns an empty slice (not nil) if the store is empty.
func (s *EventStore) Latest(n int) []Event {
	total := len(s.events)
	if total == 0 {
		return []Event{}
	}
	if n > total {
		n = total
	}

	// Iterate backwards from the end of the events slice.
	// This avoids sorting and works because events are in insertion order.
	result := make([]Event, n)
	for i := 0; i < n; i++ {
		result[i] = s.events[total-1-i]
	}
	return result
}

// Snapshot returns a deep copy of the EventStore.
// Mutations to the snapshot do not affect the original, and vice versa.
//
// Deep copy is required for three reasons:
//  1. Slices share backing arrays — copying the header isn't enough.
//  2. The type index contains []int slices that would share backing arrays.
//  3. Event.Payload maps are reference types — they need per-map copies.
func (s *EventStore) Snapshot() EventStore {
	// Deep-copy events slice (including Payload maps)
	eventsCopy := make([]Event, len(s.events))
	for i, e := range s.events {
		eventsCopy[i] = copyEvent(e)
	}

	// Deep-copy type index — each inner []int needs its own backing array
	indexCopy := make(map[string][]int, len(s.index))
	for k, positions := range s.index {
		posCopy := make([]int, len(positions))
		copy(posCopy, positions)
		indexCopy[k] = posCopy
	}

	return EventStore{
		events: eventsCopy,
		index:  indexCopy,
	}
}

// copyEvent returns a deep copy of an Event, including its Payload map.
// Without this, two EventStores sharing a snapshot would share the same
// map pointer and mutations through one would be visible through the other.
func copyEvent(e Event) Event {
	if e.Payload == nil {
		return e
	}
	payload := make(map[string]string, len(e.Payload))
	for k, v := range e.Payload {
		payload[k] = v
	}
	e.Payload = payload
	return e
}
