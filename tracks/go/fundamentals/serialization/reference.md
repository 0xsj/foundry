# Go Standard Library Reference — Serialization

> Extracted from [encoding/json](https://pkg.go.dev/encoding/json),
> [encoding/xml](https://pkg.go.dev/encoding/xml), and
> [encoding/csv](https://pkg.go.dev/encoding/csv) package documentation.
> Covers: Marshal, Unmarshal, Decoder, Encoder, struct tags, custom marshaling,
> RawMessage, and related types.

---

## encoding/json

Source: [pkg.go.dev/encoding/json](https://pkg.go.dev/encoding/json)

### Package Overview

Package `json` implements encoding and decoding of JSON as defined in RFC 7159. The mapping between JSON and Go values is described in the documentation for the `Marshal` and `Unmarshal` functions.

```go
import "encoding/json"
```

---

### func Marshal

```go
func Marshal(v any) ([]byte, error)
```

`Marshal` returns the JSON encoding of `v`.

`Marshal` traverses the value `v` recursively. If an encountered value implements the `Marshaler` interface and is not a nil pointer, `Marshal` calls its `MarshalJSON` method to produce JSON. If no `MarshalJSON` method is present but the value implements `encoding.TextMarshaler` instead, `Marshal` calls its `MarshalText` method and encodes the result as a JSON string.

**Type encoding rules:**

| Go type | JSON encoding |
|---------|---------------|
| `bool` | `true` or `false` |
| floating point, integer | number |
| `string` | JSON string (UTF-8), special chars escaped |
| `[]byte` | base64-encoded string |
| `array`, `slice` | JSON array (`null` for nil slice) |
| `struct` | JSON object |
| `map` | JSON object (key must be string, integer, or implement `encoding.TextMarshaler`) |
| pointer | value pointed to, or `null` for nil pointer |
| `interface{}` | value stored in interface, or `null` |
| `chan`, `func`, `complex` | `UnsupportedTypeError` |

**Struct fields** are encoded as JSON object members following these rules:
- Only exported fields are encoded.
- The field name is used as the JSON key unless overridden by a `json` struct tag.
- If the field tag specifies `"-"` as the name, the field is always omitted.
- If the field tag includes the `"omitempty"` option, the field is omitted when the value is the zero value for the type.

```go
// These produce the same JSON key:
Field int `json:"myName"`      // "myName"
Field int `json:"myName,omitempty"` // "myName" if non-zero

// Omit the field entirely:
Field int `json:"-"`

// Keep the field name, use omitempty:
Field int `json:",omitempty"`
```

**Anonymous struct fields** are usually marshaled as if their inner exported fields were fields in the outer struct, subject to the usual Go visibility rules. An anonymous struct field with a name given in its JSON tag is treated as having that name instead of being anonymous.

**Marshaling errors:**
- Cyclic data structures are not supported and cause infinite recursion.
- `chan`, `func`, `complex` types cause `UnsupportedTypeError`.
- Map keys must be strings, integers, or implement `encoding.TextMarshaler`.

---

### func Unmarshal

```go
func Unmarshal(data []byte, v any) error
```

`Unmarshal` parses the JSON-encoded `data` and stores the result in the value pointed to by `v`. If `v` is nil or not a pointer, `Unmarshal` returns an `InvalidUnmarshalError`.

`Unmarshal` uses the inverse of the encodings that `Marshal` uses, allocating maps, slices, and pointers as necessary.

**Type decoding rules:**

| JSON type | Default Go type (into `interface{}`) |
|-----------|--------------------------------------|
| `bool` | `bool` |
| number | `float64` |
| string | `string` |
| array | `[]interface{}` |
| object | `map[string]interface{}` |
| null | `nil` |

**Into a pointer:** If `v` is nil, `Unmarshal` allocates a new value to point to.

**Into a struct:** `Unmarshal` matches JSON object keys to struct field names (or JSON tag names) case-insensitively. Unmatched fields in the JSON are silently ignored by default.

**Into a `[]byte`:** JSON strings are decoded as base64.

**Unknown fields:** By default, unknown fields are silently discarded. Use `(*Decoder).DisallowUnknownFields()` to return an error on unknown fields.

**Number precision:** JSON numbers decoded into `interface{}` always become `float64`. To preserve large integers, use `(*Decoder).UseNumber()` which returns `json.Number` (a string type with `Int64()`, `Float64()`, `String()` methods).

---

### type Decoder

```go
type Decoder struct { /* unexported fields */ }

func NewDecoder(r io.Reader) *Decoder
func (dec *Decoder) Decode(v any) error
func (dec *Decoder) DisallowUnknownFields()
func (dec *Decoder) UseNumber()
func (dec *Decoder) Buffered() io.Reader
func (dec *Decoder) InputOffset() int64
func (dec *Decoder) More() bool
func (dec *Decoder) Token() (Token, error)
```

`NewDecoder` returns a new decoder that reads from `r`. The decoder introduces its own buffering and may read data from `r` beyond the JSON values requested.

`Decode` reads the next JSON-encoded value from its input and stores it in the value pointed to by `v`.

`DisallowUnknownFields` causes the Decoder to return an error when the destination is a struct and the input contains object keys which do not match any non-ignored, exported fields in the destination.

`UseNumber` causes the Decoder to unmarshal a number into an interface{} as a `Number` instead of a `float64`.

`More` reports whether there is another element in the current array or object being parsed.

**Reading multiple values from one stream:**

```go
dec := json.NewDecoder(r)
for dec.More() {
    var v MyType
    if err := dec.Decode(&v); err != nil {
        return err
    }
    // process v
}
```

---

### type Encoder

```go
type Encoder struct { /* unexported fields */ }

func NewEncoder(w io.Writer) *Encoder
func (enc *Encoder) Encode(v any) error
func (enc *Encoder) SetIndent(prefix, indent string)
func (enc *Encoder) SetEscapeHTML(on bool)
```

`NewEncoder` returns a new encoder that writes to `w`.

`Encode` writes the JSON encoding of `v` to the stream, followed by a newline character.

`SetIndent` instructs the encoder to format each subsequent encoded value as if indented by the package-level `MarshalIndent` with the specified prefix and indent string.

`SetEscapeHTML` specifies whether certain problematic HTML characters should be escaped inside JSON quoted strings. The default is true. Setting to false avoids `<`, `>`, `&` being escaped to `\u003c`, `\u003e`, `\u0026`.

---

### type RawMessage

```go
type RawMessage []byte

func (m RawMessage) MarshalJSON() ([]byte, error)
func (m *RawMessage) UnmarshalJSON(data []byte) error
```

`RawMessage` is a raw encoded JSON value. It implements `Marshaler` and `Unmarshaler` and can be used to delay JSON decoding or to precompute a JSON encoding.

```go
// Delay decoding of inner field
type Container struct {
    Type string          `json:"type"`
    Data json.RawMessage `json:"data"` // kept as raw bytes
}

// Precompute a JSON encoding
var msg = json.RawMessage(`{"foo":"bar"}`)
```

---

### type Number

```go
type Number string

func (n Number) Float64() (float64, error)
func (n Number) Int64() (int64, error)
func (n Number) String() string
```

`Number` represents a JSON number literal. Used when `UseNumber()` is called on a decoder to preserve the exact numeric representation.

---

### Interfaces

#### json.Marshaler

```go
type Marshaler interface {
    MarshalJSON() ([]byte, error)
}
```

Types that implement `Marshaler` control their own JSON encoding. `MarshalJSON` must return valid JSON.

#### json.Unmarshaler

```go
type Unmarshaler interface {
    UnmarshalJSON([]byte) error
}
```

Types that implement `Unmarshaler` control their own JSON decoding. The input slice may be retained after `UnmarshalJSON` returns.

---

### Struct Tag Reference

The `json` struct tag format:

```
`json:"[name][,option1][,option2]..."`
```

**Name:** The JSON key to use. If empty, the field name is used. If `"-"`, the field is always omitted (note: `json:"-,"` uses a literal hyphen as the key).

**Options:**

| Option | Description |
|--------|-------------|
| `omitempty` | Omit the field if its value is the zero value for its type: `false`, `0`, `nil` pointer, nil interface, empty string, empty array/slice/map |
| `string` | Encode the field as a JSON string. Only valid for string, floating point, integer, or boolean fields. The field value is marshaled into JSON and then encoded as a JSON string |

**Examples:**

```go
// Common patterns
Field string  `json:"field_name"`                // rename
Field string  `json:"field_name,omitempty"`       // rename + omit if empty
Field string  `json:",omitempty"`                  // keep field name, omit if empty
Field string  `json:"-"`                           // always omit
Field string  `json:"-,"`                          // JSON key is literally "-"
Field int64   `json:"count,string"`                // encode as "123" not 123
Field *string `json:"value,omitempty"`             // omit if nil pointer
```

---

### Error Types

```go
// Returned by Marshal when encoding a value of an unsupported type
type UnsupportedTypeError struct {
    Type reflect.Type
}

// Returned by Marshal when trying to encode a cyclic value
type UnsupportedValueError struct {
    Value reflect.Value
    Str   string
}

// Returned by Unmarshal when the argument is not a pointer or is nil
type InvalidUnmarshalError struct {
    Type reflect.Type
}

// Describes a JSON value that cannot be stored in a Go value of a specific type
type UnmarshalTypeError struct {
    Value  string       // JSON value description
    Type   reflect.Type // Go type
    Offset int64        // byte offset where error occurred
    Struct string       // struct type name
    Field  string       // field path from root node
}

// Describes a syntax error in JSON input
type SyntaxError struct {
    Offset int64 // error occurred after reading Offset bytes
    // msg is unexported
}
```

---

## encoding/xml

Source: [pkg.go.dev/encoding/xml](https://pkg.go.dev/encoding/xml)

### Package Overview

Package `xml` implements a simple XML 1.0 parser that understands XML name spaces.

```go
import "encoding/xml"
```

### func Marshal / MarshalIndent

```go
func Marshal(v any) ([]byte, error)
func MarshalIndent(v any, prefix, indent string) ([]byte, error)
```

### func Unmarshal

```go
func Unmarshal(data []byte, v any) error
```

### type Decoder (XML)

```go
func NewDecoder(r io.Reader) *Decoder
func (d *Decoder) Decode(v any) error
func (d *Decoder) Token() (Token, error)
```

### type Encoder (XML)

```go
func NewEncoder(w io.Writer) *Encoder
func (enc *Encoder) Encode(v any) error
func (enc *Encoder) EncodeElement(v any, start StartElement) error
func (enc *Encoder) EncodeToken(t Token) error
func (enc *Encoder) Flush() error
func (enc *Encoder) Indent(prefix, indent string)
```

### XML Struct Tag Reference

The `xml` struct tag format controls XML marshaling:

| Tag | Description |
|-----|-------------|
| `xml:"name"` | Element name |
| `xml:"name,attr"` | Attribute of enclosing element |
| `xml:",chardata"` | Character data content of element |
| `xml:",cdata"` | Character data, wrapped in CDATA tags |
| `xml:",innerxml"` | Raw inner XML verbatim |
| `xml:",comment"` | XML comment |
| `xml:"a>b>c"` | Nested elements: `<a><b><c>` |
| `xml:"-"` | Never marshal this field |
| `xml:",omitempty"` | Omit if value is empty |

### type Name

```go
type Name struct {
    Space, Local string
}
```

An XML `Name` identifies an XML element or attribute. The `XMLName` field in a struct sets the element name when marshaling:

```go
type Feed struct {
    XMLName xml.Name `xml:"feed"`
    // ...
}
// marshals as <feed>...</feed>
```

---

## encoding/csv

Source: [pkg.go.dev/encoding/csv](https://pkg.go.dev/encoding/csv)

### Package Overview

Package `csv` reads and writes comma-separated values (CSV) files. There are many kinds of CSV files; this package supports the format described in RFC 4180.

```go
import "encoding/csv"
```

### type Reader

```go
func NewReader(r io.Reader) *Reader
func (r *Reader) Read() (record []string, err error)
func (r *Reader) ReadAll() (records [][]string, err error)
```

**Reader fields:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `Comma` | `rune` | `','` | Field delimiter |
| `Comment` | `rune` | `0` | If non-zero, lines beginning with this character are ignored |
| `FieldsPerRecord` | `int` | `0` | Number of fields per record. 0 = set to number of fields in first record. -1 = variable |
| `LazyQuotes` | `bool` | `false` | If true, quotes may appear in unquoted fields and non-doubled quotes may appear in quoted fields |
| `TrimLeadingSpace` | `bool` | `false` | If true, leading whitespace in a field is ignored |
| `ReuseRecord` | `bool` | `false` | If true, `Read` may return a slice sharing backing array of previous call |

`Read` returns `io.EOF` when no more records remain.

`ReadAll` reads all remaining records from `r`.

### type Writer

```go
func NewWriter(w io.Writer) *Writer
func (w *Writer) Write(record []string) error
func (w *Writer) WriteAll(records [][]string) error
func (w *Writer) Flush()
func (w *Writer) Error() error
```

**Writer fields:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `Comma` | `rune` | `','` | Field delimiter |
| `UseCRLF` | `bool` | `false` | If true, use `\r\n` as line terminator |

`Write` writes a single CSV record. Records are buffered — call `Flush` to ensure all pending data is written.

`Flush` flushes any buffered data to the underlying `io.Writer`.

`Error` reports any error that has occurred during a previous `Write` or `Flush`.

**Important:** Always call `Flush()` after writing and check `Error()`. Forgetting `Flush` leaves data in the buffer.

```go
w := csv.NewWriter(f)
defer func() {
    w.Flush()
    if err := w.Error(); err != nil {
        log.Fatal(err)
    }
}()
```

---

## time.Time and JSON

`time.Time` implements `json.Marshaler` and `json.Unmarshaler`.

**Default format:** RFC 3339 with sub-second precision:
```
"2006-01-02T15:04:05.999999999Z07:00"
```

**Go's time format reference:** The reference time is `Mon Jan 2 15:04:05 MST 2006` (or `01/02 03:04:05PM '06 -0700`). Use these specific values to define custom formats:

```go
const (
    RFC3339     = "2006-01-02T15:04:05Z07:00"
    RFC3339Nano = "2006-01-02T15:04:05.999999999Z07:00"
    DateOnly    = "2006-01-02"
    TimeOnly    = "15:04:05"
    DateTime    = "2006-01-02 15:04:05"
)
```

**Parsing:**
```go
t, err := time.Parse(time.RFC3339, "2026-02-18T10:00:00Z")
t, err = time.Parse("2006-01-02", "2026-02-18")
```

**Formatting:**
```go
s := t.Format(time.RFC3339)        // "2026-02-18T10:00:00Z"
s = t.Format("2006-01-02")         // "2026-02-18"
s = t.UTC().Format(time.RFC3339)   // always UTC
```

**Zero value:** `time.Time{}` marshals to `"0001-01-01T00:00:00Z"`. Use `omitempty` with a pointer (`*time.Time`) to omit unset times — but note that `time.Time` value with `omitempty` does NOT omit the zero time (structs are never considered "empty" by `omitempty`).

```go
type Event struct {
    Name       string     `json:"name"`
    OccurredAt time.Time  `json:"occurred_at"`            // always included
    FinishedAt *time.Time `json:"finished_at,omitempty"` // omitted if nil
}
```

---

## json.MarshalIndent

```go
func MarshalIndent(v any, prefix, indent string) ([]byte, error)
```

`MarshalIndent` is like `Marshal` but applies an indent to format the output.

```go
data, err := json.MarshalIndent(v, "", "  ") // 2-space indent
data, err = json.MarshalIndent(v, "", "\t")  // tab indent
```

---

## json.Valid

```go
func Valid(data []byte) bool
```

`Valid` reports whether `data` is a valid JSON encoding. Useful for validating raw JSON before attempting to unmarshal.

---

## json.Compact and json.HTMLEscape

```go
func Compact(dst *bytes.Buffer, src []byte) error
func HTMLEscape(dst *bytes.Buffer, src []byte)
func Indent(dst *bytes.Buffer, src []byte, prefix, indent string) error
```

`Compact` appends the JSON-encoded `src` to `dst`, eliding insignificant whitespace.

`HTMLEscape` appends the JSON-encoded `src` to `dst` with `<`, `>`, `&`, `U+2028`, and `U+2029` characters inside string literals changed to `\u003c`, `\u003e`, `\u0026`, `\u2028`, `\u2029` so that the JSON will be safe to embed inside HTML `<script>` tags.
