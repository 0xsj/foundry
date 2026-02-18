# Serialization — Go

## What Serialization Actually Is

Serialization is the process of converting an in-memory data structure into a sequence of bytes that can be stored, transmitted, or reconstructed later. The reverse — bytes back to structure — is deserialization (or unmarshaling in Go's vocabulary).

Every time your service reads a config file, parses a webhook payload, writes a JSON response, or exports a CSV report, you're doing serialization. It's not glamorous, but it's everywhere.

Go's standard library handles three formats natively: JSON (`encoding/json`), XML (`encoding/xml`), and CSV (`encoding/csv`). For anything else — MessagePack, Protocol Buffers, YAML — third-party packages follow the same conventions.

---

## encoding/json: The Core API

### Marshal and Unmarshal

The two fundamental operations:

```go
// Marshal: Go value → JSON bytes
data, err := json.Marshal(v)

// Unmarshal: JSON bytes → Go value
err := json.Unmarshal(data, &v)
```

`json.Marshal` traverses your struct using reflection and builds a JSON byte slice. `json.Unmarshal` does the reverse — it parses JSON and populates a struct. Both return errors; always check them.

```go
type WebhookEvent struct {
    EventType string `json:"event_type"`
    Source    string `json:"source"`
    Payload   string `json:"payload"`
    Timestamp int64  `json:"timestamp"`
}

event := WebhookEvent{
    EventType: "payment.completed",
    Source:    "stripe",
    Payload:   `{"amount":4999}`,
    Timestamp: 1708000000,
}

data, err := json.Marshal(event)
if err != nil {
    log.Fatal(err)
}
// data = {"event_type":"payment.completed","source":"stripe","payload":"{\"amount\":4999}","timestamp":1708000000}

var received WebhookEvent
err = json.Unmarshal(data, &received)
// received.EventType == "payment.completed"
```

Note that `Unmarshal` takes a pointer (`&received`). The function needs to write into your variable — it can't do that with a copy. Passing a non-pointer to `Unmarshal` returns an `InvalidUnmarshalError`.

### Struct Tags

Struct tags are the mechanism for controlling how fields serialize. For `encoding/json`, the tag key is `json`:

```go
type APIResponse struct {
    RequestID   string `json:"request_id"`
    StatusCode  int    `json:"status_code"`
    Data        any    `json:"data"`
    Error       string `json:"error,omitempty"`
    internal    string // unexported — always ignored by encoding/json
}
```

The tag value is `"name,options"`. The name overrides the field name in JSON. Options follow after a comma.

**The four tag options you need:**

| Tag | Effect |
|-----|--------|
| `json:"name"` | Use `name` in JSON instead of the field name |
| `json:"name,omitempty"` | Skip field if it's the zero value (empty string, 0, false, nil, empty slice/map) |
| `json:"-"` | Always skip this field (even if exported) |
| `json:"name,string"` | Encode a number or bool as a JSON string |

```go
type Metrics struct {
    RequestCount int64   `json:"request_count,string"` // "1234" in JSON, not 1234
    ErrorRate    float64 `json:"error_rate"`
    InternalTag  string  `json:"-"`                    // never in JSON
    CacheHit     bool    `json:"cache_hit,omitempty"`  // omitted when false
}
```

The `,string` option exists for systems that represent large integers as strings to avoid floating-point precision loss (JavaScript's `number` type can't safely represent integers above 2^53). You'll see this in Twitter/X's API, for example.

### omitempty and Zero Values

`omitempty` omits a field when its value is the **zero value for its type**:

- `string`: `""`
- `int`, `int64`, etc.: `0`
- `bool`: `false`
- pointer: `nil`
- slice: `nil` (but **not** an empty slice `[]T{}`)
- map: `nil` (but **not** an empty map `map[K]V{}`)

This creates a subtle trap: **`omitempty` on an `int` field omits it when the value is `0`**. If `0` is a valid, meaningful value in your domain (HTTP status codes, retry counts, scores), `omitempty` will silently drop it.

```go
type JobResult struct {
    JobID     string `json:"job_id"`
    ExitCode  int    `json:"exit_code,omitempty"` // DANGEROUS: omits on exit code 0 (success!)
    Output    string `json:"output,omitempty"`    // safe: empty output is meaningless
}
```

The fix: use a pointer. A nil pointer serializes as `null`, a pointer to zero (`&0`) serializes as `0`. With a pointer, `omitempty` only omits when the pointer is nil — which you control explicitly.

```go
type JobResult struct {
    JobID    string `json:"job_id"`
    ExitCode *int   `json:"exit_code,omitempty"` // nil = not set, &0 = explicitly 0
    Output   string `json:"output,omitempty"`
}
```

This is the canonical pattern for "nullable" fields in Go JSON: use a pointer.

### Your notes

---

## Struct Tags in Detail

### Field Naming Convention

JSON conventionally uses `snake_case`. Go uses `PascalCase` for exported fields. Without tags, `encoding/json` uses the exact field name — `RequestID` becomes `"RequestID"` in JSON, which is unusual. Always tag fields that will appear in JSON.

```go
// Without tags — non-idiomatic JSON
type User struct {
    UserID    string
    FirstName string
    LastName  string
}
// → {"UserID":"...","FirstName":"...","LastName":"..."}

// With tags — correct
type User struct {
    UserID    string `json:"user_id"`
    FirstName string `json:"first_name"`
    LastName  string `json:"last_name"`
}
// → {"user_id":"...","first_name":"...","last_name":"..."}
```

### Embedded Structs and Promoted Fields

When you embed a struct (without a field name), its fields are **promoted** into the parent. JSON serialization honors this: promoted fields appear as if they were declared directly on the outer struct.

```go
type Metadata struct {
    CreatedAt time.Time `json:"created_at"`
    UpdatedAt time.Time `json:"updated_at"`
    Version   int       `json:"version"`
}

type UserProfile struct {
    Metadata              // embedded — fields promoted
    UserID   string       `json:"user_id"`
    Email    string       `json:"email"`
}

profile := UserProfile{
    Metadata: Metadata{
        CreatedAt: time.Now(),
        UpdatedAt: time.Now(),
        Version:   3,
    },
    UserID: "usr_123",
    Email:  "alice@example.com",
}
```

Serializes as:
```json
{
    "created_at": "2026-02-18T10:00:00Z",
    "updated_at": "2026-02-18T10:00:00Z",
    "version": 3,
    "user_id": "usr_123",
    "email": "alice@example.com"
}
```

The `Metadata` wrapper doesn't appear in JSON. If you used a named field instead (`Meta Metadata`), you'd get a nested `"meta": { ... }` object.

Embedding by pointer (`*Metadata`) works too, but if the pointer is nil, the promoted fields are omitted from JSON output — useful for optional metadata.

### Conflicting Field Names

If the outer struct and an embedded struct both have a field with the same JSON name, the outer struct's field wins (depth rule: shallowest field takes precedence). If two embedded structs at the same depth have the same name, both are omitted. This is a bug waiting to happen — avoid duplicate names across embedded types.

### Your notes

---

## Custom Marshaling

### When You Need It

The default reflection-based marshaling covers 90% of cases. You need custom marshaling when:

- `time.Time` needs a specific format (not RFC3339)
- A value should serialize differently based on its content (e.g., a union/variant type)
- You want to include computed fields that aren't stored on the struct
- You need to rename or restructure the JSON shape significantly
- You're wrapping an external type you don't own

### Implementing MarshalJSON / UnmarshalJSON

Implement `json.Marshaler` and `json.Unmarshaler`:

```go
type Marshaler interface {
    MarshalJSON() ([]byte, error)
}

type Unmarshaler interface {
    UnmarshalJSON([]byte) error
}
```

The canonical pattern for custom marshaling without losing all the struct-tag benefits: use an alias type or a shadow struct.

```go
type Money struct {
    Amount   int64  // stored in cents
    Currency string // "USD", "EUR", etc.
}

// JSON shape: {"amount_cents": 4999, "currency": "USD", "display": "$49.99"}
func (m Money) MarshalJSON() ([]byte, error) {
    return json.Marshal(struct {
        AmountCents int64  `json:"amount_cents"`
        Currency    string `json:"currency"`
        Display     string `json:"display"`
    }{
        AmountCents: m.Amount,
        Currency:    m.Currency,
        Display:     m.displayString(),
    })
}

func (m Money) displayString() string {
    switch m.Currency {
    case "USD":
        return fmt.Sprintf("$%.2f", float64(m.Amount)/100)
    default:
        return fmt.Sprintf("%d %s", m.Amount, m.Currency)
    }
}

func (m *Money) UnmarshalJSON(data []byte) error {
    var raw struct {
        AmountCents int64  `json:"amount_cents"`
        Currency    string `json:"currency"`
    }
    if err := json.Unmarshal(data, &raw); err != nil {
        return err
    }
    m.Amount = raw.AmountCents
    m.Currency = raw.Currency
    return nil
}
```

The shadow struct approach avoids infinite recursion. If you called `json.Marshal(m)` inside `MarshalJSON`, you'd recurse forever. Using an anonymous struct with the same data avoids this.

### time.Time Serialization

`time.Time` implements `json.Marshaler` — it marshals to RFC3339 format by default:

```go
t := time.Now()
data, _ := json.Marshal(t)
// "2026-02-18T10:00:00.123456789Z"
```

But you'll often need different formats — Unix timestamps for interoperability, date-only for birthdates, or a custom format for legacy system compatibility.

```go
type UnixTime struct {
    time.Time
}

func (u UnixTime) MarshalJSON() ([]byte, error) {
    return json.Marshal(u.Unix()) // int64 Unix timestamp
}

func (u *UnixTime) UnmarshalJSON(data []byte) error {
    var ts int64
    if err := json.Unmarshal(data, &ts); err != nil {
        return err
    }
    u.Time = time.Unix(ts, 0).UTC()
    return nil
}

type Event struct {
    Name      string   `json:"name"`
    OccurredAt UnixTime `json:"occurred_at"` // serializes as integer
}
```

For one-off custom formats, you can also use a string with `time.Format`:

```go
const dateFormat = "2006-01-02" // Go's reference time

type Report struct {
    ReportDate string `json:"report_date"` // store as string, parse manually
}
```

Go's reference time is `Mon Jan 2 15:04:05 MST 2006` — use those specific values to define your format. It's unusual but memorable once you know it.

### Your notes

---

## Streaming with json.Decoder and json.Encoder

### Why Streaming

`json.Marshal` / `json.Unmarshal` operate on entire byte slices. For small payloads that's fine. But if you're:
- Reading a 50MB JSON response body from an HTTP request
- Processing a newline-delimited JSON log file with millions of records
- Streaming responses to a client as data becomes available

...you want `json.Decoder` and `json.Encoder`. They operate on `io.Reader` and `io.Writer` respectively, processing data incrementally without loading everything into memory.

```go
// Decoder — read from any io.Reader (HTTP body, file, stdin)
dec := json.NewDecoder(r)
var event WebhookEvent
if err := dec.Decode(&event); err != nil {
    return err
}

// Encoder — write to any io.Writer (HTTP response, file, stdout)
enc := json.NewEncoder(w)
enc.SetIndent("", "  ") // pretty-print
if err := enc.Encode(event); err != nil {
    return err
}
```

### Decoding Multiple Objects (NDJSON)

Newline-delimited JSON (NDJSON or JSON Lines) is a common log format where each line is a separate JSON object. `json.Decoder` handles this naturally:

```go
// Each line: {"level":"info","msg":"request processed","duration_ms":42}
dec := json.NewDecoder(logFile)
for {
    var entry LogEntry
    err := dec.Decode(&entry)
    if err == io.EOF {
        break // done
    }
    if err != nil {
        return fmt.Errorf("parse error: %w", err)
    }
    process(entry)
}
```

This is the correct pattern. `Decode` returns `io.EOF` when the stream is exhausted — that's not an error, it's normal termination. Everything else is a real error.

### Peeking with DisallowUnknownFields

By default, unknown JSON fields are silently ignored during `Unmarshal`. This is often the right behavior — it allows forward compatibility (new fields from a newer server version are ignored by an older client). But when loading config files, unknown fields are usually a typo:

```go
dec := json.NewDecoder(configFile)
dec.DisallowUnknownFields()
if err := dec.Decode(&cfg); err != nil {
    return fmt.Errorf("config error: %w", err) // will mention the unknown field name
}
```

Use `DisallowUnknownFields()` for config loading and other "I control both sides" scenarios. Do not use it for external API responses.

### Your notes

---

## Handling Unknown JSON

### map[string]interface{} (or map[string]any)

Sometimes you don't know the shape of the JSON upfront. The escape hatch: unmarshal into a `map[string]interface{}`.

```go
var raw map[string]any
err := json.Unmarshal(data, &raw)

// Access values with type assertions
if name, ok := raw["name"].(string); ok {
    fmt.Println(name)
}
```

The type mapping that `encoding/json` uses for `interface{}`:

| JSON | Go type |
|------|---------|
| `true`/`false` | `bool` |
| number | `float64` |
| string | `string` |
| array | `[]interface{}` |
| object | `map[string]interface{}` |
| null | `nil` |

Note: JSON numbers always become `float64`, not `int`. `raw["count"].(int)` will always fail — you need `raw["count"].(float64)` and then convert.

Working with deeply nested `map[string]interface{}` gets unwieldy fast. Avoid it beyond the first level. If you find yourself writing a chain of type assertions, define a struct.

### json.RawMessage

`json.RawMessage` is `[]byte` with a special tag that tells `encoding/json` to leave it as raw JSON — don't try to parse it further. Use it when:

- A field's shape depends on another field's value (discriminated unions)
- You want to forward JSON to another system without parsing it
- You want to parse the outer structure immediately but defer inner parsing

```go
type Message struct {
    Type    string          `json:"type"`
    Payload json.RawMessage `json:"payload"` // kept as raw JSON bytes
}

var msg Message
json.Unmarshal(data, &msg)

// Now decode payload based on type
switch msg.Type {
case "payment":
    var p PaymentPayload
    json.Unmarshal(msg.Payload, &p)
case "refund":
    var r RefundPayload
    json.Unmarshal(msg.Payload, &r)
}
```

This is clean and efficient. The outer unmarshal is fast. The inner unmarshal only runs for the type you care about. The `json.RawMessage` retains the original bytes without allocation overhead.

`json.RawMessage` is also the right choice for **forward-compatible parsing** — your system receives JSON from an external service that may add new message types in the future. Unknown types are preserved as raw bytes and can be forwarded unchanged.

### Your notes

---

## Validation After Unmarshal

### The Gap Between Parsing and Validity

JSON parsing and business validation are separate concerns. `json.Unmarshal` tells you whether the JSON is syntactically valid and whether it fits your struct's field types. It cannot tell you whether the values make sense.

After unmarshal, you need a validation pass:

```go
type WebhookConfig struct {
    URL        string   `json:"url"`
    Secret     string   `json:"secret"`
    Events     []string `json:"events"`
    MaxRetries int      `json:"max_retries"`
    TimeoutMs  int      `json:"timeout_ms"`
}

func (c *WebhookConfig) Validate() error {
    if c.URL == "" {
        return fmt.Errorf("url is required")
    }
    if !strings.HasPrefix(c.URL, "https://") {
        return fmt.Errorf("url must use HTTPS")
    }
    if len(c.Events) == 0 {
        return fmt.Errorf("at least one event type required")
    }
    if c.MaxRetries < 0 || c.MaxRetries > 10 {
        return fmt.Errorf("max_retries must be between 0 and 10, got %d", c.MaxRetries)
    }
    if c.TimeoutMs <= 0 {
        return fmt.Errorf("timeout_ms must be positive")
    }
    return nil
}

func LoadConfig(r io.Reader) (*WebhookConfig, error) {
    var cfg WebhookConfig
    dec := json.NewDecoder(r)
    dec.DisallowUnknownFields()
    if err := dec.Decode(&cfg); err != nil {
        return nil, fmt.Errorf("parse error: %w", err)
    }
    if err := cfg.Validate(); err != nil {
        return nil, fmt.Errorf("validation error: %w", err)
    }
    return &cfg, nil
}
```

The pattern: parse first, validate second, always wrap errors with context so the caller knows where the error came from.

For complex validation (cross-field constraints, nested structs), `go-playground/validator` is the standard third-party library — it uses struct tags like `validate:"required,min=1,max=100"`. But the approach above — a `Validate()` method — works well for most cases and keeps everything in the standard library.

### Your notes

---

## encoding/xml Basics

XML is less common in new Go services but you'll encounter it with legacy systems, SOAP APIs, RSS feeds, and SVG files. The API mirrors `encoding/json`:

```go
import "encoding/xml"

type RSSFeed struct {
    XMLName xml.Name   `xml:"rss"`
    Version string     `xml:"version,attr"` // XML attribute, not child element
    Channel RSSChannel `xml:"channel"`
}

type RSSChannel struct {
    Title       string    `xml:"title"`
    Link        string    `xml:"link"`
    Description string    `xml:"description"`
    Items       []RSSItem `xml:"item"`
}

type RSSItem struct {
    Title   string `xml:"title"`
    Link    string `xml:"link"`
    PubDate string `xml:"pubDate"`
    GUID    string `xml:"guid"`
}

data, err := xml.Marshal(feed)
data, err = xml.MarshalIndent(feed, "", "  ") // pretty-print

var feed RSSFeed
err = xml.Unmarshal(data, &feed)
```

XML-specific tag options:

| Tag | Effect |
|-----|--------|
| `xml:"name"` | Element name |
| `xml:"name,attr"` | XML attribute |
| `xml:",chardata"` | Character data content of the element |
| `xml:",innerxml"` | Raw inner XML (like `json.RawMessage`) |
| `xml:"name>child"` | Nested element path |
| `xml:"-"` | Skip field |

`xml.Name` is a special type that controls the element name. When marshaling, the `XMLName` field sets the root element name.

### Your notes

---

## encoding/csv

CSV (Comma-Separated Values) has no standard encoding/decoding via struct tags — `encoding/csv` operates on `[]string` records. You read and write rows manually:

```go
import "encoding/csv"

// Writing
w := csv.NewWriter(file)
w.Write([]string{"id", "name", "email", "amount"}) // header
for _, order := range orders {
    w.Write([]string{
        order.ID,
        order.CustomerName,
        order.Email,
        strconv.FormatFloat(order.Amount, 'f', 2, 64),
    })
}
w.Flush() // csv.Writer buffers — must flush
if err := w.Error(); err != nil {
    return err
}

// Reading
r := csv.NewReader(file)
r.Read() // skip header
for {
    record, err := r.Read()
    if err == io.EOF {
        break
    }
    if err != nil {
        return err
    }
    // record[0] = id, record[1] = name, etc.
    amount, _ := strconv.ParseFloat(record[3], 64)
    orders = append(orders, Order{
        ID:           record[0],
        CustomerName: record[1],
        Email:        record[2],
        Amount:       amount,
    })
}
```

The `csv.Writer` buffers writes internally — **always call `w.Flush()` and check `w.Error()` after writing**. Forgetting `Flush` is a common bug that produces an empty or truncated file.

For large CSV files, `csv.NewReader` reads one record at a time — it doesn't load the entire file into memory.

### Configuring the Reader/Writer

```go
// Tab-separated (TSV)
r := csv.NewReader(file)
r.Comma = '\t'

// Handle variable number of columns
r.FieldsPerRecord = -1

// Skip lines starting with #
r.Comment = '#'
```

### Your notes

---

## Comparison to TypeScript/JavaScript

| Concept | Go | TypeScript/JS |
|---|---|---|
| Parse JSON | `json.Unmarshal(data, &v)` | `JSON.parse(str)` |
| Serialize JSON | `json.Marshal(v)` | `JSON.stringify(v)` |
| Field naming control | Struct tags `json:"name"` | Manual key naming or `toJSON()` |
| Omit empty fields | `json:",omitempty"` | `JSON.stringify` with replacer, or Zod `.optional()` |
| Validation | Manual `Validate()` method, or `go-playground/validator` | Zod, Yup, Joi, class-validator |
| Unknown fields | `map[string]any`, `json.RawMessage` | `Record<string, unknown>`, Zod `.passthrough()` |
| Streaming | `json.NewDecoder(reader)` | Node.js streams, `JSONStream` |
| Custom serialization | `MarshalJSON()` / `UnmarshalJSON()` | `toJSON()` method, or Zod `.transform()` |
| Schema → validation | Tags + reflection (runtime) | TypeScript types (compile-time) + Zod (runtime) |

**The fundamental difference:** TypeScript can validate types at compile time but those checks disappear at runtime — your TypeScript types don't exist in the compiled JavaScript. Runtime validation with Zod (or similar) is a separate layer you add explicitly. In Go, the struct defines the shape AND is validated at parse time — unknown JSON fields cause an error with `DisallowUnknownFields`, wrong types cause errors, missing required fields... well, that's still a separate validation step. Go doesn't have Zod's ergonomics. You either write `Validate()` methods or reach for `go-playground/validator` tags.

**Encoding/decoding errors:** In JavaScript, `JSON.parse` throws. In Go, `json.Unmarshal` returns an error. Same outcome, different control flow — Go requires you to handle the error explicitly rather than wrapping everything in try/catch.

**JSON numbers:** JavaScript's `JSON.parse` gives you `number` for all numeric types. Go unmarshals into the correct numeric type specified by the struct field. When unmarshaling into `interface{}`, Go uses `float64` for all JSON numbers — same as JavaScript, because JSON numbers are IEEE 754 doubles.

## Comparison to Rust (serde)

Rust's `serde` library is the gold standard for serialization ergonomics. Understanding the comparison sharpens how you think about Go's approach.

| Concept | Go | Rust (serde) |
|---|---|---|
| Field renaming | `json:"name"` struct tag | `#[serde(rename = "name")]` attribute |
| Skip field | `json:"-"` | `#[serde(skip)]` |
| Omit if default | `json:",omitempty"` | `#[serde(skip_serializing_if = "Option::is_none")]` |
| Custom format | `MarshalJSON()` method | `#[serde(with = "module")]` or `Serialize` trait |
| Derive | Not applicable — reflection | `#[derive(Serialize, Deserialize)]` |
| Validation | Separate step | Separate step (or `serde`+`validator` crate) |
| Multiple formats | Separate packages | Same `serde` with different serializers |

**The key architectural difference:** Rust's `serde` works at **compile time** via derive macros. The serialization code is generated and compiled — zero runtime reflection. Go's `encoding/json` uses **reflection at runtime**. This means:

- Go's approach is slower (reflection has overhead)
- Go's errors surface at runtime, not compile time
- Rust's derive macros give you compile errors for mismatched types
- But Go's approach requires zero setup — struct tags work without any code generation

Go 1.21 introduced `encoding/json/v2` experiments (not yet stable) that move toward code generation for performance, but the tag-based API remains the standard.

**Serde's `skip_serializing_if` vs Go's `omitempty`:** Rust lets you specify an arbitrary function (`Option::is_none`, `Vec::is_empty`, or your own). Go's `omitempty` only uses the zero-value check. When you need "omit if empty slice" in Go, your options are: use a pointer (nil vs non-nil), implement `MarshalJSON`, or accept the limitation.

### Your notes
