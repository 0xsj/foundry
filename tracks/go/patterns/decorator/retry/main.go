// Decorator Pattern: Function Decorators
//
// Demonstrates decorating plain function types (not interfaces).
// Builds reusable decorators: WithRetry, WithTimeout, WithLogging, WithMetrics,
// WithCircuitBreaker. Shows how they compose to create resilient operations.
//
// Run: go run ./retry/

package main

import (
	"context"
	"errors"
	"fmt"
	"math/rand"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// --- Function Type ---

// Operation is a function that performs some work and may fail.
// This is the "interface" for function decorators -- any function
// matching this signature can be decorated.
type Operation func(ctx context.Context) error

// --- Decorator: Retry ---

// WithRetry wraps an operation with automatic retry logic.
// Uses exponential backoff: wait = baseDelay * 2^attempt with jitter.
// Respects context cancellation between retries.
func WithRetry(op Operation, maxAttempts int, baseDelay time.Duration) Operation {
	return func(ctx context.Context) error {
		var lastErr error

		for attempt := 0; attempt < maxAttempts; attempt++ {
			lastErr = op(ctx)
			if lastErr == nil {
				return nil // success
			}

			// Don't sleep after the last attempt
			if attempt < maxAttempts-1 {
				// Exponential backoff with jitter
				delay := baseDelay * time.Duration(1<<uint(attempt))
				jitter := time.Duration(rand.Int63n(int64(delay / 2)))
				delay = delay + jitter

				fmt.Printf("    [retry] attempt %d/%d failed: %v (retrying in %v)\n",
					attempt+1, maxAttempts, lastErr, delay.Round(time.Millisecond))

				select {
				case <-time.After(delay):
					// continue to next attempt
				case <-ctx.Done():
					return fmt.Errorf("retry cancelled: %w", ctx.Err())
				}
			}
		}

		return fmt.Errorf("after %d attempts: %w", maxAttempts, lastErr)
	}
}

// --- Decorator: Timeout ---

// WithTimeout wraps an operation with a deadline.
// If the operation doesn't complete within the timeout, the context is cancelled.
func WithTimeout(op Operation, timeout time.Duration) Operation {
	return func(ctx context.Context) error {
		ctx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		// Run the operation in a goroutine so we can select on timeout
		done := make(chan error, 1)
		go func() {
			done <- op(ctx)
		}()

		select {
		case err := <-done:
			return err
		case <-ctx.Done():
			return fmt.Errorf("operation timed out after %v: %w", timeout, ctx.Err())
		}
	}
}

// --- Decorator: Logging ---

// WithLogging wraps an operation with entry/exit logging including duration and result.
func WithLogging(name string, op Operation) Operation {
	return func(ctx context.Context) error {
		start := time.Now()
		fmt.Printf("    [log] %s: starting\n", name)

		err := op(ctx)

		elapsed := time.Since(start).Round(time.Microsecond)
		if err != nil {
			fmt.Printf("    [log] %s: failed in %v: %v\n", name, elapsed, err)
		} else {
			fmt.Printf("    [log] %s: completed in %v\n", name, elapsed)
		}

		return err
	}
}

// --- Decorator: Metrics ---

// Metrics tracks call counts and latencies for decorated operations.
type Metrics struct {
	mu        sync.Mutex
	calls     int64
	successes int64
	failures  int64
	totalTime time.Duration
}

func (m *Metrics) String() string {
	m.mu.Lock()
	defer m.mu.Unlock()
	var avgMs float64
	if m.calls > 0 {
		avgMs = float64(m.totalTime.Milliseconds()) / float64(m.calls)
	}
	return fmt.Sprintf("calls=%d success=%d fail=%d avg=%.1fms",
		m.calls, m.successes, m.failures, avgMs)
}

// WithMetrics wraps an operation with call counting and latency tracking.
func WithMetrics(metrics *Metrics, op Operation) Operation {
	return func(ctx context.Context) error {
		start := time.Now()
		err := op(ctx)
		elapsed := time.Since(start)

		metrics.mu.Lock()
		metrics.calls++
		metrics.totalTime += elapsed
		if err != nil {
			metrics.failures++
		} else {
			metrics.successes++
		}
		metrics.mu.Unlock()

		return err
	}
}

// --- Decorator: Circuit Breaker ---

// CircuitBreaker implements the circuit breaker pattern as a decorator.
// After consecutive failures exceed the threshold, it "opens" the circuit
// and fast-fails without calling the underlying operation.
type CircuitBreaker struct {
	maxFailures   int
	resetTimeout  time.Duration
	failures      int64
	state         int32 // 0=closed, 1=open, 2=half-open
	lastFailure   time.Time
	mu            sync.Mutex
}

const (
	circuitClosed   int32 = 0
	circuitOpen     int32 = 1
	circuitHalfOpen int32 = 2
)

var ErrCircuitOpen = errors.New("circuit breaker is open")

// NewCircuitBreaker creates a circuit breaker that opens after maxFailures
// consecutive failures and attempts recovery after resetTimeout.
func NewCircuitBreaker(maxFailures int, resetTimeout time.Duration) *CircuitBreaker {
	return &CircuitBreaker{
		maxFailures:  maxFailures,
		resetTimeout: resetTimeout,
	}
}

// WithCircuitBreaker wraps an operation with circuit breaker protection.
func WithCircuitBreaker(cb *CircuitBreaker, op Operation) Operation {
	return func(ctx context.Context) error {
		state := atomic.LoadInt32(&cb.state)

		if state == circuitOpen {
			cb.mu.Lock()
			// Check if enough time has passed to try again
			if time.Since(cb.lastFailure) > cb.resetTimeout {
				atomic.StoreInt32(&cb.state, circuitHalfOpen)
				cb.mu.Unlock()
				fmt.Printf("    [circuit] half-open: attempting recovery\n")
			} else {
				cb.mu.Unlock()
				return ErrCircuitOpen
			}
		}

		err := op(ctx)

		if err != nil {
			cb.mu.Lock()
			cb.failures++
			cb.lastFailure = time.Now()
			if cb.failures >= int64(cb.maxFailures) {
				atomic.StoreInt32(&cb.state, circuitOpen)
				fmt.Printf("    [circuit] OPEN after %d failures\n", cb.failures)
			}
			cb.mu.Unlock()
		} else {
			cb.mu.Lock()
			cb.failures = 0
			atomic.StoreInt32(&cb.state, circuitClosed)
			cb.mu.Unlock()
		}

		return err
	}
}

// --- Simulated Operations ---

// flakeyDatabaseQuery simulates a database call that fails intermittently.
func flakeyDatabaseQuery() Operation {
	callCount := 0
	return func(ctx context.Context) error {
		callCount++
		// Simulate latency
		time.Sleep(10 * time.Millisecond)

		// Fail on attempts 1 and 2, succeed on attempt 3+
		if callCount <= 2 {
			return fmt.Errorf("connection refused (attempt %d)", callCount)
		}
		fmt.Printf("    [db] query executed successfully on attempt %d\n", callCount)
		return nil
	}
}

// alwaysFailingService simulates a downstream service that is completely down.
func alwaysFailingService() Operation {
	return func(ctx context.Context) error {
		time.Sleep(5 * time.Millisecond)
		return errors.New("service unavailable")
	}
}

// slowOperation simulates an operation that takes too long.
func slowOperation() Operation {
	return func(ctx context.Context) error {
		select {
		case <-time.After(5 * time.Second):
			return nil
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

// --- Demo Functions ---

func demoRetryWithLogging() {
	fmt.Println("=== Demo 1: Retry + Logging ===")
	fmt.Println("Scenario: Database query that fails twice then succeeds.")
	fmt.Println()

	op := WithLogging("db-query",
		WithRetry(
			flakeyDatabaseQuery(),
			5,
			50*time.Millisecond,
		),
	)

	ctx := context.Background()
	err := op(ctx)

	if err != nil {
		fmt.Printf("  Result: FAILED - %v\n", err)
	} else {
		fmt.Printf("  Result: SUCCESS\n")
	}
	fmt.Println()
}

func demoTimeout() {
	fmt.Println("=== Demo 2: Timeout ===")
	fmt.Println("Scenario: Operation with 100ms timeout wrapping a 5-second operation.")
	fmt.Println()

	op := WithLogging("slow-service",
		WithTimeout(
			slowOperation(),
			100*time.Millisecond,
		),
	)

	ctx := context.Background()
	err := op(ctx)

	if err != nil {
		fmt.Printf("  Result: FAILED (expected) - %v\n", err)
	} else {
		fmt.Printf("  Result: SUCCESS\n")
	}
	fmt.Println()
}

func demoMetrics() {
	fmt.Println("=== Demo 3: Metrics ===")
	fmt.Println("Scenario: Track call counts and latencies across multiple invocations.")
	fmt.Println()

	metrics := &Metrics{}

	op := WithMetrics(metrics,
		WithRetry(
			flakeyDatabaseQuery(),
			5,
			10*time.Millisecond,
		),
	)

	ctx := context.Background()
	for i := 0; i < 3; i++ {
		err := op(ctx)
		status := "ok"
		if err != nil {
			status = "err"
		}
		fmt.Printf("  Call %d: %s\n", i+1, status)
	}

	fmt.Printf("  Metrics: %s\n", metrics)
	fmt.Println()
}

func demoCircuitBreaker() {
	fmt.Println("=== Demo 4: Circuit Breaker ===")
	fmt.Println("Scenario: Service is down. After 3 failures, circuit opens and fast-fails.")
	fmt.Println()

	cb := NewCircuitBreaker(3, 500*time.Millisecond)

	op := WithCircuitBreaker(cb,
		alwaysFailingService(),
	)

	ctx := context.Background()
	for i := 1; i <= 6; i++ {
		err := op(ctx)
		fmt.Printf("  Call %d: %v\n", i, err)
	}

	fmt.Println()
	fmt.Println("  Calls 4-6 returned instantly (circuit open, no actual call made).")
	fmt.Println()
}

func demoFullStack() {
	fmt.Println("=== Demo 5: Full Decorator Stack ===")
	fmt.Println("Scenario: Production-grade operation with all decorators composed.")
	fmt.Println()

	metrics := &Metrics{}
	cb := NewCircuitBreaker(5, time.Second)

	// Build the decorator chain (read inside-out):
	//   Logging → CircuitBreaker → Metrics → Timeout → Retry → actual operation
	//
	// Execution order:
	//   1. Logging: log start
	//   2. CircuitBreaker: check if circuit is open
	//   3. Metrics: start timer
	//   4. Timeout: set deadline
	//   5. Retry: attempt with backoff
	//   6. Operation: execute
	//   5. Retry: retry if failed
	//   4. Timeout: cancel if too slow
	//   3. Metrics: record latency
	//   2. CircuitBreaker: update failure count
	//   1. Logging: log completion

	op := WithLogging("config-fetch",
		WithCircuitBreaker(cb,
			WithMetrics(metrics,
				WithTimeout(
					WithRetry(
						flakeyDatabaseQuery(),
						3,
						20*time.Millisecond,
					),
					2*time.Second,
				),
			),
		),
	)

	ctx := context.Background()
	err := op(ctx)

	if err != nil {
		fmt.Printf("  Result: FAILED - %v\n", err)
	} else {
		fmt.Printf("  Result: SUCCESS\n")
	}
	fmt.Printf("  Final metrics: %s\n", metrics)
	fmt.Println()
}

func main() {
	fmt.Println("=== Function Decorator Composition ===")
	fmt.Println()
	fmt.Println("Each decorator wraps an Operation (func(context.Context) error)")
	fmt.Println("and returns a new Operation with added behavior.")
	fmt.Println()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demoRetryWithLogging()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demoTimeout()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demoMetrics()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demoCircuitBreaker()
	fmt.Println(strings.Repeat("-", 60))
	fmt.Println()

	demoFullStack()

	fmt.Println("=== Key Takeaways ===")
	fmt.Println()
	fmt.Println("- Define a named function type: type Operation func(ctx) error")
	fmt.Println("- Each decorator: func With*(op Operation, ...) Operation")
	fmt.Println("- Decorators compose: WithA(WithB(WithC(op)))")
	fmt.Println("- Each decorator is independent, testable, reusable")
	fmt.Println("- Same principle as HTTP middleware, applied to plain functions")
}
