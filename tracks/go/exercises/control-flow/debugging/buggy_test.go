package main

import "testing"

func TestValidateConfig_ValidConfig(t *testing.T) {
	cfg := Config{
		Port:        8080,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) != 0 {
		t.Errorf("valid config should have 0 errors, got %d: %v", len(errs), errs)
	}
}

func TestValidateConfig_InvalidPort_Negative(t *testing.T) {
	cfg := Config{
		Port:        -1,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) == 0 {
		t.Error("expected port error for negative port")
	}

	foundPortError := false
	for _, err := range errs {
		if err.Field == "Port" {
			foundPortError = true
		}
	}
	if !foundPortError {
		t.Error("expected Port validation error")
	}
}

func TestValidateConfig_InvalidPort_TooLarge(t *testing.T) {
	cfg := Config{
		Port:        70000,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) == 0 {
		t.Error("expected port error for port > 65535")
	}

	foundPortError := false
	for _, err := range errs {
		if err.Field == "Port" {
			foundPortError = true
		}
	}
	if !foundPortError {
		t.Error("expected Port validation error")
	}
}

func TestValidateConfig_InvalidTimeout(t *testing.T) {
	cfg := Config{
		Port:        8080,
		Timeout:     -5,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) == 0 {
		t.Error("expected timeout error for negative timeout")
	}

	foundTimeoutError := false
	for _, err := range errs {
		if err.Field == "Timeout" {
			foundTimeoutError = true
		}
	}
	if !foundTimeoutError {
		t.Error("expected Timeout validation error")
	}
}

func TestValidateConfig_NoAdminEmails(t *testing.T) {
	cfg := Config{
		Port:        8080,
		Timeout:     30,
		AdminEmails: []string{},
	}

	errs := ValidateConfig(cfg)
	if len(errs) == 0 {
		t.Error("expected error for missing admin emails")
	}

	foundEmailError := false
	for _, err := range errs {
		if err.Field == "AdminEmails" {
			foundEmailError = true
		}
	}
	if !foundEmailError {
		t.Error("expected AdminEmails validation error")
	}
}

func TestValidateConfig_InvalidAdminEmail(t *testing.T) {
	cfg := Config{
		Port:        8080,
		Timeout:     30,
		AdminEmails: []string{"notanemail"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) == 0 {
		t.Error("expected error for invalid admin email")
	}

	foundEmailError := false
	for _, err := range errs {
		if err.Field == "AdminEmails" {
			foundEmailError = true
		}
	}
	if !foundEmailError {
		t.Error("expected AdminEmails validation error")
	}
}

func TestValidateConfig_MultipleErrors(t *testing.T) {
	cfg := Config{
		Port:        -1,      // invalid
		Timeout:     -5,      // invalid
		AdminEmails: []string{}, // invalid
	}

	errs := ValidateConfig(cfg)

	// Should report ALL errors, not just the first one
	if len(errs) < 3 {
		t.Errorf("expected 3 errors (port, timeout, emails), got %d: %v", len(errs), errs)
	}
}

func TestValidateConfig_EdgeCase_Port1(t *testing.T) {
	cfg := Config{
		Port:        1,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) != 0 {
		t.Errorf("port 1 should be valid, got errors: %v", errs)
	}
}

func TestValidateConfig_EdgeCase_Port65535(t *testing.T) {
	cfg := Config{
		Port:        65535,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
	}

	errs := ValidateConfig(cfg)
	if len(errs) != 0 {
		t.Errorf("port 65535 should be valid, got errors: %v", errs)
	}
}
