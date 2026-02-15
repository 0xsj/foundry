package main

import (
	"fmt"
	"strings"
)

// Config represents service configuration
type Config struct {
	Port         int
	Timeout      int
	AdminEmails  []string
	EnableDebug  bool
}

// ValidationError represents a configuration validation error
type ValidationError struct {
	Field   string
	Message string
}

func (e ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

// ValidateConfig checks if configuration is valid
// Returns slice of validation errors (empty if valid)
func ValidateConfig(cfg Config) []ValidationError {
	var errors []ValidationError

	// BUG: Port range check is incorrect
	// Port should be 1-65535, but this checks 0-65534
	if cfg.Port < 0 && cfg.Port > 65535 {
		errors = append(errors, ValidationError{
			Field:   "Port",
			Message: "must be between 1 and 65535",
		})
	}

	// BUG: Early return prevents checking other fields
	// Should collect all errors, not return on first one
	if cfg.Timeout <= 0 {
		return []ValidationError{{
			Field:   "Timeout",
			Message: "must be positive",
		}}
	}

	// BUG: Loop exits on first valid email, doesn't check if ANY are valid
	// This means if the first email is valid, it returns even if others are invalid
	for i := range cfg.AdminEmails {
		if strings.Contains(cfg.AdminEmails[i], "@") {
			break // BUG: Should be checking if at least ONE is valid, not breaking early
		}
	}

	// BUG: This check happens even if we found a valid email above
	// Should track whether we found a valid email
	if len(cfg.AdminEmails) == 0 {
		errors = append(errors, ValidationError{
			Field:   "AdminEmails",
			Message: "at least one admin email required",
		})
	}

	return errors
}

func main() {
	// Test with a valid config
	validConfig := Config{
		Port:        8080,
		Timeout:     30,
		AdminEmails: []string{"admin@example.com"},
		EnableDebug: false,
	}

	if errs := ValidateConfig(validConfig); len(errs) > 0 {
		fmt.Println("Valid config rejected (BUG):")
		for _, err := range errs {
			fmt.Printf("  - %s\n", err)
		}
	} else {
		fmt.Println("✓ Valid config accepted")
	}

	// Test with invalid config
	invalidConfig := Config{
		Port:        -1,      // invalid
		Timeout:     -5,      // invalid
		AdminEmails: []string{}, // invalid
	}

	if errs := ValidateConfig(invalidConfig); len(errs) > 0 {
		fmt.Printf("✓ Invalid config rejected with %d errors:\n", len(errs))
		for _, err := range errs {
			fmt.Printf("  - %s\n", err)
		}
	} else {
		fmt.Println("Invalid config accepted (BUG)")
	}
}
