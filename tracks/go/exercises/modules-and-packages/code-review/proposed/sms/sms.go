// Package sms provides SMS notification delivery via a gateway API.
package sms

import (
	"fmt"

	"github.com/foundry/notify/driver"
)

// init registers the SMS driver when this package is imported.
// Same init() registration pattern as email — same tradeoffs apply.
func init() {
	driver.Register("sms", &smsSender{
		apiKey:  "placeholder-key",
		gateway: "api.smsgateway.example.com",
	})
}

// smsSender implements driver.Sender for SMS delivery.
// ISSUE (opposite of email): smsSender is unexported (lowercase), which is
// correct for the implementation type. But the zero-value of smsSender has
// an empty apiKey — if someone constructs one directly (bypassing init),
// it silently sends with no credentials. Unexported types prevent this from
// external packages, but within the sms package itself there's no guard.
// A constructor that validates credentials would be safer.
type smsSender struct {
	apiKey  string
	gateway string
}

// Send delivers an SMS notification.
func (s *smsSender) Send(to, subject, body string) error {
	if s.apiKey == "" {
		// ISSUE: This check should happen at construction time, not at Send time.
		// If apiKey is empty, every Send call fails — but the failure is silent
		// until the first actual send attempt, which could be minutes after startup.
		return fmt.Errorf("sms: no API key configured")
	}
	fmt.Printf("[sms] sending to %s via %s: %s\n", to, s.gateway, subject)
	return nil
}

// Protocol returns "sms".
func (s *smsSender) Protocol() string {
	return "sms"
}
