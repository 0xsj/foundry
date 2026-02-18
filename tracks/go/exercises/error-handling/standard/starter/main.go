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
// errors.Is(err, ErrInvalidConfig) returns true when err is or wraps a *ValidationError.
var ErrInvalidConfig = errors.New("invalid config")

// ============================================================================
// TYPES — provided, do not change
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
// Implement the error interface (Error() string) on each.
// Use pointer receivers. Implement Unwrap() where applicable.
// ============================================================================

// FieldError is returned when a field is present but has an invalid value.
type FieldError struct {
	Field   string
	Value   any
	Message string
}

// TODO: implement Error() string on *FieldError
// Format: `field "name": message (got value)`
func (e *FieldError) Error() string {
	// TODO: replace this stub with a real implementation
	return fmt.Sprintf("field %q: %s (got %v)", e.Field, e.Message, e.Value)
}

// MissingFieldError is returned when a required field is absent.
type MissingFieldError struct {
	Field string
}

// TODO: implement Error() string on *MissingFieldError
// Format: `required field "name" is missing`
func (e *MissingFieldError) Error() string {
	// TODO: replace this stub with a real implementation
	return fmt.Sprintf("required field %q is missing", e.Field)
}

// ParseError is returned when the config file cannot be parsed.
type ParseError struct {
	Line    int
	Message string
}

// TODO: implement Error() string on *ParseError
// Format: `parse error at line N: message`
func (e *ParseError) Error() string {
	// TODO: replace this stub with a real implementation
	return fmt.Sprintf("parse error at line %d: %s", e.Line, e.Message)
}

// ValidationError wraps all field/missing errors collected during Validate.
// errors.Is(err, ErrInvalidConfig) must return true for any *ValidationError.
type ValidationError struct {
	// TODO: store the joined errors
	// Hint: use an unexported `err error` field
	err error
}

// TODO: implement Error() string on *ValidationError
func (e *ValidationError) Error() string {
	// TODO: replace this stub
	return "validation failed"
}

// TODO: implement Unwrap() error on *ValidationError (returns the inner joined error)
func (e *ValidationError) Unwrap() error {
	// TODO: return the inner err field
	return e.err
}

// TODO: implement Is(target error) bool so errors.Is(err, ErrInvalidConfig) works
func (e *ValidationError) Is(target error) bool {
	// TODO: return true when target == ErrInvalidConfig
	return false
}

// ============================================================================
// PARSE
// Parse reads a key=value config format.
// Lines starting with # are comments. Empty lines are ignored.
// Returns *ParseError on malformed lines.
// ============================================================================

// Parse parses a key=value config string into a Config.
// Default: Timeout = 30 if not specified.
func Parse(data string) (*Config, error) {
	// TODO: Implement
	// - Split on newlines
	// - Skip empty lines and lines starting with "#"
	// - Split each line on "=" (first occurrence only — use SplitN)
	// - Return *ParseError if a line doesn't have exactly key=value
	// - Populate cfg fields: name, host, port, timeout, tags
	// - tags: comma-separated values, e.g. "billing,critical"
	// - Return *ParseError if port or timeout can't be parsed as int
	_ = strconv.Atoi   // you'll need this
	_ = strings.Split  // and this
	_ = strings.SplitN // and this
	_ = fmt.Sprintf    // and this
	return nil, errors.New("not implemented")
}

// ============================================================================
// VALIDATE
// Validate checks a parsed Config for all constraint violations.
// It MUST collect ALL errors, not stop at the first.
// Returns nil if valid, *ValidationError if any fields fail.
// ============================================================================

// Validate checks cfg for all constraint violations.
func Validate(cfg Config) error {
	// TODO: Implement
	// Collect errors into a []error slice, then:
	//   - *MissingFieldError if Name is ""
	//   - *MissingFieldError if Host is ""
	//   - *MissingFieldError if Port is 0
	//   - *FieldError if Port != 0 && (Port < 1 || Port > 65535)
	//   - *FieldError if Timeout <= 0 (only if explicitly set — hint: 0 means unset, Timeout defaults to 30 in Parse)
	// If no errors: return nil
	// If errors: return &ValidationError{...} with errors.Join(errs...)
	return errors.New("not implemented")
}

// ============================================================================
// LOAD
// Load reads a file, parses it, and validates it.
// Each step wraps errors with context using fmt.Errorf("load: %w", err).
// ============================================================================

// fileReader is an injectable dependency so tests can avoid real file I/O.
// In production, pass os.ReadFile. In tests, pass a fake.
type fileReader func(path string) ([]byte, error)

// Load reads path using reader, parses, and validates the config.
// Wraps all errors with "load [path]: " context.
func Load(path string, reader fileReader) (*Config, error) {
	// TODO: Implement
	// 1. Call reader(path) — wrap error as fmt.Errorf("load %q: %w", path, err)
	// 2. Call Parse(string(data)) — wrap error as fmt.Errorf("load %q: %w", path, err)
	// 3. Call Validate(*cfg) — wrap error as fmt.Errorf("load %q: %w", path, err)
	// 4. Return cfg, nil on success
	return nil, errors.New("not implemented")
}
