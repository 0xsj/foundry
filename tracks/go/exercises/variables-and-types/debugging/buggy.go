package tracker

import "fmt"

// EventStats tracks how many times an event type has been seen
// and the byte size of its last payload.
type EventStats struct {
	Count       int
	LastPayload string
	PayloadSize int
}

// Tracker records incoming webhook events by type.
type Tracker struct {
	events map[string]EventStats
}

// NewTracker creates a new Tracker ready to receive events.
func NewTracker() *Tracker {
	return &Tracker{}
}

// Record processes an incoming webhook event.
func (t *Tracker) Record(eventType string, payload string) {
	stats := t.events[eventType]
	stats.Count++
	stats.LastPayload = payload
	stats.PayloadSize = len(payload)
	t.events[eventType] = stats
}

// Get returns the stats for a given event type.
func (t *Tracker) Get(eventType string) (EventStats, bool) {
	stats, ok := t.events[eventType]
	return stats, ok
}

// Summary prints a summary of all tracked events.
func (t *Tracker) Summary() string {
	result := ""
	for eventType, stats := range t.events {
		result += fmt.Sprintf("%s: count=%d, lastSize=%d chars\n",
			eventType, stats.Count, stats.PayloadSize)
	}
	return result
}

// CharCount returns the number of characters in a payload.
// Used for display purposes on the dashboard.
func CharCount(s string) int {
	return len(s)
}
