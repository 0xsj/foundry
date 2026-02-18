// Package configvalidator validates service configuration files.
// Run tests: go test -v ./...
package configvalidator

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// ============================================================================
// SENTINEL ERRORS
// ============================================================================

// ErrInvalidConfig is the sentinel for any validation failure.
// errors.Is(err, ErrInvalidConfig) returns true for any *ValidationError.
var ErrInvalidConfig = errors.New("invalid config")

// ============================================================================
// TYPES
// ============================================================================

// Config holds the parsed and validated service configuration.
type Config struct {
	Name    string
	Host    string
	Port    int
	Timeout int      // seconds; default 30 if not set
	Tags    []string // optional
}

// ============================================================================
// ERROR TYPES
// ============================================================================

// FieldError is returned when a field is present but fails a constraint.
// Callers use errors.As to extract the field name and invalid value.
type FieldError struct {
	Field   string
	Value   any
	Message string
}

func (e *FieldError) Error() string {
	return fmt.Sprintf("field %q: %s (got %v)", e.Field, e.Message, e.Value)
}

// MissingFieldError is returned when a required field is absent.
type MissingFieldError struct {
	Field string
}

func (e *MissingFieldError) Error() string {
	return fmt.Sprintf("required field %q is missing", e.Field)
}

// ParseError is returned when the config file cannot be parsed.
type ParseError struct {
	Line    int
	Message string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("parse error at line %d: %s", e.Line, e.Message)
}

// ValidationError wraps all field/missing errors collected during Validate.
//
// Design notes:
// - Stores errors as a joined error internally for proper Unwrap support
// - Implements Is(target) so errors.Is(err, ErrInvalidConfig) works
// - Implements Unwrap() so errors.As can find individual *FieldError, *MissingFieldError
//
// The Is method is the key insight: rather than making ErrInvalidConfig a type,
// we make *ValidationError declare "I am an ErrInvalidConfig". This lets callers
// use a simple sentinel check while the full type is available via errors.As.
type ValidationError struct {
	err error // joined errors from Validate
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("validation failed:\n%v", e.err)
}

// Unwrap lets errors.Is and errors.As traverse the joined errors.
func (e *ValidationError) Unwrap() error {
	return e.err
}

// Is makes errors.Is(err, ErrInvalidConfig) return true for any *ValidationError.
// Without this, callers would have to use errors.As to find *ValidationError first.
func (e *ValidationError) Is(target error) bool {
	return target == ErrInvalidConfig
}

// ============================================================================
// PARSE
// ============================================================================

// Parse reads a key=value config format into a Config.
// Lines starting with # are comments. Empty lines are ignored.
// Default: Timeout = 30 if not specified.
func Parse(data string) (*Config, error) {
	cfg := &Config{Timeout: 30} // default timeout

	for i, raw := range strings.Split(data, "\n") {
		line := strings.TrimSpace(raw)

		// Skip empty lines and comments
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Expect exactly one "=" — SplitN(line, "=", 2) gives at most 2 parts
		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			return nil, &ParseError{
				Line:    i + 1,
				Message: fmt.Sprintf("expected key=value, got %q", line),
			}
		}

		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])

		switch key {
		case "name":
			cfg.Name = value
		case "host":
			cfg.Host = value
		case "port":
			n, err := strconv.Atoi(value)
			if err != nil {
				return nil, &ParseError{
					Line:    i + 1,
					Message: fmt.Sprintf("port must be an integer, got %q", value),
				}
			}
			cfg.Port = n
		case "timeout":
			n, err := strconv.Atoi(value)
			if err != nil {
				return nil, &ParseError{
					Line:    i + 1,
					Message: fmt.Sprintf("timeout must be an integer, got %q", value),
				}
			}
			cfg.Timeout = n
		case "tags":
			// tags=billing,critical → []string{"billing", "critical"}
			for _, tag := range strings.Split(value, ",") {
				if t := strings.TrimSpace(tag); t != "" {
					cfg.Tags = append(cfg.Tags, t)
				}
			}
		}
		// Unknown keys are silently ignored — permissive parsing
	}

	return cfg, nil
}

// ============================================================================
// VALIDATE
// ============================================================================

// Validate checks cfg for all constraint violations.
// Collects ALL errors — does not stop at the first failure.
// Returns nil if valid, *ValidationError containing all failures otherwise.
func Validate(cfg Config) error {
	var errs []error

	// Required: Name
	if cfg.Name == "" {
		errs = append(errs, &MissingFieldError{Field: "name"})
	}

	// Required: Host
	if cfg.Host == "" {
		errs = append(errs, &MissingFieldError{Field: "host"})
	}

	// Required: Port (0 means unset, since Port is not explicitly missing from the type)
	if cfg.Port == 0 {
		errs = append(errs, &MissingFieldError{Field: "port"})
	} else if cfg.Port < 1 || cfg.Port > 65535 {
		errs = append(errs, &FieldError{
			Field:   "port",
			Value:   cfg.Port,
			Message: "must be between 1 and 65535",
		})
	}

	// Timeout: must be positive if set (0 is the "default applied by Parse" case,
	// but Parse sets it to 30, so 0 here means caller constructed Config manually
	// with zero value — treat as invalid)
	if cfg.Timeout <= 0 {
		errs = append(errs, &FieldError{
			Field:   "timeout",
			Value:   cfg.Timeout,
			Message: "must be a positive integer",
		})
	}

	if len(errs) == 0 {
		return nil
	}

	// errors.Join combines all errors. Its Unwrap() []error lets errors.As
	// find *FieldError or *MissingFieldError anywhere in the collection.
	return &ValidationError{err: errors.Join(errs...)}
}

// ============================================================================
// LOAD
// ============================================================================

// fileReader is an injectable dependency so tests can avoid real file I/O.
type fileReader func(path string) ([]byte, error)

// Load reads path using reader, parses, and validates the config.
// Each failure is wrapped with "load <path>:" context.
//
// Error wrapping design:
//   - Reader failure → wrapped with fmt.Errorf("load %q: %w", path, err)
//   - Parse failure  → wrapped with fmt.Errorf("load %q: %w", path, err) — *ParseError accessible via errors.As
//   - Validation failure → wrapped with fmt.Errorf("load %q: %w", path, err) — *ValidationError accessible via errors.Is/As
//
// This means callers can do:
//   errors.Is(err, ErrInvalidConfig)       — was it a validation error?
//   errors.As(err, &fieldErr)              — which field failed?
//   strings.Contains(err.Error(), path)    — which file failed?
func Load(path string, reader fileReader) (*Config, error) {
	data, err := reader(path)
	if err != nil {
		return nil, fmt.Errorf("load %q: %w", path, err)
	}

	cfg, err := Parse(string(data))
	if err != nil {
		return nil, fmt.Errorf("load %q: %w", path, err)
	}

	if err := Validate(*cfg); err != nil {
		return nil, fmt.Errorf("load %q: %w", path, err)
	}

	return cfg, nil
}
