// Package webhookprocessor processes incoming webhook events.
// This implementation is correct — do not modify it.
// All bugs are in buggy_test.go.
package webhookprocessor

import (
	"fmt"
	"strings"
	"time"
)

// Event represents a parsed webhook payload.
type Event struct {
	ID        string
	Type      string
	Timestamp time.Time
	Payload   map[string]string
}

// ProcessEvent parses and normalizes a raw webhook event string.
// Format: "TYPE:ID:key=value,key=value"
// Example: "payment.succeeded:evt-001:amount=100,currency=USD"
func ProcessEvent(raw string) (Event, error) {
	parts := strings.SplitN(raw, ":", 3)
	if len(parts) < 2 {
		return Event{}, fmt.Errorf("invalid event format: %q", raw)
	}

	event := Event{
		Type:      parts[0],
		ID:        parts[1],
		Timestamp: time.Now(),
		Payload:   make(map[string]string),
	}

	if len(parts) == 3 {
		for _, kv := range strings.Split(parts[2], ",") {
			pair := strings.SplitN(kv, "=", 2)
			if len(pair) == 2 {
				event.Payload[pair[0]] = pair[1]
			}
		}
	}

	return event, nil
}

// FormatEventSummary returns a human-readable summary of an event.
func FormatEventSummary(e Event) string {
	return fmt.Sprintf("[%s] %s", e.Type, e.ID)
}

// IsRetryable reports whether an event type should be retried on processing failure.
func IsRetryable(eventType string) bool {
	retryable := map[string]bool{
		"payment.succeeded": false,
		"payment.failed":    true,
		"subscription.created": true,
		"webhook.ping":      false,
	}
	return retryable[eventType]
}
