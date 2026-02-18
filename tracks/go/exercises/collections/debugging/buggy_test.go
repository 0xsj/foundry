package auditlog

import (
	"reflect"
	"sort"
	"testing"
)

// ============================================================================
// Bug 1: ProcessBatch — nil map panic
// ============================================================================

func TestProcessBatch_DoesNotPanic(t *testing.T) {
	entries := []LogEntry{
		{ID: "1", UserID: "alice", Service: "auth", Action: "login"},
		{ID: "2", UserID: "bob", Service: "payments", Action: "purchase"},
		{ID: "3", UserID: "alice", Service: "payments", Action: "purchase"},
	}
	// Should not panic
	got := ProcessBatch(entries)
	if got == nil {
		t.Fatal("ProcessBatch returned nil map")
	}
}

func TestProcessBatch_ActionCounts(t *testing.T) {
	entries := []LogEntry{
		{ID: "1", UserID: "alice", Service: "auth", Action: "login"},
		{ID: "2", UserID: "alice", Service: "auth", Action: "logout"},
		{ID: "3", UserID: "alice", Service: "payments", Action: "purchase"},
		{ID: "4", UserID: "bob", Service: "auth", Action: "login"},
	}
	got := ProcessBatch(entries)
	if got["alice"].ActionCount != 3 {
		t.Errorf("alice ActionCount: got %d, want 3", got["alice"].ActionCount)
	}
	if got["bob"].ActionCount != 1 {
		t.Errorf("bob ActionCount: got %d, want 1", got["bob"].ActionCount)
	}
}

func TestProcessBatch_UniqueServices(t *testing.T) {
	entries := []LogEntry{
		{ID: "1", UserID: "alice", Service: "auth", Action: "login"},
		{ID: "2", UserID: "alice", Service: "auth", Action: "logout"}, // same service, should not duplicate
		{ID: "3", UserID: "alice", Service: "payments", Action: "purchase"},
	}
	got := ProcessBatch(entries)
	services := make([]string, len(got["alice"].Services))
	copy(services, got["alice"].Services)
	sort.Strings(services)

	want := []string{"auth", "payments"}
	if !reflect.DeepEqual(services, want) {
		t.Errorf("alice services: got %v, want %v", services, want)
	}
}

// ============================================================================
// Bug 2: SplitByService — shared backing array corruption
// ============================================================================

func TestSplitByService_ReturnsCorrectGroups(t *testing.T) {
	entries := []LogEntry{
		{ID: "1", Service: "auth"},
		{ID: "2", Service: "payments"},
		{ID: "3", Service: "auth"},
		{ID: "4", Service: "inventory"},
		{ID: "5", Service: "auth"},
	}

	matched, rest := SplitByService(entries, "auth")

	matchedIDs := make([]string, len(matched))
	for i, e := range matched {
		matchedIDs[i] = e.ID
	}
	restIDs := make([]string, len(rest))
	for i, e := range rest {
		restIDs[i] = e.ID
	}

	sort.Strings(matchedIDs)
	sort.Strings(restIDs)

	if !reflect.DeepEqual(matchedIDs, []string{"1", "3", "5"}) {
		t.Errorf("matched IDs: got %v, want [1 3 5]", matchedIDs)
	}
	if !reflect.DeepEqual(restIDs, []string{"2", "4"}) {
		t.Errorf("rest IDs: got %v, want [2 4]", restIDs)
	}
}

func TestSplitByService_SlicesAreIndependent(t *testing.T) {
	// This test exposes the backing array corruption bug.
	// If matched and rest share a backing array, appending to matched
	// will overwrite the start of rest.
	entries := []LogEntry{
		{ID: "1", Service: "auth"},
		{ID: "2", Service: "payments"},  // rest[0]
		{ID: "3", Service: "auth"},
		{ID: "4", Service: "inventory"}, // rest[1]
	}

	matched, rest := SplitByService(entries, "auth")

	// Capture rest before corruption
	originalRestID0 := rest[0].ID
	originalRestID1 := rest[1].ID

	// Append to matched (within the old backing array capacity)
	matched = append(matched, LogEntry{ID: "corrupted", Service: "auth"})
	_ = matched

	// rest must not be affected
	if rest[0].ID != originalRestID0 {
		t.Errorf("rest[0].ID corrupted: got %q, want %q — shared backing array bug!", rest[0].ID, originalRestID0)
	}
	if rest[1].ID != originalRestID1 {
		t.Errorf("rest[1].ID corrupted: got %q, want %q — shared backing array bug!", rest[1].ID, originalRestID1)
	}
}

// ============================================================================
// Bug 3: MarkProcessed — range value copy
// ============================================================================

func TestMarkProcessed_ModifiesOriginalSlice(t *testing.T) {
	entries := []LogEntry{
		{ID: "1", Service: "auth", Action: "login"},
		{ID: "2", Service: "payments", Action: "purchase"},
		{ID: "3", Service: "auth", Action: "logout"},
	}

	MarkProcessed(entries, "auth")

	if entries[0].Action != "processed" {
		t.Errorf("entries[0].Action: got %q, want %q", entries[0].Action, "processed")
	}
	if entries[1].Action != "purchase" {
		t.Errorf("entries[1].Action should be unchanged: got %q", entries[1].Action)
	}
	if entries[2].Action != "processed" {
		t.Errorf("entries[2].Action: got %q, want %q", entries[2].Action, "processed")
	}
}

// ============================================================================
// Bug 4: DeduplicateIDs — append result ignored
// ============================================================================

func TestDeduplicateIDs_RemovesDuplicates(t *testing.T) {
	ids := []string{"a", "b", "a", "c", "b", "d", "a"}
	DeduplicateIDs(&ids)

	// Order must be preserved (first occurrence kept)
	want := []string{"a", "b", "c", "d"}
	if !reflect.DeepEqual(ids, want) {
		t.Errorf("DeduplicateIDs: got %v, want %v", ids, want)
	}
}

func TestDeduplicateIDs_NoDuplicates_Unchanged(t *testing.T) {
	ids := []string{"x", "y", "z"}
	DeduplicateIDs(&ids)
	if len(ids) != 3 {
		t.Errorf("DeduplicateIDs on unique slice: got len=%d, want 3", len(ids))
	}
}

func TestDeduplicateIDs_Empty(t *testing.T) {
	ids := []string{}
	DeduplicateIDs(&ids)
	if len(ids) != 0 {
		t.Errorf("DeduplicateIDs on empty slice: got %v", ids)
	}
}
