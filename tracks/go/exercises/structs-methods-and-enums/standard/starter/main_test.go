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
	if r.ShouldRetry(10) {
		t.Error("ShouldRetry(10) should be false when MaxAttempts=3")
	}
}

// ============================================================================
// DELIVERY LOG TESTS
// ============================================================================

func TestDeliveryLogAppendAndEntries(t *testing.T) {
	log := &DeliveryLog{}

	if entries := log.Entries(); len(entries) != 0 {
		t.Errorf("empty log should have 0 entries, got %d", len(entries))
	}

	log.Append("attempt 1")
	log.Append("attempt 2")

	entries := log.Entries()
	if len(entries) != 2 {
		t.Errorf("expected 2 entries, got %d", len(entries))
	}
	if !strings.Contains(entries[0], "attempt 1") {
		t.Errorf("first entry should contain 'attempt 1', got %q", entries[0])
	}
	if !strings.Contains(entries[1], "attempt 2") {
		t.Errorf("second entry should contain 'attempt 2', got %q", entries[1])
	}
}

// ============================================================================
// NOTIFICATION CONSTRUCTOR TESTS
// ============================================================================

func TestNewNotificationValidation(t *testing.T) {
	t.Run("valid notification", func(t *testing.T) {
		n, err := NewNotification("n1", TypeEmail, "alice@example.com", "Hello", "body")
		if err != nil {
			t.Errorf("expected no error, got %v", err)
		}
		if n == nil {
			t.Fatal("expected non-nil notification")
		}
		if n.ID != "n1" {
			t.Errorf("ID = %q, want %q", n.ID, "n1")
		}
	})

	t.Run("missing ID", func(t *testing.T) {
		_, err := NewNotification("", TypeEmail, "alice@example.com", "Hello", "body")
		if err == nil {
			t.Error("expected error for empty ID, got nil")
		}
	})

	t.Run("missing recipient", func(t *testing.T) {
		_, err := NewNotification("n1", TypeEmail, "", "Hello", "body")
		if err == nil {
			t.Error("expected error for empty recipient, got nil")
		}
	})
}

// ============================================================================
// CHANNEL STRUCT TESTS
// ============================================================================

func TestEmailChannelName(t *testing.T) {
	e := NewEmailChannel("smtp.example.com", "noreply@example.com")
	if e.Name() != "email" {
		t.Errorf("Name() = %q, want %q", e.Name(), "email")
	}
}

func TestSMSChannelName(t *testing.T) {
	s := NewSMSChannel("twilio", "+15550001234")
	if s.Name() != "sms" {
		t.Errorf("Name() = %q, want %q", s.Name(), "sms")
	}
}

func TestWebhookChannelName(t *testing.T) {
	w := NewWebhookChannel("https://hooks.example.com", "secret")
	if w.Name() != "webhook" {
		t.Errorf("Name() = %q, want %q", w.Name(), "webhook")
	}
}

func TestSlackChannelName(t *testing.T) {
	s := NewSlackChannel("acme", "alerts")
	if s.Name() != "slack" {
		t.Errorf("Name() = %q, want %q", s.Name(), "slack")
	}
}

func TestEmailChannelEmbeddedRetryPolicy(t *testing.T) {
	// Verify the RetryPolicy is embedded and accessible via promoted fields.
	e := NewEmailChannel("smtp.example.com", "noreply@example.com")
	if e.MaxAttempts != 3 {
		t.Errorf("MaxAttempts = %d, want 3", e.MaxAttempts)
	}
	if !e.ShouldRetry(0) {
		t.Error("ShouldRetry(0) should be true for a fresh EmailChannel")
	}
	if e.ShouldRetry(3) {
		t.Error("ShouldRetry(3) should be false when MaxAttempts=3")
	}
}

func TestEmailChannelDeliverLogsAttempts(t *testing.T) {
	e := NewEmailChannel("smtp.example.com", "noreply@example.com")
	n := Notification{ID: "n1", Type: TypeEmail, Recipient: "alice@example.com"}

	// First call to Deliver should log at least one entry.
	_ = e.Deliver(n)
	entries := e.Entries()
	if len(entries) == 0 {
		t.Error("Deliver should append at least one entry to DeliveryLog")
	}
}

// ============================================================================
// DISPATCHER TESTS
// ============================================================================

func TestNewDispatcherHasInitializedMap(t *testing.T) {
	// Registering a channel immediately should not panic.
	d := NewDispatcher()
	// This will panic if the map was not initialized with make().
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))
}

func TestDispatchNoChannelRegistered(t *testing.T) {
	d := NewDispatcher()
	n := Notification{ID: "n1", Type: TypeEmail, Recipient: "alice@example.com"}

	report := d.Dispatch(n)
	if report.Status != StatusFailed {
		t.Errorf("expected StatusFailed for unregistered channel, got %v", report.Status)
	}
	if report.NotificationID != "n1" {
		t.Errorf("NotificationID = %q, want %q", report.NotificationID, "n1")
	}
	if len(report.Log) == 0 {
		t.Error("expected at least one log entry explaining no channel registered")
	}
}

func TestDispatchSuccessfulDelivery(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))

	// The stub Deliver fails on attempt 0 (odd index behavior) and succeeds on retry.
	// With MaxAttempts=3, it should eventually succeed.
	n := Notification{
		ID:        "n2",
		Type:      TypeEmail,
		Recipient: "bob@example.com",
		Subject:   "Test",
		Body:      "body",
	}

	report := d.Dispatch(n)
	if report.Status != StatusDelivered {
		t.Errorf("expected StatusDelivered, got %v (log: %v)", report.Status, report.Log)
	}
	if report.NotificationID != "n2" {
		t.Errorf("NotificationID = %q, want %q", report.NotificationID, "n2")
	}
	if report.Channel != "email" {
		t.Errorf("Channel = %q, want %q", report.Channel, "email")
	}
	if report.Attempts == 0 {
		t.Error("Attempts should be > 0 after dispatch")
	}
}

func TestDispatchRetriesOnFailure(t *testing.T) {
	d := NewDispatcher()
	// Use a channel with MaxAttempts=1 so it fails after 1 retry attempt.
	ch := NewEmailChannel("smtp.example.com", "noreply@example.com")
	ch.MaxAttempts = 1 // only 1 attempt allowed
	d.Register(TypeEmail, ch)

	n := Notification{
		ID:        "n3",
		Type:      TypeEmail,
		Recipient: "carol@example.com",
	}

	report := d.Dispatch(n)
	// With MaxAttempts=1, the first attempt (index 0) fails (odd behavior),
	// ShouldRetry(1) is false, so delivery fails.
	if report.Status != StatusFailed {
		t.Errorf("expected StatusFailed with MaxAttempts=1, got %v", report.Status)
	}
}

func TestDispatchMultipleChannels(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeEmail, NewEmailChannel("smtp.example.com", "noreply@example.com"))
	d.Register(TypeSMS, NewSMSChannel("twilio", "+15550001234"))
	d.Register(TypeWebhook, NewWebhookChannel("https://hooks.example.com", "secret"))
	d.Register(TypeSlack, NewSlackChannel("acme", "alerts"))

	tests := []struct {
		id      string
		nType   NotificationType
		channel string
	}{
		{"e1", TypeEmail, "email"},
		{"s1", TypeSMS, "sms"},
		{"w1", TypeWebhook, "webhook"},
		{"sl1", TypeSlack, "slack"},
	}

	for _, tc := range tests {
		t.Run(tc.channel, func(t *testing.T) {
			n := Notification{
				ID:        tc.id,
				Type:      tc.nType,
				Recipient: "user@example.com",
			}
			report := d.Dispatch(n)
			if report.Channel != tc.channel {
				t.Errorf("Channel = %q, want %q", report.Channel, tc.channel)
			}
		})
	}
}

func TestDispatchReportContainsLogs(t *testing.T) {
	d := NewDispatcher()
	d.Register(TypeWebhook, NewWebhookChannel("https://hooks.example.com", "secret"))

	n := Notification{
		ID:        "w1",
		Type:      TypeWebhook,
		Recipient: "https://customer.example.com/hook",
	}

	report := d.Dispatch(n)
	if len(report.Log) == 0 {
		t.Error("DeliveryReport should include delivery log entries")
	}
}
