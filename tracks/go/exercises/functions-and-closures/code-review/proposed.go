// Package backoff provides retry logic with exponential backoff.
// Intended for use across all platform services for retrying transient failures.
package backoff

import (
	"fmt"
	"math"
	"time"
)

// Op is any operation that might fail transiently.
type Op func() error

// Result captures the outcome of a retried operation.
type Result struct {
	Attempts int
	LastErr  error
	Duration time.Duration
}

// Backoff returns a function that retries op up to maxAttempts times
// with exponential backoff starting at baseDelay.
// The returned function can be called multiple times to retry the full sequence.
func Backoff(op Op, maxAttempts int, baseDelay time.Duration) func() Result {
	attempt := 0  // BUG: shared across calls to the returned function

	return func() Result {
		start := time.Now()

		for attempt < maxAttempts {
			err := op()
			attempt++

			if err == nil {
				return Result{Attempts: attempt, Duration: time.Since(start)}
			}

			if attempt < maxAttempts {
				delay := time.Duration(float64(baseDelay) * math.Pow(2, float64(attempt-1)))
				time.Sleep(delay)
			}
		}

		return Result{
			Attempts: attempt,
			LastErr:  fmt.Errorf("failed after %d attempts", maxAttempts),  // BUG: loses original error
			Duration: time.Since(start),
		}
	}
}

// WithHooks wraps an Op with before/after hooks for observability.
// before is called before each attempt, after is called after each attempt
// with the attempt number and error (nil on success).
func WithHooks(op Op, before func(), after func(int, error)) Op {
	attempt := 0
	return func() error {
		before()
		err := op()
		attempt++
		// BUG: defer argument evaluated at defer time — attempt is always 0 on first call
		defer after(attempt, err)
		return err
	}
}

// MakeAttempts creates a slice of functions, one per attempt number.
// Each function calls op and returns (attemptNumber, error).
func MakeAttempts(op Op, maxAttempts int) []func() (int, error) {
	result := make([]func() (int, error), maxAttempts)
	for i := 0; i < maxAttempts; i++ {
		// BUG: closure captures i by reference
		result[i] = func() (int, error) {
			return i + 1, op()
		}
	}
	return result
}

// Schedule runs each op in fns sequentially, stopping on first error.
// Returns how many ops completed and any error.
// BUG: non-idiomatic return type — should use (int, error) not a struct for simple cases
func Schedule(fns []Op) struct {
	Completed int
	Err       error
} {
	for i, fn := range fns {
		if err := fn(); err != nil {
			return struct {
				Completed int
				Err       error
			}{Completed: i, Err: err}
		}
	}
	return struct {
		Completed int
		Err       error
	}{Completed: len(fns)}
}

// Jitter adds random variation to a delay to prevent thundering herd.
// Returns a duration between delay*0.5 and delay*1.5.
// Minor: unexported helper with exported-style name
func Jitter(delay time.Duration) time.Duration {
	// Uses a simple deterministic "jitter" for testability (not truly random)
	// Minor: this is not actually jitter — it always returns the same value
	return time.Duration(float64(delay) * 1.5)
}
