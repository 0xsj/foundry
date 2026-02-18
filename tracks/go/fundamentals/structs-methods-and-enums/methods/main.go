// Run with: go run ./methods/
package main

import (
	"fmt"
	"strings"
	"time"
)

// ============================================================================
// TYPE DEFINITION
// The type we'll add methods to.
// ============================================================================

// Notification represents a message to be sent through some channel.
type Notification struct {
	ID        string
	Recipient string
	Subject   string
	Body      string
	CreatedAt time.Time
	Retries   int
	MaxRetries int
	Sent      bool
	SentAt    *time.Time // pointer — nil means "not sent yet"
}

// ============================================================================
// CONSTRUCTOR PATTERN
// NewXxx functions validate and initialize structs.
// Return *Notification because the struct will be mutated via pointer receivers.
// ============================================================================

func NewNotification(id, recipient, subject, body string) (*Notification, error) {
	if id == "" {
		return nil, fmt.Errorf("notification id is required")
	}
	if recipient == "" {
		return nil, fmt.Errorf("recipient is required")
	}
	if !strings.Contains(recipient, "@") {
		return nil, fmt.Errorf("recipient %q does not look like an email address", recipient)
	}

	return &Notification{
		ID:         id,
		Recipient:  recipient,
		Subject:    subject,
		Body:       body,
		CreatedAt:  time.Now(),
		MaxRetries: 3,
	}, nil
}

// ============================================================================
// VALUE RECEIVERS
// The method receives a copy of the struct. Cannot modify the original.
// Use for: read-only access, small structs, when you want copy semantics.
// ============================================================================

// String implements fmt.Stringer. Value receiver — only reads fields.
func (n Notification) String() string {
	status := "pending"
	if n.Sent {
		status = "sent"
	} else if n.Retries > 0 {
		status = fmt.Sprintf("retried(%d)", n.Retries)
	}
	return fmt.Sprintf("[%s] to=%s status=%s subject=%q",
		n.ID, n.Recipient, status, n.Subject)
}

// IsRetryable returns whether the notification can be attempted again.
// Value receiver — computes from current state, doesn't modify anything.
func (n Notification) IsRetryable() bool {
	return !n.Sent && n.Retries < n.MaxRetries
}

// Summary returns a one-line description for dashboard display.
func (n Notification) Summary() string {
	bodyPreview := n.Body
	if len(bodyPreview) > 50 {
		bodyPreview = bodyPreview[:47] + "..."
	}
	return fmt.Sprintf("%s | %s | %s", n.ID, n.Recipient, bodyPreview)
}

// ============================================================================
// POINTER RECEIVERS
// The method receives a pointer — it can modify the original struct.
// Use for: mutation, large structs (avoid copying), consistent method sets.
// ============================================================================

// MarkSent records a successful delivery.
// Pointer receiver — modifies Sent and SentAt on the original.
func (n *Notification) MarkSent() {
	now := time.Now()
	n.Sent = true
	n.SentAt = &now
	n.Retries = 0 // reset retry count on success
}

// IncrementRetry records a failed delivery attempt.
func (n *Notification) IncrementRetry() {
	n.Retries++
}

// SetMaxRetries configures the retry limit.
// If you forget to use a pointer receiver here but use pointer receivers
// everywhere else, you'll hit interface compliance issues.
func (n *Notification) SetMaxRetries(max int) {
	n.MaxRetries = max
}

// ============================================================================
// DEMONSTRATING VALUE RECEIVER PITFALL
// A value receiver gets a copy. Mutations inside the method don't affect
// the original — this is a common source of bugs.
// ============================================================================

// wrongMarkSent uses a VALUE receiver — it modifies a copy, not the original.
// This is intentionally wrong to show the pitfall.
func (n Notification) wrongMarkSent() {
	n.Sent = true // modifies the copy, original is unchanged
}

func demoValueReceiverPitfall() {
	fmt.Println("=== Value Receiver Pitfall ===")

	n, _ := NewNotification("n1", "alice@example.com", "Test", "body")
	fmt.Printf("Before wrongMarkSent: Sent=%v\n", n.Sent)
	n.wrongMarkSent()
	fmt.Printf("After wrongMarkSent:  Sent=%v  (unchanged!)\n", n.Sent)

	n.MarkSent() // correct — pointer receiver
	fmt.Printf("After MarkSent:       Sent=%v  (changed)\n\n", n.Sent)
}

// ============================================================================
// METHOD SETS AND AUTO-ADDRESS-TAKING
// Go automatically takes the address of an addressable value when calling
// a pointer-receiver method. But map elements are not addressable.
// ============================================================================

func demoMethodSets() {
	fmt.Println("=== Method Sets ===")

	// Addressable variable — Go auto-takes address for pointer receivers.
	n, _ := NewNotification("n2", "bob@example.com", "Alert", "system alert")
	n.MarkSent() // actually (&n).MarkSent() — Go handles this for addressable vars
	fmt.Println("Auto-addressed:", n)

	// Map of values — NOT addressable. Cannot call pointer-receiver methods.
	byValueMap := map[string]Notification{
		"n3": {ID: "n3", Recipient: "carol@example.com"},
	}
	_ = byValueMap["n3"].String() // OK — String() has value receiver
	// byValueMap["n3"].MarkSent() // COMPILE ERROR: cannot take address of map value
	fmt.Println("Map of values: direct pointer-receiver call is illegal (compile error)")

	// Map of pointers — all good.
	byPtrMap := map[string]*Notification{
		"n4": {ID: "n4", Recipient: "dave@example.com"},
	}
	byPtrMap["n4"].MarkSent() // OK — byPtrMap["n4"] is already *Notification
	fmt.Printf("Map of pointers: Sent=%v\n\n", byPtrMap["n4"].Sent)
}

// ============================================================================
// INTERFACE SATISFACTION
// A *Notification satisfies interfaces that require pointer-receiver methods.
// A Notification value does NOT — even if the method is auto-addressable.
// ============================================================================

type Sender interface {
	Send() error
	String() string
}

// Send delivers the notification (stub implementation).
func (n *Notification) Send() error {
	if n.Recipient == "" {
		return fmt.Errorf("no recipient")
	}
	n.MarkSent()
	return nil
}

func demoInterfaceSatisfaction() {
	fmt.Println("=== Interface Satisfaction ===")

	// *Notification has methods: String (from T) + MarkSent, Send, SetMaxRetries (from *T)
	// *Notification satisfies Sender.
	n, _ := NewNotification("n5", "eve@example.com", "Invoice", "see attachment")
	var s Sender = n         // OK: n is *Notification
	_ = s.Send()
	fmt.Printf("Sent via interface: %v\n", n.Sent)

	// Notification (value) would NOT satisfy Sender because Sender.Send requires *Notification.
	// var s2 Sender = *n  // COMPILE ERROR: Notification does not implement Sender
	//                     // (Send method has pointer receiver)
	fmt.Println("Notification value cannot satisfy Sender (pointer receiver on Send)\n")
}

// ============================================================================
// METHODS ON NON-STRUCT TYPES
// Any named type in the package can have methods, not just structs.
// ============================================================================

type StatusCode int

const (
	CodePending StatusCode = iota
	CodeSent
	CodeFailed
)

func (s StatusCode) String() string {
	switch s {
	case CodePending:
		return "PENDING"
	case CodeSent:
		return "SENT"
	case CodeFailed:
		return "FAILED"
	default:
		return fmt.Sprintf("StatusCode(%d)", int(s))
	}
}

func (s StatusCode) IsFinal() bool {
	return s == CodeSent || s == CodeFailed
}

func demoNonStructMethods() {
	fmt.Println("=== Methods on Non-Struct Types ===")

	code := CodeFailed
	fmt.Printf("code=%v isFinal=%v\n\n", code, code.IsFinal())
}

func main() {
	demoValueReceiverPitfall()
	demoMethodSets()
	demoInterfaceSatisfaction()
	demoNonStructMethods()

	// Constructor validation
	fmt.Println("=== Constructor Validation ===")
	_, err := NewNotification("", "alice@example.com", "Test", "body")
	fmt.Println("Missing ID:", err)

	_, err = NewNotification("n1", "notanemail", "Test", "body")
	fmt.Println("Bad email:", err)

	n, err := NewNotification("n1", "alice@example.com", "Test", "body")
	fmt.Println("Valid:", err, "→", n)
}
