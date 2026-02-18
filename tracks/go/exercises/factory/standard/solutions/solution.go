package main

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"sync"
)

// ---------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------

// Notification represents a message to be sent through any channel.
type Notification struct {
	Subject  string
	Body     string
	Priority string            // "low", "normal", "high", "critical"
	Metadata map[string]string // channel-specific metadata
}

// ValidPriorities defines the allowed priority levels.
var ValidPriorities = map[string]bool{
	"low":      true,
	"normal":   true,
	"high":     true,
	"critical": true,
}

// ChannelConfig holds configuration for creating a dispatcher.
type ChannelConfig struct {
	Type    string            // "email", "sms", "push", "webhook"
	Params  map[string]string // channel-specific parameters
	Enabled bool
}

// ---------------------------------------------------------------------
// Dispatcher interface
// ---------------------------------------------------------------------

// Dispatcher sends notifications through a specific channel.
type Dispatcher interface {
	// Send delivers a notification to the given recipient.
	Send(ctx context.Context, recipient string, notification Notification) error

	// Channel returns the name of this dispatcher's channel.
	Channel() string
}

// ---------------------------------------------------------------------
// Factory registry
//
// Key design decision: the registry is package-level with a RWMutex.
// Registration happens in init() (before main), lookups happen at runtime.
// The mutex protects against the rare case of lazy registration.
// ---------------------------------------------------------------------

// FactoryFunc creates a Dispatcher from channel-specific parameters.
type FactoryFunc func(params map[string]string) (Dispatcher, error)

var (
	registryMu sync.RWMutex
	factories  = make(map[string]FactoryFunc)
)

// Register adds a dispatcher factory to the registry.
// It panics if a factory with the same name is already registered.
func Register(name string, factory FactoryFunc) {
	registryMu.Lock()
	defer registryMu.Unlock()

	if _, exists := factories[name]; exists {
		panic(fmt.Sprintf("notification: duplicate channel registration: %s", name))
	}
	factories[name] = factory
}

// NewDispatcher creates a Dispatcher based on the given ChannelConfig.
// Returns an error if the channel type is unknown, disabled, or config is invalid.
func NewDispatcher(cfg ChannelConfig) (Dispatcher, error) {
	if cfg.Type == "" {
		return nil, fmt.Errorf("notification: channel type cannot be empty")
	}

	if !cfg.Enabled {
		return nil, fmt.Errorf("notification: channel %q is disabled", cfg.Type)
	}

	registryMu.RLock()
	factory, exists := factories[cfg.Type]
	registryMu.RUnlock()

	if !exists {
		return nil, fmt.Errorf("notification: unknown channel type %q", cfg.Type)
	}

	return factory(cfg.Params)
}

// ListChannels returns the names of all registered channels.
func ListChannels() []string {
	registryMu.RLock()
	defer registryMu.RUnlock()

	names := make([]string, 0, len(factories))
	for name := range factories {
		names = append(names, name)
	}
	return names
}

// ---------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------

// requireParams checks that all required parameters are present and non-empty.
func requireParams(params map[string]string, required ...string) error {
	for _, key := range required {
		if params[key] == "" {
			return fmt.Errorf("missing required parameter: %s", key)
		}
	}
	return nil
}

// validateNotification checks that a notification has valid fields.
func validateNotification(n Notification) error {
	if n.Body == "" {
		return fmt.Errorf("notification body cannot be empty")
	}
	if n.Priority != "" && !ValidPriorities[n.Priority] {
		return fmt.Errorf("invalid priority %q: must be one of low, normal, high, critical", n.Priority)
	}
	return nil
}

// validateSend performs common validation for all Send methods.
func validateSend(ctx context.Context, recipient string, n Notification) error {
	// Check context first -- fast fail on cancelled requests
	if err := ctx.Err(); err != nil {
		return fmt.Errorf("context error: %w", err)
	}

	if recipient == "" {
		return fmt.Errorf("recipient cannot be empty")
	}

	return validateNotification(n)
}

// ---------------------------------------------------------------------
// Email Dispatcher
//
// Validates: smtp_host, smtp_port, from_address
// In production, this would use net/smtp or a library like gomail.
// ---------------------------------------------------------------------

type emailDispatcher struct {
	smtpHost string
	smtpPort string
	from     string
}

func newEmailDispatcher(params map[string]string) (Dispatcher, error) {
	if err := requireParams(params, "smtp_host", "smtp_port", "from_address"); err != nil {
		return nil, fmt.Errorf("email: %w", err)
	}

	return &emailDispatcher{
		smtpHost: params["smtp_host"],
		smtpPort: params["smtp_port"],
		from:     params["from_address"],
	}, nil
}

func (d *emailDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	if err := validateSend(ctx, recipient, n); err != nil {
		return fmt.Errorf("email: %w", err)
	}

	// In production: dial SMTP, send email
	fmt.Printf("[email] Sending to %s via %s:%s from %s | subject=%q priority=%s\n",
		recipient, d.smtpHost, d.smtpPort, d.from, n.Subject, n.Priority)

	return nil
}

func (d *emailDispatcher) Channel() string { return "email" }

// ---------------------------------------------------------------------
// SMS Dispatcher
//
// Validates: api_key, from_number
// In production, this would call a Twilio-like HTTP API.
// ---------------------------------------------------------------------

type smsDispatcher struct {
	apiKey     string
	fromNumber string
}

func newSMSDispatcher(params map[string]string) (Dispatcher, error) {
	if err := requireParams(params, "api_key", "from_number"); err != nil {
		return nil, fmt.Errorf("sms: %w", err)
	}

	return &smsDispatcher{
		apiKey:     params["api_key"],
		fromNumber: params["from_number"],
	}, nil
}

func (d *smsDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	if err := validateSend(ctx, recipient, n); err != nil {
		return fmt.Errorf("sms: %w", err)
	}

	// In production: POST to Twilio API
	fmt.Printf("[sms] Sending to %s from %s | body=%q priority=%s\n",
		recipient, d.fromNumber, n.Body, n.Priority)

	return nil
}

func (d *smsDispatcher) Channel() string { return "sms" }

// ---------------------------------------------------------------------
// Push Notification Dispatcher
//
// Validates: server_key, project_id
// In production, this would call the FCM HTTP API.
// ---------------------------------------------------------------------

type pushDispatcher struct {
	serverKey string
	projectID string
}

func newPushDispatcher(params map[string]string) (Dispatcher, error) {
	if err := requireParams(params, "server_key", "project_id"); err != nil {
		return nil, fmt.Errorf("push: %w", err)
	}

	return &pushDispatcher{
		serverKey: params["server_key"],
		projectID: params["project_id"],
	}, nil
}

func (d *pushDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	if err := validateSend(ctx, recipient, n); err != nil {
		return fmt.Errorf("push: %w", err)
	}

	// In production: POST to FCM endpoint
	fmt.Printf("[push] Sending to device %s via project %s | subject=%q priority=%s\n",
		recipient, d.projectID, n.Subject, n.Priority)

	return nil
}

func (d *pushDispatcher) Channel() string { return "push" }

// ---------------------------------------------------------------------
// Webhook Dispatcher
//
// Validates: url, secret
// Signs payloads with HMAC-SHA256 to verify authenticity.
// In production, this would POST to the configured URL.
// ---------------------------------------------------------------------

type webhookDispatcher struct {
	url    string
	secret string
}

func newWebhookDispatcher(params map[string]string) (Dispatcher, error) {
	if err := requireParams(params, "url", "secret"); err != nil {
		return nil, fmt.Errorf("webhook: %w", err)
	}

	return &webhookDispatcher{
		url:    params["url"],
		secret: params["secret"],
	}, nil
}

func (d *webhookDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	if err := validateSend(ctx, recipient, n); err != nil {
		return fmt.Errorf("webhook: %w", err)
	}

	// Compute HMAC-SHA256 signature of the body
	signature := d.Sign([]byte(n.Body))

	// In production: POST to d.url with X-Signature header
	fmt.Printf("[webhook] POST %s/%s | sig=%s... body=%q\n",
		d.url, recipient, signature[:16], n.Body)

	return nil
}

// Sign computes an HMAC-SHA256 signature for the given payload.
func (d *webhookDispatcher) Sign(payload []byte) string {
	mac := hmac.New(sha256.New, []byte(d.secret))
	mac.Write(payload)
	return hex.EncodeToString(mac.Sum(nil))
}

func (d *webhookDispatcher) Channel() string { return "webhook" }

// ---------------------------------------------------------------------
// Self-registration -- each channel registers its factory at init time.
// In a larger codebase, these would be in separate packages.
// ---------------------------------------------------------------------

func init() {
	Register("email", newEmailDispatcher)
	Register("sms", newSMSDispatcher)
	Register("push", newPushDispatcher)
	Register("webhook", newWebhookDispatcher)
}

// ---------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------

func main() {
	fmt.Println("=== Notification Factory Demo ===")
	fmt.Printf("Registered channels: %v\n\n", ListChannels())

	configs := []ChannelConfig{
		{
			Type: "email",
			Params: map[string]string{
				"smtp_host":    "smtp.example.com",
				"smtp_port":    "587",
				"from_address": "alerts@example.com",
			},
			Enabled: true,
		},
		{
			Type: "sms",
			Params: map[string]string{
				"api_key":     "sk_live_abc123",
				"from_number": "+15551234567",
			},
			Enabled: true,
		},
		{
			Type: "webhook",
			Params: map[string]string{
				"url":    "https://hooks.slack.com/services/T00/B00/xxx",
				"secret": "whsec_signing_secret",
			},
			Enabled: true,
		},
	}

	ctx := context.Background()

	for _, cfg := range configs {
		d, err := NewDispatcher(cfg)
		if err != nil {
			fmt.Printf("Error creating %s dispatcher: %v\n", cfg.Type, err)
			continue
		}

		err = d.Send(ctx, "user@example.com", Notification{
			Subject:  "Deployment Complete",
			Body:     "Service api-gateway deployed to production (v2.1.0)",
			Priority: "high",
		})
		if err != nil {
			fmt.Printf("Error sending via %s: %v\n", d.Channel(), err)
		}
	}
}
