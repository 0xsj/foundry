package webhookprocessor

import (
	"encoding/json"
	"strings"
	"testing"
	"time"
)

// TestEventRoundTrip verifies that marshaling a PaymentEvent preserves the amount.
// FAILS: the amount field is unexported — encoding/json ignores unexported fields.
// amount is always absent from JSON output, so a round-trip loses the value.
func TestEventRoundTrip(t *testing.T) {
	event := NewPaymentEvent("evt_001", "payment.completed", 4999, "USD")

	data, err := event.ExportJSON()
	if err != nil {
		t.Fatalf("ExportJSON error: %v", err)
	}

	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	amountRaw, ok := raw["amount"]
	if !ok {
		t.Fatal("amount field missing from JSON — is the field exported?")
	}
	amount, ok := amountRaw.(float64)
	if !ok {
		t.Fatalf("amount should be a number, got %T", amountRaw)
	}
	if int64(amount) != 4999 {
		t.Errorf("amount = %v, want 4999", amount)
	}
}

// TestZeroScoreOmitted verifies that a score of 0 is preserved in JSON output.
// FAILS: omitempty on int omits zero — but score=0 means "no fraud detected",
// which is a meaningful value that must appear in the audit log.
func TestZeroScoreOmitted(t *testing.T) {
	rs := RiskScore{
		EventID: "evt_001",
		Score:   0, // legitimate payment — explicitly no fraud risk
		Reason:  "",
	}

	data, err := ExportRiskScore(rs)
	if err != nil {
		t.Fatalf("ExportRiskScore error: %v", err)
	}

	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	if _, ok := raw["score"]; !ok {
		t.Error("score field missing from JSON — score=0 is a valid, meaningful value and must not be omitted")
	}
	if raw["score"] != float64(0) {
		t.Errorf("score = %v, want 0", raw["score"])
	}
}

// TestStreamMultipleEvents verifies that StreamEvents correctly reads all
// events from an NDJSON stream.
// FAILS: a new json.Decoder is created inside the loop on each iteration.
// After the first event is decoded, subsequent Decoders see an advanced reader
// position but their buffers are empty — subsequent Decode calls return errors
// or never advance, causing the loop to terminate early or incorrectly.
func TestStreamMultipleEvents(t *testing.T) {
	ndjson := `{"event_id":"evt_001","event_type":"payment.completed","currency":"USD"}
{"event_id":"evt_002","event_type":"payment.completed","currency":"EUR"}
{"event_id":"evt_003","event_type":"refund.created","currency":"USD"}
`

	events, err := StreamEvents(strings.NewReader(ndjson))
	if err != nil {
		t.Fatalf("StreamEvents error: %v", err)
	}

	if len(events) != 3 {
		t.Fatalf("expected 3 events, got %d — decoder-per-loop bug: each new Decoder starts fresh", len(events))
	}
	if events[0].EventID != "evt_001" {
		t.Errorf("events[0].EventID = %q, want evt_001", events[0].EventID)
	}
	if events[1].EventID != "evt_002" {
		t.Errorf("events[1].EventID = %q, want evt_002", events[1].EventID)
	}
	if events[2].EventID != "evt_003" {
		t.Errorf("events[2].EventID = %q, want evt_003", events[2].EventID)
	}
}

// TestTimestampFormat verifies that occurred_at serializes as a Unix integer,
// not as an RFC3339 string.
// FAILS: EventExport uses time.Time which defaults to RFC3339 string output.
// The data warehouse rejects string timestamps — it expects integer seconds.
func TestTimestampFormat(t *testing.T) {
	ts := time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	export := EventExport{
		EventID:     "evt_001",
		EventType:   "payment.completed",
		AmountCents: 4999,
		OccurredAt:  ts,
	}

	data, err := ExportForWarehouse(export)
	if err != nil {
		t.Fatalf("ExportForWarehouse error: %v", err)
	}

	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("unmarshal error: %v", err)
	}

	val, ok := raw["occurred_at"]
	if !ok {
		t.Fatal("occurred_at field missing")
	}

	// The data warehouse expects a Unix integer, not a string
	if _, isString := val.(string); isString {
		t.Fatalf("occurred_at should be a Unix integer, got string %q — change OccurredAt to a UnixTime type", val)
	}
	num, isNum := val.(float64)
	if !isNum {
		t.Fatalf("occurred_at should be a number (Unix timestamp), got %T: %v", val, val)
	}

	want := float64(ts.Unix())
	if num != want {
		t.Errorf("occurred_at = %v, want %v (Unix timestamp for %v)", num, want, ts)
	}
}
