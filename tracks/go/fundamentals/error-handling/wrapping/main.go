// Package main demonstrates error wrapping, unwrapping, and the errors.Is/As API.
//
// Run: go run ./wrapping/
package main

import (
	"errors"
	"fmt"
	"os"
)

// ============================================================================
// Sentinel errors — package-level variables representing known conditions
// ============================================================================

var (
	ErrNotFound    = errors.New("not found")
	ErrUnauthorized = errors.New("unauthorized")
	ErrConflict    = errors.New("conflict")
)

// ============================================================================
// Error wrapping with %w
//
// %w wraps the original error, preserving it in the chain.
// %v does NOT wrap — it creates a new error string, severing the chain.
// ============================================================================

// findRecord simulates looking up a record by ID.
func findRecord(id string) (string, error) {
	if id == "missing" {
		return "", ErrNotFound // return sentinel directly
	}
	if id == "secret" {
		return "", ErrUnauthorized
	}
	return "record:" + id, nil
}

// loadRecord wraps errors from findRecord with context.
func loadRecord(id string) (string, error) {
	record, err := findRecord(id)
	if err != nil {
		// %w wraps — errors.Is(result, ErrNotFound) will be true
		return "", fmt.Errorf("loadRecord %q: %w", id, err)
	}
	return record, nil
}

// processRecord wraps errors from loadRecord with more context.
func processRecord(id string) (string, error) {
	record, err := loadRecord(id)
	if err != nil {
		// Each layer adds context: processRecord → loadRecord → ErrNotFound
		return "", fmt.Errorf("processRecord: %w", err)
	}
	return "[processed] " + record, nil
}

// ============================================================================
// errors.Is — test for a specific error anywhere in the chain
// ============================================================================

func demonstrateIs() {
	fmt.Println("=== errors.Is ===")

	// Direct sentinel — errors.Is works just like ==
	err := ErrNotFound
	fmt.Printf("errors.Is(ErrNotFound, ErrNotFound): %v\n", errors.Is(err, ErrNotFound)) // true

	// Wrapped once
	wrapped1 := fmt.Errorf("layer1: %w", ErrNotFound)
	fmt.Printf("errors.Is(wrapped1, ErrNotFound): %v\n", errors.Is(wrapped1, ErrNotFound)) // true

	// Wrapped twice — still finds it
	wrapped2 := fmt.Errorf("layer2: %w", wrapped1)
	fmt.Printf("errors.Is(wrapped2, ErrNotFound): %v\n", errors.Is(wrapped2, ErrNotFound)) // true

	// Wrong sentinel — not in chain
	fmt.Printf("errors.Is(wrapped2, ErrUnauthorized): %v\n", errors.Is(wrapped2, ErrUnauthorized)) // false

	// %v does NOT wrap — chain is severed
	severed := fmt.Errorf("lost: %v", ErrNotFound) // note: %v not %w
	fmt.Printf("errors.Is(severed, ErrNotFound): %v\n", errors.Is(severed, ErrNotFound)) // false!
}

// ============================================================================
// errors.As — extract a typed error from the chain
// ============================================================================

// DatabaseError carries structured information about a database failure.
type DatabaseError struct {
	Query   string
	Message string
	Cause   error
}

func (e *DatabaseError) Error() string {
	return fmt.Sprintf("database error on %q: %s", e.Query, e.Message)
}

// Unwrap lets errors.Is/As traverse through DatabaseError
func (e *DatabaseError) Unwrap() error {
	return e.Cause
}

// queryUser simulates a database query failure.
func queryUser(userID string) (string, error) {
	if userID == "bad" {
		return "", &DatabaseError{
			Query:   "SELECT * FROM users WHERE id = ?",
			Message: "connection reset by peer",
			Cause:   os.ErrDeadlineExceeded, // simulate a timeout
		}
	}
	return "Alice", nil
}

// getUserDisplay wraps the database error with request-level context.
func getUserDisplay(userID string) (string, error) {
	name, err := queryUser(userID)
	if err != nil {
		return "", fmt.Errorf("getUserDisplay: %w", err)
	}
	return name, nil
}

func demonstrateAs() {
	fmt.Println("\n=== errors.As ===")

	_, err := getUserDisplay("bad")
	fmt.Printf("Error: %v\n\n", err)
	// Output: getUserDisplay: database error on "SELECT ...": connection reset by peer

	// errors.As finds *DatabaseError anywhere in the chain
	var dbErr *DatabaseError
	if errors.As(err, &dbErr) {
		fmt.Printf("Found DatabaseError:\n")
		fmt.Printf("  Query: %s\n", dbErr.Query)
		fmt.Printf("  Message: %s\n", dbErr.Message)
	}

	// errors.Is can also find the cause through DatabaseError.Unwrap()
	fmt.Printf("\nerrors.Is(err, os.ErrDeadlineExceeded): %v\n",
		errors.Is(err, os.ErrDeadlineExceeded))
}

// ============================================================================
// errors.Join — aggregate multiple errors (Go 1.20+)
// ============================================================================

func validateConfig(host string, port int, timeout int) error {
	var errs []error

	if host == "" {
		errs = append(errs, errors.New("host is required"))
	}
	if port < 1 || port > 65535 {
		errs = append(errs, fmt.Errorf("port %d out of range (1-65535)", port))
	}
	if timeout <= 0 {
		errs = append(errs, errors.New("timeout must be positive"))
	}

	return errors.Join(errs...) // nil if no errors
}

func demonstrateJoin() {
	fmt.Println("\n=== errors.Join ===")

	// All fields invalid
	err := validateConfig("", 99999, -1)
	if err != nil {
		fmt.Printf("Validation failed:\n%v\n", err)
		// Output:
		// host is required
		// port 99999 out of range (1-65535)
		// timeout must be positive
	}

	// All fields valid — returns nil
	err = validateConfig("localhost", 8080, 30)
	fmt.Printf("Valid config error: %v\n", err) // <nil>
}

// ============================================================================
// Full chain demonstration
// ============================================================================

func fullChain() {
	fmt.Println("\n=== Full chain: processRecord ===")

	// Success case
	result, err := processRecord("rec-42")
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Println("Result:", result)
	}

	// Error case — chain: processRecord → loadRecord → ErrNotFound
	_, err = processRecord("missing")
	if err != nil {
		fmt.Println("\nWrapped error:", err)
		// processRecord: loadRecord "missing": not found

		fmt.Printf("errors.Is ErrNotFound: %v\n", errors.Is(err, ErrNotFound))
		fmt.Printf("errors.Is ErrUnauthorized: %v\n", errors.Is(err, ErrUnauthorized))
	}
}

func main() {
	demonstrateIs()
	demonstrateAs()
	demonstrateJoin()
	fullChain()
}
