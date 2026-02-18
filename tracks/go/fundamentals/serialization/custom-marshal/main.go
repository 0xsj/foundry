// Run with: go run ./custom-marshal/
//
// Demonstrates: custom MarshalJSON/UnmarshalJSON, time.Time serialization,
// json.Decoder for streaming, json.RawMessage for deferred parsing,
// and DisallowUnknownFields for strict config loading.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"strings"
	"time"
)

// ============================================================================
// Custom MarshalJSON / UnmarshalJSON
//
// Use case: Money — stored in cents internally, but the JSON representation
// must include a display string and use a specific field layout.
// ============================================================================

// Money stores a monetary amount in the smallest currency unit (cents).
// JSON shape: {"amount_cents": 4999, "currency": "USD", "display": "$49.99"}
type Money struct {
	Amount   int64  // stored in cents
	Currency string // "USD", "EUR", "GBP"
}

// MarshalJSON produces the custom JSON representation.
// The shadow struct avoids infinite recursion — if we called json.Marshal(m),
// it would call MarshalJSON again forever.
func (m Money) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		AmountCents int64  `json:"amount_cents"`
		Currency    string `json:"currency"`
		Display     string `json:"display"`
	}{
		AmountCents: m.Amount,
		Currency:    m.Currency,
		Display:     m.display(),
	})
}

func (m Money) display() string {
	switch m.Currency {
	case "USD":
		return fmt.Sprintf("$%.2f", float64(m.Amount)/100)
	case "EUR":
		return fmt.Sprintf("€%.2f", float64(m.Amount)/100)
	case "GBP":
		return fmt.Sprintf("£%.2f", float64(m.Amount)/100)
	default:
		return fmt.Sprintf("%d %s", m.Amount, m.Currency)
	}
}

// UnmarshalJSON reads the custom format back into a Money.
// UnmarshalJSON must have a pointer receiver so it can modify the value.
func (m *Money) UnmarshalJSON(data []byte) error {
	var raw struct {
		AmountCents int64  `json:"amount_cents"`
		Currency    string `json:"currency"`
		// Display is computed — we ignore it when reading
	}
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	m.Amount = raw.AmountCents
	m.Currency = raw.Currency
	return nil
}

func customMarshal() {
	fmt.Println("=== Custom MarshalJSON / UnmarshalJSON ===")

	price := Money{Amount: 4999, Currency: "USD"}

	data, err := json.MarshalIndent(price, "", "  ")
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("Marshaled:", string(data))

	// Round-trip
	var decoded Money
	if err := json.Unmarshal(data, &decoded); err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Decoded: Amount=%d Currency=%s Display=%s\n",
		decoded.Amount, decoded.Currency, decoded.display())
}

// ============================================================================
// time.Time with a custom format
//
// Default time.Time marshals to RFC3339 ("2026-02-18T10:00:00Z").
// Sometimes you need Unix timestamps, date-only strings, or legacy formats.
// ============================================================================

// UnixTime wraps time.Time and marshals/unmarshals as a Unix integer timestamp.
// Useful for APIs that use epoch seconds instead of ISO strings.
type UnixTime struct {
	time.Time
}

func (u UnixTime) MarshalJSON() ([]byte, error) {
	return json.Marshal(u.Unix()) // produces a JSON integer
}

func (u *UnixTime) UnmarshalJSON(data []byte) error {
	var ts int64
	if err := json.Unmarshal(data, &ts); err != nil {
		return err
	}
	u.Time = time.Unix(ts, 0).UTC()
	return nil
}

// AuditEvent uses UnixTime for its timestamp field.
type AuditEvent struct {
	Action    string   `json:"action"`
	ActorID   string   `json:"actor_id"`
	OccuredAt UnixTime `json:"occurred_at"` // serializes as integer
	// Compare: if this were time.Time, it would serialize as RFC3339 string
}

func timeSerialization() {
	fmt.Println("\n=== time.Time serialization ===")

	// Default time.Time — RFC3339
	type StandardEvent struct {
		Name string    `json:"name"`
		At   time.Time `json:"at"`
	}
	se := StandardEvent{Name: "deploy", At: time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)}
	seData, _ := json.Marshal(se)
	fmt.Printf("RFC3339 (default):   %s\n", seData)

	// Custom UnixTime wrapper
	ae := AuditEvent{
		Action:    "user.login",
		ActorID:   "usr_001",
		OccuredAt: UnixTime{time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)},
	}
	aeData, _ := json.Marshal(ae)
	fmt.Printf("Unix timestamp:      %s\n", aeData)

	// Round-trip the audit event
	var decoded AuditEvent
	_ = json.Unmarshal(aeData, &decoded)
	fmt.Printf("Decoded time: %s\n", decoded.OccuredAt.Format(time.RFC3339))
}

// ============================================================================
// json.Decoder for streaming (NDJSON)
//
// Newline-delimited JSON is common in log pipelines and event streams.
// Each line is a self-contained JSON object.
// json.Decoder handles this without loading the full stream into memory.
// ============================================================================

type LogEntry struct {
	Level   string `json:"level"`
	Message string `json:"msg"`
	Ms      int    `json:"duration_ms"`
}

func streamingDecoder() {
	fmt.Println("\n=== Streaming Decoder (NDJSON) ===")

	// Simulate a log file or streaming response body
	ndjson := `{"level":"info","msg":"server started","duration_ms":0}
{"level":"info","msg":"request processed","duration_ms":42}
{"level":"warn","msg":"slow query detected","duration_ms":1503}
{"level":"error","msg":"connection refused","duration_ms":5001}
`

	dec := json.NewDecoder(strings.NewReader(ndjson))
	var entries []LogEntry

	for {
		var entry LogEntry
		err := dec.Decode(&entry)
		if err == io.EOF {
			break // stream exhausted — not an error
		}
		if err != nil {
			log.Fatalf("parse error: %v", err)
		}
		entries = append(entries, entry)
	}

	fmt.Printf("Decoded %d log entries:\n", len(entries))
	for _, e := range entries {
		fmt.Printf("  [%s] %s (%dms)\n", e.Level, e.Message, e.Ms)
	}
}

// ============================================================================
// json.RawMessage for deferred / discriminated parsing
//
// Use case: a message bus where the "payload" field shape depends on "type".
// We parse the outer envelope immediately; defer parsing the inner payload
// until we know the type.
// ============================================================================

type Message struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload"` // raw bytes — not parsed yet
}

type PaymentPayload struct {
	Amount   int64  `json:"amount"`
	Currency string `json:"currency"`
	InvoiceID string `json:"invoice_id"`
}

type RefundPayload struct {
	OriginalPaymentID string `json:"original_payment_id"`
	Reason            string `json:"reason"`
	Amount            int64  `json:"amount"`
}

func rawMessage() {
	fmt.Println("\n=== json.RawMessage for discriminated types ===")

	messages := []string{
		`{"type":"payment","payload":{"amount":4999,"currency":"USD","invoice_id":"inv_001"}}`,
		`{"type":"refund","payload":{"original_payment_id":"pay_abc","reason":"customer_request","amount":4999}}`,
		`{"type":"unknown_future_type","payload":{"whatever":"we don't know this yet"}}`,
	}

	for _, raw := range messages {
		var msg Message
		if err := json.Unmarshal([]byte(raw), &msg); err != nil {
			log.Fatal(err)
		}

		// Outer parse succeeded. Now branch on type.
		switch msg.Type {
		case "payment":
			var p PaymentPayload
			if err := json.Unmarshal(msg.Payload, &p); err != nil {
				log.Fatal(err)
			}
			fmt.Printf("Payment: $%.2f on invoice %s\n", float64(p.Amount)/100, p.InvoiceID)
		case "refund":
			var r RefundPayload
			if err := json.Unmarshal(msg.Payload, &r); err != nil {
				log.Fatal(err)
			}
			fmt.Printf("Refund: $%.2f (%s) for %s\n", float64(r.Amount)/100, r.Reason, r.OriginalPaymentID)
		default:
			// Forward unknown message types without losing the payload
			fmt.Printf("Unknown type %q — payload preserved: %s\n", msg.Type, msg.Payload)
		}
	}
}

// ============================================================================
// DisallowUnknownFields — strict config parsing
// ============================================================================

type ServerConfig struct {
	Host    string `json:"host"`
	Port    int    `json:"port"`
	TLS     bool   `json:"tls"`
	Timeout int    `json:"timeout_ms"`
}

func strictConfigLoad() {
	fmt.Println("\n=== DisallowUnknownFields (strict config loading) ===")

	valid := `{"host":"api.example.com","port":8443,"tls":true,"timeout_ms":30000}`
	typo := `{"host":"api.example.com","port":8443,"tls":true,"timeoout_ms":30000}` // note the typo

	for _, input := range []string{valid, typo} {
		var cfg ServerConfig
		dec := json.NewDecoder(strings.NewReader(input))
		dec.DisallowUnknownFields()
		err := dec.Decode(&cfg)
		if err != nil {
			fmt.Printf("Config error: %v\n", err)
		} else {
			fmt.Printf("Config loaded: %+v\n", cfg)
		}
	}
	// valid → loads fine
	// typo  → error: json: unknown field "timeoout_ms" — catches the typo at load time
}

func main() {
	customMarshal()
	timeSerialization()
	streamingDecoder()
	rawMessage()
	strictConfigLoad()
}
