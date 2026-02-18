// Package dispatcher implements a multi-channel notification dispatcher.
// It demonstrates: struct definitions, pointer vs value receivers,
// struct embedding for shared behavior, iota enums with String() methods,
// interface satisfaction, and constructor patterns with validation.
package dispatcher

import (
	"fmt"
	"time"
)

// ============================================================================
// ENUMS
// Typed int constants with iota + String() methods.
// Type safety: can't accidentally pass a raw int where NotificationType is needed.
// ============================================================================

// NotificationType identifies which delivery channel to use.
type NotificationType int

const (
	TypeEmail   NotificationType = iota // 0
	TypeSMS                             // 1
	TypeWebhook                         // 2
	TypeSlack                           // 3
)

// String implements fmt.Stringer so Printf/Println print names, not numbers.
func (t NotificationType) String() string {
	switch t {
	case TypeEmail:
		return "email"
	case TypeSMS:
		return "sms"
	case TypeWebhook:
		return "webhook"
	case TypeSlack:
		return "slack"
	default:
		return fmt.Sprintf("NotificationType(%d)", int(t))
	}
}

// DeliveryStatus represents the final outcome of a dispatch attempt.
type DeliveryStatus int

const (
	StatusPending   DeliveryStatus = iota // 0
	StatusDelivered                       // 1
	StatusFailed                          // 2
)

func (s DeliveryStatus) String() string {
	switch s {
	case StatusPending:
		return "pending"
	case StatusDelivered:
		return "delivered"
	case StatusFailed:
		return "failed"
	default:
		return fmt.Sprintf("DeliveryStatus(%d)", int(s))
	}
}

// ============================================================================
// RETRY POLICY
// Shared retry configuration. Embedded into each channel struct by value.
// Value receiver on ShouldRetry — it only reads, doesn't mutate.
// ============================================================================

// RetryPolicy configures how many times a delivery should be retried.
type RetryPolicy struct {
	MaxAttempts int
	BackoffMs   int
}

// ShouldRetry returns true if another delivery attempt is allowed.
// Value receiver: only reads MaxAttempts, returns a bool.
func (r RetryPolicy) ShouldRetry(attempts int) bool {
	return attempts < r.MaxAttempts
}

// ============================================================================
// DELIVERY LOG
// Collects timestamped log entries during a delivery sequence.
// Append uses a pointer receiver — it modifies the slice in place.
// ============================================================================

// DeliveryLog records the history of delivery attempts.
type DeliveryLog struct {
	entries []string
}

// Append adds a timestamped entry. Pointer receiver — modifies the real slice.
// If this were a value receiver, the append would be lost.
func (d *DeliveryLog) Append(msg string) {
	ts := time.Now().Format("15:04:05.000")
	d.entries = append(d.entries, fmt.Sprintf("[%s] %s", ts, msg))
}

// Entries returns all log entries. Value receiver — no mutation, returns copy of slice header.
func (d DeliveryLog) Entries() []string {
	return d.entries
}

// ============================================================================
// NOTIFICATION
// Pure data struct. Validated at construction time — invalid notifications
// can't be created without going through NewNotification.
// ============================================================================

// Notification is the message to be delivered.
type Notification struct {
	ID        string
	Type      NotificationType
	Recipient string
	Subject   string
	Body      string
}

// NewNotification constructs and validates a Notification.
// Returns a pointer — notifications are typically passed around by pointer
// after creation to avoid copying.
func NewNotification(id string, nType NotificationType, recipient, subject, body string) (*Notification, error) {
	if id == "" {
		return nil, fmt.Errorf("notification ID is required")
	}
	if recipient == "" {
		return nil, fmt.Errorf("recipient is required")
	}
	return &Notification{
		ID:        id,
		Type:      nType,
		Recipient: recipient,
		Subject:   subject,
		Body:      body,
	}, nil
}

// ============================================================================
// DELIVERER INTERFACE
// Implemented by all four channel types.
// Each channel type uses pointer receivers for Deliver and Name,
// so *ChannelType satisfies Deliverer (not ChannelType).
// ============================================================================

// Deliverer is implemented by any type that can send a notification.
type Deliverer interface {
	Deliver(n Notification) error
	Name() string
}

// ============================================================================
// CHANNEL STRUCTS
// Each embeds RetryPolicy (by value) and DeliveryLog (by value).
// Promoted fields: e.g. emailChannel.MaxAttempts, emailChannel.ShouldRetry(n)
// Promoted methods: e.g. emailChannel.Append("msg"), emailChannel.Entries()
//
// All Deliver methods use pointer receivers — they append to DeliveryLog
// (mutation) and need to see the real embedded struct, not a copy.
//
// Stub delivery behavior:
//   - Attempt index 0, 2, 4 (even) → fail
//   - Attempt index 1, 3, 5 (odd) → succeed
// This ensures each dispatch needs at least one retry to succeed.
// ============================================================================

// EmailChannel delivers notifications via SMTP.
type EmailChannel struct {
	RetryPolicy // embedded by value — ShouldRetry, MaxAttempts promoted
	DeliveryLog // embedded by value — Append, Entries promoted
	SMTPHost    string
	FromAddr    string
	attempts    int // internal attempt counter for stub behavior
}

// NewEmailChannel creates an EmailChannel with standard SMTP retry settings.
func NewEmailChannel(host, from string) *EmailChannel {
	return &EmailChannel{
		RetryPolicy: RetryPolicy{MaxAttempts: 3, BackoffMs: 500},
		SMTPHost:    host,
		FromAddr:    from,
	}
}

// Deliver attempts SMTP delivery.
// Pointer receiver: appends to DeliveryLog (mutation) and increments attempts.
func (e *EmailChannel) Deliver(n Notification) error {
	e.attempts++
	e.Append(fmt.Sprintf("email attempt %d to %s via %s", e.attempts, n.Recipient, e.SMTPHost))

	if e.attempts%2 == 1 { // odd attempt (1st, 3rd, ...) → fail
		e.Append("SMTP: connection refused")
		return fmt.Errorf("SMTP connection refused (attempt %d)", e.attempts)
	}

	e.Append(fmt.Sprintf("email delivered to %s", n.Recipient))
	return nil
}

func (e *EmailChannel) Name() string { return "email" }

// SMSChannel delivers notifications via an SMS provider.
type SMSChannel struct {
	RetryPolicy
	DeliveryLog
	Provider  string
	SenderNum string
	attempts  int
}

func NewSMSChannel(provider, senderNum string) *SMSChannel {
	return &SMSChannel{
		RetryPolicy: RetryPolicy{MaxAttempts: 2, BackoffMs: 200},
		Provider:    provider,
		SenderNum:   senderNum,
	}
}

func (s *SMSChannel) Deliver(n Notification) error {
	s.attempts++
	s.Append(fmt.Sprintf("sms attempt %d to %s via %s", s.attempts, n.Recipient, s.Provider))

	if s.attempts%2 == 1 {
		s.Append("SMS: gateway timeout")
		return fmt.Errorf("SMS gateway timeout (attempt %d)", s.attempts)
	}

	s.Append(fmt.Sprintf("sms delivered to %s", n.Recipient))
	return nil
}

func (s *SMSChannel) Name() string { return "sms" }

// WebhookChannel delivers notifications via HTTP POST.
type WebhookChannel struct {
	RetryPolicy
	DeliveryLog
	Endpoint string
	Secret   string
	attempts int
}

func NewWebhookChannel(endpoint, secret string) *WebhookChannel {
	return &WebhookChannel{
		RetryPolicy: RetryPolicy{MaxAttempts: 5, BackoffMs: 100},
		Endpoint:    endpoint,
		Secret:      secret,
	}
}

func (w *WebhookChannel) Deliver(n Notification) error {
	w.attempts++
	w.Append(fmt.Sprintf("webhook attempt %d to %s", w.attempts, w.Endpoint))

	if w.attempts%2 == 1 {
		w.Append("webhook: upstream 503")
		return fmt.Errorf("upstream returned 503 (attempt %d)", w.attempts)
	}

	w.Append(fmt.Sprintf("webhook delivered: %s", w.Endpoint))
	return nil
}

func (w *WebhookChannel) Name() string { return "webhook" }

// SlackChannel delivers notifications to a Slack workspace.
type SlackChannel struct {
	RetryPolicy
	DeliveryLog
	Workspace string
	ChannelID string
	attempts  int
}

func NewSlackChannel(workspace, channelID string) *SlackChannel {
	return &SlackChannel{
		RetryPolicy: RetryPolicy{MaxAttempts: 2, BackoffMs: 300},
		Workspace:   workspace,
		ChannelID:   channelID,
	}
}

func (s *SlackChannel) Deliver(n Notification) error {
	s.attempts++
	s.Append(fmt.Sprintf("slack attempt %d to #%s (%s)", s.attempts, s.ChannelID, s.Workspace))

	if s.attempts%2 == 1 {
		s.Append("Slack: rate limited")
		return fmt.Errorf("Slack rate limited (attempt %d)", s.attempts)
	}

	s.Append(fmt.Sprintf("slack message posted to #%s", s.ChannelID))
	return nil
}

func (s *SlackChannel) Name() string { return "slack" }

// ============================================================================
// DELIVERY REPORT
// The final result of dispatching one notification.
// Returned by value — it's a pure data snapshot, not a live object.
// ============================================================================

// DeliveryReport summarizes the result of dispatching a single notification.
type DeliveryReport struct {
	NotificationID string
	Channel        string
	Status         DeliveryStatus
	Attempts       int
	Log            []string
}

// ============================================================================
// DISPATCHER
// Routes notifications to the correct channel and drives the retry loop.
// The channel map must be initialized with make() — nil map panics on write.
// ============================================================================

// Dispatcher routes notifications to registered delivery channels.
type Dispatcher struct {
	channels map[NotificationType]Deliverer
}

// NewDispatcher creates a Dispatcher ready to accept channel registrations.
// The channels map is initialized here — writing to a nil map would panic.
func NewDispatcher() *Dispatcher {
	return &Dispatcher{
		channels: make(map[NotificationType]Deliverer),
	}
}

// Register associates a delivery channel with a notification type.
// Calling Register replaces any previously registered channel for that type.
func (d *Dispatcher) Register(t NotificationType, ch Deliverer) {
	d.channels[t] = ch
}

// Dispatch delivers a notification through the registered channel.
// If no channel is registered, returns a failed report without panicking.
// Drives the retry loop per the channel's embedded RetryPolicy.
func (d *Dispatcher) Dispatch(n Notification) DeliveryReport {
	ch, ok := d.channels[n.Type]
	if !ok {
		return DeliveryReport{
			NotificationID: n.ID,
			Status:         StatusFailed,
			Log:            []string{fmt.Sprintf("no channel registered for type %v", n.Type)},
		}
	}

	var lastErr error
	attempts := 0

	for {
		err := ch.Deliver(n)
		attempts++

		if err == nil {
			// Successful delivery
			return DeliveryReport{
				NotificationID: n.ID,
				Channel:        ch.Name(),
				Status:         StatusDelivered,
				Attempts:       attempts,
				Log:            getLog(ch),
			}
		}

		lastErr = err

		// Check if we should retry (attempts is the count of tries so far)
		if !ch.(interface{ ShouldRetry(int) bool }).ShouldRetry(attempts) {
			break
		}
	}

	return DeliveryReport{
		NotificationID: n.ID,
		Channel:        ch.Name(),
		Status:         StatusFailed,
		Attempts:       attempts,
		Log:            append(getLog(ch), fmt.Sprintf("final error: %v", lastErr)),
	}
}

// getLog extracts delivery log entries from a Deliverer using a type assertion.
// Channels that embed DeliveryLog expose Entries() — we check for that interface.
func getLog(ch Deliverer) []string {
	type logProvider interface {
		Entries() []string
	}
	if lp, ok := ch.(logProvider); ok {
		return lp.Entries()
	}
	return nil
}
