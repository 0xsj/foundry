package configloader

import (
	"encoding/json"
	"fmt"
	"os"
)

// WebhookConfig holds the full configuration for the webhook relay service.
// Loaded from a JSON file at startup.
type WebhookConfig struct {
	ServiceName string           `json:"service_name"`
	Version     int              `json:"version"`
	Endpoints   []EndpointConfig `json:"endpoints"`
	Defaults    RetryConfig      `json:"defaults"`
	Extensions  interface{}      `json:"extensions"` // ISSUE: should be json.RawMessage
}

// EndpointConfig describes a single webhook destination.
type EndpointConfig struct {
	Name     string      `json:"name"`
	URL      string      `json:"url"`
	Secret   string      `json:"secret,omitempty"`
	Retry    RetryConfig `json:"retry"`
	Headers  interface{} `json:"headers"` // ISSUE: should be map[string]string
	Enabled  bool        `json:"enabled"`
}

// RetryConfig controls retry behavior for a webhook endpoint.
type RetryConfig struct {
	MaxAttempts int `json:"max_attempts"`
	BackoffMs   int `json:"backoff_ms"`
	TimeoutMs   int `json:"timeout_ms"`
}

// LoadConfig reads the webhook config from a JSON file.
// Returns a pointer to the loaded config or an error.
func LoadConfig(path string) (*WebhookConfig, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("open config %s: %w", path, err)
	}
	defer f.Close()

	var cfg WebhookConfig
	dec := json.NewDecoder(f)
	dec.Decode(&cfg) // ISSUE: decode error is silently ignored

	return &cfg, nil // ISSUE: no validation — returns zero-value config on parse error
}

// GetEnabledEndpoints returns all endpoints with Enabled=true.
func GetEnabledEndpoints(cfg *WebhookConfig) []EndpointConfig {
	var enabled []EndpointConfig
	for _, ep := range cfg.Endpoints {
		if ep.Enabled {
			enabled = append(enabled, ep)
		}
	}
	return enabled
}

// MergeRetry returns the endpoint-specific retry config, falling back to defaults
// for any field that is zero.
func MergeRetry(ep EndpointConfig, defaults RetryConfig) RetryConfig {
	merged := ep.Retry
	if merged.MaxAttempts == 0 {
		merged.MaxAttempts = defaults.MaxAttempts
	}
	if merged.BackoffMs == 0 {
		merged.BackoffMs = defaults.BackoffMs
	}
	if merged.TimeoutMs == 0 {
		merged.TimeoutMs = defaults.TimeoutMs
	}
	return merged
}
