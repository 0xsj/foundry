// Package utils provides shared utility functions for the platform team.
// This is the proposed version — find the issues before reading expert-review.md.
package utils

import (
	"fmt"
	"strings"
	"time"
)

// ============================================================================
// FUNCTION 1: Transform
//
// Intended to be a general-purpose "apply function to every element" helper.
// ============================================================================

// Transform applies fn to every element of in and returns the results.
func Transform[A, B, C any](in []A, fn func(A) B, postProcess func(B) C) []C {
	result := make([]C, len(in))
	for i, v := range in {
		result[i] = postProcess(fn(v))
	}
	return result
}

// ============================================================================
// FUNCTION 2: ValidateAll
//
// Runs a list of validators against a value and collects all errors.
// Validators are expressed as generic functions.
// ============================================================================

// Validator is a function that validates a value and returns an error or nil.
type Validator[T any] func(T) error

// ValidateAll runs all validators against value and returns all errors found.
func ValidateAll[T any](value T, validators ...Validator[T]) []error {
	var errs []error
	for _, v := range validators {
		if err := v(value); err != nil {
			errs = append(errs, err)
		}
	}
	return errs
}

// ============================================================================
// FUNCTION 3: LookupOrDefault
//
// Returns a value from a map, or a default if the key is absent.
// ============================================================================

// LookupOrDefault returns m[key] if present, otherwise def.
func LookupOrDefault[K any, V any](m map[K]V, key K, def V) V {
	if v, ok := m[key]; ok {
		return v
	}
	return def
}

// ============================================================================
// FUNCTION 4: FormatDuration
//
// Formats a duration-like value as a human-readable string.
// Generic so it works with time.Duration and "any numeric type".
// ============================================================================

// Numeric is a constraint for all number types.
type Numeric interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 |
		~float32 | ~float64
}

// FormatDuration formats a duration-like numeric value as a human-readable string.
// val is assumed to be in milliseconds.
func FormatDuration[T Numeric](val T) string {
	ms := float64(val)
	switch {
	case ms < 1000:
		return fmt.Sprintf("%.0fms", ms)
	case ms < 60000:
		return fmt.Sprintf("%.1fs", ms/1000)
	default:
		return fmt.Sprintf("%.1fm", ms/60000)
	}
}

// ============================================================================
// USAGE EXAMPLES (not part of the PR, shown for context)
// ============================================================================

func exampleUsage() {
	// Transform: the caller ALWAYS uses B=C in practice — the postProcess
	// is always an identity function or a simple string conversion
	names := []string{"alice", "bob", "carol"}
	_ = Transform(names,
		strings.ToUpper,
		func(s string) string { return "[" + s + "]" },
	)

	// ValidateAll in practice — T is always a concrete struct
	type Config struct {
		Host    string
		Timeout time.Duration
	}
	cfg := Config{Host: "localhost", Timeout: 5 * time.Second}
	errs := ValidateAll(cfg,
		func(c Config) error {
			if c.Host == "" {
				return fmt.Errorf("host required")
			}
			return nil
		},
		func(c Config) error {
			if c.Timeout <= 0 {
				return fmt.Errorf("timeout must be positive")
			}
			return nil
		},
	)
	_ = errs

	// LookupOrDefault in practice — always called with string keys
	features := map[string]bool{"dark_mode": true}
	_ = LookupOrDefault(features, "dark_mode", false)
	_ = LookupOrDefault(features, "new_ui", false)

	// FormatDuration in practice — always called with time.Duration or int
	_ = FormatDuration(1500)                 // "1.5s"
	_ = FormatDuration(time.Second * 2 / 1e6) // note: time.Duration is int64 nanoseconds
	_ = FormatDuration(250)                  // "250ms"
}
