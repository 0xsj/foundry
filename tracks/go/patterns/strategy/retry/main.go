// Strategy Pattern: Retry Policies with Pluggable Backoff
//
// Demonstrates function-type-based strategy pattern. A retry executor
// accepts a backoff function that determines the delay between attempts.
// Different backoff strategies (fixed, exponential, jittered) are plain
// functions -- no interfaces, no structs, just function signatures.
//
// Run: go run ./retry/

package main

import (
	"context"
	"errors"
	"fmt"
	"math/rand"
	"strings"
	"time"
)

// --- Strategy: Function Type ---

// BackoffFunc determines how long to wait before the next retry attempt.
// This is the strategy — a function type, not an interface.
// Any function with this signature is a valid backoff strategy.
type BackoffFunc func(attempt int) time.Duration

// --- Concrete Strategies (Factory Functions) ---

// FixedBackoff returns a strategy that always waits the same duration.
// Use case: polling a resource at a fixed interval.
func FixedBackoff(delay time.Duration) BackoffFunc {
	return func(attempt int) time.Duration {
		return delay
	}
}

// ExponentialBackoff returns a strategy that doubles the wait each attempt.
// base is the initial delay; maxDelay caps the growth.
// Use case: backing off from an overloaded service.
func ExponentialBackoff(base time.Duration, maxDelay time.Duration) BackoffFunc {
	return func(attempt int) time.Duration {
		delay := base * (1 << uint(attempt)) // base * 2^attempt
		if delay > maxDelay {
			return maxDelay
		}
		return delay
	}
}

// JitteredBackoff returns a strategy with exponential growth plus randomness.
// Prevents the "thundering herd" problem where many clients retry simultaneously.
// Use case: distributed systems with many clients hitting the same service.
func JitteredBackoff(base time.Duration, maxDelay time.Duration) BackoffFunc {
	return func(attempt int) time.Duration {
		delay := base * (1 << uint(attempt))
		if delay > maxDelay {
			delay = maxDelay
		}
		// Add jitter: random value between [delay/2, delay)
		half := int64(delay) / 2
		if half <= 0 {
			return delay
		}
		jitter := time.Duration(rand.Int63n(half))
		return time.Duration(half) + jitter
	}
}

// LinearBackoff returns a strategy that increases the delay linearly.
// delay(n) = base * (n + 1), capped at maxDelay.
// Use case: gradual backoff for non-critical background tasks.
func LinearBackoff(base time.Duration, maxDelay time.Duration) BackoffFunc {
	return func(attempt int) time.Duration {
		delay := base * time.Duration(attempt+1)
		if delay > maxDelay {
			return maxDelay
		}
		return delay
	}
}

// --- Retry Executor ---

// RetryConfig holds configuration for the retry executor.
type RetryConfig struct {
	MaxAttempts int
	Backoff     BackoffFunc
	// RetryIf is an optional strategy that determines whether an error is retryable.
	// If nil, all errors are retried.
	RetryIf func(err error) bool
}

// Retry executes an operation with retries using the configured backoff strategy.
// It returns the result of the first successful attempt, or the last error
// after all attempts are exhausted.
func Retry[T any](ctx context.Context, cfg RetryConfig, operation func(ctx context.Context) (T, error)) (T, error) {
	var zero T
	var lastErr error

	for attempt := 0; attempt < cfg.MaxAttempts; attempt++ {
		result, err := operation(ctx)
		if err == nil {
			return result, nil
		}

		lastErr = err

		// Check if the error is retryable (if a filter is configured)
		if cfg.RetryIf != nil && !cfg.RetryIf(err) {
			return zero, fmt.Errorf("non-retryable error on attempt %d: %w", attempt+1, err)
		}

		// Don't sleep after the last attempt
		if attempt < cfg.MaxAttempts-1 {
			delay := cfg.Backoff(attempt)
			fmt.Printf("  attempt %d failed: %v (retrying in %v)\n", attempt+1, err, delay)

			select {
			case <-time.After(delay):
				// continue to next attempt
			case <-ctx.Done():
				return zero, fmt.Errorf("context cancelled during retry: %w", ctx.Err())
			}
		}
	}

	return zero, fmt.Errorf("all %d attempts failed, last error: %w", cfg.MaxAttempts, lastErr)
}

// --- Simulated Operations ---

// Transient errors that should be retried
var (
	ErrServiceUnavailable = errors.New("service temporarily unavailable")
	ErrTimeout            = errors.New("request timed out")
	ErrRateLimited        = errors.New("rate limited")
)

// Permanent errors that should NOT be retried
var (
	ErrUnauthorized = errors.New("unauthorized: invalid API key")
	ErrNotFound     = errors.New("resource not found")
)

// simulateUnstableAPI creates an operation that fails a specified number of
// times before succeeding. Simulates a flaky external service.
func simulateUnstableAPI(failCount int) func(ctx context.Context) (string, error) {
	attempts := 0
	transientErrors := []error{ErrServiceUnavailable, ErrTimeout, ErrRateLimited}

	return func(ctx context.Context) (string, error) {
		attempts++
		if attempts <= failCount {
			err := transientErrors[attempts%len(transientErrors)]
			return "", err
		}
		return fmt.Sprintf("success after %d attempts", attempts), nil
	}
}

// isRetryable returns true for transient errors that warrant a retry.
// This is itself a strategy -- a function that decides retry eligibility.
func isRetryable(err error) bool {
	switch {
	case errors.Is(err, ErrServiceUnavailable):
		return true
	case errors.Is(err, ErrTimeout):
		return true
	case errors.Is(err, ErrRateLimited):
		return true
	default:
		return false
	}
}

// --- Demonstration ---

func demonstrateBackoffStrategy(name string, backoff BackoffFunc, attempts int) {
	fmt.Printf("\n%s:\n", name)
	for i := 0; i < attempts; i++ {
		delay := backoff(i)
		bar := strings.Repeat("#", int(delay.Milliseconds())/50)
		fmt.Printf("  attempt %d: wait %v %s\n", i+1, delay, bar)
	}
}

func main() {
	fmt.Println("=== Backoff Strategy Comparison ===")
	fmt.Println("Visualizing delay progression for each strategy:")

	demonstrateBackoffStrategy(
		"Fixed (500ms)",
		FixedBackoff(500*time.Millisecond),
		6,
	)

	demonstrateBackoffStrategy(
		"Exponential (100ms base, 5s max)",
		ExponentialBackoff(100*time.Millisecond, 5*time.Second),
		8,
	)

	demonstrateBackoffStrategy(
		"Jittered Exponential (100ms base, 5s max)",
		JitteredBackoff(100*time.Millisecond, 5*time.Second),
		8,
	)

	demonstrateBackoffStrategy(
		"Linear (200ms base, 2s max)",
		LinearBackoff(200*time.Millisecond, 2*time.Second),
		8,
	)

	// --- Retry with different strategies ---
	fmt.Println("\n" + strings.Repeat("=", 60))
	fmt.Println("=== Retry Execution Demo ===")

	ctx := context.Background()

	// Scenario 1: Exponential backoff, service recovers after 3 failures
	fmt.Println("\n--- Scenario 1: Exponential backoff (service recovers after 3 failures) ---")
	result, err := Retry(ctx, RetryConfig{
		MaxAttempts: 5,
		Backoff:     ExponentialBackoff(10*time.Millisecond, 1*time.Second),
		RetryIf:     isRetryable,
	}, simulateUnstableAPI(3))

	if err != nil {
		fmt.Printf("  FAILED: %v\n", err)
	} else {
		fmt.Printf("  RESULT: %s\n", result)
	}

	// Scenario 2: Fixed backoff, service doesn't recover in time
	fmt.Println("\n--- Scenario 2: Fixed backoff (service never recovers, exhausts retries) ---")
	result, err = Retry(ctx, RetryConfig{
		MaxAttempts: 3,
		Backoff:     FixedBackoff(10 * time.Millisecond),
		RetryIf:     isRetryable,
	}, simulateUnstableAPI(10))

	if err != nil {
		fmt.Printf("  FAILED: %v\n", err)
	} else {
		fmt.Printf("  RESULT: %s\n", result)
	}

	// Scenario 3: Non-retryable error (unauthorized)
	fmt.Println("\n--- Scenario 3: Non-retryable error (stops immediately) ---")
	result, err = Retry(ctx, RetryConfig{
		MaxAttempts: 5,
		Backoff:     ExponentialBackoff(10*time.Millisecond, 1*time.Second),
		RetryIf:     isRetryable,
	}, func(ctx context.Context) (string, error) {
		return "", ErrUnauthorized // permanent error — should not retry
	})

	if err != nil {
		fmt.Printf("  FAILED (correctly): %v\n", err)
	}

	// Scenario 4: Inline anonymous backoff strategy
	fmt.Println("\n--- Scenario 4: Custom inline backoff (always 42ms) ---")
	result, err = Retry(ctx, RetryConfig{
		MaxAttempts: 4,
		Backoff: func(attempt int) time.Duration {
			return 42 * time.Millisecond // custom inline strategy
		},
	}, simulateUnstableAPI(2))

	if err != nil {
		fmt.Printf("  FAILED: %v\n", err)
	} else {
		fmt.Printf("  RESULT: %s\n", result)
	}

	fmt.Println("\n" + strings.Repeat("=", 60))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- BackoffFunc is a function type, not an interface")
	fmt.Println("- Each factory (FixedBackoff, ExponentialBackoff, etc.) returns a closure")
	fmt.Println("- Callers can provide inline anonymous functions too")
	fmt.Println("- RetryIf is also a function-type strategy (retry eligibility)")
	fmt.Println("- No structs, no interfaces — just functions all the way down")
	fmt.Println("- The Retry function is generic (Go 1.18+) — works with any return type")
}
