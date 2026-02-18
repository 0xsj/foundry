# Solution: Audit Log Processor

## Bug 1: Nil Map Panic in ProcessBatch

**Location:** `ProcessBatch()` — `var summaries map[string]UserSummary` (line 26)

**Symptom:** Panic on the first call: `assignment to entry in nil map`.

**What:** `var summaries map[string]UserSummary` declares a map variable with the zero value for maps: `nil`. A nil map is readable (returns the zero value for the value type) but panics on write. The first write — `summaries[entry.UserID] = summary` — causes the panic.

**Why it's subtle:** The read on line `summary := summaries[entry.UserID]` works fine (returns a zero-value `UserSummary`). It's only the write that panics. This asymmetry trips people up.

**Fix:**
```go
func ProcessBatch(entries []LogEntry) map[string]UserSummary {
    summaries := make(map[string]UserSummary)  // initialized, not nil
    // ...
}
```

**JS comparison:** In JS, `const obj = {}` is always initialized. You never get a panic from `obj.key = value`. Go requires explicit initialization for maps — the zero value is not usable for writing.

**Related:** [[pitfalls/go-nil-map-panic]]

---

## Bug 2: Shared Backing Array in SplitByService

**Location:** `SplitByService()` — the partition-in-place approach (lines 58-64)

**Symptom:** Appending to `matched` corrupts the first element of `rest` (or vice versa). Tests see wrong IDs in the `rest` slice after the split.

**What:** The "partition in place" pattern reuses the input slice's backing array. After partitioning:

```
entries backing array: [auth1, auth2, payments1, inventory1]
                        ^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
                        matched (len=2)   rest (len=2)
```

Both `matched` and `rest` are sub-slices of the same array. `matched` has cap 4 (it starts at index 0 of a 4-element array). Appending within that capacity writes to `entries[2]` — which is `rest[0]`. Silent data corruption.

**Fix:**

```go
func SplitByService(entries []LogEntry, targetService string) (matched, rest []LogEntry) {
    matched = make([]LogEntry, 0)
    rest = make([]LogEntry, 0)
    for _, e := range entries {
        if e.Service == targetService {
            matched = append(matched, e)
        } else {
            rest = append(rest, e)
        }
    }
    return matched, rest
}
```

This builds two fully independent slices. Each `append` allocates its own backing array. No sharing.

**When the in-place pattern IS safe:** The partition-in-place approach is valid if you immediately restrict capacity using a three-index slice and never append to either result:
```go
matched = entries[:i:i]   // cap bounded to i — appending forces reallocation
rest = entries[i:]        // rest starts at i
```
But this is subtle and error-prone. The explicit-append version is safer and clear.

---

## Bug 3: Range Value Copy in MarkProcessed

**Location:** `MarkProcessed()` — `for _, entry := range entries` (line 77)

**Symptom:** After calling `MarkProcessed`, entries with the target service still show their original Action. The function appears to do nothing.

**What:** The `range` loop variable `entry` is a **copy** of `entries[i]`. Setting `entry.Action = "processed"` modifies the copy. The original slice element is unchanged. After the loop body, the modified `entry` is thrown away.

**Fix: Use the index to modify in place:**

```go
func MarkProcessed(entries []LogEntry, service string) {
    for i := range entries {
        if entries[i].Service == service {
            entries[i].Action = "processed"  // modify original, not copy
        }
    }
}
```

**JS comparison:** In JS, `for (const item of arr) { item.field = x }` works if `item` is an object — objects are references, and you're mutating through the reference. In Go, struct values are always copied into the loop variable. The mental model is different.

**Pointer alternative:** If `entries` were `[]*LogEntry`, the loop variable would be a pointer (copied), and `entry.Action = "processed"` would work because you're dereferencing the pointer, not mutating the pointer value.

---

## Bug 4: Ignored append Result in DeduplicateIDs

**Location:** `DeduplicateIDs()` — missing `*ids = result` at the end (line 100)

**Symptom:** The caller's slice appears unchanged after calling `DeduplicateIDs`. Duplicates are still present.

**What:** The function builds a correct deduplicated slice in `result`, but never assigns it back to `*ids`. The local `result` variable goes out of scope and is garbage collected. The caller's slice pointer is never updated.

**Fix:**

```go
func DeduplicateIDs(ids *[]string) {
    seen := make(map[string]struct{})
    result := make([]string, 0, len(*ids))
    for _, id := range *ids {
        if _, ok := seen[id]; !ok {
            seen[id] = struct{}{}
            result = append(result, id)
        }
    }
    *ids = result  // assign back through the pointer
}
```

**Better design:** The pointer-receiver pattern is unusual in Go. The idiomatic approach is to return the new slice:

```go
func DeduplicateIDs(ids []string) []string {
    seen := make(map[string]struct{})
    result := make([]string, 0, len(ids))
    for _, id := range ids {
        if _, ok := seen[id]; !ok {
            seen[id] = struct{}{}
            result = append(result, id)
        }
    }
    return result
}

// Caller:
ids = DeduplicateIDs(ids)
```

This is the same reason `append` returns a new slice instead of taking a `*[]T` — returning is cleaner than mutating through a pointer.

---

## Summary

| Bug | Location | Concept | Root Cause |
|-----|----------|---------|------------|
| Nil map panic | `ProcessBatch` | Zero values | `var m map[K]V` is nil, not empty |
| Shared backing array | `SplitByService` | Slice internals | Sub-slices share backing array; appending within capacity corrupts |
| Mutation not visible | `MarkProcessed` | range semantics | `range` copies each element; mutating the copy doesn't affect original |
| Append result lost | `DeduplicateIDs` | Append semantics | `append` returns a new slice; result must be assigned or returned |
