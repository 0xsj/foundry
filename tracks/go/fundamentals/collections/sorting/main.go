// Sorting — sort.Slice, custom sort.Interface, slices package (Go 1.21+).
//
// Run with: go run ./sorting/
package main

import (
	"cmp"
	"fmt"
	"slices"
	"sort"
)

type Event struct {
	ID        string
	Timestamp int64
	Priority  int
	Type      string
}

func main() {
	builtinSort()
	sortSlice()
	sortInterface()
	slicesPackage()
	multiFieldSort()
	binarySearch()
}

// ============================================================================
// Built-in Sort Functions
// ============================================================================

func builtinSort() {
	fmt.Println("=== Built-in Sort Functions ===")

	nums := []int{5, 2, 8, 1, 9, 3}
	sort.Ints(nums)
	fmt.Printf("sort.Ints:    %v\n", nums)

	strs := []string{"banana", "apple", "cherry", "date"}
	sort.Strings(strs)
	fmt.Printf("sort.Strings: %v\n", strs)

	fmt.Printf("is sorted: %v\n", sort.IntsAreSorted(nums))

	fmt.Println()
}

// ============================================================================
// sort.Slice — Ad-Hoc Comparators
// ============================================================================

func sortSlice() {
	fmt.Println("=== sort.Slice: Ad-Hoc Comparators ===")

	events := []Event{
		{ID: "e1", Timestamp: 1000, Priority: 2, Type: "click"},
		{ID: "e2", Timestamp: 500,  Priority: 1, Type: "hover"},
		{ID: "e3", Timestamp: 750,  Priority: 3, Type: "submit"},
		{ID: "e4", Timestamp: 250,  Priority: 2, Type: "keydown"},
	}

	// Sort by timestamp ascending
	sort.Slice(events, func(i, j int) bool {
		return events[i].Timestamp < events[j].Timestamp
	})
	fmt.Println("by timestamp ascending:")
	for _, e := range events {
		fmt.Printf("  %s  t=%d\n", e.ID, e.Timestamp)
	}

	// Sort by priority descending (higher priority first)
	sort.Slice(events, func(i, j int) bool {
		return events[i].Priority > events[j].Priority
	})
	fmt.Println("by priority descending:")
	for _, e := range events {
		fmt.Printf("  %s  p=%d\n", e.ID, e.Priority)
	}

	// sort.SliceStable preserves order of equal elements
	// Equal priority elements keep their timestamp-sorted order
	sort.SliceStable(events, func(i, j int) bool {
		return events[i].Priority > events[j].Priority
	})
	fmt.Println("stable by priority (equal elements keep relative order):")
	for _, e := range events {
		fmt.Printf("  %s  p=%d  t=%d\n", e.ID, e.Priority, e.Timestamp)
	}

	fmt.Println()
}

// ============================================================================
// sort.Interface — Reusable Sorted Type
// ============================================================================

// ByTimestamp implements sort.Interface for []Event sorted by Timestamp.
type ByTimestamp []Event

func (s ByTimestamp) Len() int           { return len(s) }
func (s ByTimestamp) Less(i, j int) bool { return s[i].Timestamp < s[j].Timestamp }
func (s ByTimestamp) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }

// ByPriority implements sort.Interface for []Event sorted by Priority (desc).
type ByPriority []Event

func (s ByPriority) Len() int           { return len(s) }
func (s ByPriority) Less(i, j int) bool { return s[i].Priority > s[j].Priority }
func (s ByPriority) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }

func sortInterface() {
	fmt.Println("=== sort.Interface: Reusable Sorted Types ===")

	events := []Event{
		{ID: "e1", Timestamp: 1000, Priority: 2},
		{ID: "e2", Timestamp: 500,  Priority: 1},
		{ID: "e3", Timestamp: 750,  Priority: 3},
	}

	sort.Sort(ByTimestamp(events))
	fmt.Println("ByTimestamp:")
	for _, e := range events {
		fmt.Printf("  %s  t=%d\n", e.ID, e.Timestamp)
	}

	sort.Sort(ByPriority(events))
	fmt.Println("ByPriority (desc):")
	for _, e := range events {
		fmt.Printf("  %s  p=%d\n", e.ID, e.Priority)
	}

	// sort.Search: binary search using sort.Interface pattern
	// Find first event with timestamp >= 800
	sort.Sort(ByTimestamp(events)) // must be sorted first
	target := int64(800)
	i := sort.Search(len(events), func(i int) bool {
		return events[i].Timestamp >= target
	})
	if i < len(events) {
		fmt.Printf("first event with timestamp >= %d: %s (t=%d)\n",
			target, events[i].ID, events[i].Timestamp)
	}

	fmt.Println()
}

// ============================================================================
// slices Package (Go 1.21+) — Generic, Type-Safe
// ============================================================================

func slicesPackage() {
	fmt.Println("=== slices Package (Go 1.21+) ===")

	nums := []int{5, 2, 8, 1, 9, 3}
	slices.Sort(nums)
	fmt.Printf("slices.Sort:    %v\n", nums)

	fmt.Printf("IsSorted:       %v\n", slices.IsSorted(nums))

	slices.Reverse(nums)
	fmt.Printf("Reverse:        %v\n", nums)

	fmt.Printf("Contains 8:     %v\n", slices.Contains(nums, 8))
	fmt.Printf("Index of 8:     %d\n", slices.Index(nums, 8))

	fmt.Println()
}

// ============================================================================
// Multi-Field Sort with slices.SortFunc
// ============================================================================

func multiFieldSort() {
	fmt.Println("=== Multi-Field Sort with slices.SortFunc ===")

	events := []Event{
		{ID: "e1", Priority: 2, Timestamp: 1000, Type: "click"},
		{ID: "e2", Priority: 1, Timestamp: 500,  Type: "hover"},
		{ID: "e3", Priority: 2, Timestamp: 750,  Type: "submit"},
		{ID: "e4", Priority: 3, Timestamp: 900,  Type: "keydown"},
	}

	// Primary: priority descending; secondary: timestamp ascending for ties
	slices.SortFunc(events, func(a, b Event) int {
		// cmp.Compare returns negative/zero/positive — exactly what SortFunc needs
		if n := cmp.Compare(b.Priority, a.Priority); n != 0 {
			return n // descending priority
		}
		return cmp.Compare(a.Timestamp, b.Timestamp) // ascending timestamp
	})

	fmt.Println("priority desc, then timestamp asc:")
	for _, e := range events {
		fmt.Printf("  %s  p=%d  t=%d\n", e.ID, e.Priority, e.Timestamp)
	}

	fmt.Println()
}

// ============================================================================
// Binary Search on Sorted Slice
// ============================================================================

func binarySearch() {
	fmt.Println("=== Binary Search ===")

	sorted := []int{1, 3, 5, 7, 9, 11, 13, 15}

	// slices.BinarySearch — returns (index, found)
	idx, found := slices.BinarySearch(sorted, 7)
	fmt.Printf("search 7:  idx=%d found=%v\n", idx, found)

	idx, found = slices.BinarySearch(sorted, 6)
	fmt.Printf("search 6:  idx=%d found=%v (insertion point)\n", idx, found)

	// BinarySearchFunc for custom types
	events := []Event{
		{ID: "e1", Timestamp: 100},
		{ID: "e2", Timestamp: 500},
		{ID: "e3", Timestamp: 900},
	}
	// Already sorted by timestamp — search for timestamp 500
	idx, found = slices.BinarySearchFunc(events, int64(500), func(e Event, t int64) int {
		return cmp.Compare(e.Timestamp, t)
	})
	fmt.Printf("search t=500: idx=%d found=%v event=%s\n", idx, found, events[idx].ID)

	fmt.Println()
}
