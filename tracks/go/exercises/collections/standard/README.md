# Exercise: In-Memory Event Store

## Scenario

You're building the event store component for an observability platform. Services emit structured events (user actions, system alerts, errors) that need to be stored and queried efficiently. The store lives in-process — no database — and must support several access patterns: fetching the latest N events, querying by time range, and filtering by event type. The system receives thousands of events per minute, so allocations matter.

## Brief

Implement an `EventStore` that:

1. Accepts events via `Append` and stores them in insertion order
2. Maintains a type index (`map[string][]int`) storing the slice positions of events by type, for O(1) type lookups
3. Queries events by time range (inclusive on both ends)
4. Queries events by type using the index
5. Returns the latest N events in reverse-chronological order
6. Supports snapshots (a read-only point-in-time copy)

## Acceptance Criteria

- [ ] `EventStore` struct with `Append(Event)` method
- [ ] `All() []Event` returns all events in insertion order
- [ ] `ByType(t string) []Event` returns all events of that type, using the index
- [ ] `ByTimeRange(from, to int64) []Event` returns events with `from <= e.Timestamp <= to`
- [ ] `Latest(n int) []Event` returns the last n events, most-recent first. If n > len, return all.
- [ ] `Snapshot() EventStore` returns a deep copy — mutations to the snapshot do not affect the original and vice versa
- [ ] Type index is updated atomically with every `Append` — never stale
- [ ] All query methods return `[]Event{}` (not nil) when no results match
- [ ] `Len() int` returns total event count

## Event Type

```go
type Event struct {
    ID        string
    Type      string  // e.g., "error", "purchase", "login"
    Timestamp int64   // Unix milliseconds
    Payload   map[string]string
}
```

## Constraints

- Standard library only
- Do not sort on every query — store in insertion order and let callers sort if needed
- `ByTimeRange` should scan linearly (no pre-built time index needed)
- The type index must be a `map[string][]int` (indices into the main events slice)
- `Snapshot()` must deep-copy both the events slice and the type index (and event payloads)

## Concepts Exercised

- Slice append patterns and pre-allocation
- Map creation and the type index (`map[string][]int`)
- Backing array sharing — why `Snapshot()` can't just copy the slice header
- `copy()` to prevent aliasing in snapshots
- nil vs empty slice in return values
- range over slices and maps
- Reverse iteration for `Latest(n)`

## Hints

<details>
<summary>Hint 1: Type index structure</summary>

The type index maps event types to their positions in the main events slice:

```go
type EventStore struct {
    events []Event
    index  map[string][]int  // "error" -> [0, 3, 7, ...]
}
```

When you append event at position `len(store.events)` (before appending), record that index.
</details>

<details>
<summary>Hint 2: Returning empty instead of nil</summary>

```go
func (s *EventStore) ByType(t string) []Event {
    positions, ok := s.index[t]
    if !ok {
        return []Event{}  // not nil — callers can always range over it safely
    }
    // ...
}
```
</details>

<details>
<summary>Hint 3: Deep copy for Snapshot</summary>

A shallow copy (`copy(dst, src)`) copies the slice headers of each event's Payload map — they still share the same underlying map. You need to copy each Payload map explicitly:

```go
func copyEvent(e Event) Event {
    if e.Payload == nil {
        return e
    }
    p := make(map[string]string, len(e.Payload))
    for k, v := range e.Payload {
        p[k] = v
    }
    e.Payload = p
    return e
}
```
</details>

<details>
<summary>Hint 4: Latest(n) in reverse order</summary>

```go
func (s *EventStore) Latest(n int) []Event {
    if n >= len(s.events) {
        n = len(s.events)
    }
    result := make([]Event, n)
    for i := 0; i < n; i++ {
        result[i] = s.events[len(s.events)-1-i]
    }
    return result
}
```
</details>
