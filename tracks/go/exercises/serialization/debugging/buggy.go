package webhookprocessor

import (
	"encoding/json"
	"io"
	"time"
)

// PaymentEvent represents an inbound event from the payment provider.
// Stored internally after parsing; exported as JSON for downstream systems.
type PaymentEvent struct {
	EventID    string    `json:"event_id"`
	EventType  string    `json:"event_type"`
	amount     int64     // BUG 1: unexported field — encoding/json ignores unexported fields.
	Currency   string    `json:"currency"`
	Score      int       `json:"score,omitempty"` // BUG 2: omitempty on int — score=0 is a valid "no fraud" score
	OccurredAt time.Time `json:"occurred_at"`     // BUG 4: time.Time → RFC3339, but warehouse expects Unix int
}

// NewPaymentEvent constructs a PaymentEvent from parsed data.
func NewPaymentEvent(id, eventType string, amount int64, currency string) PaymentEvent {
	return PaymentEvent{
		EventID:    id,
		EventType:  eventType,
		amount:     amount, // stored correctly internally, but invisible to encoding/json
		Currency:   currency,
		OccurredAt: time.Now(),
	}
}

// Amount returns the event amount (unexported field accessor).
func (e PaymentEvent) Amount() int64 {
	return e.amount
}

// ExportJSON serializes the event for downstream consumption.
func (e PaymentEvent) ExportJSON() ([]byte, error) {
	return json.MarshalIndent(e, "", "  ")
}

// ============================================================================
// RiskScore (BUG 2)
// ============================================================================

// RiskScore holds the fraud assessment for a payment event.
// Score of 0 means "no fraud detected" — a meaningful value that must appear in output.
type RiskScore struct {
	EventID string `json:"event_id"`
	Score   int    `json:"score,omitempty"` // BUG 2: drops score=0 (legitimate payment)
	Reason  string `json:"reason,omitempty"`
}

// ExportRiskScore serializes a RiskScore for audit logging.
func ExportRiskScore(rs RiskScore) ([]byte, error) {
	return json.Marshal(rs)
}

// ============================================================================
// StreamEvents (BUG 3)
//
// Reads multiple JSON objects from a stream (NDJSON format).
// BUG: a new json.Decoder is created on each loop iteration.
// The Decoder reads ahead and buffers data. Creating a new one each time
// resets to the start of the stream — only the first event is ever read,
// and the loop never terminates (io.EOF is never returned because the first
// event is always decodeable).
// ============================================================================

// StreamEvents reads all PaymentEvents from an NDJSON stream.
func StreamEvents(r io.Reader) ([]PaymentEvent, error) {
	var events []PaymentEvent

	for {
		// BUG 3: new Decoder created inside the loop.
		// The Decoder maintains its own internal buffer. Creating a new Decoder
		// each iteration throws away any buffering progress — it always starts
		// re-reading from the current position of r, but since json.Decoder
		// buffers ahead, subsequent decoders will stall or re-read data.
		//
		// For an io.Reader backed by a strings.Reader or bytes.Buffer, this means
		// the first Decode always succeeds (first object), and subsequent Decoders
		// see an already-advanced reader position but their internal state thinks
		// they're fresh — leading to errors or duplicate reads depending on the
		// underlying reader implementation.
		//
		// Fix: create ONE Decoder outside the loop.
		dec := json.NewDecoder(r)
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

// ============================================================================
// EventExport (BUG 4)
// ============================================================================

// EventExport is the shape sent to the data warehouse.
// The warehouse ingestion pipeline expects occurred_at as a Unix integer timestamp.
// Using time.Time causes RFC3339 string output, which the pipeline rejects.
type EventExport struct {
	EventID     string    `json:"event_id"`
	EventType   string    `json:"event_type"`
	AmountCents int64     `json:"amount_cents"`
	OccurredAt  time.Time `json:"occurred_at"` // BUG 4: should be a UnixTime type
}

// ExportForWarehouse serializes an EventExport for the data warehouse.
func ExportForWarehouse(e EventExport) ([]byte, error) {
	return json.Marshal(e)
}
