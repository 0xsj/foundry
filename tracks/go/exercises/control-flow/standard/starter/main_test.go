package main

import (
	"errors"
	"fmt"
	"testing"
	"time"
)

func TestRetry_ImmediateSuccess(t *testing.T) {
	attempts := 0
	op := func() error {
		attempts++
		return nil
	}

	err := Retry(op, 3)
	if err != nil {
		t.Errorf("expected success, got error: %v", err)
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt, got %d", attempts)
	}
}

func TestRetry_SuccessAfterRetries(t *testing.T) {
	attempts := 0
	op := func() error {
		attempts++
		if attempts < 3 {
			return &RetryableError{Err: fmt.Errorf("temporary failure")}
		}
		return nil
	}

	err := Retry(op, 5)
	if err != nil {
		t.Errorf("expected success, got error: %v", err)
	}
	if attempts != 3 {
		t.Errorf("expected 3 attempts, got %d", attempts)
	}
}

func TestRetry_NonRetryableError(t *testing.T) {
	attempts := 0
	op := func() error {
		attempts++
		return fmt.Errorf("permanent failure")
	}

	err := Retry(op, 5)
	if err == nil {
		t.Error("expected error, got nil")
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt (no retries), got %d", attempts)
	}
}

func TestRetry_AllRetriesExhausted(t *testing.T) {
	attempts := 0
	maxRetries := 3
	op := func() error {
		attempts++
		return &RetryableError{Err: fmt.Errorf("always fails")}
	}

	err := Retry(op, maxRetries)
	if err == nil {
		t.Error("expected error, got nil")
	}
	expectedAttempts := maxRetries + 1 // initial attempt + retries
	if attempts != expectedAttempts {
		t.Errorf("expected %d attempts, got %d", expectedAttempts, attempts)
	}
}

func TestRetry_NilOperation(t *testing.T) {
	err := Retry(nil, 3)
	if err == nil {
		t.Error("expected error for nil operation, got nil")
	}
}

func TestRetry_NegativeMaxRetries(t *testing.T) {
	op := func() error { return nil }
	err := Retry(op, -1)
	if err == nil {
		t.Error("expected error for negative maxRetries, got nil")
	}
}

func TestRetry_ExponentialBackoff(t *testing.T) {
	attempts := 0
	startTime := time.Now()
	op := func() error {
		attempts++
		if attempts < 4 {
			return &RetryableError{Err: fmt.Errorf("temporary failure")}
		}
		return nil
	}

	_ = Retry(op, 5)
	elapsed := time.Since(startTime)

	// With exponential backoff: 1s + 2s + 4s = 7s minimum
	// Allow some margin for execution time
	if elapsed < 6*time.Second {
		t.Errorf("backoff too short: expected ~7s, got %v", elapsed)
	}
	if elapsed > 10*time.Second {
		t.Errorf("backoff too long: expected ~7s, got %v", elapsed)
	}
}

func TestRetry_ZeroRetries(t *testing.T) {
	attempts := 0
	op := func() error {
		attempts++
		return &RetryableError{Err: fmt.Errorf("failure")}
	}

	err := Retry(op, 0)
	if err == nil {
		t.Error("expected error, got nil")
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt (no retries), got %d", attempts)
	}
}
