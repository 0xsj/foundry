// Run with: go run ./embedding/
package main

import (
	"fmt"
	"time"
)

// ============================================================================
// BASE TYPES TO EMBED
// These represent reusable behaviors that multiple channel types need.
// ============================================================================

// RetryConfig holds retry behavior. Any channel type can embed this
// to get retry logic without duplicating it.
type RetryConfig struct {
	MaxAttempts int
	BackoffMs   int
	Attempts    int
}

func (r *RetryConfig) ShouldRetry() bool {
	return r.Attempts < r.MaxAttempts
}

func (r *RetryConfig) RecordAttempt() {
	r.Attempts++
}

func (r *RetryConfig) Backoff() time.Duration {
	if r.BackoffMs == 0 {
		return 100 * time.Millisecond
	}
	// Exponential backoff: BackoffMs * 2^attempts
	delay := r.BackoffMs
	for i := 0; i < r.Attempts; i++ {
		delay *= 2
	}
	return time.Duration(delay) * time.Millisecond
}

func (r *RetryConfig) Reset() {
	r.Attempts = 0
}

// AuditLog provides structured logging for delivery attempts.
type AuditLog struct {
	Events []string
}

func (a *AuditLog) Log(msg string) {
	entry := fmt.Sprintf("[%s] %s", time.Now().Format(time.RFC3339), msg)
	a.Events = append(a.Events, entry)
	fmt.Println(entry)
}

func (a *AuditLog) EventCount() int {
	return len(a.Events)
}

// ============================================================================
// EMBEDDING BY VALUE
// The embedded struct is part of the outer struct. Zero value is valid.
// ============================================================================

// EmailChannel embeds RetryConfig and AuditLog by value.
// Both embedded types are initialized as zero values unless explicitly set.
type EmailChannel struct {
	RetryConfig       // embedded by value: RetryConfig{} is the zero value
	AuditLog          // embedded by value
	SMTPHost  string
	SMTPPort  int
	FromAddr  string
}

// NewEmailChannel creates an EmailChannel with sensible retry defaults.
func NewEmailChannel(host string, port int, from string) *EmailChannel {
	return &EmailChannel{
		RetryConfig: RetryConfig{MaxAttempts: 3, BackoffMs: 500},
		SMTPHost:    host,
		SMTPPort:    port,
		FromAddr:    from,
	}
}

// Send attempts delivery. Uses promoted methods from embedded types.
func (e *EmailChannel) Send(recipient, subject, body string) error {
	e.Log(fmt.Sprintf("attempting email to %s via %s:%d", recipient, e.SMTPHost, e.SMTPPort))
	e.RecordAttempt() // promoted from RetryConfig

	// Simulate occasional failure
	if e.Attempts%2 == 0 && e.Attempts > 0 {
		e.Log("delivery failed")
		return fmt.Errorf("SMTP connection refused")
	}

	e.Log(fmt.Sprintf("delivered to %s", recipient))
	return nil
}

func demoEmbeddingByValue() {
	fmt.Println("=== Embedding by Value ===")

	email := NewEmailChannel("smtp.example.com", 587, "noreply@example.com")

	// Promoted field access — same as email.RetryConfig.MaxAttempts
	fmt.Printf("MaxAttempts: %d\n", email.MaxAttempts)
	fmt.Printf("BackoffMs: %d\n", email.BackoffMs)

	// Promoted method calls
	fmt.Printf("ShouldRetry: %v\n", email.ShouldRetry())   // RetryConfig.ShouldRetry
	email.Log("channel initialized")                         // AuditLog.Log
	fmt.Printf("EventCount: %d\n\n", email.EventCount())    // AuditLog.EventCount
}

// ============================================================================
// EMBEDDING BY POINTER
// The embedded pointer is nil by default — must be initialized before use.
// ============================================================================

// WebhookChannel embeds *RetryConfig (pointer).
// If RetryConfig is not set, calling any promoted method panics.
type WebhookChannel struct {
	*RetryConfig      // embedded pointer — nil by default!
	AuditLog          // embedded by value — safe to use immediately
	Endpoint string
	Secret   string
}

func demoEmbeddingByPointer() {
	fmt.Println("=== Embedding by Pointer ===")

	// This webhook channel has no RetryConfig — pointer is nil.
	broken := &WebhookChannel{
		Endpoint: "https://api.example.com/webhook",
	}
	fmt.Println("WebhookChannel with nil *RetryConfig:")
	fmt.Println("  Logging works (AuditLog embedded by value):")
	broken.Log("channel created") // AuditLog is by value — OK

	// Checking nil before calling promoted methods from *RetryConfig
	if broken.RetryConfig != nil {
		fmt.Println("  ShouldRetry:", broken.ShouldRetry())
	} else {
		fmt.Println("  Skipping ShouldRetry — RetryConfig is nil")
	}

	// Properly initialized version
	working := &WebhookChannel{
		RetryConfig: &RetryConfig{MaxAttempts: 5, BackoffMs: 200},
		Endpoint:    "https://api.example.com/webhook",
	}
	fmt.Printf("\nWebhookChannel with initialized *RetryConfig:\n")
	fmt.Printf("  ShouldRetry: %v\n\n", working.ShouldRetry())
}

// ============================================================================
// METHOD SHADOWING ("OVERRIDING")
// The outer struct can define a method with the same name as an embedded method.
// The outer method wins. The embedded method is still accessible explicitly.
// ============================================================================

// AlertChannel overrides Log from AuditLog to add alert-specific formatting.
type AlertChannel struct {
	AuditLog          // embedded
	RetryConfig
	Severity string
	Endpoint string
}

// Log shadows AuditLog.Log. When you call alertChannel.Log(...),
// this method runs — not AuditLog.Log.
func (a *AlertChannel) Log(msg string) {
	// Add alert-specific context before delegating to the embedded logger.
	formatted := fmt.Sprintf("[ALERT][severity=%s] %s", a.Severity, msg)
	a.AuditLog.Log(formatted) // explicit call to the embedded (shadowed) method
}

func demoMethodShadowing() {
	fmt.Println("=== Method Shadowing ===")

	alert := &AlertChannel{
		RetryConfig: RetryConfig{MaxAttempts: 1, BackoffMs: 0},
		Severity:    "critical",
		Endpoint:    "https://pagerduty.example.com",
	}

	// Calls AlertChannel.Log — the outer method (shadowing AuditLog.Log)
	alert.Log("disk at 95% capacity")

	// Calls AuditLog.Log directly — bypass the shadow
	alert.AuditLog.Log("direct audit entry")

	fmt.Printf("Total events logged: %d\n\n", alert.EventCount())
}

// ============================================================================
// EMBEDDING AND INTERFACES
// Embedding promotes methods into the method set of the outer type.
// If the embedded type satisfies an interface, the outer type can too
// (as long as it doesn't shadow the required methods).
// ============================================================================

type Retrier interface {
	ShouldRetry() bool
	RecordAttempt()
	Backoff() time.Duration
}

// EmailChannel embeds RetryConfig, which provides ShouldRetry, RecordAttempt, Backoff.
// Therefore *EmailChannel also satisfies Retrier — for free.
func useRetrier(r Retrier) {
	for r.ShouldRetry() {
		fmt.Printf("  attempt %v, backoff=%v\n", "next", r.Backoff())
		r.RecordAttempt()
	}
	fmt.Println("  done retrying")
}

func demoEmbeddingAndInterfaces() {
	fmt.Println("=== Embedding and Interfaces ===")

	email := NewEmailChannel("smtp.example.com", 587, "noreply@example.com")
	email.RetryConfig.MaxAttempts = 2 // limit for demo

	var r Retrier = email // *EmailChannel satisfies Retrier via promotion
	useRetrier(r)
	fmt.Println()
}

// ============================================================================
// AMBIGUOUS SELECTORS
// Two embedded types with the same field/method name cause a compile error
// unless you use explicit qualification.
// ============================================================================

type Logger1 struct{ Prefix string }
func (l *Logger1) Log(msg string) { fmt.Printf("[L1][%s] %s\n", l.Prefix, msg) }

type Logger2 struct{ Prefix string }
func (l *Logger2) Log(msg string) { fmt.Printf("[L2][%s] %s\n", l.Prefix, msg) }

// DualLogger embeds both Logger1 and Logger2 — both have Log and Prefix.
type DualLogger struct {
	Logger1
	Logger2
}

func demoAmbiguousSelectors() {
	fmt.Println("=== Ambiguous Selectors ===")

	dl := DualLogger{
		Logger1: Logger1{Prefix: "primary"},
		Logger2: Logger2{Prefix: "backup"},
	}

	// dl.Log("msg")       // COMPILE ERROR: ambiguous selector dl.Log
	// dl.Prefix           // COMPILE ERROR: ambiguous selector dl.Prefix
	dl.Logger1.Log("explicit Logger1 call")
	dl.Logger2.Log("explicit Logger2 call")
	fmt.Printf("Logger1 prefix: %s\n", dl.Logger1.Prefix)
	fmt.Printf("Logger2 prefix: %s\n\n", dl.Logger2.Prefix)
}

func main() {
	demoEmbeddingByValue()
	demoEmbeddingByPointer()
	demoMethodShadowing()
	demoEmbeddingAndInterfaces()
	demoAmbiguousSelectors()
}
