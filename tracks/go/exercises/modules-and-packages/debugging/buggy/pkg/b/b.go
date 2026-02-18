// Package b provides notification senders (email, SMS, webhook).
package b

import (
	// BUG 1 (continued): Package b imports package a, completing the cycle.
	// This is the other half of the circular dependency.
	"github.com/foundry/notify/pkg/a"
)

// Sender is the interface that all notification senders must implement.
type Sender interface {
	Send(recipient, message string) error
	Protocol() string
}

// sender is the concrete SMTP email sender.
// BUG 3: sender starts with a lowercase letter — it is unexported.
// NewSender returns *sender, but the caller in main.go tries to assign
// it to a variable of type b.Sender (the interface). An unexported type
// *sender cannot be used as an exported interface type by an external package.
// The fix: either export the type (Sender → SMTPSender), or change NewSender
// to return the Sender interface directly.
type sender struct {
	protocol string
	maxRetry int
}

// Send delivers a notification via SMTP.
func (s *sender) Send(recipient, message string) error {
	_ = recipient
	_ = message
	// In a real implementation, this would dial an SMTP server.
	return nil
}

// Protocol returns the transport protocol name.
func (s *sender) Protocol() string {
	return s.protocol
}

// NewSender creates a new sender for the given protocol.
// Currently only "smtp" is supported.
func NewSender(protocol string) *sender { // returns unexported type
	if protocol != "smtp" {
		return nil
	}
	return &sender{protocol: protocol, maxRetry: 3}
}

// RouteForCapability uses package a to get routing config.
// This is the second half of the circular import.
func RouteForCapability(eventType string) string {
	routes := a.RouteEvent(eventType)
	if len(routes) == 0 {
		return "none"
	}
	return routes[0].Channel
}
