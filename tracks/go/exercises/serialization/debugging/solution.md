# Debugging Solution: Serialization Bugs

## Bug 1: Unexported Field Not Serialized

**Location:** `PaymentEvent.amount` field declaration

**Symptom:** `TestEventRoundTrip` fails — `amount` field is missing from JSON output.

**Root cause:**

```go
type PaymentEvent struct {
    EventID   string    `json:"event_id"`
    amount    int64     // unexported — encoding/json skips it
    // ...
}
```

`encoding/json` only marshals exported (uppercase) fields. The `amount` field starts with a lowercase letter, making it unexported — invisible to packages outside `webhookprocessor`, and also invisible to `encoding/json` (which uses reflection from outside the package boundary).

The field can be read/written within the package, and the `Amount()` getter works, but JSON serialization never sees it.

**Fix:**

```go
type PaymentEvent struct {
    EventID   string `json:"event_id"`
    Amount    int64  `json:"amount"`    // exported — uppercase A
    // ...
}
```

Update all references to use `Amount` (uppercase). The `Amount()` getter becomes redundant once the field is exported — remove it or keep it if the accessor pattern is intentional.

**Key insight:** In Go, exported/unexported applies to the whole package. The rule "packages outside cannot access lowercase names" also applies to `encoding/json`, because `encoding/json` uses reflection and reflection respects the same visibility rules. `reflect.StructField.IsExported()` returns false for unexported fields, and `encoding/json` checks this explicitly.

**Related pitfall:** [[go-unexported-field-json]] — common source of "field is missing from API response" bugs.

---

## Bug 2: omitempty Drops a Valid Zero Value

**Location:** `RiskScore.Score` field tag

**Symptom:** `TestZeroScoreOmitted` fails — `score` is missing from JSON when the payment is legitimate (score = 0).

**Root cause:**

```go
type RiskScore struct {
    Score int `json:"score,omitempty"` // drops score=0
}
```

`omitempty` omits a field when its value is the **zero value for its type**. For `int`, the zero value is `0`. A score of `0` means "no fraud detected" — a meaningful business value. But `omitempty` can't distinguish between "not set" and "explicitly zero". Both result in the field being dropped.

**Fix: use a pointer**

```go
type RiskScore struct {
    Score *int `json:"score,omitempty"` // nil = not set, &0 = explicitly 0
}
```

With a pointer, `omitempty` only omits when the pointer is `nil`. A pointer to `0` (`&0`) serializes as `0`. Now you can distinguish between "score not yet computed" (`nil`) and "score is 0" (`*int` pointing to 0).

**Alternative fix: remove omitempty**

If a score of `0` should always be present (no use case for "not set"), simply remove `omitempty`:

```go
Score int `json:"score"` // always included, even when 0
```

**When to use `omitempty` on integers:**
- Safe: fields where zero means "absent" or "not applicable" (e.g., `TotalPages int` — 0 pages means no pagination)
- Dangerous: fields where 0 is a valid domain value (scores, counts, prices, exit codes, timestamps)

**Rule of thumb:** If "is this field 0?" and "was this field set?" are two different questions, use a pointer.

---

## Bug 3: json.Decoder Created Inside the Loop

**Location:** `StreamEvents` function body

**Symptom:** `TestStreamMultipleEvents` fails — only 1 event returned from a 3-event stream.

**Root cause:**

```go
func StreamEvents(r io.Reader) ([]PaymentEvent, error) {
    var events []PaymentEvent
    for {
        dec := json.NewDecoder(r)  // ← new decoder every iteration
        var event PaymentEvent
        err := dec.Decode(&event)
        if err == io.EOF {
            break
        }
        // ...
    }
}
```

`json.NewDecoder(r)` creates a decoder that wraps `r` and maintains its own internal read buffer. When `Decode` is called, it reads ahead from `r` — potentially more bytes than needed for one JSON object. This buffered data is stored inside the decoder.

When you create a new decoder at the top of the next loop iteration, the internal buffer is thrown away. The new decoder starts fresh, reading from `r` — but `r` has already had some bytes consumed (by the first decoder's read-ahead). Depending on the underlying reader type, this causes:
- The second decoder to start in the middle of the second JSON object → parse error
- Or the second decoder to return `io.EOF` immediately because the underlying `strings.Reader` was fully consumed

In practice with `strings.NewReader`, the first decoder reads all data eagerly, so subsequent decoders get `io.EOF` immediately. Only 1 event is decoded.

**Fix:**

```go
func StreamEvents(r io.Reader) ([]PaymentEvent, error) {
    var events []PaymentEvent
    dec := json.NewDecoder(r)  // ONE decoder, created outside the loop
    for {
        var event PaymentEvent
        err := dec.Decode(&event)
        if err == io.EOF {
            break
        }
        if err != nil {
            return nil, err
        }
        events = append(events, event)
    }
    return events, nil
}
```

The decoder maintains its position across iterations, reading forward through the stream correctly.

**General rule:** When using `json.Decoder` in a loop, create it once before the loop. The same applies to any buffered reader/writer (`bufio.Scanner`, `csv.Reader`, etc.) — the buffer is the state, and recreating it discards that state.

---

## Bug 4: time.Time Serializes as RFC3339, Not Unix Integer

**Location:** `EventExport.OccurredAt` field type

**Symptom:** `TestTimestampFormat` fails — `occurred_at` is a string like `"2026-02-18T10:00:00Z"` but the warehouse expects an integer like `1771408800`.

**Root cause:**

```go
type EventExport struct {
    OccurredAt time.Time `json:"occurred_at"` // marshals to "2026-02-18T10:00:00Z"
}
```

`time.Time` implements `json.Marshaler`. Its `MarshalJSON` method always produces an RFC3339 string. You cannot change this behavior with a struct tag alone.

**Fix: use a custom UnixTime wrapper**

Define a type that wraps `time.Time` and overrides `MarshalJSON`:

```go
type UnixTime struct {
    time.Time
}

func (u UnixTime) MarshalJSON() ([]byte, error) {
    return json.Marshal(u.Unix())
}

func (u *UnixTime) UnmarshalJSON(data []byte) error {
    var ts int64
    if err := json.Unmarshal(data, &ts); err != nil {
        return err
    }
    u.Time = time.Unix(ts, 0).UTC()
    return nil
}
```

Then update the field:

```go
type EventExport struct {
    OccurredAt UnixTime `json:"occurred_at"` // now marshals as integer
}
```

**Why you can't use a struct tag for this:** Struct tags can rename fields, omit them, or change quote behavior for primitives — but they cannot change the serialization logic for a type that implements `json.Marshaler`. The tag is metadata; the type's `MarshalJSON` method always wins.

**Alternative: manually produce the integer in a custom MarshalJSON for EventExport**

If `UnixTime` is too much indirection for your taste, implement `MarshalJSON` on `EventExport` directly. But `UnixTime` is more reusable — define it once, use it anywhere you need Unix timestamps.

---

## Summary

| Bug | Root Cause | Fix |
|-----|-----------|-----|
| 1 — Missing amount | Unexported field ignored by `encoding/json` | Export the field: `Amount int64` |
| 2 — Missing score=0 | `omitempty` on `int` drops zero value | Change to `*int` with `omitempty`, or remove `omitempty` |
| 3 — Only first event | `json.Decoder` created inside loop discards buffer | Create one decoder outside the loop |
| 4 — Wrong timestamp format | `time.Time.MarshalJSON()` always produces RFC3339 | Use a `UnixTime` wrapper with custom `MarshalJSON` |

## Related Concepts

- [[fundamentals/serialization]] — lesson covering all four of these patterns
- [[pitfalls/go-omitempty-zero-int]] — the omitempty zero-value trap
- [[pitfalls/go-unexported-json]] — unexported fields and encoding/json
