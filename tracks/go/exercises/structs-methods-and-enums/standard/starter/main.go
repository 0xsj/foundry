package dispatcher

import "fmt"

// ============================================================================
// ENUMS
// Define NotificationType and DeliveryStatus as typed int constants using iota.
// Add String() methods so they print human-readable values.
// ============================================================================

// NotificationType identifies which delivery channel to use.
type NotificationType int

const (
	// TODO: Define TypeEmail, TypeSMS, TypeWebhook, TypeSlack using iota
)

// TODO: func (t NotificationType) String() string

// DeliveryStatus represents the final outcome of a dispatch attempt.
type DeliveryStatus int

const (
	// TODO: Define StatusPending, StatusDelivered, StatusFailed using iota
)

// TODO: func (s DeliveryStatus) String() string

// ============================================================================
// RETRY POLICY
// Holds retry configuration. Embedded into each channel struct.
// ============================================================================

// RetryPolicy configures how many times a delivery should be retried.
type RetryPolicy struct {
	MaxAttempts int
	BackoffMs   int
}

// ShouldRetry returns true if another delivery attempt is allowed.
// TODO: Implement
func (r RetryPolicy) ShouldRetry(attempts int) bool {
	return false
}

// ============================================================================
// DELIVERY LOG
// Records timestamped delivery attempt log entries.
// Embedded into each channel struct.
// ============================================================================

// DeliveryLog records the history of delivery attempts for a single dispatch.
type DeliveryLog struct {
	// TODO: Add a field to hold log entries
}

// Append adds a new entry to the log.
// Think about: does this need a pointer receiver?
// TODO: Implement
func (d *DeliveryLog) Append(msg string) {
}

// Entries returns all recorded log entries.
// TODO: Implement
func (d DeliveryLog) Entries() []string {
	return nil
}

// ============================================================================
// NOTIFICATION
// The message being dispatched. Validated at construction time.
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
// Returns an error if ID or Recipient are empty.
// TODO: Implement
func NewNotification(id string, nType NotificationType, recipient, subject, body string) (*Notification, error) {
	return nil, fmt.Errorf("not implemented")
}

// ============================================================================
// DELIVERER INTERFACE
// All channel types implement this interface.
// ============================================================================

// Deliverer is implemented by all notification channels.
type Deliverer interface {
	// Deliver attempts to send the notification.
	// Returns nil on success, an error on failure.
	Deliver(n Notification) error

	// Name returns the channel's display name.
	Name() string
}

// ============================================================================
// CHANNEL STRUCTS
// Each channel embeds RetryPolicy and DeliveryLog.
// Deliver stubs: succeed when attempts is even, fail when odd.
// ============================================================================

// EmailChannel delivers notifications via SMTP.
type EmailChannel struct {
	// TODO: Embed RetryPolicy and DeliveryLog
	SMTPHost string
	FromAddr string
}

// NewEmailChannel creates an EmailChannel with the given SMTP config.
func NewEmailChannel(host, from string) *EmailChannel {
	return &EmailChannel{
		// TODO: Initialize embedded RetryPolicy with MaxAttempts: 3, BackoffMs: 500
		SMTPHost: host,
		FromAddr: from,
	}
}

// Deliver attempts SMTP delivery.
// Stub: fails on first attempt (attempts==0 is odd), succeeds on retry (attempts==1 is even).
// TODO: Implement using pointer receiver. Append log entries.
func (e *EmailChannel) Deliver(n Notification) error {
	return fmt.Errorf("not implemented")
}

// TODO: func (e *EmailChannel) Name() string

// SMSChannel delivers notifications via SMS provider.
type SMSChannel struct {
	// TODO: Embed RetryPolicy and DeliveryLog
	Provider   string
	SenderNum  string
}

// NewSMSChannel creates an SMSChannel.
func NewSMSChannel(provider, senderNum string) *SMSChannel {
	return &SMSChannel{
		// TODO: Initialize embedded RetryPolicy with MaxAttempts: 2, BackoffMs: 200
		Provider:  provider,
		SenderNum: senderNum,
	}
}

// TODO: Deliver — same stub pattern
// TODO: Name

// WebhookChannel delivers notifications via HTTP POST.
type WebhookChannel struct {
	// TODO: Embed RetryPolicy and DeliveryLog
	Endpoint string
	Secret   string
}

// NewWebhookChannel creates a WebhookChannel.
func NewWebhookChannel(endpoint, secret string) *WebhookChannel {
	return &WebhookChannel{
		// TODO: Initialize embedded RetryPolicy with MaxAttempts: 5, BackoffMs: 100
		Endpoint: endpoint,
		Secret:   secret,
	}
}

// TODO: Deliver — same stub pattern
// TODO: Name

// SlackChannel delivers notifications to a Slack workspace.
type SlackChannel struct {
	// TODO: Embed RetryPolicy and DeliveryLog
	Workspace string
	ChannelID string
}

// NewSlackChannel creates a SlackChannel.
func NewSlackChannel(workspace, channelID string) *SlackChannel {
	return &SlackChannel{
		// TODO: Initialize embedded RetryPolicy with MaxAttempts: 2, BackoffMs: 300
		Workspace: workspace,
		ChannelID: channelID,
	}
}

// TODO: Deliver — same stub pattern
// TODO: Name

// ============================================================================
// DELIVERY REPORT
// Returned by Dispatch — the final outcome of one notification delivery.
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
// Routes notifications to the right channel and drives the retry loop.
// ============================================================================

// Dispatcher routes notifications to registered delivery channels.
type Dispatcher struct {
	// TODO: Add a map field: channels map[NotificationType]Deliverer
	// Remember: the zero value of a map is nil — initialize with make().
}

// NewDispatcher creates a Dispatcher with an initialized channel map.
// TODO: Implement
func NewDispatcher() *Dispatcher {
	return &Dispatcher{}
}

// Register adds a delivery channel for the given notification type.
// TODO: Implement
func (d *Dispatcher) Register(t NotificationType, ch Deliverer) {
}

// Dispatch delivers a notification through the registered channel.
// If no channel is registered for the notification type, returns a failed report.
// Drives the retry loop: attempt delivery, retry on failure per the channel's RetryPolicy.
// TODO: Implement
func (d *Dispatcher) Dispatch(n Notification) DeliveryReport {
	return DeliveryReport{}
}
