// Strategy Pattern: Notification Dispatcher
//
// Demonstrates interface-based strategy pattern. A notification service
// selects a delivery strategy (email, SMS, webhook, Slack) based on
// user preferences. Each strategy satisfies the same interface; the
// dispatcher doesn't know or care which one it's using.
//
// Run: go run ./notification/

package main

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"
)

// --- Strategy Interface ---

// Message represents a notification payload.
type Message struct {
	Subject string
	Body    string
	Level   string // "info", "warning", "critical"
}

// DeliveryStrategy defines how a notification is delivered.
// This interface is defined in the consuming package (here), not in the
// implementing packages. Implementations don't know this interface exists.
type DeliveryStrategy interface {
	Deliver(ctx context.Context, recipient string, msg Message) error
	Name() string
}

// --- Concrete Strategies ---

// EmailStrategy delivers notifications via SMTP.
type EmailStrategy struct {
	SMTPHost string
	SMTPPort int
	From     string
}

// Compile-time interface guard.
var _ DeliveryStrategy = (*EmailStrategy)(nil)

func (e *EmailStrategy) Deliver(ctx context.Context, recipient string, msg Message) error {
	// In production, this would open an SMTP connection and send.
	fmt.Printf("[EMAIL] To: %s | From: %s | Subject: %s\n", recipient, e.From, msg.Subject)
	fmt.Printf("        Body: %s\n", msg.Body)
	fmt.Printf("        Via: %s:%d\n", e.SMTPHost, e.SMTPPort)
	return nil
}

func (e *EmailStrategy) Name() string { return "email" }

// SMSStrategy delivers notifications via SMS gateway.
type SMSStrategy struct {
	APIKey    string
	FromPhone string
}

var _ DeliveryStrategy = (*SMSStrategy)(nil)

func (s *SMSStrategy) Deliver(ctx context.Context, recipient string, msg Message) error {
	// Truncate for SMS: subject + first 100 chars of body
	text := msg.Subject + ": " + msg.Body
	if len(text) > 160 {
		text = text[:157] + "..."
	}
	fmt.Printf("[SMS]   To: %s | From: %s\n", recipient, s.FromPhone)
	fmt.Printf("        Text: %s\n", text)
	return nil
}

func (s *SMSStrategy) Name() string { return "sms" }

// WebhookStrategy delivers notifications via HTTP POST to a webhook URL.
type WebhookStrategy struct {
	TargetURL string
	Secret    string
}

var _ DeliveryStrategy = (*WebhookStrategy)(nil)

func (w *WebhookStrategy) Deliver(ctx context.Context, recipient string, msg Message) error {
	// In production, this would POST JSON to the webhook URL with HMAC signature.
	fmt.Printf("[WEBHOOK] URL: %s | Recipient: %s\n", w.TargetURL, recipient)
	fmt.Printf("          Payload: {subject: %q, body: %q, level: %q}\n",
		msg.Subject, msg.Body, msg.Level)
	return nil
}

func (w *WebhookStrategy) Name() string { return "webhook" }

// SlackStrategy delivers notifications to a Slack channel.
type SlackStrategy struct {
	WebhookURL string
	Channel    string
}

var _ DeliveryStrategy = (*SlackStrategy)(nil)

func (s *SlackStrategy) Deliver(ctx context.Context, recipient string, msg Message) error {
	// In production, this would POST to Slack's webhook API.
	emoji := map[string]string{
		"info":     ":information_source:",
		"warning":  ":warning:",
		"critical": ":rotating_light:",
	}
	icon := emoji[msg.Level]
	if icon == "" {
		icon = ":bell:"
	}
	fmt.Printf("[SLACK] Channel: %s | Mention: %s\n", s.Channel, recipient)
	fmt.Printf("        %s *%s*: %s\n", icon, msg.Subject, msg.Body)
	return nil
}

func (s *SlackStrategy) Name() string { return "slack" }

// --- Notification Service (Consumer) ---

// NotificationService dispatches notifications using pluggable strategies.
// It doesn't know which strategy it's using -- it just calls Deliver.
type NotificationService struct {
	strategies map[string]DeliveryStrategy
	fallback   DeliveryStrategy
}

// NewNotificationService creates a service with registered strategies.
func NewNotificationService(fallback DeliveryStrategy, strategies ...DeliveryStrategy) *NotificationService {
	m := make(map[string]DeliveryStrategy, len(strategies))
	for _, s := range strategies {
		m[s.Name()] = s
	}
	return &NotificationService{
		strategies: m,
		fallback:   fallback,
	}
}

// User represents a notification recipient with delivery preferences.
type User struct {
	Name             string
	PreferredChannel string // "email", "sms", "webhook", "slack"
	ContactInfo      string // email address, phone number, user ID, etc.
}

// Notify sends a notification using the user's preferred delivery channel.
// Falls back to the default strategy if the preferred channel is unavailable.
func (ns *NotificationService) Notify(ctx context.Context, user User, msg Message) error {
	strategy, ok := ns.strategies[user.PreferredChannel]
	if !ok {
		fmt.Printf("  (preferred channel %q not available, falling back to %s)\n",
			user.PreferredChannel, ns.fallback.Name())
		strategy = ns.fallback
	}
	return strategy.Deliver(ctx, user.ContactInfo, msg)
}

// --- Decorator: Logging Wrapper ---

// loggingStrategy wraps any DeliveryStrategy with logging.
// This demonstrates the decorator pattern applied to strategies.
type loggingStrategy struct {
	delegate DeliveryStrategy
}

func withLogging(s DeliveryStrategy) DeliveryStrategy {
	return &loggingStrategy{delegate: s}
}

func (l *loggingStrategy) Deliver(ctx context.Context, recipient string, msg Message) error {
	start := time.Now()
	fmt.Printf("  -> delivering via %s to %s...\n", l.delegate.Name(), recipient)
	err := l.delegate.Deliver(ctx, recipient, msg)
	elapsed := time.Since(start)
	if err != nil {
		fmt.Printf("  -> FAILED after %v: %v\n", elapsed, err)
	} else {
		fmt.Printf("  -> delivered in %v\n", elapsed)
	}
	return err
}

func (l *loggingStrategy) Name() string { return l.delegate.Name() }

// --- Main: Wire everything together ---

func main() {
	// Create strategies (in production, these come from configuration)
	emailStrategy := withLogging(&EmailStrategy{
		SMTPHost: "smtp.example.com",
		SMTPPort: 587,
		From:     "alerts@example.com",
	})

	smsStrategy := withLogging(&SMSStrategy{
		APIKey:    "sk-sms-xxxxx",
		FromPhone: "+1-555-0100",
	})

	webhookStrategy := withLogging(&WebhookStrategy{
		TargetURL: "https://hooks.example.com/notify",
		Secret:    "whsec_xxxxx",
	})

	slackStrategy := withLogging(&SlackStrategy{
		WebhookURL: "https://hooks.slack.com/services/T00/B00/xxxx",
		Channel:    "#ops-alerts",
	})

	// Create the notification service with all strategies
	svc := NewNotificationService(
		emailStrategy, // fallback
		emailStrategy,
		smsStrategy,
		webhookStrategy,
		slackStrategy,
	)

	// Users with different delivery preferences
	users := []User{
		{Name: "Alice", PreferredChannel: "email", ContactInfo: "alice@example.com"},
		{Name: "Bob", PreferredChannel: "sms", ContactInfo: "+1-555-0199"},
		{Name: "Charlie", PreferredChannel: "slack", ContactInfo: "@charlie"},
		{Name: "Diana", PreferredChannel: "webhook", ContactInfo: "team-diana"},
		{Name: "Eve", PreferredChannel: "carrier-pigeon", ContactInfo: "eve@example.com"},
	}

	// The alert to send
	alert := Message{
		Subject: "Database CPU > 90%",
		Body:    "Primary database cpu-prod-01 has been above 90% CPU for 5 minutes. Consider scaling up or investigating slow queries.",
		Level:   "critical",
	}

	ctx := context.Background()

	fmt.Println("=== Sending Alert to All Users ===")
	fmt.Println(strings.Repeat("-", 60))

	for _, user := range users {
		fmt.Printf("\nNotifying %s (prefers %s):\n", user.Name, user.PreferredChannel)
		if err := svc.Notify(ctx, user, alert); err != nil {
			log.Printf("Failed to notify %s: %v", user.Name, err)
		}
	}

	fmt.Println(strings.Repeat("-", 60))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Each strategy satisfies DeliveryStrategy independently")
	fmt.Println("- NotificationService doesn't import any strategy package")
	fmt.Println("- The logging decorator wraps any strategy transparently")
	fmt.Println("- Eve's unknown channel fell back to the default (email)")
	fmt.Println("- Adding a new channel means writing one struct — zero changes to existing code")
}
