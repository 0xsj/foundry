package config

import "testing"

// Run with: go test -v ./...

func TestLoadConfig_FullEnv(t *testing.T) {
	// All fields provided — nothing should use defaults
	env := map[string]string{
		"HOST":         "api.example.com",
		"PORT":         "9090",
		"DEBUG":        "true",
		"MAX_RETRIES":  "5",
		"TIMEOUT_SEC":  "30.5",
		"ENVIRONMENT":  "production",
	}

	cfg, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if cfg.Host != "api.example.com" {
		t.Errorf("Host = %q, want %q", cfg.Host, "api.example.com")
	}
	if cfg.Port != 9090 {
		t.Errorf("Port = %d, want %d", cfg.Port, 9090)
	}
	if cfg.Debug != true {
		t.Errorf("Debug = %v, want %v", cfg.Debug, true)
	}
	if cfg.MaxRetries == nil {
		t.Fatal("MaxRetries is nil, want *5")
	}
	if *cfg.MaxRetries != 5 {
		t.Errorf("MaxRetries = %d, want %d", *cfg.MaxRetries, 5)
	}
	if cfg.TimeoutSec != 30.5 {
		t.Errorf("TimeoutSec = %f, want %f", cfg.TimeoutSec, 30.5)
	}
	if cfg.Environment != "production" {
		t.Errorf("Environment = %q, want %q", cfg.Environment, "production")
	}
}

func TestLoadConfig_DefaultsApplied(t *testing.T) {
	// Only required field provided — everything else should use defaults
	env := map[string]string{
		"HOST": "localhost",
	}

	cfg, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if cfg.Host != "localhost" {
		t.Errorf("Host = %q, want %q", cfg.Host, "localhost")
	}
	if cfg.Port != DefaultPort {
		t.Errorf("Port = %d, want default %d", cfg.Port, DefaultPort)
	}
	if cfg.Debug != false {
		t.Errorf("Debug = %v, want default false", cfg.Debug)
	}
	if cfg.MaxRetries != nil {
		t.Errorf("MaxRetries = %v, want nil (not set)", cfg.MaxRetries)
	}
	if cfg.TimeoutSec != DefaultTimeoutSec {
		t.Errorf("TimeoutSec = %f, want default %f", cfg.TimeoutSec, DefaultTimeoutSec)
	}
	if cfg.Environment != DefaultEnvironment {
		t.Errorf("Environment = %q, want default %q", cfg.Environment, DefaultEnvironment)
	}
}

func TestLoadConfig_MissingHost(t *testing.T) {
	// Host is required — should error
	env := map[string]string{
		"PORT": "8080",
	}

	_, err := LoadConfig(env)
	if err == nil {
		t.Fatal("expected error for missing HOST, got nil")
	}
}

func TestLoadConfig_EmptyHost(t *testing.T) {
	// Host present but empty — should error
	env := map[string]string{
		"HOST": "",
	}

	_, err := LoadConfig(env)
	if err == nil {
		t.Fatal("expected error for empty HOST, got nil")
	}
}

func TestLoadConfig_InvalidPort(t *testing.T) {
	env := map[string]string{
		"HOST": "localhost",
		"PORT": "not-a-number",
	}

	_, err := LoadConfig(env)
	if err == nil {
		t.Fatal("expected error for invalid PORT, got nil")
	}
}

func TestLoadConfig_MaxRetriesZero(t *testing.T) {
	// MAX_RETRIES=0 is intentional — should be *int pointing to 0, not nil
	env := map[string]string{
		"HOST":        "localhost",
		"MAX_RETRIES": "0",
	}

	cfg, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if cfg.MaxRetries == nil {
		t.Fatal("MaxRetries is nil, want *0 (explicitly set to zero)")
	}
	if *cfg.MaxRetries != 0 {
		t.Errorf("MaxRetries = %d, want 0", *cfg.MaxRetries)
	}
}

func TestLoadConfig_DebugVariants(t *testing.T) {
	tests := []struct {
		input string
		want  bool
	}{
		{"true", true},
		{"false", false},
		{"1", true},
		{"0", false},
	}

	for _, tt := range tests {
		env := map[string]string{
			"HOST":  "localhost",
			"DEBUG": tt.input,
		}

		cfg, err := LoadConfig(env)
		if err != nil {
			t.Errorf("DEBUG=%q: unexpected error: %v", tt.input, err)
			continue
		}
		if cfg.Debug != tt.want {
			t.Errorf("DEBUG=%q: got %v, want %v", tt.input, cfg.Debug, tt.want)
		}
	}
}

func TestLoadConfig_InvalidTimeout(t *testing.T) {
	env := map[string]string{
		"HOST":        "localhost",
		"TIMEOUT_SEC": "abc",
	}

	_, err := LoadConfig(env)
	if err == nil {
		t.Fatal("expected error for invalid TIMEOUT_SEC, got nil")
	}
}

func TestConfig_String(t *testing.T) {
	env := map[string]string{
		"HOST": "localhost",
	}

	cfg, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	s := cfg.String()
	if s == "" {
		t.Error("String() returned empty string")
	}
	// Just verify it doesn't panic and returns something non-empty.
	// Exact format is up to the implementer.
	t.Logf("Config.String() output:\n%s", s)
}
