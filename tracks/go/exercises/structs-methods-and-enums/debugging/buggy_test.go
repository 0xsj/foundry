package channelmanager

import "testing"

// TestDisablePersists verifies that calling Disable actually marks the channel
// as disabled in the manager's stored state.
// FAILS: value receiver on Disable — mutation is lost.
func TestDisablePersists(t *testing.T) {
	m := NewChannelManager()
	m.Register(ChannelConfig{Name: "email", Enabled: true, MaxRetries: 3, TimeoutMs: 5000})

	if err := m.Disable("email"); err != nil {
		t.Fatalf("Disable returned error: %v", err)
	}

	cfg, ok := m.Get("email")
	if !ok {
		t.Fatal("channel 'email' not found after Disable")
	}
	if cfg.Enabled {
		t.Errorf("expected Enabled=false after Disable, got Enabled=true")
	}
}

// TestPingChannelDoesNotPanic verifies that PingChannel works on a freshly
// created manager without panicking.
// FAILS: PingClient is nil — calling Ping through it panics.
func TestPingChannelDoesNotPanic(t *testing.T) {
	m := NewChannelManager()
	m.Register(ChannelConfig{Name: "webhook", Enabled: true})

	// This panics because m.PingClient is nil.
	m.PingChannel("webhook")

	if m.PingClient.PingCount != 1 {
		t.Errorf("PingCount = %d, want 1", m.PingClient.PingCount)
	}
	if m.PingClient.LastPingedChannel != "webhook" {
		t.Errorf("LastPingedChannel = %q, want %q", m.PingClient.LastPingedChannel, "webhook")
	}
}

// TestUpdateTimeoutModifiesCorrectChannel verifies that updating a channel's
// timeout affects only that channel.
// FAILS: UpdateTimeout writes back to the wrong map key (m.lastUpdated instead of name).
func TestUpdateTimeoutModifiesCorrectChannel(t *testing.T) {
	m := NewChannelManager()
	m.Register(ChannelConfig{Name: "email", Enabled: true, TimeoutMs: 1000})
	m.Register(ChannelConfig{Name: "sms", Enabled: true, TimeoutMs: 2000})

	if err := m.UpdateTimeout("sms", 9999); err != nil {
		t.Fatalf("UpdateTimeout returned error: %v", err)
	}

	smsCfg, _ := m.Get("sms")
	if smsCfg.TimeoutMs != 9999 {
		t.Errorf("sms TimeoutMs = %d, want 9999", smsCfg.TimeoutMs)
	}

	emailCfg, _ := m.Get("email")
	if emailCfg.TimeoutMs != 1000 {
		t.Errorf("email TimeoutMs = %d, want 1000 (should be unchanged)", emailCfg.TimeoutMs)
	}
}

// TestLastUpdatedTracksCorrectChannel verifies that LastUpdated returns the name
// of the channel most recently modified via Disable or UpdateTimeout.
// FAILS: Disable uses a value receiver, so m.lastUpdated is never set via Disable.
func TestLastUpdatedTracksCorrectChannel(t *testing.T) {
	m := NewChannelManager()
	m.Register(ChannelConfig{Name: "email", Enabled: true, TimeoutMs: 1000})
	m.Register(ChannelConfig{Name: "slack", Enabled: true, TimeoutMs: 500})

	_ = m.Disable("slack")

	if last := m.LastUpdated(); last != "slack" {
		t.Errorf("LastUpdated() = %q, want %q", last, "slack")
	}
}

// TestMultiplePings verifies ping count accumulates correctly.
func TestMultiplePings(t *testing.T) {
	m := NewChannelManager()
	m.Register(ChannelConfig{Name: "email"})
	m.Register(ChannelConfig{Name: "sms"})

	m.PingChannel("email")
	m.PingChannel("sms")
	m.PingChannel("email")

	if m.PingClient.PingCount != 3 {
		t.Errorf("PingCount = %d, want 3", m.PingClient.PingCount)
	}
	if m.PingClient.LastPingedChannel != "email" {
		t.Errorf("LastPingedChannel = %q, want 'email'", m.PingClient.LastPingedChannel)
	}
}
