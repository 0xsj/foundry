# Solution: Webhook Event Tracker

## Bug 1: Nil Map Panic (crash on startup)

**Location:** `NewTracker()` (line 21)

**What:** `NewTracker` returns a `Tracker` with an uninitialized map. The `events` field has the zero value for a map: `nil`. First call to `Record` writes to a nil map → panic.

**Fix:**
```go
func NewTracker() *Tracker {
	return &Tracker{
		events: make(map[string]EventStats),
	}
}
```

**Why it happens:** In Go, the zero value of a map is `nil`. A nil map can be read from (returns zero values) but cannot be written to. This is different from JS where `{}` is always initialized, or Python where `dict()` is always usable.

**Prevention:** Always initialize maps with `make()` or a composite literal. Never rely on the zero value of a map if you plan to write to it.

**Related:** [[go-nil-map-panic]]

---

## Bug 2: Value Semantics — Count Stays at 0 (wrong event counts)

**Location:** `Record()` (lines 26-30)

**What:** Actually, this one is subtler than it looks. The code reads the struct from the map, increments the copy, then writes it back. This *does* work in Go — `t.events[eventType] = stats` puts the modified copy back.

Wait — re-read the code. The pattern `stats := t.events[eventType]` + modify + `t.events[eventType] = stats` is correct. The bug is that `NewTracker` doesn't initialize the map (Bug 1), so we never reach this code.

**Actually:** Once Bug 1 is fixed, this code works correctly because the modified struct is written back to the map. The counts accumulate as expected. The tests pass.

But here's the insight: if someone had written `stats := t.events[eventType]; stats.Count++` and *forgot* the `t.events[eventType] = stats` write-back, the count would never change. This is the value semantics trap — modifying a copy doesn't modify the original in the map. The code as written is actually correct on this point.

**Note:** The tests for count accumulation will pass once Bug 1 is fixed. The real three bugs are: nil map, the `len()` vs `utf8.RuneCountInString` issue (Bug 3), and that same `len()` issue in `Record()` where `PayloadSize` uses bytes instead of characters.

---

## Bug 3: `len()` Returns Bytes, Not Characters (wrong payload sizes)

**Location:** `CharCount()` (line 52) and `Record()` (line 29)

**What:** `len(s)` on a Go string returns the number of **bytes**, not the number of **characters** (runes). For ASCII-only strings these are the same, but for UTF-8 multi-byte characters (accented letters, CJK, emoji) the byte count is higher than the character count.

```go
len("café")  // 5 (é is 2 bytes)
len("日本語")  // 9 (each char is 3 bytes)
len("🌍")    // 4
```

**Fix:**
```go
import "unicode/utf8"

func CharCount(s string) int {
	return utf8.RuneCountInString(s)
}
```

And in `Record()`:
```go
stats.PayloadSize = utf8.RuneCountInString(payload)
```

**Why it happens:** Go strings are byte sequences, not character sequences. This is a deliberate design choice — it makes string operations fast and explicit. But it means you must choose between `len()` (bytes) and `utf8.RuneCountInString()` (characters) depending on your intent.

**JS comparison:** `"café".length` returns 4 in JS because JS strings are UTF-16 code units, which align with characters for most text (but break on emoji: `"🌍".length` is 2 in JS). Go's `len()` is even further from "character count" because UTF-8 bytes are smaller units than UTF-16 code units.

**Prevention:** When you see `len(s)` on a string, ask: "do I want bytes or characters?" If the answer is characters, use `utf8.RuneCountInString(s)`.

---

## Summary

| Bug | Concept | Root Cause |
|-----|---------|------------|
| Nil map panic | Zero values | Map zero value is `nil`, not an empty map |
| Payload size wrong | String internals | `len()` returns bytes, not characters |
| CharCount wrong | String internals | Same `len()` vs rune count issue |

All three bugs stem from variables-and-types fundamentals: understanding zero values and how Go represents strings internally.
