package config

import (
	"testing"
	"time"
)

func TestLoadConfig_AllDefaults(t *testing.T) {
	env := map[string]string{}

	got, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	want := Config{
		Host:           "localhost",
		Port:           8080,
		Timeout:        30 * time.Second,
		MaxConnections: 100,
	}

	if got != want {
		t.Errorf("got %+v, want %+v", got, want)
	}
}

func TestLoadConfig_CustomValues(t *testing.T) {
	env := map[string]string{
		"HOST":            "0.0.0.0",
		"PORT":            "3000",
		"TIMEOUT":         "60s",
		"MAX_CONNECTIONS": "200",
	}

	got, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if got.Host != "0.0.0.0" {
		t.Errorf("Host: got %q, want %q", got.Host, "0.0.0.0")
	}
	if got.Port != 3000 {
		t.Errorf("Port: got %d, want %d", got.Port, 3000)
	}
	if got.Timeout != 60*time.Second {
		t.Errorf("Timeout: got %v, want %v", got.Timeout, 60*time.Second)
	}
	if got.MaxConnections != 200 {
		t.Errorf("MaxConnections: got %d, want %d", got.MaxConnections, 200)
	}
}

func TestLoadConfig_InvalidPort(t *testing.T) {
	tests := []struct {
		name string
		port string
	}{
		{"negative", "-1"},
		{"zero", "0"},
		{"too large", "70000"},
		{"not a number", "abc"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			env := map[string]string{"PORT": tt.port}
			_, err := LoadConfig(env)
			if err == nil {
				t.Errorf("expected error for port %q, got nil", tt.port)
			}
		})
	}
}

func TestLoadConfig_InvalidTimeout(t *testing.T) {
	env := map[string]string{"TIMEOUT": "not-a-duration"}

	_, err := LoadConfig(env)
	if err == nil {
		t.Error("expected error for invalid timeout, got nil")
	}
}

func TestLoadConfig_PartialOverride(t *testing.T) {
	env := map[string]string{
		"PORT": "9000",
	}

	got, err := LoadConfig(env)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Port should be overridden, others should be defaults
	if got.Port != 9000 {
		t.Errorf("Port: got %d, want %d", got.Port, 9000)
	}
	if got.Host != "localhost" {
		t.Errorf("Host: got %q, want %q (default)", got.Host, "localhost")
	}
	if got.Timeout != 30*time.Second {
		t.Errorf("Timeout: got %v, want %v (default)", got.Timeout, 30*time.Second)
	}
}
