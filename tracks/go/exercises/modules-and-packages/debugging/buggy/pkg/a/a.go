// Package a provides notification routing configuration.
// It decides which channels (email, SMS, webhook) to use for a given event type.
package a

import (
	// BUG 1: Package a imports package b, and package b imports package a.
	// This creates an import cycle — the Go compiler does not allow cycles.
	// The import graph must be a directed acyclic graph (DAG).
	"github.com/foundry/notify/pkg/b"
)

// ChannelConfig holds routing configuration for a notification channel.
type ChannelConfig struct {
	Channel  string
	Priority int
}

// RouteEvent returns the list of channels to use for a given event type.
// It uses package b to look up the sender's capabilities.
func RouteEvent(eventType string) []ChannelConfig {
	// Check if the email sender supports this event type
	sender := b.NewSender("smtp")
	if sender == nil {
		return nil
	}

	// Route critical events to high-priority channels
	if eventType == "payment.failed" || eventType == "auth.breach" {
		return []ChannelConfig{
			{Channel: "email", Priority: 1},
			{Channel: "sms", Priority: 2},
		}
	}

	return []ChannelConfig{
		{Channel: "email", Priority: 1},
	}
}
