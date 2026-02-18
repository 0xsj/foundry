package main

import (
	"context"
	"strings"
	"testing"
	"time"
)

// =============================================================
// Registration Tests
// =============================================================

func TestListChannels(t *testing.T) {
	channels := ListChannels()
	expected := map[string]bool{
		"email":   false,
		"sms":     false,
		"push":    false,
		"webhook": false,
	}

	for _, ch := range channels {
		if _, ok := expected[ch]; ok {
			expected[ch] = true
		}
	}

	for ch, found := range expected {
		if !found {
			t.Errorf("expected channel %q to be registered", ch)
		}
	}
}

// =============================================================
// Email Dispatcher Tests
// =============================================================

func TestEmailDispatcher_Create(t *testing.T) {
	cfg := ChannelConfig{
		Type: "email",
		Params: map[string]string{
			"smtp_host":    "smtp.example.com",
			"smtp_port":    "587",
			"from_address": "noreply@example.com",
		},
		Enabled: true,
	}

	d, err := NewDispatcher(cfg)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if d.Channel() != "email" {
		t.Errorf("expected channel 'email', got %q", d.Channel())
	}
}

func TestEmailDispatcher_MissingParams(t *testing.T) {
	cfg := ChannelConfig{
		Type: "email",
		Params: map[string]string{
			"smtp_host": "smtp.example.com",
		},
		Enabled: true,
	}

	_, err := NewDispatcher(cfg)
	if err == nil {
		t.Fatal("expected error for missing params")
	}
	if !strings.Contains(err.Error(), "smtp_port") && !strings.Contains(err.Error(), "from_address") {
		t.Errorf("error should mention missing param, got: %v", err)
	}
}

func TestEmailDispatcher_Send(t *testing.T) {
	d := createEmailDispatcher(t)

	err := d.Send(context.Background(), "user@example.com", Notification{
		Subject:  "Test",
		Body:     "Hello world",
		Priority: "normal",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestEmailDispatcher_EmptyRecipient(t *testing.T) {
	d := createEmailDispatcher(t)

	err := d.Send(context.Background(), "", Notification{
		Body: "Hello",
	})
	if err == nil {
		t.Fatal("expected error for empty recipient")
	}
}

// =============================================================
// SMS Dispatcher Tests
// =============================================================

func TestSMSDispatcher_Create(t *testing.T) {
	cfg := ChannelConfig{
		Type: "sms",
		Params: map[string]string{
			"api_key":     "sk_test_abc123",
			"from_number": "+15551234567",
		},
		Enabled: true,
	}

	d, err := NewDispatcher(cfg)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if d.Channel() != "sms" {
		t.Errorf("expected channel 'sms', got %q", d.Channel())
	}
}

func TestSMSDispatcher_MissingAPIKey(t *testing.T) {
	cfg := ChannelConfig{
		Type: "sms",
		Params: map[string]string{
			"from_number": "+15551234567",
		},
		Enabled: true,
	}

	_, err := NewDispatcher(cfg)
	if err == nil {
		t.Fatal("expected error for missing api_key")
	}
}

func TestSMSDispatcher_Send(t *testing.T) {
	d := createSMSDispatcher(t)

	err := d.Send(context.Background(), "+15559876543", Notification{
		Body:     "Your verification code is 123456",
		Priority: "high",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

// =============================================================
// Push Dispatcher Tests
// =============================================================

func TestPushDispatcher_Create(t *testing.T) {
	cfg := ChannelConfig{
		Type: "push",
		Params: map[string]string{
			"server_key": "AAAA-fake-server-key",
			"project_id": "my-project-123",
		},
		Enabled: true,
	}

	d, err := NewDispatcher(cfg)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if d.Channel() != "push" {
		t.Errorf("expected channel 'push', got %q", d.Channel())
	}
}

func TestPushDispatcher_Send(t *testing.T) {
	d := createPushDispatcher(t)

	err := d.Send(context.Background(), "device-token-xyz", Notification{
		Subject:  "New message",
		Body:     "You have a new message from Alice",
		Priority: "normal",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

// =============================================================
// Webhook Dispatcher Tests
// =============================================================

func TestWebhookDispatcher_Create(t *testing.T) {
	cfg := ChannelConfig{
		Type: "webhook",
		Params: map[string]string{
			"url":    "https://hooks.example.com/notify",
			"secret": "whsec_test_secret_key",
		},
		Enabled: true,
	}

	d, err := NewDispatcher(cfg)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if d.Channel() != "webhook" {
		t.Errorf("expected channel 'webhook', got %q", d.Channel())
	}
}

func TestWebhookDispatcher_Send(t *testing.T) {
	d := createWebhookDispatcher(t)

	err := d.Send(context.Background(), "hook-endpoint-1", Notification{
		Subject:  "deployment.completed",
		Body:     `{"service":"api","version":"1.2.3"}`,
		Priority: "normal",
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestWebhookDispatcher_Sign(t *testing.T) {
	cfg := ChannelConfig{
		Type: "webhook",
		Params: map[string]string{
			"url":    "https://hooks.example.com/notify",
			"secret": "test-secret",
		},
		Enabled: true,
	}

	d, err := NewDispatcher(cfg)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	signer, ok := d.(*webhookDispatcher)
	if !ok {
		t.Fatal("webhook dispatcher should be *webhookDispatcher")
	}

	sig := signer.Sign([]byte("hello"))
	if sig == "" {
		t.Error("signature should not be empty")
	}
	if len(sig) != 64 {
		t.Errorf("expected 64-char hex signature, got %d chars", len(sig))
	}
}

// =============================================================
// Error Handling Tests
// =============================================================

func TestNewDispatcher_UnknownChannel(t *testing.T) {
	cfg := ChannelConfig{
		Type:    "carrier_pigeon",
		Params:  map[string]string{},
		Enabled: true,
	}

	_, err := NewDispatcher(cfg)
	if err == nil {
		t.Fatal("expected error for unknown channel type")
	}
	if !strings.Contains(err.Error(), "carrier_pigeon") {
		t.Errorf("error should mention the unknown channel name, got: %v", err)
	}
}

func TestNewDispatcher_DisabledChannel(t *testing.T) {
	cfg := ChannelConfig{
		Type: "email",
		Params: map[string]string{
			"smtp_host":    "smtp.example.com",
			"smtp_port":    "587",
			"from_address": "noreply@example.com",
		},
		Enabled: false,
	}

	_, err := NewDispatcher(cfg)
	if err == nil {
		t.Fatal("expected error for disabled channel")
	}
}

func TestSend_InvalidPriority(t *testing.T) {
	d := createEmailDispatcher(t)

	err := d.Send(context.Background(), "user@example.com", Notification{
		Body:     "Hello",
		Priority: "ultra-mega-critical",
	})
	if err == nil {
		t.Fatal("expected error for invalid priority")
	}
}

func TestSend_EmptyBody(t *testing.T) {
	d := createEmailDispatcher(t)

	err := d.Send(context.Background(), "user@example.com", Notification{
		Subject:  "Subject but no body",
		Body:     "",
		Priority: "normal",
	})
	if err == nil {
		t.Fatal("expected error for empty body")
	}
}

func TestSend_CancelledContext(t *testing.T) {
	d := createEmailDispatcher(t)

	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err := d.Send(ctx, "user@example.com", Notification{
		Body:     "Hello",
		Priority: "normal",
	})
	if err == nil {
		t.Fatal("expected error for cancelled context")
	}
}

func TestSend_TimedOutContext(t *testing.T) {
	d := createEmailDispatcher(t)

	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Nanosecond)
	defer cancel()
	time.Sleep(1 * time.Millisecond)

	err := d.Send(ctx, "user@example.com", Notification{
		Body:     "Hello",
		Priority: "normal",
	})
	if err == nil {
		t.Fatal("expected error for timed out context")
	}
}

// =============================================================
// Helpers
// =============================================================

func createEmailDispatcher(t *testing.T) Dispatcher {
	t.Helper()
	d, err := NewDispatcher(ChannelConfig{
		Type: "email",
		Params: map[string]string{
			"smtp_host":    "smtp.example.com",
			"smtp_port":    "587",
			"from_address": "noreply@example.com",
		},
		Enabled: true,
	})
	if err != nil {
		t.Fatalf("failed to create email dispatcher: %v", err)
	}
	return d
}

func createSMSDispatcher(t *testing.T) Dispatcher {
	t.Helper()
	d, err := NewDispatcher(ChannelConfig{
		Type: "sms",
		Params: map[string]string{
			"api_key":     "sk_test_abc123",
			"from_number": "+15551234567",
		},
		Enabled: true,
	})
	if err != nil {
		t.Fatalf("failed to create SMS dispatcher: %v", err)
	}
	return d
}

func createPushDispatcher(t *testing.T) Dispatcher {
	t.Helper()
	d, err := NewDispatcher(ChannelConfig{
		Type: "push",
		Params: map[string]string{
			"server_key": "AAAA-fake-server-key",
			"project_id": "my-project-123",
		},
		Enabled: true,
	})
	if err != nil {
		t.Fatalf("failed to create push dispatcher: %v", err)
	}
	return d
}

func createWebhookDispatcher(t *testing.T) Dispatcher {
	t.Helper()
	d, err := NewDispatcher(ChannelConfig{
		Type: "webhook",
		Params: map[string]string{
			"url":    "https://hooks.example.com/notify",
			"secret": "whsec_test_secret_key",
		},
		Enabled: true,
	})
	if err != nil {
		t.Fatalf("failed to create webhook dispatcher: %v", err)
	}
	return d
}
