// Package main demonstrates Go interface basics:
// - Interface definition and implicit satisfaction
// - Polymorphism: multiple types behind one interface
// - Interface guards (compile-time satisfaction checks)
// - Accept interfaces, return structs
// - Common stdlib interfaces: fmt.Stringer, error, io.Writer
//
// Run with: go run ./basics/
package main

import (
	"errors"
	"fmt"
	"strings"
)

// ============================================================================
// DEFINING AN INTERFACE
//
// A Notifier can send a message. Any type with a Send method and a Name
// method satisfies this interface — no declaration needed.
// ============================================================================

// Notifier represents any delivery channel that can send a notification.
// Naming convention: single-method interfaces use the method name + "er".
// Multi-method interfaces describe a role (Notifier, Handler, Processor).
type Notifier interface {
	Send(recipient, subject, body string) error
	Name() string
}

// ============================================================================
// CONCRETE TYPES THAT SATISFY THE INTERFACE
//
// Neither EmailNotifier nor SlackNotifier declares "implements Notifier".
// The compiler verifies the method set at the point of use.
// ============================================================================

// EmailNotifier sends notifications via SMTP.
type EmailNotifier struct {
	SMTPHost string
	FromAddr string
}

// Send implements Notifier. Pointer receiver because EmailNotifier has state
// that might be updated (e.g., tracking sent count — not shown here, but
// once you use pointer receivers on any method, use them on all).
func (e *EmailNotifier) Send(recipient, subject, body string) error {
	if recipient == "" {
		return fmt.Errorf("email: recipient is required")
	}
	fmt.Printf("[email] %s → %s | subject: %s\n", e.FromAddr, recipient, subject)
	return nil
}

func (e *EmailNotifier) Name() string { return "email" }

// SlackNotifier sends notifications to a Slack workspace.
type SlackNotifier struct {
	Workspace string
	BotToken  string
}

func (s *SlackNotifier) Send(recipient, subject, body string) error {
	if recipient == "" {
		return fmt.Errorf("slack: channel is required")
	}
	fmt.Printf("[slack] workspace=%s channel=%s subject=%s\n", s.Workspace, recipient, subject)
	return nil
}

func (s *SlackNotifier) Name() string { return "slack" }

// SMSNotifier sends notifications via SMS.
type SMSNotifier struct {
	Provider  string
	SenderNum string
}

func (s *SMSNotifier) Send(recipient, subject, body string) error {
	if recipient == "" {
		return fmt.Errorf("sms: phone number is required")
	}
	fmt.Printf("[sms] %s → %s via %s\n", s.SenderNum, recipient, s.Provider)
	return nil
}

func (s *SMSNotifier) Name() string { return "sms" }

// ============================================================================
// INTERFACE GUARDS
//
// Compile-time assertions that each concrete type satisfies Notifier.
// These produce zero runtime overhead — the variable is discarded.
// Place them directly below the type definitions or at the top of the file.
//
// If *EmailNotifier is missing any Notifier method, this line fails to compile
// with a clear error: "does not implement Notifier (missing method X)".
// ============================================================================

var _ Notifier = (*EmailNotifier)(nil)
var _ Notifier = (*SlackNotifier)(nil)
var _ Notifier = (*SMSNotifier)(nil)

// ============================================================================
// ACCEPT INTERFACES — POLYMORPHISM
//
// broadcastAlert accepts a Notifier — it doesn't know or care which
// concrete type it's talking to. Email, Slack, SMS: all the same to it.
// ============================================================================

// broadcastAlert sends the same notification through multiple channels.
// Accepts []Notifier — any type satisfying the interface works.
func broadcastAlert(notifiers []Notifier, recipient, subject, body string) []error {
	var errs []error
	for _, n := range notifiers {
		if err := n.Send(recipient, subject, body); err != nil {
			errs = append(errs, fmt.Errorf("channel %s: %w", n.Name(), err))
		}
	}
	return errs
}

// ============================================================================
// RETURN STRUCTS
//
// Constructor functions return the concrete type, not the interface.
// Callers get the full type — all methods, not just the interface methods.
// They can always assign to a Notifier variable themselves if needed.
// ============================================================================

func NewEmailNotifier(host, from string) *EmailNotifier {
	return &EmailNotifier{SMTPHost: host, FromAddr: from}
}

func NewSlackNotifier(workspace, token string) *SlackNotifier {
	return &SlackNotifier{Workspace: workspace, BotToken: token}
}

// ============================================================================
// STDLIB INTERFACES: fmt.Stringer
//
// Any type with String() string implements fmt.Stringer.
// fmt.Println, fmt.Sprintf("%v"), and friends call String() automatically.
// ============================================================================

// NotificationPriority is an enum-like type with a String() method.
// The String() method makes it implement fmt.Stringer implicitly.
type NotificationPriority int

const (
	PriorityLow NotificationPriority = iota
	PriorityNormal
	PriorityHigh
	PriorityCritical
)

func (p NotificationPriority) String() string {
	switch p {
	case PriorityLow:
		return "low"
	case PriorityNormal:
		return "normal"
	case PriorityHigh:
		return "high"
	case PriorityCritical:
		return "critical"
	default:
		return fmt.Sprintf("NotificationPriority(%d)", int(p))
	}
}

// NotificationPriority satisfies fmt.Stringer — fmt uses String() automatically.
var _ fmt.Stringer = NotificationPriority(0)

// ============================================================================
// STDLIB INTERFACES: error
//
// The built-in error interface: type error interface { Error() string }
// Any type with Error() string implements it. This is how custom errors work.
// ============================================================================

// DeliveryError captures structured information about a failed delivery.
type DeliveryError struct {
	Channel   string
	Recipient string
	Cause     error
}

func (e *DeliveryError) Error() string {
	return fmt.Sprintf("delivery failed on %s to %s: %v", e.Channel, e.Recipient, e.Cause)
}

// Unwrap enables errors.Is and errors.As to traverse the error chain.
func (e *DeliveryError) Unwrap() error { return e.Cause }

// *DeliveryError satisfies the built-in error interface.
var _ error = (*DeliveryError)(nil)

// ============================================================================
// STDLIB INTERFACES: io.Writer
//
// io.Writer has one method: Write(p []byte) (n int, err error).
// A LogWriter captures output as a slice of lines — useful for testing.
// ============================================================================

// LogWriter is an io.Writer that collects lines written to it.
// Useful for capturing output in tests or routing logs to multiple sinks.
type LogWriter struct {
	lines []string
	buf   strings.Builder
}

func (w *LogWriter) Write(p []byte) (n int, err error) {
	w.buf.Write(p)
	// Flush complete lines
	for {
		s := w.buf.String()
		idx := strings.Index(s, "\n")
		if idx < 0 {
			break
		}
		w.lines = append(w.lines, s[:idx])
		w.buf.Reset()
		w.buf.WriteString(s[idx+1:])
	}
	return len(p), nil
}

func (w *LogWriter) Lines() []string { return w.lines }

// Note: *LogWriter satisfies io.Writer (via Write method) but NOT fmt.Stringer
// (no String() method). Interface guards only compile for interfaces you implement.

// ============================================================================
// MAIN
// ============================================================================

func main() {
	fmt.Println("=== Interface Basics ===")
	fmt.Println()

	// --- Polymorphism ---
	fmt.Println("--- Polymorphism ---")
	notifiers := []Notifier{
		NewEmailNotifier("smtp.example.com", "alerts@example.com"),
		NewSlackNotifier("acme", "xoxb-token-xxx"),
		&SMSNotifier{Provider: "twilio", SenderNum: "+15550001234"},
	}

	errs := broadcastAlert(notifiers, "ops@example.com", "Disk usage critical", "Disk at 95%")
	for _, err := range errs {
		fmt.Println("error:", err)
	}
	fmt.Println()

	// --- fmt.Stringer ---
	fmt.Println("--- fmt.Stringer ---")
	p := PriorityCritical
	fmt.Printf("Priority: %v\n", p)           // calls p.String() automatically
	fmt.Printf("Priority: %s\n", p)           // also calls String()
	fmt.Printf("Priority raw: %d\n", int(p))  // bypass String(), print integer
	fmt.Println()

	// --- Custom error type ---
	fmt.Println("--- Custom error (error interface) ---")
	err := &DeliveryError{
		Channel:   "email",
		Recipient: "alice@example.com",
		Cause:     errors.New("connection refused"),
	}
	fmt.Println("Error:", err)

	var delivErr *DeliveryError
	if errors.As(err, &delivErr) {
		fmt.Printf("Channel that failed: %s\n", delivErr.Channel)
	}
	fmt.Println()

	// --- io.Writer ---
	fmt.Println("--- io.Writer (LogWriter) ---")
	lw := &LogWriter{}
	fmt.Fprintln(lw, "starting up")
	fmt.Fprintln(lw, "connected to redis")
	fmt.Fprintln(lw, "ready to serve")

	for i, line := range lw.Lines() {
		fmt.Printf("log[%d]: %s\n", i, line)
	}
	fmt.Println()

	// --- Accept interfaces: demonstrate with io.Writer ---
	fmt.Println("--- Writing to multiple sinks (io.Writer polymorphism) ---")
	// writeReport accepts io.Writer — works with anything: files, buffers, network
	report := strings.NewReader("system report: all checks passed\n")
	buf := &strings.Builder{}
	fmt.Fprint(buf, "")
	_ = report
	// In real code: io.Copy(someWriter, someReader)
	fmt.Println("(io.Copy, io.MultiWriter, bufio.Writer all work via io.Writer)")
}
