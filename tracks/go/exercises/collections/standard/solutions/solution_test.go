package eventstore

import (
	"reflect"
	"testing"
)

// ============================================================================
// Helpers
// ============================================================================

func makeEvent(id, typ string, ts int64) Event {
	return Event{ID: id, Type: typ, Timestamp: ts, Payload: map[string]string{"key": "val"}}
}

func eventIDs(events []Event) []string {
	ids := make([]string, len(events))
	for i, e := range events {
		ids[i] = e.ID
	}
	return ids
}

// ============================================================================
// NewEventStore
// ============================================================================

func TestNewEventStore_IsEmpty(t *testing.T) {
	s := NewEventStore()
	if s.Len() != 0 {
		t.Errorf("expected empty store, got len=%d", s.Len())
	}
}

func TestNewEventStore_AllReturnsEmpty(t *testing.T) {
	s := NewEventStore()
	got := s.All()
	if got == nil {
		t.Fatal("All() returned nil; want []Event{}")
	}
	if len(got) != 0 {
		t.Errorf("All() on empty store: got %d events", len(got))
	}
}

// ============================================================================
// Append and Len
// ============================================================================

func TestAppend_Len(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "purchase", 200))
	s.Append(makeEvent("e3", "login", 300))

	if s.Len() != 3 {
		t.Errorf("expected Len()=3, got %d", s.Len())
	}
}

func TestAll_InsertionOrder(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "purchase", 200))
	s.Append(makeEvent("e3", "error", 300))

	got := eventIDs(s.All())
	want := []string{"e1", "e2", "e3"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("All() order: got %v, want %v", got, want)
	}
}

// ============================================================================
// ByType
// ============================================================================

func TestByType_ReturnsMatchingEvents(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))
	s.Append(makeEvent("e3", "login", 300))
	s.Append(makeEvent("e4", "purchase", 400))
	s.Append(makeEvent("e5", "login", 500))

	got := eventIDs(s.ByType("login"))
	want := []string{"e1", "e3", "e5"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("ByType(login): got %v, want %v", got, want)
	}
}

func TestByType_EmptyForUnknownType(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))

	got := s.ByType("nonexistent")
	if got == nil {
		t.Fatal("ByType returned nil; want []Event{}")
	}
	if len(got) != 0 {
		t.Errorf("expected empty, got %v", got)
	}
}

func TestByType_IndexIsUpdatedAfterEachAppend(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))

	if len(s.ByType("login")) != 1 {
		t.Fatal("index not updated after first append")
	}

	s.Append(makeEvent("e2", "login", 200))
	if len(s.ByType("login")) != 2 {
		t.Fatal("index not updated after second append")
	}
}

// ============================================================================
// ByTimeRange
// ============================================================================

func TestByTimeRange_InclusiveBothEnds(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))
	s.Append(makeEvent("e3", "purchase", 300))
	s.Append(makeEvent("e4", "login", 400))
	s.Append(makeEvent("e5", "error", 500))

	got := eventIDs(s.ByTimeRange(200, 400))
	want := []string{"e2", "e3", "e4"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("ByTimeRange(200,400): got %v, want %v", got, want)
	}
}

func TestByTimeRange_EmptyWhenNoMatch(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))

	got := s.ByTimeRange(500, 1000)
	if got == nil {
		t.Fatal("ByTimeRange returned nil; want []Event{}")
	}
	if len(got) != 0 {
		t.Errorf("expected empty, got %v", got)
	}
}

func TestByTimeRange_SingleEvent(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))

	got := eventIDs(s.ByTimeRange(100, 100))
	want := []string{"e1"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("ByTimeRange(100,100): got %v, want %v", got, want)
	}
}

func TestByTimeRange_EmptyStore(t *testing.T) {
	s := NewEventStore()
	got := s.ByTimeRange(0, 1000)
	if got == nil {
		t.Fatal("ByTimeRange on empty store returned nil; want []Event{}")
	}
}

// ============================================================================
// Latest
// ============================================================================

func TestLatest_ReturnsNMostRecentInReverseOrder(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))
	s.Append(makeEvent("e3", "purchase", 300))
	s.Append(makeEvent("e4", "login", 400))
	s.Append(makeEvent("e5", "error", 500))

	got := eventIDs(s.Latest(3))
	want := []string{"e5", "e4", "e3"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Latest(3): got %v, want %v", got, want)
	}
}

func TestLatest_NGreaterThanLen(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))

	got := s.Latest(100)
	if len(got) != 2 {
		t.Errorf("Latest(100) with 2 events: expected 2, got %d", len(got))
	}
}

func TestLatest_EmptyStore(t *testing.T) {
	s := NewEventStore()
	got := s.Latest(5)
	if got == nil {
		t.Fatal("Latest on empty store returned nil; want []Event{}")
	}
	if len(got) != 0 {
		t.Errorf("expected empty, got %v", got)
	}
}

func TestLatest_One(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))

	got := eventIDs(s.Latest(1))
	want := []string{"e2"}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Latest(1): got %v, want %v", got, want)
	}
}

// ============================================================================
// Snapshot
// ============================================================================

func TestSnapshot_IsIndependent(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "error", 200))

	snap := s.Snapshot()

	// Appending to original does not affect snapshot
	s.Append(makeEvent("e3", "purchase", 300))
	if snap.Len() != 2 {
		t.Errorf("snapshot Len changed after original was appended to: got %d, want 2", snap.Len())
	}
}

func TestSnapshot_AppendToSnapshotDoesNotAffectOriginal(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))

	snap := s.Snapshot()
	snap.Append(makeEvent("e2", "error", 200))

	if s.Len() != 1 {
		t.Errorf("original Len changed after appending to snapshot: got %d, want 1", s.Len())
	}
}

func TestSnapshot_PayloadIsDeepCopied(t *testing.T) {
	s := NewEventStore()
	e := Event{
		ID:        "e1",
		Type:      "login",
		Timestamp: 100,
		Payload:   map[string]string{"user": "alice"},
	}
	s.Append(e)

	snap := s.Snapshot()

	// Mutate via All() — we get a copy of the slice, but the Payload map
	// is still shared unless we deep-copied it.
	all := s.All()
	all[0].Payload["user"] = "hacker"

	snapAll := snap.All()
	if snapAll[0].Payload["user"] == "hacker" {
		t.Error("snapshot payload was not deep-copied: mutation in original leaked into snapshot")
	}
}

func TestSnapshot_TypeIndexIsDeepCopied(t *testing.T) {
	s := NewEventStore()
	s.Append(makeEvent("e1", "login", 100))
	s.Append(makeEvent("e2", "login", 200))

	snap := s.Snapshot()

	// Appending to original should not change snapshot's ByType result
	s.Append(makeEvent("e3", "login", 300))

	snapLogins := snap.ByType("login")
	if len(snapLogins) != 2 {
		t.Errorf("snapshot type index leaked: expected 2 login events, got %d", len(snapLogins))
	}
}
