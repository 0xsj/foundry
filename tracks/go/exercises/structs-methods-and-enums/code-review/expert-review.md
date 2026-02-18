# Expert Review: Priority Notification Queue

## Critical Issues

### 1. `*QueueStats` embedded pointer is never initialized — nil panic on first Enqueue/Dequeue

**Location:** `New()` (line `return &NotifyQueue{...}`) and `Enqueue` / `Dequeue`

`NotifyQueue` embeds `*QueueStats` — a pointer to a struct, not the struct itself. The zero value of any pointer is `nil`. `New()` initializes `Buckets` and `Paused` but omits `QueueStats`:

```go
return &NotifyQueue{
    // QueueStats not set — nil pointer
    Buckets: make(map[Priority][]Notification),
    Paused:  make(map[Priority]bool),
    maxPerBucket: maxPerBucket,
}
```

The first call to `Enqueue` calls `q.RecordEnqueue(...)`. The promoted `RecordEnqueue` method dereferences `q.QueueStats` (nil) → **nil pointer dereference panic**.

**Fix:** Initialize `QueueStats` in `New()`:

```go
return &NotifyQueue{
    QueueStats:   &QueueStats{},
    Buckets:      make(map[Priority][]Notification),
    Paused:       make(map[Priority]bool),
    maxPerBucket: maxPerBucket,
}
```

Or better: embed by value if `QueueStats` is always present:
```go
type NotifyQueue struct {
    QueueStats           // by value — zero value is valid, no nil risk
    buckets    map[Priority][]Notification
    ...
}
```

**Concept:** Embedding by pointer (`*T`) leaves the field nil until explicitly initialized. Embedding by value (`T`) gives you the zero value of the embedded struct — always safe to use. Only embed by pointer when the embedded type is optional or requires lazy initialization.

---

### 2. `RecordEnqueue` has a value receiver — stats are never updated

**Location:** `func (s QueueStats) RecordEnqueue(currentSize int)`

`RecordEnqueue` increments `s.EnqueueCount` and updates `s.PeakSize`, but the receiver is `QueueStats` (value), not `*QueueStats` (pointer). The method operates on a copy. When it returns, `s.EnqueueCount++` is discarded. `q.EnqueueCount` in the embedded struct is never incremented.

Note: this bug is masked by Bug 1 (the nil pointer panics first), but once Bug 1 is fixed, every `Enqueue` call silently fails to update stats.

```go
// Bug
func (s QueueStats) RecordEnqueue(currentSize int) {
    s.EnqueueCount++  // modifies copy — original unchanged
}

// Fix
func (s *QueueStats) RecordEnqueue(currentSize int) {
    s.EnqueueCount++
    if currentSize > s.PeakSize {
        s.PeakSize = currentSize
    }
}
```

**Concept:** Any method that modifies struct fields must use a pointer receiver. Value receivers are for read-only access. Since `RecordDequeue` already uses `*QueueStats`, having `RecordEnqueue` use a value receiver is inconsistent and wrong. Go's guideline: if any method uses a pointer receiver, all methods should.

---

## Major Concerns

### 3. `Pause` uses a value receiver — pause state is never persisted

**Location:** `func (q NotifyQueue) Pause(priority Priority)`

`Pause` sets `q.Paused[priority] = true`. Maps are reference types — modifying the map through a copy does work (the map header is copied, but it points to the same underlying data). So `q.Paused[priority] = true` *does* persist to the caller's map.

However, there's still a problem with consistency and principle. `NotifyQueue` has other pointer-receiver methods (`Enqueue`, `Dequeue`, `Resume`). Mixing value and pointer receivers is a code smell that sets a precedent for future bugs. If `Pause` were modified to also update a field on `NotifyQueue` itself (e.g., a `PausedCount`), that change would be silently lost.

More importantly: `Pause` has a value receiver while `Resume` has a pointer receiver — asymmetric design in a method pair that should behave consistently.

**Fix:**
```go
func (q *NotifyQueue) Pause(priority Priority) {
    q.Paused[priority] = true
}
```

**Concept:** Keep receiver types consistent across a type's methods. The Go spec notes: "if any method of a type has a pointer receiver, all methods should, to avoid confusion about which method set satisfies which interface."

---

### 4. `Buckets` and `Paused` are exported — internal state is directly mutable by callers

**Location:** Field declarations in `NotifyQueue`

Exported fields are part of the public API. Any caller can write:

```go
q.Buckets[PriorityCritical] = nil           // silently drain the queue
q.Paused[PriorityHigh] = true               // bypass the Pause method
q.Buckets[PriorityCritical] = append(q.Buckets[PriorityCritical], n) // bypass Enqueue
```

This defeats `maxPerBucket` enforcement, bypass `Pause`, and makes it impossible to add invariants later (e.g., max queue depth, thread safety) without breaking callers who touch the fields directly.

**Fix:** Unexport the fields and expose behavior through methods only:

```go
type NotifyQueue struct {
    stats         QueueStats    // unexported
    buckets       map[Priority][]Notification  // unexported
    paused        map[Priority]bool            // unexported
    maxPerBucket  int
}
```

Add accessor methods if callers need to read the data:
```go
func (q *NotifyQueue) IsPaused(p Priority) bool { return q.paused[p] }
func (q *NotifyQueue) BucketSize(p Priority) int { return len(q.buckets[p]) }
```

**Concept:** In Go, the unit of encapsulation is the package, not the type. Unexported fields are accessible within the package but not from external packages. For types meant to be used across package boundaries, unexported fields enforce the invariants that the methods guarantee.

---

## Minor Suggestions

### 5. `PriorityFromInt` silently returns `PriorityLow` for invalid input

**Location:** `func PriorityFromInt(n int) Priority`

```go
func PriorityFromInt(n int) Priority {
    switch Priority(n) {
    case PriorityCritical, PriorityHigh, PriorityNormal, PriorityLow:
        return Priority(n)
    }
    return PriorityLow  // silent fallback
}
```

An invalid input (e.g., `PriorityFromInt(99)`) returns `PriorityLow` silently. The caller has no way to distinguish "the caller passed 3 (PriorityLow)" from "the caller passed 99 (invalid, silently degraded to PriorityLow)". A notification intended to fail validation is silently enqueued at low priority.

**Better:**
```go
func PriorityFromInt(n int) (Priority, error) {
    p := Priority(n)
    switch p {
    case PriorityCritical, PriorityHigh, PriorityNormal, PriorityLow:
        return p, nil
    }
    return 0, fmt.Errorf("unknown priority %d: must be 0-3", n)
}
```

**Concept:** Parsing/conversion functions that can fail should return `(T, error)`. Silent fallbacks mask configuration errors and make debugging harder. The caller should be able to decide how to handle invalid input.

---

### 6. `Priority` has no `String()` method — error messages print integers

**Location:** `Priority` type and its use in error messages

```go
return fmt.Errorf("priority %d is paused", n.Priority)
// prints: "priority 2 is paused" — what does 2 mean?
```

Without a `String()` method on `Priority`, error messages and log output show raw integers. After three hours debugging a live incident, `"priority 2 is paused"` is less useful than `"priority normal is paused"`.

**Fix:**
```go
func (p Priority) String() string {
    switch p {
    case PriorityCritical:
        return "critical"
    case PriorityHigh:
        return "high"
    case PriorityNormal:
        return "normal"
    case PriorityLow:
        return "low"
    default:
        return fmt.Sprintf("Priority(%d)", int(p))
    }
}
```

Then the error message becomes:
```go
return fmt.Errorf("priority %v is paused", n.Priority)
// prints: "priority normal is paused"
```

**Concept:** Any `iota` enum that will appear in logs, errors, or user-facing output should implement `fmt.Stringer`. The `default` case is important — it handles values added in future or values from external sources that don't match known constants.

---

## Positive Feedback

- Correct use of `iota` for `Priority` constants — idiomatic Go
- The priority ordering in `Dequeue` (critical first) is explicit and easy to follow
- `RecordDequeue` correctly uses a pointer receiver
- `Resume` and `Size` are well-implemented
- The `maxPerBucket` capacity enforcement in `Enqueue` is a good design touch
- Separating queue stats into an embeddable `QueueStats` struct is a good instinct — promotes reuse

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | `*QueueStats` nil — panics on first use | Embedding by pointer, nil initialization |
| 2 | Critical | `RecordEnqueue` value receiver — stats never updated | Value vs pointer receivers |
| 3 | Major | `Pause` value receiver — inconsistent with `Resume` | Consistent receiver types |
| 4 | Major | `Buckets` and `Paused` exported — bypasses invariants | Exported vs unexported fields |
| 5 | Minor | `PriorityFromInt` silent fallback — hides invalid input | Error returns vs sentinel values |
| 6 | Minor | `Priority` missing `String()` — integers in error messages | fmt.Stringer on enums |
