# Solution: In-Memory Event Store

## Approach

The core design is an **append-only slice with a type index**:

```
EventStore {
    events []Event              // insertion-ordered primary store
    index  map[string][]int    // type → positions in events
}
```

Events are never moved or deleted. Every `Append` records the position (current `len(events)`) before appending, then adds that position to the type index. All queries read from the primary slice — either directly (linear scan for time ranges) or via the index (O(1) type lookup).

## Key Decisions

### 1. Index stores positions, not copies

`index` maps type names to `[]int` (positions in the events slice), not `[]Event` (copies). This means:
- One copy of each event in memory, always
- ByType() is O(k) where k is the number of events of that type — just index lookups into the slice, no searching
- The index is always consistent with the events slice

Alternative considered: `map[string][]Event` — simpler to read, but duplicates every event. At 10k events with 5 types, that's 2x memory for type lookups.

### 2. Linear scan for ByTimeRange

Time ranges are not indexed. The decision: a time index (sorted by timestamp) would complicate `Append` (maintain sort invariant) and `Snapshot` (copy another sorted structure). Since events are appended in roughly-chronological order, a linear scan is cache-friendly and fast enough for most event store use cases.

When to add a time index: when ByTimeRange is called millions of times per second on stores with millions of events. At that scale, you'd likely move off an in-memory store entirely.

### 3. Deep copy in Snapshot

The `Snapshot()` method does three things:
- Deep-copies the events slice (each Event gets its own Payload map)
- Deep-copies each inner `[]int` in the type index (to prevent shared backing arrays)
- Returns a value (not pointer) — a snapshot is a read-mostly view

Without deep copying the Payload maps, two stores would share the same map pointer. Mutating a payload in one store would silently corrupt the other.

Without deep copying the inner `[]int` slices in the index, appending new events to one store could overwrite index entries in the other (the shared backing array problem).

### 4. Always return []Event{}, never nil

All query methods return `[]Event{}` for "no results". This is a deliberate API contract: callers can always range over results without a nil check. `nil` is reserved for "uninitialized" — something that should never be returned from a query method.

## Variant Tradeoffs

| Approach | Reads | Writes | Memory | Complexity |
|---|---|---|---|---|
| Events slice + type index (this solution) | O(k) by type, O(n) by time | O(1) amortized | 1x events | Low |
| Events slice only | O(n) for all queries | O(1) amortized | 1x events | Very low |
| Separate slice per type | O(k) by type, O(n) by time | O(1) amortized | 1x events | Medium |
| Events slice + time index (sorted) | O(log n + k) by time | O(log n) | 1.5x events | High |

For most observability event stores, the type index + linear time scan is the sweet spot.

## Slice Mechanics Demonstrated

- `make([]Event, 0)` — empty, non-nil slice for consistent JSON marshaling
- `append(s.index[e.Type], pos)` — appending to nil map value (first event of a type) works without initialization
- `copy(result, s.events)` in `All()` — returns a copy to prevent callers mutating the internal slice
- Reverse iteration in `Latest()` — `events[total-1-i]` gives most-recent-first without sorting
- Deep copy of `map[string][]int` in `Snapshot()` — requires copying each inner slice separately
