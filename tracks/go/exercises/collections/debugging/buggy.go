// Package auditlog processes structured audit log entries from multiple services.
package auditlog

// LogEntry represents a single audit log entry.
type LogEntry struct {
	ID      string
	UserID  string
	Service string // e.g., "payments", "auth", "inventory"
	Action  string // e.g., "login", "purchase", "refund"
}

// UserSummary aggregates a user's actions across all services.
type UserSummary struct {
	UserID      string
	ActionCount int
	Services    []string // unique services the user interacted with
}

// ============================================================================
// Bug 1: Nil map panic on first call
//
// ProcessBatch aggregates a batch of log entries by user.
// Returns a map of userID → UserSummary.
// ============================================================================

func ProcessBatch(entries []LogEntry) map[string]UserSummary {
	// BUG: summaries is declared but never initialized.
	// Writing to a nil map panics on the first entry.
	var summaries map[string]UserSummary

	for _, entry := range entries {
		summary := summaries[entry.UserID]
		summary.UserID = entry.UserID
		summary.ActionCount++
		// Add service if not already present
		if !containsService(summary.Services, entry.Service) {
			summary.Services = append(summary.Services, entry.Service)
		}
		summaries[entry.UserID] = summary // PANIC HERE
	}
	return summaries
}

func containsService(services []string, s string) bool {
	for _, svc := range services {
		if svc == s {
			return true
		}
	}
	return false
}

// ============================================================================
// Bug 2: Shared backing array corruption
//
// SplitByService takes a slice of entries and splits them into two groups:
// entries matching targetService and all others.
// The two result slices must be completely independent.
// ============================================================================

func SplitByService(entries []LogEntry, targetService string) (matched, rest []LogEntry) {
	// BUG: Both matched and rest are built as sub-slices of entries using
	// the "filter in place" trick — but they share the same backing array.
	// Appending to matched can overwrite elements of rest.
	//
	// The fix: use separate, independent slices built with append from scratch.

	// "Partition in place" approach — looks clever, is buggy:
	i := 0
	for _, e := range entries {
		if e.Service == targetService {
			entries[i] = e
			i++
		}
	}
	matched = entries[:i]  // shares backing array with entries
	rest = entries[i:]     // shares backing array with entries, starts right after matched
	// If caller appends to matched within its capacity, it overwrites rest[0]
	return matched, rest
}

// ============================================================================
// Bug 3: range loop value semantics — struct mutation in range doesn't stick
//
// MarkProcessed sets the Action field to "processed" on all entries
// that match the given service. Should modify entries in-place.
// ============================================================================

func MarkProcessed(entries []LogEntry, service string) {
	// BUG: `entry` in the range loop is a COPY of entries[i].
	// Modifying entry.Action changes the local copy, not the original slice element.
	// After the loop, entries is unchanged.
	for _, entry := range entries {
		if entry.Service == service {
			entry.Action = "processed"
			_ = entry // silence "unused variable" — the real bug is we don't write back
		}
	}
}

// ============================================================================
// Bug 4: append result ignored — caller's slice appears unmodified
//
// DeduplicateIDs removes duplicate IDs from the given slice and returns
// a new deduplicated slice. The caller passes a pointer to the slice so
// the function can "modify it in place."
//
// But the implementation is broken: it builds a new local slice via append,
// but the pointer-based approach doesn't work the way the author intended.
// ============================================================================

func DeduplicateIDs(ids *[]string) {
	// BUG: The author tried to "return via pointer" so the caller sees the result.
	// But `result` is a completely new local slice. Writing result back to *ids
	// is correct in principle — but the assignment is missing.
	//
	// The local `result` slice is built correctly but never assigned to *ids.
	// After this function returns, the caller's slice is unchanged.
	seen := make(map[string]struct{})
	result := make([]string, 0, len(*ids))
	for _, id := range *ids {
		if _, ok := seen[id]; !ok {
			seen[id] = struct{}{}
			result = append(result, id)
		}
	}
	// BUG: Missing assignment: *ids = result
	// result is built correctly, but the caller never sees it.
	_ = result
}
