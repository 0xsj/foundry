package retry

import (
	"errors"
	"testing"
)

var errTransient = errors.New("transient error")

// ============================================================================
// TESTS FOR buildAttempts (Bug 1)
// ============================================================================

// TestBuildAttempts_IndependentIndices verifies that each attempt function
// returns its own index, not a shared final value.
func TestBuildAttempts_IndependentIndices(t *testing.T) {
	op := func() error { return errTransient } // always fails

	attempts := buildAttempts(op, 3)

	if len(attempts) != 3 {
		t.Fatalf("expected 3 attempts, got %d", len(attempts))
	}

	// Each attempt should return index 1, 2, 3 respectively
	for wantIndex, fn := range attempts {
		got, _ := fn()
		if got != wantIndex+1 {
			t.Errorf("attempt[%d] returned index %d, want %d", wantIndex, got, wantIndex+1)
		}
	}
}

// TestBuildAttempts_AllReturnOwnIndex checks all attempt functions
// return different indices (not all the same final value).
func TestBuildAttempts_AllReturnOwnIndex(t *testing.T) {
	op := func() error { return nil }

	attempts := buildAttempts(op, 5)
	seen := make(map[int]bool)
	for _, fn := range attempts {
		idx, _ := fn()
		if seen[idx] {
			t.Errorf("duplicate index %d — closures are sharing the loop variable", idx)
		}
		seen[idx] = true
	}

	if len(seen) != 5 {
		t.Errorf("expected 5 unique indices, got %d: %v", len(seen), seen)
	}
}

// ============================================================================
// TESTS FOR WithCleanup (Bug 2)
// ============================================================================

// TestWithCleanup_SuccessCallsCleanupOnce checks that cleanup is called exactly
// once when the operation succeeds on the first try.
func TestWithCleanup_SuccessCallsCleanupOnce(t *testing.T) {
	var cleanupCalls []int
	cleanup := func(attempt int) {
		cleanupCalls = append(cleanupCalls, attempt)
	}

	op := func() error { return nil } // succeeds immediately

	_, err := WithCleanup(op, 3, cleanup)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(cleanupCalls) != 1 {
		t.Errorf("cleanup called %d times, want 1", len(cleanupCalls))
	}
	if len(cleanupCalls) == 1 && cleanupCalls[0] != 1 {
		t.Errorf("cleanup called with attempt=%d, want 1 (first attempt succeeded)", cleanupCalls[0])
	}
}

// TestWithCleanup_FailureCallsCleanupOnce checks that cleanup is called once
// with the last attempt number when all attempts fail.
func TestWithCleanup_FailureCallsCleanupOnce(t *testing.T) {
	var cleanupCalls []int
	cleanup := func(attempt int) {
		cleanupCalls = append(cleanupCalls, attempt)
	}

	op := func() error { return errTransient } // always fails

	_, err := WithCleanup(op, 3, cleanup)
	if err == nil {
		t.Fatal("expected error when all attempts fail")
	}

	// Cleanup should be called exactly once, with the last attempt number (3)
	if len(cleanupCalls) != 1 {
		t.Errorf("cleanup called %d times, want 1 (defer runs once when function returns)", len(cleanupCalls))
	}
	if len(cleanupCalls) > 0 && cleanupCalls[0] != 3 {
		t.Errorf("cleanup called with attempt=%d, want 3 (last attempt)", cleanupCalls[0])
	}
}

// TestWithCleanup_RetrySuccessOnSecond verifies cleanup is called with
// the correct attempt number when op succeeds on the 2nd try.
func TestWithCleanup_RetrySuccessOnSecond(t *testing.T) {
	var cleanupCalls []int
	cleanup := func(attempt int) {
		cleanupCalls = append(cleanupCalls, attempt)
	}

	callCount := 0
	op := func() error {
		callCount++
		if callCount < 2 {
			return errTransient
		}
		return nil
	}

	_, err := WithCleanup(op, 3, cleanup)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Succeeded on attempt 2, cleanup should reflect that
	if len(cleanupCalls) != 1 {
		t.Errorf("cleanup called %d times, want 1", len(cleanupCalls))
	}
	if len(cleanupCalls) > 0 && cleanupCalls[0] != 2 {
		t.Errorf("cleanup called with %d, want 2", cleanupCalls[0])
	}
}

// ============================================================================
// TESTS FOR Retry (Bug 3)
// ============================================================================

// TestRetry_TimersStoppedPerAttempt verifies that each attempt's timer is
// stopped before the next attempt begins, not deferred to function return.
func TestRetry_TimersStoppedPerAttempt(t *testing.T) {
	callCount := 0
	// Op fails for first 2 calls, then succeeds
	op := func() error {
		callCount++
		if callCount < 3 {
			return errTransient
		}
		return nil
	}

	attempts, timers, err := Retry(op, 5)
	if err != nil {
		t.Fatalf("expected success, got: %v", err)
	}
	if attempts != 3 {
		t.Errorf("expected 3 attempts, got %d", attempts)
	}
	if len(timers) != 3 {
		t.Fatalf("expected 3 timers created, got %d", len(timers))
	}

	// All timers should be stopped now that Retry has returned
	for i, timer := range timers {
		if !timer.stopped {
			t.Errorf("timer[%d] was not stopped — resource leak", i)
		}
	}
}

// TestRetry_TimerStoppedBeforeNextAttempt verifies that timer for attempt N
// is stopped before attempt N+1 begins.
// This tests the core semantics: stop-per-attempt, not stop-at-end.
func TestRetry_TimerStoppedBeforeNextAttempt(t *testing.T) {
	var stoppedAt []int // attempt numbers when each timer was stopped
	var allTimers []*fakeTimer

	// We intercept by wrapping op to check timer state at each call
	callNum := 0
	op := func() error {
		callNum++
		if callNum > 1 {
			// Check that the previous attempt's timer was stopped
			if len(allTimers) > 0 {
				prev := allTimers[len(allTimers)-1]
				if prev.stopped {
					stoppedAt = append(stoppedAt, callNum-1)
				}
			}
		}
		if callNum < 3 {
			return errTransient
		}
		return nil
	}

	_, timers, _ := Retry(op, 5)
	allTimers = timers

	// Timers for attempts 1 and 2 should have been stopped before attempt 2 and 3
	// (i.e., stopped mid-loop, not deferred to function return)
	if len(stoppedAt) != 2 {
		t.Errorf("expected 2 timers stopped mid-loop (before next attempt), got %d stopped: %v", len(stoppedAt), stoppedAt)
	}
}

// TestRetry_AllAttemptsFailReturnsError verifies the error path.
func TestRetry_AllAttemptsFailReturnsError(t *testing.T) {
	op := func() error { return errTransient }

	attempts, _, err := Retry(op, 3)
	if err == nil {
		t.Error("expected error when all attempts fail")
	}
	if attempts != 3 {
		t.Errorf("expected 3 attempts, got %d", attempts)
	}
}

// TestRetry_SuccessOnFirstAttempt verifies early exit.
func TestRetry_SuccessOnFirstAttempt(t *testing.T) {
	op := func() error { return nil }

	attempts, timers, err := Retry(op, 5)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt, got %d", attempts)
	}
	if len(timers) != 1 {
		t.Errorf("expected 1 timer, got %d", len(timers))
	}
	if !timers[0].stopped {
		t.Error("timer was not stopped after success")
	}
}
