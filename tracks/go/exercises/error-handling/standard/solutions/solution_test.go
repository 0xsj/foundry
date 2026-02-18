package configvalidator

import (
	"errors"
	"strings"
	"testing"
)

// ============================================================================
// Parse tests
// ============================================================================

func TestParse_ValidConfig(t *testing.T) {
	input := `
# service config
name=payment-service
host=0.0.0.0
port=8080
timeout=30
tags=billing,critical
`
	cfg, err := Parse(input)
	if err != nil {
		t.Fatalf("Parse() error = %v, want nil", err)
	}
	if cfg.Name != "payment-service" {
		t.Errorf("Name = %q, want %q", cfg.Name, "payment-service")
	}
	if cfg.Host != "0.0.0.0" {
		t.Errorf("Host = %q, want %q", cfg.Host, "0.0.0.0")
	}
	if cfg.Port != 8080 {
		t.Errorf("Port = %d, want %d", cfg.Port, 8080)
	}
	if cfg.Timeout != 30 {
		t.Errorf("Timeout = %d, want %d", cfg.Timeout, 30)
	}
	if len(cfg.Tags) != 2 || cfg.Tags[0] != "billing" || cfg.Tags[1] != "critical" {
		t.Errorf("Tags = %v, want [billing critical]", cfg.Tags)
	}
}

func TestParse_DefaultTimeout(t *testing.T) {
	input := "name=svc\nhost=localhost\nport=8080\n"
	cfg, err := Parse(input)
	if err != nil {
		t.Fatalf("Parse() error = %v, want nil", err)
	}
	if cfg.Timeout != 30 {
		t.Errorf("Timeout = %d, want default 30", cfg.Timeout)
	}
}

func TestParse_MalformedLine(t *testing.T) {
	input := "name=svc\nthis is not valid\nport=8080\n"
	_, err := Parse(input)
	if err == nil {
		t.Fatal("Parse() error = nil, want *ParseError")
	}
	var parseErr *ParseError
	if !errors.As(err, &parseErr) {
		t.Errorf("errors.As(*ParseError) = false, want true; err = %v", err)
	}
	if parseErr.Line != 2 {
		t.Errorf("ParseError.Line = %d, want 2", parseErr.Line)
	}
}

func TestParse_InvalidPortType(t *testing.T) {
	input := "name=svc\nhost=localhost\nport=notanumber\n"
	_, err := Parse(input)
	if err == nil {
		t.Fatal("Parse() error = nil, want *ParseError")
	}
	var parseErr *ParseError
	if !errors.As(err, &parseErr) {
		t.Errorf("errors.As(*ParseError) = false; err = %v", err)
	}
}

func TestParse_CommentsAndEmptyLines(t *testing.T) {
	input := `
# This is a comment

name=svc

# another comment
host=localhost
port=9090
`
	cfg, err := Parse(input)
	if err != nil {
		t.Fatalf("Parse() error = %v, want nil", err)
	}
	if cfg.Name != "svc" {
		t.Errorf("Name = %q, want %q", cfg.Name, "svc")
	}
}

// ============================================================================
// Validate tests
// ============================================================================

func TestValidate_Valid(t *testing.T) {
	cfg := Config{Name: "svc", Host: "localhost", Port: 8080, Timeout: 30}
	if err := Validate(cfg); err != nil {
		t.Errorf("Validate() error = %v, want nil", err)
	}
}

func TestValidate_MissingName(t *testing.T) {
	cfg := Config{Host: "localhost", Port: 8080, Timeout: 30}
	err := Validate(cfg)
	if err == nil {
		t.Fatal("Validate() error = nil, want error")
	}
	var missing *MissingFieldError
	if !errors.As(err, &missing) {
		t.Errorf("errors.As(*MissingFieldError) = false; err = %v", err)
	}
	if missing.Field != "name" {
		t.Errorf("MissingFieldError.Field = %q, want %q", missing.Field, "name")
	}
}

func TestValidate_MissingMultipleFields(t *testing.T) {
	cfg := Config{Timeout: 30}
	err := Validate(cfg)
	if err == nil {
		t.Fatal("Validate() error = nil, want error")
	}
	var valErr *ValidationError
	if !errors.As(err, &valErr) {
		t.Fatalf("errors.As(*ValidationError) = false; err = %v", err)
	}
	msg := err.Error()
	for _, field := range []string{"name", "host", "port"} {
		if !strings.Contains(msg, field) {
			t.Errorf("error message %q should mention field %q", msg, field)
		}
	}
}

func TestValidate_InvalidPort(t *testing.T) {
	cfg := Config{Name: "svc", Host: "localhost", Port: 99999, Timeout: 30}
	err := Validate(cfg)
	if err == nil {
		t.Fatal("Validate() error = nil, want error")
	}
	var fieldErr *FieldError
	if !errors.As(err, &fieldErr) {
		t.Errorf("errors.As(*FieldError) = false; err = %v", err)
	}
	if fieldErr.Field != "port" {
		t.Errorf("FieldError.Field = %q, want %q", fieldErr.Field, "port")
	}
}

func TestValidate_ErrInvalidConfig(t *testing.T) {
	cfg := Config{Timeout: 30}
	err := Validate(cfg)
	if !errors.Is(err, ErrInvalidConfig) {
		t.Errorf("errors.Is(err, ErrInvalidConfig) = false, want true; err = %v", err)
	}
}

func TestValidate_CollectsAllErrors(t *testing.T) {
	// Port is present but invalid (so MissingFieldError for port is not returned)
	// Name is missing, Host is missing, Port is out of range
	cfg := Config{Port: 99999, Timeout: 30}
	err := Validate(cfg)
	if err == nil {
		t.Fatal("Validate() error = nil, want error")
	}

	msg := err.Error()
	// Should mention name missing and host missing and port invalid
	if !strings.Contains(msg, "name") {
		t.Errorf("error should mention 'name'; got: %s", msg)
	}
	if !strings.Contains(msg, "host") {
		t.Errorf("error should mention 'host'; got: %s", msg)
	}
	if !strings.Contains(msg, "port") {
		t.Errorf("error should mention 'port'; got: %s", msg)
	}
}

// ============================================================================
// Load tests
// ============================================================================

func TestLoad_Success(t *testing.T) {
	reader := func(path string) ([]byte, error) {
		return []byte("name=svc\nhost=localhost\nport=8080\n"), nil
	}
	cfg, err := Load("config.yaml", reader)
	if err != nil {
		t.Fatalf("Load() error = %v, want nil", err)
	}
	if cfg.Name != "svc" {
		t.Errorf("Name = %q, want %q", cfg.Name, "svc")
	}
}

func TestLoad_FileReadError(t *testing.T) {
	readErr := errors.New("no such file")
	reader := func(path string) ([]byte, error) {
		return nil, readErr
	}
	_, err := Load("missing.yaml", reader)
	if err == nil {
		t.Fatal("Load() error = nil, want error")
	}
	if !errors.Is(err, readErr) {
		t.Errorf("errors.Is(err, readErr) = false; err = %v", err)
	}
}

func TestLoad_ParseError(t *testing.T) {
	reader := func(path string) ([]byte, error) {
		return []byte("name=svc\nbad line here\nport=8080\n"), nil
	}
	_, err := Load("config.yaml", reader)
	if err == nil {
		t.Fatal("Load() error = nil, want error")
	}
	var parseErr *ParseError
	if !errors.As(err, &parseErr) {
		t.Errorf("errors.As(*ParseError) = false; err = %v", err)
	}
}

func TestLoad_ValidationError(t *testing.T) {
	reader := func(path string) ([]byte, error) {
		return []byte("name=svc\ntimeout=30\n"), nil
	}
	_, err := Load("config.yaml", reader)
	if err == nil {
		t.Fatal("Load() error = nil, want error")
	}
	if !errors.Is(err, ErrInvalidConfig) {
		t.Errorf("errors.Is(err, ErrInvalidConfig) = false; err = %v", err)
	}
}

func TestLoad_ErrorContainsPath(t *testing.T) {
	reader := func(path string) ([]byte, error) {
		return []byte("bad line"), nil
	}
	_, err := Load("production.yaml", reader)
	if err == nil {
		t.Fatal("Load() error = nil, want error")
	}
	if !strings.Contains(err.Error(), "production.yaml") {
		t.Errorf("error %q should contain path %q", err.Error(), "production.yaml")
	}
}

func TestLoad_CanExtractFieldError(t *testing.T) {
	// Port out of range — Load wraps ValidationError which wraps FieldError
	reader := func(path string) ([]byte, error) {
		return []byte("name=svc\nhost=localhost\nport=99999\n"), nil
	}
	_, err := Load("config.yaml", reader)
	if err == nil {
		t.Fatal("Load() error = nil, want error")
	}

	var fieldErr *FieldError
	if !errors.As(err, &fieldErr) {
		t.Errorf("errors.As(*FieldError) through Load wrapping = false; err = %v", err)
	}
}
