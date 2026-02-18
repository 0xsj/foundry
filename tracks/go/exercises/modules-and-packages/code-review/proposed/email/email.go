// Package email provides SMTP email notification delivery.
package email

import (
	"fmt"

	"github.com/foundry/notify/driver"
)

// init registers the email driver automatically when this package is imported.
// ISSUE: The caller must blank-import this package to activate it:
//
//	import _ "github.com/foundry/notify/email"
//
// If they forget the blank import, the "email" driver silently doesn't exist.
// This pattern (from database/sql's driver model) is reasonable for open-ended
// plugin systems, but for a fixed set of known drivers (email, SMS), explicit
// registration is clearer and easier to test.
func init() {
	driver.Register("email", &EmailSender{
		host: "smtp.example.com",
		port: 587,
	})
}

// EmailSender sends notifications via SMTP.
// ISSUE: EmailSender is exported, but it's an implementation detail.
// Callers retrieve it through driver.Get("email") and use it as driver.Sender.
// They never need to construct an EmailSender directly — the init() function
// handles that. Exporting it leaks the implementation and prevents the
// struct from being changed without a breaking change.
// Should be: type emailSender struct (unexported).
type EmailSender struct {
	host string
	port int
}

// Send delivers an email notification.
func (e *EmailSender) Send(to, subject, body string) error {
	// In a real implementation, this would dial the SMTP server.
	fmt.Printf("[email] sending to %s via %s:%d: %s\n", to, e.host, e.port, subject)
	return nil
}

// Protocol returns "email".
func (e *EmailSender) Protocol() string {
	return "email"
}
