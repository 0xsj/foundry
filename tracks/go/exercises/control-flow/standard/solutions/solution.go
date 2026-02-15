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
//
// This implementation demonstrates:
// - Guard clauses for input validation
// - For loop for retry logic
// - Early returns on success and non-retryable errors
// - Exponential backoff calculation
// - Error wrapping with context
func Retry(op Operation, maxRetries int) error {
	// Guard clause: validate operation
	if op == nil {
		return fmt.Errorf("operation cannot be nil")
	}

	// Guard clause: validate maxRetries
	if maxRetries < 0 {
		return fmt.Errorf("maxRetries must be >= 0")
	}

	// Attempt the operation up to maxRetries + 1 times (initial + retries)
	for attempt := 0; attempt <= maxRetries; attempt++ {
		// Try the operation
		err := op()

		// Early return: success
		if err == nil {
			if attempt > 0 {
				fmt.Printf("Operation succeeded after %d retries\n", attempt)
			}
			return nil
		}

		// Check if error is retryable
		var retryableErr *RetryableError
		if !errors.As(err, &retryableErr) {
			// Early return: non-retryable error
			return fmt.Errorf("non-retryable error on attempt %d: %w", attempt+1, err)
		}

		// If this was the last attempt, return the error
		if attempt == maxRetries {
			return fmt.Errorf("all retries exhausted after %d attempts: %w", attempt+1, err)
		}

		// Calculate exponential backoff: 2^attempt seconds
		delay := time.Second * time.Duration(1<<attempt)
		fmt.Printf("Attempt %d failed, retrying in %v...\n", attempt+1, delay)
		time.Sleep(delay)
	}

	// This should never be reached due to the loop logic above,
	// but included for completeness
	return fmt.Errorf("unexpected retry loop exit")
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
		fmt.Println("  ✓ Success\n")
	}

	// Example 2: Operation that fails then succeeds
	fmt.Println("Test 2: Fails twice, then succeeds")
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
		fmt.Printf("  ✓ Success after %d attempts\n\n", attempts)
	}

	// Example 3: Non-retryable error
	fmt.Println("Test 3: Non-retryable error")
	attempts = 0
	err = Retry(func() error {
		attempts++
		fmt.Printf("  Attempt %d...\n", attempts)
		return fmt.Errorf("permanent failure") // Not wrapped in RetryableError
	}, 5)
	if err != nil {
		fmt.Printf("  Failed after %d attempts: %v\n\n", attempts, err)
	}

	// Example 4: All retries exhausted
	fmt.Println("Test 4: All retries exhausted")
	attempts = 0
	err = Retry(func() error {
		attempts++
		fmt.Printf("  Attempt %d...\n", attempts)
		return &RetryableError{Err: fmt.Errorf("always fails")}
	}, 3)
	if err != nil {
		fmt.Printf("  Failed after %d attempts: %v\n", attempts, err)
	}
}
