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

	// Channel returns the name of this dispatcher's channel (e.g., "email").
	Channel() string
}

// ---------------------------------------------------------------------
// Factory registry
// ---------------------------------------------------------------------

// FactoryFunc creates a Dispatcher from channel-specific parameters.
type FactoryFunc func(params map[string]string) (Dispatcher, error)

// TODO: Declare package-level registry variables:
// - A sync.RWMutex for thread safety
// - A map[string]FactoryFunc for storing factories

// Register adds a dispatcher factory to the registry.
// It panics if a factory with the same name is already registered.
func Register(name string, factory FactoryFunc) {
	// TODO: Implement registration with mutex protection
	// - Lock the mutex
	// - Check for duplicate registration (panic if duplicate)
	// - Store the factory
}

// NewDispatcher creates a Dispatcher based on the given ChannelConfig.
// Returns an error if the channel type is unknown or config is invalid.
func NewDispatcher(cfg ChannelConfig) (Dispatcher, error) {
	// TODO: Implement factory lookup
	// - Validate that cfg is not empty
	// - Check if channel is enabled
	// - Look up the factory in the registry (use RLock for reads)
	// - Return a clear error if the channel type is not found
	// - Call the factory function with cfg.Params
	return nil, fmt.Errorf("not implemented")
}

// ListChannels returns the names of all registered channels.
func ListChannels() []string {
	// TODO: Return a slice of all registered channel names
	return nil
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

// ---------------------------------------------------------------------
// Email Dispatcher
// ---------------------------------------------------------------------

type emailDispatcher struct {
	smtpHost string
	smtpPort string
	from     string
}

func newEmailDispatcher(params map[string]string) (Dispatcher, error) {
	// TODO: Implement email dispatcher creation
	// - Validate required params: smtp_host, smtp_port, from_address
	// - Return a configured emailDispatcher
	return nil, fmt.Errorf("not implemented")
}

func (d *emailDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	// TODO: Implement send
	// - Validate recipient is not empty
	// - Validate the notification
	// - Check context cancellation
	// - Simulate sending (fmt.Printf is fine)
	return fmt.Errorf("not implemented")
}

func (d *emailDispatcher) Channel() string {
	return "email"
}

// ---------------------------------------------------------------------
// SMS Dispatcher
// ---------------------------------------------------------------------

type smsDispatcher struct {
	apiKey     string
	fromNumber string
}

func newSMSDispatcher(params map[string]string) (Dispatcher, error) {
	// TODO: Implement SMS dispatcher creation
	// - Validate required params: api_key, from_number
	// - Return a configured smsDispatcher
	return nil, fmt.Errorf("not implemented")
}

func (d *smsDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	// TODO: Implement send
	// - Validate recipient is not empty
	// - Validate the notification
	// - Check context cancellation
	// - Simulate sending
	return fmt.Errorf("not implemented")
}

func (d *smsDispatcher) Channel() string {
	return "sms"
}

// ---------------------------------------------------------------------
// Push Notification Dispatcher
// ---------------------------------------------------------------------

type pushDispatcher struct {
	serverKey string
	projectID string
}

func newPushDispatcher(params map[string]string) (Dispatcher, error) {
	// TODO: Implement push dispatcher creation
	// - Validate required params: server_key, project_id
	// - Return a configured pushDispatcher
	return nil, fmt.Errorf("not implemented")
}

func (d *pushDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	// TODO: Implement send
	// - Validate recipient (this is the device token) is not empty
	// - Validate the notification
	// - Check context cancellation
	// - Simulate sending
	return fmt.Errorf("not implemented")
}

func (d *pushDispatcher) Channel() string {
	return "push"
}

// ---------------------------------------------------------------------
// Webhook Dispatcher
// ---------------------------------------------------------------------

type webhookDispatcher struct {
	url    string
	secret string
}

func newWebhookDispatcher(params map[string]string) (Dispatcher, error) {
	// TODO: Implement webhook dispatcher creation
	// - Validate required params: url, secret
	// - Return a configured webhookDispatcher
	return nil, fmt.Errorf("not implemented")
}

func (d *webhookDispatcher) Send(ctx context.Context, recipient string, n Notification) error {
	// TODO: Implement send
	// - Validate recipient (the webhook endpoint path or ID) is not empty
	// - Validate the notification
	// - Check context cancellation
	// - Compute HMAC-SHA256 signature of the body using d.secret
	// - Simulate sending with the signature
	return fmt.Errorf("not implemented")
}

// Sign computes an HMAC-SHA256 signature for the given payload.
func (d *webhookDispatcher) Sign(payload []byte) string {
	mac := hmac.New(sha256.New, []byte(d.secret))
	mac.Write(payload)
	return hex.EncodeToString(mac.Sum(nil))
}

func (d *webhookDispatcher) Channel() string {
	return "webhook"
}

// ---------------------------------------------------------------------
// Self-registration
// ---------------------------------------------------------------------

func init() {
	// TODO: Register all four dispatcher factories
	// Register("email", newEmailDispatcher)
	// Register("sms", newSMSDispatcher)
	// Register("push", newPushDispatcher)
	// Register("webhook", newWebhookDispatcher)
}

// ---------------------------------------------------------------------
// Demo (optional -- tests are the primary validation)
// ---------------------------------------------------------------------

func main() {
	fmt.Println("=== Notification Factory Demo ===")
	fmt.Printf("Registered channels: %v\n", ListChannels())

	// This is a placeholder -- implement the TODOs and run tests instead.
	fmt.Println("Run 'go test -v' to validate your implementation.")
}

// Ensure these are used (compilation guard)
var (
	_ = context.Background
	_ = sync.RWMutex{}
	_ = hmac.New
	_ = sha256.New
	_ = hex.EncodeToString
)
