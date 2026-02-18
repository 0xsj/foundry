// Package retry provides a retry wrapper for transient operations.
// There are 3 bugs in this file. Find them.
package retry

import (
	"fmt"
	"time"
)

// Operation is any function that might fail transiently.
type Operation func() error

// Result holds the outcome of a retried operation.
type Result struct {
	Attempts int
	Err      error
}

// attemptFunc is a function that executes one attempt and returns its index and error.
type attemptFunc func() (int, error)

// ============================================================================
// BUG 1 LIVES HERE
// buildAttempts creates a slice of functions, one per attempt.
// Each function should capture its own attempt index.
// ============================================================================

// buildAttempts creates attempt functions for each retry index.
// Each returned function, when called, executes op and returns (attemptNumber, error).
func buildAttempts(op Operation, maxAttempts int) []attemptFunc {
	attempts := make([]attemptFunc, maxAttempts)
	for i := 0; i < maxAttempts; i++ {
		attempts[i] = func() (int, error) {
			// BUG 1: i is captured by reference — what value does it have when called?
			return i + 1, op()
		}
	}
	return attempts
}

// ============================================================================
// BUG 2 LIVES HERE
// WithCleanup wraps a retry operation and defers a cleanup callback.
// The cleanup should log which attempt number we're cleaning up after.
// ============================================================================

// WithCleanup runs op with retries and calls cleanup(attemptNumber) when done.
// The cleanup is called with the number of the last attempt that ran.
func WithCleanup(op Operation, maxAttempts int, cleanup func(int)) (Result, error) {
	var lastAttempt int

	for attempt := 1; attempt <= maxAttempts; attempt++ {
		lastAttempt = attempt
		// BUG 2: defer argument is evaluated now — what is lastAttempt at this point?
		defer cleanup(lastAttempt)

		err := op()
		if err == nil {
			return Result{Attempts: attempt}, nil
		}
		if attempt < maxAttempts {
			time.Sleep(time.Millisecond) // small delay between retries
		}
	}
	return Result{Attempts: maxAttempts}, fmt.Errorf("all %d attempts failed", maxAttempts)
}

// ============================================================================
// BUG 3 LIVES HERE
// Retry runs op up to maxAttempts times. On each attempt, it creates a
// timer for observability. The timer should be released at the end of each
// attempt, not when Retry returns.
// ============================================================================

// fakeTimer simulates an external resource (like a metrics timer or tracing span).
type fakeTimer struct {
	id      int
	stopped bool
}

func newTimer(attempt int) *fakeTimer {
	return &fakeTimer{id: attempt}
}

func (t *fakeTimer) Stop() {
	t.stopped = true
}

// Retry runs op up to maxAttempts times, releasing a timer after each attempt.
// Returns the number of attempts made and any final error.
func Retry(op Operation, maxAttempts int) (attempts int, timers []*fakeTimer, err error) {
	for i := 0; i < maxAttempts; i++ {
		t := newTimer(i)
		timers = append(timers, t)

		// BUG 3: defer inside a loop — when does t.Stop() actually run?
		defer t.Stop()

		err = op()
		attempts++
		if err == nil {
			return
		}
	}
	return
}
