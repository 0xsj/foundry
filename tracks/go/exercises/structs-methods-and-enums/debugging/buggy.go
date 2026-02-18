package channelmanager

import "fmt"

// ChannelConfig holds runtime configuration for a delivery channel.
type ChannelConfig struct {
	Name        string
	Enabled     bool
	MaxRetries  int
	TimeoutMs   int
}

// PingClient handles test pings to verify channel connectivity.
// It is embedded in ChannelManager to provide ping functionality.
type PingClient struct {
	LastPingedChannel string
	PingCount         int
}

// Ping records a test ping for the given channel.
func (p *PingClient) Ping(channelName string) {
	p.LastPingedChannel = channelName
	p.PingCount++
	fmt.Printf("ping: %s (total pings: %d)\n", channelName, p.PingCount)
}

// ChannelManager tracks all registered delivery channels.
type ChannelManager struct {
	*PingClient                        // BUG 2: embedded pointer, never initialized
	channels    map[string]ChannelConfig
	lastUpdated string
}

// NewChannelManager creates a ready-to-use ChannelManager.
func NewChannelManager() *ChannelManager {
	return &ChannelManager{
		// PingClient is a pointer — its zero value is nil.
		// Calling Ping() on a nil *PingClient will panic.
		channels: make(map[string]ChannelConfig),
	}
}

// Register adds a channel configuration.
func (m *ChannelManager) Register(cfg ChannelConfig) {
	m.channels[cfg.Name] = cfg
}

// Get returns the config for a channel. Returns false if not found.
func (m *ChannelManager) Get(name string) (ChannelConfig, bool) {
	cfg, ok := m.channels[name]
	return cfg, ok
}

// Disable marks a channel as disabled.
// BUG 1: value receiver — mutates a copy of ChannelManager, not the original.
// The change to m.lastUpdated is lost when this method returns.
// But that's not the full story — even m.channels[name].Enabled = false
// is the wrong approach (can't assign to map element field directly).
func (m ChannelManager) Disable(name string) error {
	cfg, ok := m.channels[name]
	if !ok {
		return fmt.Errorf("channel %q not found", name)
	}
	cfg.Enabled = false
	m.channels[name] = cfg
	m.lastUpdated = name  // this write is lost — value receiver
	return nil
}

// UpdateTimeout changes the timeout for a channel.
// BUG 3: reads cfg from map, modifies it, then writes it back to the wrong key.
// The key used for write-back is hardcoded to the first registered channel's name
// (because lastUpdated was never properly updated — see Bug 1).
func (m *ChannelManager) UpdateTimeout(name string, timeoutMs int) error {
	cfg, ok := m.channels[name]
	if !ok {
		return fmt.Errorf("channel %q not found", name)
	}
	cfg.TimeoutMs = timeoutMs

	// Write the modification back — but to the wrong entry.
	// Should be: m.channels[name] = cfg
	// Instead writes to m.lastUpdated, which is always "" or stale.
	m.channels[m.lastUpdated] = cfg

	m.lastUpdated = name
	return nil
}

// LastUpdated returns the name of the most recently updated channel.
func (m *ChannelManager) LastUpdated() string {
	return m.lastUpdated
}

// PingChannel sends a test ping to verify the channel is reachable.
// Delegates to the embedded *PingClient.
func (m *ChannelManager) PingChannel(name string) {
	m.Ping(name) // calls (*PingClient).Ping — panics if PingClient is nil
}
