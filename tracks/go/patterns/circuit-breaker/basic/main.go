// Circuit Breaker -- Basic Implementation
//
// Implements the three-state machine (Closed, Open, Half-Open) from scratch.
// Wraps a simulated HTTP call and demonstrates state transitions with logging.
//
// Run: go run ./basic/
package main

import (
	"errors"
	"fmt"
	"sync"
	"time"
)

// State represents the circuit breaker state.
type State int

const (
	StateClosed   State = iota // Normal operation -- requests pass through
	StateOpen                  // Fast-fail -- requests rejected immediately
	StateHalfOpen              // Probing -- limited requests to test recovery
)

func (s State) String() string {
	switch s {
	case StateClosed:
		return "CLOSED"
	case StateOpen:
		return "OPEN"
	case StateHalfOpen:
		return "HALF-OPEN"
	default:
		return "UNKNOWN"
	}
}

// ErrCircuitOpen is returned when the breaker is open and rejecting requests.
var ErrCircuitOpen = errors.New("circuit breaker is open")

// CircuitBreaker implements a basic three-state circuit breaker.
type CircuitBreaker struct {
	mu sync.Mutex

	name  string
	state State

	// Counters
	failureCount int
	successCount int

	// Configuration
	failureThreshold int           // failures before opening
	successThreshold int           // successes in half-open before closing
	timeout          time.Duration // how long to stay open

	// Timing
	lastStateChange time.Time

	// Callbacks
	onStateChange func(name string, from, to State)
}

// Option configures a CircuitBreaker.
type Option func(*CircuitBreaker)

// WithFailureThreshold sets the number of failures before the breaker opens.
func WithFailureThreshold(n int) Option {
	return func(cb *CircuitBreaker) {
		cb.failureThreshold = n
	}
}

// WithSuccessThreshold sets the number of successes in half-open before closing.
func WithSuccessThreshold(n int) Option {
	return func(cb *CircuitBreaker) {
		cb.successThreshold = n
	}
}

// WithTimeout sets how long the breaker stays open before probing.
func WithTimeout(d time.Duration) Option {
	return func(cb *CircuitBreaker) {
		cb.timeout = d
	}
}

// WithOnStateChange registers a callback for state transitions.
func WithOnStateChange(fn func(name string, from, to State)) Option {
	return func(cb *CircuitBreaker) {
		cb.onStateChange = fn
	}
}

// NewCircuitBreaker creates a CircuitBreaker with the given options.
func NewCircuitBreaker(name string, opts ...Option) *CircuitBreaker {
	cb := &CircuitBreaker{
		name:             name,
		state:            StateClosed,
		failureThreshold: 5,
		successThreshold: 2,
		timeout:          30 * time.Second,
		lastStateChange:  time.Now(),
	}
	for _, opt := range opts {
		opt(cb)
	}
	return cb
}

// Execute runs the given function through the circuit breaker.
//
// If the breaker is open, it returns ErrCircuitOpen immediately.
// If closed or half-open, it runs the function and records the outcome.
func (cb *CircuitBreaker) Execute(fn func() error) error {
	cb.mu.Lock()

	if !cb.canExecute() {
		cb.mu.Unlock()
		return ErrCircuitOpen
	}

	// Capture state before releasing lock -- we need to know if we were half-open
	wasHalfOpen := cb.state == StateHalfOpen
	cb.mu.Unlock()

	// Execute WITHOUT holding the lock.
	// The wrapped call may take milliseconds or seconds -- we must not
	// block other goroutines from checking breaker state.
	err := fn()

	// Re-acquire lock to record outcome
	cb.mu.Lock()
	defer cb.mu.Unlock()

	if err != nil {
		cb.recordFailure(wasHalfOpen)
		return err
	}

	cb.recordSuccess(wasHalfOpen)
	return nil
}

// State returns the current breaker state.
func (cb *CircuitBreaker) State() State {
	cb.mu.Lock()
	defer cb.mu.Unlock()

	// Check if open->half-open transition should happen
	if cb.state == StateOpen && time.Since(cb.lastStateChange) > cb.timeout {
		cb.setState(StateHalfOpen)
	}
	return cb.state
}

// canExecute checks if the breaker should allow a request through.
// Must be called with cb.mu held.
func (cb *CircuitBreaker) canExecute() bool {
	switch cb.state {
	case StateClosed:
		return true

	case StateOpen:
		// Check if timeout has elapsed -- transition to half-open
		if time.Since(cb.lastStateChange) > cb.timeout {
			cb.setState(StateHalfOpen)
			return true
		}
		return false

	case StateHalfOpen:
		// In half-open, only allow one probe at a time.
		// If successCount > 0, a probe is already in flight.
		return cb.successCount == 0 && cb.failureCount == 0

	default:
		return false
	}
}

// recordFailure handles a failed execution.
// Must be called with cb.mu held.
func (cb *CircuitBreaker) recordFailure(wasHalfOpen bool) {
	if wasHalfOpen {
		// Probe failed -- back to open
		fmt.Printf("  [%s] probe FAILED -- reopening circuit\n", cb.name)
		cb.setState(StateOpen)
		return
	}

	// Closed state -- increment failure counter
	cb.failureCount++
	fmt.Printf("  [%s] failure %d/%d\n", cb.name, cb.failureCount, cb.failureThreshold)

	if cb.failureCount >= cb.failureThreshold {
		cb.setState(StateOpen)
	}
}

// recordSuccess handles a successful execution.
// Must be called with cb.mu held.
func (cb *CircuitBreaker) recordSuccess(wasHalfOpen bool) {
	if wasHalfOpen {
		cb.successCount++
		fmt.Printf("  [%s] probe SUCCESS %d/%d\n", cb.name, cb.successCount, cb.successThreshold)

		if cb.successCount >= cb.successThreshold {
			cb.setState(StateClosed)
		}
		return
	}

	// Closed state -- reset failure counter on success
	cb.failureCount = 0
}

// setState transitions the breaker to a new state.
// Must be called with cb.mu held.
func (cb *CircuitBreaker) setState(newState State) {
	old := cb.state
	if old == newState {
		return
	}

	cb.state = newState
	cb.failureCount = 0
	cb.successCount = 0
	cb.lastStateChange = time.Now()

	fmt.Printf("  [%s] STATE: %s -> %s\n", cb.name, old, newState)

	if cb.onStateChange != nil {
		cb.onStateChange(cb.name, old, newState)
	}
}

// --- Simulation helpers ---

// simulateService returns a function that fails for the first N calls,
// then succeeds. Simulates a service that goes down and recovers.
func simulateService(failFor int) func() error {
	callCount := 0
	return func() error {
		callCount++
		if callCount <= failFor {
			return fmt.Errorf("service unavailable (call %d)", callCount)
		}
		return nil
	}
}

func main() {
	fmt.Println("=== Circuit Breaker: Basic State Machine ===")
	fmt.Println()

	// Create a breaker with low thresholds for demonstration
	cb := NewCircuitBreaker("payment-api",
		WithFailureThreshold(3),
		WithSuccessThreshold(2),
		WithTimeout(2*time.Second), // short timeout for demo
	)

	// Simulate a service that fails for 5 calls, then recovers
	service := simulateService(5)

	// Phase 1: Send requests until the breaker trips
	fmt.Println("--- Phase 1: Normal operation with failures ---")
	for i := 1; i <= 5; i++ {
		err := cb.Execute(service)
		if err != nil {
			if errors.Is(err, ErrCircuitOpen) {
				fmt.Printf("  Request %d: REJECTED (circuit open)\n", i)
			} else {
				fmt.Printf("  Request %d: ERROR: %v\n", i, err)
			}
		} else {
			fmt.Printf("  Request %d: OK\n", i)
		}
	}

	// Phase 2: Breaker is open -- requests fail fast
	fmt.Println("\n--- Phase 2: Circuit is open -- fast failures ---")
	for i := 1; i <= 3; i++ {
		err := cb.Execute(service)
		if errors.Is(err, ErrCircuitOpen) {
			fmt.Printf("  Request %d: REJECTED (circuit open) -- no network call made\n", i)
		}
	}

	// Phase 3: Wait for timeout, then probe
	fmt.Printf("\n--- Phase 3: Waiting %v for timeout ---\n", 2*time.Second)
	time.Sleep(2*time.Second + 100*time.Millisecond)

	// The service is still failing (call 4 and 5 will fail)
	// But we already made 5 calls in phase 1, so the next call (6th) will succeed
	fmt.Println("\n--- Phase 4: Half-open probing ---")
	for i := 1; i <= 4; i++ {
		err := cb.Execute(service)
		if err != nil {
			if errors.Is(err, ErrCircuitOpen) {
				fmt.Printf("  Probe %d: REJECTED (circuit open or already probing)\n", i)
			} else {
				fmt.Printf("  Probe %d: FAILED: %v\n", i, err)
			}
		} else {
			fmt.Printf("  Probe %d: SUCCESS\n", i)
		}

		// Small delay to let state transition happen
		time.Sleep(50 * time.Millisecond)

		// If breaker re-opened after a failed probe, wait for timeout again
		if cb.State() == StateOpen {
			fmt.Printf("  (waiting for timeout...)\n")
			time.Sleep(2*time.Second + 100*time.Millisecond)
		}
	}

	// Phase 5: Verify the breaker is closed
	fmt.Println("\n--- Phase 5: Circuit recovered ---")
	fmt.Printf("  Final state: %s\n", cb.State())

	for i := 1; i <= 3; i++ {
		err := cb.Execute(func() error { return nil })
		if err != nil {
			fmt.Printf("  Request %d: ERROR: %v\n", i, err)
		} else {
			fmt.Printf("  Request %d: OK (normal operation restored)\n", i)
		}
	}
}
