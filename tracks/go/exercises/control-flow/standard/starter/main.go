package main

import (
	"errors"
	"fmt"
	"time"
)

// Operation is a function that might fail
type Operation func() error

// RetryableError wraps errors that can be retried
type RetryableError struct {
	Err error
}

func (e *RetryableError) Error() string {
	return e.Err.Error()
}

// Retry attempts an operation multiple times with exponential backoff
func Retry(op Operation, maxRetries int) error {
	// TODO: Implement retry logic with exponential backoff
	// 1. Validate inputs (guard clauses)
	// 2. Loop up to maxRetries + 1 times (initial + retries)
	// 3. Try the operation
	// 4. On success, return nil immediately
	// 5. On non-retryable error, return error immediately
	// 6. On retryable error, calculate backoff and sleep
	// 7. If all retries exhausted, return final error
	return fmt.Errorf("not implemented")
}

func main() {
	fmt.Println("=== Retry Mechanism Demo ===\n")

	// Example 1: Operation that succeeds immediately
	fmt.Println("Test 1: Immediate success")
	err := Retry(func() error {
		fmt.Println("  Attempting...")
		return nil
	}, 3)
	if err != nil {
		fmt.Println("  Failed:", err)
	} else {
		fmt.Println("  ✓ Success")
	}

	// Example 2: Operation that fails then succeeds
	fmt.Println("\nTest 2: Fails twice, then succeeds")
	attempts := 0
	err = Retry(func() error {
		attempts++
		fmt.Printf("  Attempt %d...\n", attempts)
		if attempts < 3 {
			return &RetryableError{Err: fmt.Errorf("temporary failure")}
		}
		return nil
	}, 5)
	if err != nil {
		fmt.Println("  Failed:", err)
	} else {
		fmt.Println("  ✓ Success after", attempts, "attempts")
	}

	// Example 3: Non-retryable error
	fmt.Println("\nTest 3: Non-retryable error")
	attempts = 0
	err = Retry(func() error {
		attempts++
		fmt.Printf("  Attempt %d...\n", attempts)
		return fmt.Errorf("permanent failure")  // Not wrapped in RetryableError
	}, 5)
	if err != nil {
		fmt.Println("  Failed after", attempts, "attempts:", err)
	}

	// Example 4: All retries exhausted
	fmt.Println("\nTest 4: All retries exhausted")
	attempts = 0
	err = Retry(func() error {
		attempts++
		fmt.Printf("  Attempt %d...\n", attempts)
		return &RetryableError{Err: fmt.Errorf("always fails")}
	}, 3)
	if err != nil {
		fmt.Println("  Failed after", attempts, "attempts:", err)
	}
}
