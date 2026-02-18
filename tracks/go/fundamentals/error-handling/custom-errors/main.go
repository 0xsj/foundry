// Package main demonstrates custom error types, sentinel errors, panic/recover,
// and a realistic package-level error design.
//
// Run: go run ./custom-errors/
package main

import (
	"errors"
	"fmt"
)

// ============================================================================
// A realistic package-level error design
//
// A config package typically has:
// - Sentinel errors for simple "did this happen?" conditions
// - Custom types for errors that carry data the caller needs
// ============================================================================

// Sentinel errors — exported so callers can test with errors.Is
var (
	ErrNotFound     = errors.New("not found")
	ErrUnauthorized = errors.New("unauthorized")
	ErrConflict     = errors.New("conflict: resource already exists")
)

// FieldError is returned when a specific field fails validation.
// The caller can use errors.As to extract the field name and message.
type FieldError struct {
	Field   string
	Value   any
	Message string
}

func (e *FieldError) Error() string {
	return fmt.Sprintf("field %q: %s (got %v)", e.Field, e.Message, e.Value)
}

// ParseError is returned when a config file cannot be parsed.
type ParseError struct {
	Line    int
	Column  int
	Message string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("parse error at line %d, col %d: %s", e.Line, e.Column, e.Message)
}

// ConfigError wraps lower-level errors with the config file path.
// It implements Unwrap so errors.Is/As can traverse the chain.
type ConfigError struct {
	Path  string
	Cause error
}

func (e *ConfigError) Error() string {
	return fmt.Sprintf("config %q: %v", e.Path, e.Cause)
}

func (e *ConfigError) Unwrap() error {
	return e.Cause
}

// ============================================================================
// Functions returning custom errors
// ============================================================================

func parseField(name, value string) error {
	if value == "" {
		return &FieldError{Field: name, Value: value, Message: "cannot be empty"}
	}
	if len(value) > 100 {
		return &FieldError{Field: name, Value: value[:20] + "...", Message: "exceeds 100 characters"}
	}
	return nil
}

func loadConfigFile(path string) error {
	if path == "bad.yaml" {
		// Wrap a ParseError in a ConfigError
		return &ConfigError{
			Path: path,
			Cause: &ParseError{
				Line:    42,
				Column:  7,
				Message: "unexpected token ':'",
			},
		}
	}
	if path == "missing.yaml" {
		return &ConfigError{
			Path:  path,
			Cause: ErrNotFound,
		}
	}
	return nil
}

// ============================================================================
// Using errors.As to extract typed errors
// ============================================================================

func demonstrateCustomErrors() {
	fmt.Println("=== Custom error types ===")

	// FieldError
	if err := parseField("email", ""); err != nil {
		fmt.Printf("Raw error: %v\n", err)

		var fieldErr *FieldError
		if errors.As(err, &fieldErr) {
			fmt.Printf("  Field: %s\n", fieldErr.Field)
			fmt.Printf("  Message: %s\n", fieldErr.Message)
		}
	}

	fmt.Println()

	// ConfigError wrapping ParseError — chain traversal
	if err := loadConfigFile("bad.yaml"); err != nil {
		fmt.Printf("Raw error: %v\n", err)

		// Extract ConfigError
		var configErr *ConfigError
		if errors.As(err, &configErr) {
			fmt.Printf("  Config path: %s\n", configErr.Path)
		}

		// Extract ParseError from deep in the chain
		var parseErr *ParseError
		if errors.As(err, &parseErr) {
			fmt.Printf("  Parse line: %d, col: %d\n", parseErr.Line, parseErr.Column)
		}
	}

	fmt.Println()

	// ConfigError wrapping sentinel — errors.Is works through chain
	if err := loadConfigFile("missing.yaml"); err != nil {
		fmt.Printf("Raw error: %v\n", err)
		fmt.Printf("  errors.Is ErrNotFound: %v\n", errors.Is(err, ErrNotFound))
	}
}

// ============================================================================
// panic / recover
//
// panic: for programming errors and unrecoverable conditions
// recover: to prevent panics from crashing the whole process
// mustXxx pattern: panics on init, only for static/hardcoded values
// ============================================================================

// mustPositive panics if n <= 0. Used for compile-time constants
// where a negative value indicates a programming error, not user error.
func mustPositive(n int, name string) int {
	if n <= 0 {
		panic(fmt.Sprintf("mustPositive: %s must be > 0, got %d", name, n))
	}
	return n
}

// safeCall executes fn and catches any panic, returning it as an error.
// This pattern is common in HTTP servers: catch panics from handlers
// so one bad request doesn't kill the whole server.
func safeCall(fn func()) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("panic: %v", r)
		}
	}()
	fn()
	return nil
}

func demonstratePanic() {
	fmt.Println("\n=== panic / recover ===")

	// mustPositive used correctly — static value, won't panic
	workerCount := mustPositive(5, "workerCount")
	fmt.Printf("workerCount: %d\n", workerCount)

	// safeCall catching a panic
	err := safeCall(func() {
		panic("something terrible happened")
	})
	fmt.Printf("safeCall caught: %v\n", err)

	// safeCall with no panic — returns nil
	err = safeCall(func() {
		fmt.Println("function ran successfully")
	})
	fmt.Printf("safeCall no panic: %v\n", err)

	// WRONG: panic for a user error (demonstrating what NOT to do)
	// func parseUserInput(s string) int {
	//     n, err := strconv.Atoi(s)
	//     if err != nil {
	//         panic("invalid input") // BAD: library/user-facing code should return error
	//     }
	//     return n
	// }
}

// ============================================================================
// Pointer vs value receiver — always use pointer receiver on error types
// ============================================================================

// ValueReceiverError (wrong way — for demonstration)
type ValueReceiverError struct {
	Message string
}

func (e ValueReceiverError) Error() string { return e.Message } // value receiver

// PointerReceiverError (right way)
type PointerReceiverError struct {
	Message string
}

func (e *PointerReceiverError) Error() string { return e.Message } // pointer receiver

func demonstrateReceivers() {
	fmt.Println("\n=== Pointer vs value receiver ===")

	// Value receiver: errors.As with *ValueReceiverError won't match the value
	err1 := ValueReceiverError{Message: "value receiver error"}
	var vErr *ValueReceiverError
	matched := errors.As(err1, &vErr)
	fmt.Printf("errors.As with value receiver error, looking for pointer: %v\n", matched)
	// false — we returned ValueReceiverError (value), but target is *ValueReceiverError

	// Pointer receiver: errors.As works correctly
	err2 := &PointerReceiverError{Message: "pointer receiver error"}
	var pErr *PointerReceiverError
	matched = errors.As(err2, &pErr)
	fmt.Printf("errors.As with pointer receiver error, looking for pointer: %v\n", matched)
	// true — we returned *PointerReceiverError, target is *PointerReceiverError
}

func main() {
	demonstrateCustomErrors()
	demonstratePanic()
	demonstrateReceivers()
}
