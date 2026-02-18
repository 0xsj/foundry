package dispatcher

import (
	"strings"
	"testing"
)

// ============================================================================
// ENUM TESTS
// ============================================================================

func TestNotificationTypeString(t *testing.T) {
	cases := []struct {
		typ  NotificationType
		want string
	}{
		{TypeEmail, "email"},
		{TypeSMS, "sms"},
		{TypeWebhook, "webhook"},
		{TypeSlack, "slack"},
		{NotificationType(99), "NotificationType(99)"},
	}
	for _, tc := range cases {
		t.Run(tc.want, func(t *testing.T) {
			if got := tc.typ.String(); got != tc.want {
				t.Errorf("String() = %q, want %q", got, tc.want)
			}
		})
	}
}

func TestDeliveryStatusString(t *testing.T) {
	cases := []struct {
		status DeliveryStatus
		want   string
	}{
		{StatusPending, "pending"},
		{StatusDelivered, "delivered"},
		{StatusFailed, "failed"},
		{DeliveryStatus(99), "DeliveryStatus(99)"},
	}
	for _, tc := range cases {
		t.Run(tc.want, func(t *testing.T) {
			if got := tc.status.String(); got != tc.want {
				t.Errorf("String() = %q, want %q", got, tc.want)
			}
		})
	}
}

// ============================================================================
// RETRY POLICY TESTS
// ============================================================================

func TestRetryPolicyShouldRetry(t *testing.T) {
	r := RetryPolicy{MaxAttempts: 3}

	if !r.ShouldRetry(0) {
		t.Error("ShouldRetry(0) should be true when MaxAttempts=3")
	}
	if !r.ShouldRetry(2) {
		t.Error("ShouldRetry(2) should be true when MaxAttempts=3")
	}
	if r.ShouldRetry(3) {
		t.Error("ShouldRetry(3) should be false when MaxAttempts=3")
	}
}

func TestRetryPolicyZeroMaxAttempts(t *testing.T) {
	r := RetryPolicy{MaxAttempts: 0}
	if r.ShouldRetry(0) {
		t.Error("ShouldRetry(0) should be false when MaxAttempts=0")
	}
}

// ============================================================================
// DELIVERY LOG TESTS
// ============================================================================

func TestDeliveryLogStartsEmpty(t *testing.T) {
	log := &DeliveryLog{}
	if got := log.Entries(); len(got) != 0 {
		t.Errorf("new DeliveryLog should have 0 entries, got %d", len(got))
	}
}

func TestDeliveryLogAppend(t *testing.T) {
	log := &DeliveryLog{}
	log.Append("first")
	log.Append("second")

	entries := log.Entries()
	if len(entries) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(entries))
	}
	if !strings.Contains(entries[0], "first") {
		t.Errorf("first entry should contain 'first', got %q", entries[0])
	}
	if !strings.Contains(entries[1], "second") {
		t.Errorf("second entry should contain 'second', got %q", entries[1])
	}
}

func TestDeliveryLogTimestamp(t *testing.T) {
	log := &DeliveryLog{}
	log.Append("test message")
	entries := log.Entries()
	// Timestamp format: [HH:MM:SS.mmm] — check for bracket prefix
	if !strings.HasPrefix(entries[0], "[") {
		t.Errorf("log entry should have timestamp prefix, got %q", entries[0])
	}
}

// ============================================================================
// NOTIFICATION CONSTRUCTOR TESTS
// ============================================================================

func TestNewNotificationValid(t *testing.T) {
	n, err := NewNotification("n1", TypeEmail, "alice@example.com", "Hello", "World")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if n.ID != "n1" {
		t.Errorf("ID = %q, want %q", n.ID, "n1")
	}
	if n.Type != TypeEmail {
		t.Errorf("Type = %v, want %v", n.Type, TypeEmail)
	}
	if n.Recipient != "alice@example.com" {
		t.Errorf("Recipient = %q, want %q", n.Recipient, "alice@example.com")
	}
}

func TestNewNotificationMissingID(t *testing.T) {
	_, err := NewNotification("", TypeEmail, "alice@example.com", "subj", "body")
	if err == nil {
		t.Error("expected error for empty ID")
	}
}

func TestNewNotificationMissingRecipient(t *testing.T) {
	_, err := NewNotification("n1", TypeEmail, "", "subj", "body")
	if err == nil {
		t.Error("expected error for empty recipient")
	}
}

// ============================================================================
// CHANNEL STRUCT TESTS
// ============================================================================

func TestChannelNames(t *testing.T) {
	cases := []struct {
		ch   Deliverer
		want string
	}{
		{NewEmailChannel("smtp.example.com", "noreply@example.com"), "email"},
		{NewSMSChannel("twilio", "+15550001234"), "sms"},
		{NewWebhookChannel("https://hooks.example.com", "secret"), "webhook"},
		{NewSlackChannel("acme", "alerts"), "slack"},
	}
	for _, tc := range cases {
		t.Run(tc.want, func(t *testing.T) {
			if got := tc.ch.Name(); got != tc.want {
				t.Errorf("Name() = %q, want %q", got, tc.want)
			}
		})
	}
}

func TestEmailChannelPromotedRetryFields(t *testing.T) {
	e := NewEmailChannel("smtp.example.com", "noreply@example.com")
	// Verify that RetryPolicy fields are promoted and accessible directly.
	if e.MaxAttempts != 3 {
		t.Errorf("MaxAttempts = %d, want 3", e.MaxAttempts)
	}
	if !e.ShouldRetry(0) {
		t.Error("ShouldRetry(0) should be true")
	}
}

func TestChannelDeliverAppendsToLog(t *testing.T) {
	channels := []Deliverer{
		NewEmailChannel("smtp.example.com", "noreply@example.com"),
		NewSMSChannel("twilio", "+15550001234"),
		NewWebhookChannel("https://hooks.example.com", "secret"),
		NewSlackChannel("acme", "alerts"),
	}

	n := Notification{ID: "x1", Recipient: "test@example.com"}

	for _, ch := range channels {
		_ = ch.Deliver(n)
		type logProvider interface{ Entries() []string }
		if lp, ok := ch.(logProvider); ok {
			if len(lp.Entries()) == 0 {
				t.Errorf("%s.Deliver did not append any log entries", ch.Name())
			}
		} else {
			t.Errorf("%T does not expose Entries() — is DeliveryLog embedded?", ch)
		}
	}
}

// ============================================================================
// DISPATCHER TESTS
// ============================================================================

func TestNewDispatcherDoesNotPanicOnRegister(t *testing.T) {
	d := NewDispatcher()
	// This panics if channels map is nil.
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))
}

func TestDispatchUnregisteredType(t *testing.T) {
	d := NewDispatcher()
	n := Notification{ID: "n1", Type: TypeEmail}
	report := d.Dispatch(n)

	if report.Status != StatusFailed {
		t.Errorf("expected StatusFailed, got %v", report.Status)
	}
	if report.NotificationID != "n1" {
		t.Errorf("NotificationID = %q, want %q", report.NotificationID, "n1")
	}
	if len(report.Log) == 0 {
		t.Error("expected log entry explaining no channel registered")
	}
}

func TestDispatchEventualSuccess(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))

	n := Notification{ID: "n1", Type: TypeEmail, Recipient: "alice@example.com"}
	report := d.Dispatch(n)

	if report.Status != StatusDelivered {
		t.Errorf("expected StatusDelivered, got %v (log: %v)", report.Status, report.Log)
	}
	if report.Channel != "email" {
		t.Errorf("Channel = %q, want %q", report.Channel, "email")
	}
	if report.Attempts < 2 {
		t.Errorf("expected at least 2 attempts (stub fails on first), got %d", report.Attempts)
	}
	if len(report.Log) == 0 {
		t.Error("report should include delivery log")
	}
}

func TestDispatchExhaustsRetries(t *testing.T) {
	d := NewDispatcher()
	ch := NewEmailChannel("smtp.example.com", "noreply@example.com")
	ch.MaxAttempts = 1 // 1 attempt allowed; stub fails on attempt 1 (odd)
	d.Register(TypeEmail, ch)

	n := Notification{ID: "n2", Type: TypeEmail, Recipient: "bob@example.com"}
	report := d.Dispatch(n)

	if report.Status != StatusFailed {
		t.Errorf("expected StatusFailed with MaxAttempts=1, got %v", report.Status)
	}
}

func TestDispatchAllChannelTypes(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))
	d.Register(TypeSMS, NewSMSChannel("twilio", "+15550001234"))
	d.Register(TypeWebhook, NewWebhookChannel("https://hooks.example.com", "secret"))
	d.Register(TypeSlack, NewSlackChannel("acme", "alerts"))

	types := []struct {
		nType NotificationType
		name  string
	}{
		{TypeEmail, "email"},
		{TypeSMS, "sms"},
		{TypeWebhook, "webhook"},
		{TypeSlack, "slack"},
	}

	for _, tc := range types {
		t.Run(tc.name, func(t *testing.T) {
			n := Notification{
				ID:        tc.name + "-1",
				Type:      tc.nType,
				Recipient: "user@example.com",
			}
			report := d.Dispatch(n)
			if report.Channel != tc.name {
				t.Errorf("Channel = %q, want %q", report.Channel, tc.name)
			}
		})
	}
}

func TestDispatchReportHasLogs(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeSlack, NewSlackChannel("acme", "alerts"))

	n := Notification{ID: "s1", Type: TypeSlack, Recipient: "alerts"}
	report := d.Dispatch(n)

	if len(report.Log) == 0 {
		t.Error("DeliveryReport.Log should contain entries from the delivery")
	}
}

// ============================================================================
// INTERFACE COMPLIANCE (compile-time checks)
// These assignments fail at compile time if the interface isn't satisfied.
// ============================================================================

var (
	_ Deliverer = (*EmailChannel)(nil)
	_ Deliverer = (*SMSChannel)(nil)
	_ Deliverer = (*WebhookChannel)(nil)
	_ Deliverer = (*SlackChannel)(nil)
)
