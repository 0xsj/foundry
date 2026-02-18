// Package main demonstrates the fundamentals of Go error handling:
// creating errors, returning them, checking for nil, and the error interface.
//
// Run: go run ./errors-basics/
package main

import (
	"errors"
	"fmt"
	"strconv"
)

// ============================================================================
// The error interface
//
// error is defined as:
//   type error interface { Error() string }
//
// Any type with an Error() string method satisfies it.
// nil error means "no error".
// ============================================================================

// divide returns (result, error). On success, error is nil.
// On failure, error is non-nil and result is the zero value.
func divide(a, b float64) (float64, error) {
	if b == 0 {
		return 0, errors.New("division by zero")
	}
	return a / b, nil
}

// ============================================================================
// errors.New vs fmt.Errorf
//
// errors.New: fixed message, simple errors
// fmt.Errorf: formatted message, can include values
// ============================================================================

// parsePort converts a string to a port number (1–65535).
// Uses fmt.Errorf to include the bad value in the message.
func parsePort(s string) (int, error) {
	n, err := strconv.Atoi(s)
	if err != nil {
		// fmt.Errorf includes context — the caller knows what value caused the issue
		return 0, fmt.Errorf("parsePort: %q is not a number", s)
	}
	if n < 1 || n > 65535 {
		return 0, fmt.Errorf("parsePort: %d is out of range (1–65535)", n)
	}
	return n, nil
}

// ============================================================================
// The early return pattern
//
// Check errors immediately, return early on failure.
// The happy path proceeds without indentation.
// ============================================================================

// processValue parses a port and divides 1000 by it.
// Demonstrates chaining two operations with early returns.
func processValue(portStr string, divisor float64) (float64, error) {
	port, err := parsePort(portStr)
	if err != nil {
		// Return early: add context about what THIS function was doing
		return 0, fmt.Errorf("processValue: %w", err)
	}

	result, err := divide(float64(port), divisor)
	if err != nil {
		return 0, fmt.Errorf("processValue: %w", err)
	}

	return result, nil
}

// ============================================================================
// Ignoring errors — when it's OK and when it's not
// ============================================================================

func ignoringErrors() {
	// OK to ignore: fmt.Println's Write rarely fails in practice
	// and there's nothing useful to do if it does
	fmt.Println("hello") // Println returns (n int, err error) — we ignore both

	// OK to ignore: writing to a strings.Builder never fails
	// var sb strings.Builder
	// sb.WriteString("hello")  // (int, error) — error is always nil

	// NOT OK: ignoring a real error
	// result, _ := os.ReadFile("config.json")  // _ discards the error — dangerous
	// Using result without knowing if err != nil leads to bugs

	// The _ assignment makes the ignore explicit and visible in code review
}

// ============================================================================
// Error equality: each errors.New call is a distinct value
// ============================================================================

var (
	// These are package-level sentinel errors — we'll cover them more in custom-errors
	errA = errors.New("something failed")
	errB = errors.New("something failed") // same string, different value
)

func errorEquality() {
	fmt.Printf("errA == errB: %v\n", errA == errB)   // false — different allocations
	fmt.Printf("errA == errA: %v\n", errA == errA)   // true — same variable
	fmt.Printf("errA is errA: %v\n", errors.Is(errA, errA)) // true
	fmt.Printf("errA is errB: %v\n", errors.Is(errA, errB)) // false
}

func main() {
	fmt.Println("=== divide ===")
	result, err := divide(10, 3)
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Printf("10 / 3 = %.4f\n", result)
	}

	_, err = divide(5, 0)
	if err != nil {
		fmt.Println("Error:", err) // division by zero
	}

	fmt.Println("\n=== parsePort ===")
	cases := []string{"8080", "99999", "abc", "443"}
	for _, c := range cases {
		port, err := parsePort(c)
		if err != nil {
			fmt.Printf("  parsePort(%q): %v\n", c, err)
		} else {
			fmt.Printf("  parsePort(%q): %d\n", c, port)
		}
	}

	fmt.Println("\n=== processValue (chained errors) ===")
	v, err := processValue("8080", 4)
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Printf("result: %.2f\n", v)
	}

	_, err = processValue("abc", 4)
	if err != nil {
		fmt.Println("Error:", err)
		// Output: processValue: parsePort: "abc" is not a number
		// The error chain: processValue → parsePort → original
	}

	fmt.Println("\n=== error equality ===")
	errorEquality()

	ignoringErrors()
}
