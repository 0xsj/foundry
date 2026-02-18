# Expert Review: Leaderboard System

## Critical Issues

### 1. Uninitialized map in New() — panic on any write

`New()` returns a `&Leaderboard{}` with `scores` at its zero value: `nil`. The first call to `AddScore` or `Import` writes to a nil map and panics.

```go
// Bug
return &Leaderboard{}

// Fix
return &Leaderboard{
    scores: make(map[string][]Entry),
}
```

**Concept:** Map zero value is nil. Nil maps are readable but panic on write. Every constructor must initialize maps that will be written to. This is such a common mistake it's worth having a linter rule: if a struct has a map field, the constructor must `make` it.

---

### 2. AddScore updates a copy, not the slice element

In `AddScore`, the range loop copies each `Entry` into `e`. Setting `e.Score = score` modifies the copy; the original slice element in `lb.scores[mode]` is unchanged. The update is silently lost.

```go
// Bug: e is a copy
for _, e := range entries {
    if e.PlayerID == playerID {
        e.Score = score  // updates local copy — slice element unchanged
        return
    }
}

// Fix: use the index to modify the original
for i := range entries {
    if entries[i].PlayerID == playerID {
        entries[i].Score = score
        lb.scores[mode] = entries  // write back (entries is a copy of the header)
        return
    }
}
```

Wait — there's a subtlety: `entries := lb.scores[mode]` copies the slice header. Modifying `entries[i]` modifies the backing array (since the element type is a value, not a pointer). But to be safe and clear, use `lb.scores[mode][i].Score = score` directly — no intermediate variable needed.

```go
for i, e := range lb.scores[mode] {
    if e.PlayerID == playerID {
        lb.scores[mode][i].Score = score
        return
    }
}
```

**Concept:** `range` copies values into loop variables. Modifying the copy doesn't modify the original. Use the index form `for i := range s` when you need to mutate elements.

---

### 3. TopN returns nil instead of empty slice

When no entries exist for a mode, `TopN` returns `nil`. Callers must nil-check before ranging, which breaks the contract of "query methods always return safe results."

```go
// Bug
if len(entries) == 0 {
    return nil
}

// Fix
if len(entries) == 0 {
    return []Entry{}
}
```

This is a consistency issue. All query methods should either always return nil (if callers expect to nil-check) or always return a non-nil empty slice (if callers expect to iterate safely). Pick one and document it. The idiomatic Go choice for query methods is `[]T{}`.

---

## Major Concerns

### 4. TopN sorts the internal slice, not a copy

`sorted := entries` copies the slice header — not the data. Both `sorted` and `lb.scores[mode]` point to the same backing array. Sorting `sorted` sorts `lb.scores[mode]`. After `TopN`, the stored order for that game mode is permanently changed, even though the comment says "don't mutate the stored order."

```go
// Bug: sorted shares backing array with lb.scores[mode]
sorted := entries
sort.Slice(sorted, ...)

// Fix: copy into a new slice first
sorted := make([]Entry, len(entries))
copy(sorted, entries)
sort.Slice(sorted, ...)
```

This is one of the most common Go slice bugs. The comment even correctly identifies the intent — but the implementation does the opposite of what the comment says.

**Concept:** Assigning a slice value copies the header (pointer, len, cap), not the data. Both slices share the same backing array. Mutations through one are visible through the other.

---

### 5. RankAll sets Rank on copies, not originals

`for rank, e := range entries` copies each `Entry` into `e`. Setting `e.Rank = rank + 1` updates the copy. The `Rank` field in `lb.scores[mode]` is never modified.

```go
// Bug
for rank, e := range entries {
    e.Rank = rank + 1  // copy — original unchanged
}

// Fix: use the index
for i := range entries {
    entries[i].Rank = i + 1
}
// But also need to write back if entries was a copy of the header:
// (In this case it shares the backing array, so element writes do reach lb.scores[mode])
```

Same root cause as Bug 2. Range copies values. Use `for i := range s` and `s[i]` when mutation is needed.

---

### 6. HasPlayer is O(n×m) — needs a player index

`HasPlayer` scans every entry in every game mode. For a leaderboard with 100 game modes and 10,000 players each, this is 1,000,000 comparisons per call. `MergeMode` calls `playerInMode` for every source entry — another nested linear scan.

```go
// Bug: O(n×m) — scans all modes and all entries
func (lb *Leaderboard) HasPlayer(playerID string) bool {
    for _, entries := range lb.scores {
        for _, e := range entries {
            if e.PlayerID == playerID { return true }
        }
    }
    return false
}
```

The fix is a secondary index: `players map[string]bool` (or `map[string]struct{}`), updated in `AddScore` and `Import`. This makes `HasPlayer` O(1) at the cost of a second map.

```go
type Leaderboard struct {
    scores  map[string][]Entry
    players map[string]struct{}  // set of all player IDs across all modes
}

func (lb *Leaderboard) HasPlayer(playerID string) bool {
    _, ok := lb.players[playerID]
    return ok
}
```

**Concept:** Linear scans are fine for small data. In production, "leaderboard" suggests potentially millions of entries. Any O(n) operation that runs per request becomes a latency problem. The fix (a secondary index) is the same pattern as a database index — pay a small cost at write time to make reads fast.

---

### 7. Import stores the caller's slice — aliasing risk

```go
func (lb *Leaderboard) Import(mode string, entries []Entry) {
    lb.scores[mode] = entries  // stores caller's slice header directly
}
```

The `Leaderboard` and the caller now share the same backing array. If the caller modifies their slice (or appends within capacity), the leaderboard's data is silently corrupted. If the leaderboard modifies entries via `AddScore`, the caller's slice is affected.

```go
// Fix: copy on import
func (lb *Leaderboard) Import(mode string, entries []Entry) {
    imported := make([]Entry, len(entries))
    copy(imported, entries)
    lb.scores[mode] = imported
}
```

This is the defensive copy pattern: when accepting external data that you'll store, always copy. You give up the caller's backing array and own your own data.

**Concept:** A type that stores a slice it received from outside its own package should always copy it. Shared backing arrays create action-at-a-distance bugs that are very hard to debug.

---

## Minor Suggestions

### 8. AddScore linear scan — consider a per-mode player map

`AddScore` scans all entries to find a player. This is O(k) where k is entries per mode. If modes have thousands of players and `AddScore` is called frequently, this adds up. A `map[string]int` per mode (player → slice index) would give O(1) lookup.

This is a design question, not a bug. For a leaderboard with <1,000 players per mode, the linear scan is fine. Document the decision.

### 9. ScoreStats is correct but could document the return convention

`ScoreStats` returns `(0, 0, 0)` for empty modes. This is correct but undocumented. A comment clarifying what callers should expect for empty modes avoids confusion.

---

## Positive Feedback

- `ScoreStats` correctly handles the empty case and computes min/max/avg in one pass — no extra allocations, clean logic.
- Using `sort.Slice` with a `func(i, j int) bool` is idiomatic Go for ad-hoc sorting.
- The struct and field naming is clear (`Entry`, `PlayerID`, `Score`, `Rank`).
- `playerInMode` is correctly extracted as a helper rather than inlined in `MergeMode`.

---

## Summary

| # | Severity | Issue | Concept |
|---|----------|-------|---------|
| 1 | Critical | Nil map panic in `New()` | Map zero value |
| 2 | Critical | `AddScore` updates copy, not original | range value semantics |
| 3 | Major | `TopN` returns nil for empty | nil vs empty slice contract |
| 4 | Major | `TopN` sorts internal slice — comment says it doesn't | Slice header copy ≠ data copy |
| 5 | Major | `RankAll` sets Rank on copies | range value semantics (again) |
| 6 | Major | `HasPlayer` is O(n×m) — no player index | Missing secondary index |
| 7 | Major | `Import` stores caller's slice — aliasing risk | Defensive copy pattern |
| 8 | Minor | `AddScore` linear scan | Performance tradeoff (acceptable, document it) |
| 9 | Minor | `ScoreStats` empty-mode behavior undocumented | API clarity |

Bugs 2 and 5 are the same root cause — range value copy semantics. This is the lesson's most important takeaway: **always reach for `for i := range s` when you need to mutate slice elements.**
