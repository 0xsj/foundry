// Package testdoubles demonstrates test doubles in Go: stubs and fakes.
//
// Key patterns shown:
//  1. StubClock — controls time without sleeping
//  2. FakeNotifier — records calls, verifiable in tests
//  3. ErrorNotifier — stub that always returns an error
//  4. t.Cleanup for per-test teardown
//
// The goal: tests that are fast, deterministic, and don't touch external services.
//
// Run the tests: go test -v ./test-doubles/
package testdoubles

import (
	"errors"
	"fmt"
	"sync"
	"testing"
	"time"
)

// ============================================================================
// Test doubles (live in the _test.go file — not shipped in the binary)
// ============================================================================

// StubClock is a controllable clock. Tests advance time explicitly — no sleeping.
//
// This is a STUB: it has behavior (returns the configured time) but no
// assertion logic. It's purely for input control.
type StubClock struct {
	mu  sync.Mutex
	now time.Time
}

func newStubClock(t time.Time) *StubClock {
	return &StubClock{now: t}
}

func (c *StubClock) Now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.now
}

// Advance moves the stub clock forward by d.
func (c *StubClock) Advance(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.now = c.now.Add(d)
}

// FakeNotifier records every notification sent to it.
//
// This is a FAKE: it has real logic (stores calls) but doesn't send actual
// notifications. Tests can inspect what was recorded.
type FakeNotifier struct {
	mu   sync.Mutex
	sent []notification
}

type notification struct {
	userID  string
	message string
}

func (f *FakeNotifier) Notify(userID, message string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.sent = append(f.sent, notification{userID: userID, message: message})
	return nil
}

// SentTo returns all notifications sent to a specific user.
func (f *FakeNotifier) SentTo(userID string) []string {
	f.mu.Lock()
	defer f.mu.Unlock()
	var msgs []string
	for _, n := range f.sent {
		if n.userID == userID {
			msgs = append(msgs, n.message)
		}
	}
	return msgs
}

// Count returns total notifications sent.
func (f *FakeNotifier) Count() int {
	f.mu.Lock()
	defer f.mu.Unlock()
	return len(f.sent)
}

// ErrorNotifier is a stub that always returns an error.
// Used to test error handling paths.
type ErrorNotifier struct {
	err error
}

func (e *ErrorNotifier) Notify(_, _ string) error {
	return e.err
}

// ============================================================================
// Helper
// ============================================================================

// newTestManager creates an AlertManager with test doubles already wired in.
// Returns the manager plus references to the doubles for inspection.
func newTestManager(t *testing.T, start time.Time, threshold float64, cooldown time.Duration) (*AlertManager, *StubClock, *FakeNotifier) {
	t.Helper()
	clock := newStubClock(start)
	notifier := &FakeNotifier{}
	mgr := NewAlertManager(clock, notifier, threshold, cooldown)
	return mgr, clock, notifier
}

// ============================================================================
// Tests
// ============================================================================

func TestAlertManager_BelowThreshold_NoAlert(t *testing.T) {
	mgr, _, notifier := newTestManager(t, time.Unix(0, 0), 90.0, time.Minute)

	if err := mgr.Record("cpu", 50.0, "user-1"); err != nil {
		t.Fatalf("Record: %v", err)
	}

	if notifier.Count() != 0 {
		t.Errorf("expected 0 notifications, got %d", notifier.Count())
	}
}

func TestAlertManager_ExceedsThreshold_AlertFired(t *testing.T) {
	mgr, _, notifier := newTestManager(t, time.Unix(0, 0), 90.0, time.Minute)

	if err := mgr.Record("cpu", 95.0, "user-1"); err != nil {
		t.Fatalf("Record: %v", err)
	}

	msgs := notifier.SentTo("user-1")
	if len(msgs) != 1 {
		t.Fatalf("expected 1 notification, got %d", len(msgs))
	}

	alerts := mgr.AlertsSent()
	if len(alerts) != 1 {
		t.Fatalf("expected 1 alert sent, got %d", len(alerts))
	}
	if alerts[0].Metric != "cpu" {
		t.Errorf("alert metric = %q, want %q", alerts[0].Metric, "cpu")
	}
	if alerts[0].Value != 95.0 {
		t.Errorf("alert value = %f, want %f", alerts[0].Value, 95.0)
	}
}

func TestAlertManager_Cooldown_SuppressDuplicates(t *testing.T) {
	// StubClock lets us test cooldown behavior without sleeping.
	// A real clock test would require time.Sleep(time.Minute) — slow and fragile.
	mgr, clock, notifier := newTestManager(t, time.Unix(0, 0), 90.0, time.Minute)

	// First alert: fires immediately
	if err := mgr.Record("cpu", 95.0, "user-1"); err != nil {
		t.Fatalf("first Record: %v", err)
	}

	// Second alert: within cooldown — suppressed
	clock.Advance(30 * time.Second)
	if err := mgr.Record("cpu", 98.0, "user-1"); err != nil {
		t.Fatalf("second Record: %v", err)
	}

	if notifier.Count() != 1 {
		t.Errorf("expected 1 notification (cooldown suppressed 2nd), got %d", notifier.Count())
	}

	// Third alert: after cooldown expires — fires
	clock.Advance(31 * time.Second) // total 61s > 1 minute cooldown
	if err := mgr.Record("cpu", 99.0, "user-1"); err != nil {
		t.Fatalf("third Record: %v", err)
	}

	if notifier.Count() != 2 {
		t.Errorf("expected 2 notifications after cooldown, got %d", notifier.Count())
	}
}

func TestAlertManager_IndependentCooldownPerMetric(t *testing.T) {
	mgr, _, notifier := newTestManager(t, time.Unix(0, 0), 90.0, time.Minute)

	// Two different metrics — each has its own cooldown bucket
	if err := mgr.Record("cpu", 95.0, "user-1"); err != nil {
		t.Fatalf("cpu Record: %v", err)
	}
	if err := mgr.Record("memory", 95.0, "user-1"); err != nil {
		t.Fatalf("memory Record: %v", err)
	}

	// Both should fire even though they're within the same second
	if notifier.Count() != 2 {
		t.Errorf("expected 2 notifications (one per metric), got %d", notifier.Count())
	}
}

func TestAlertManager_NotifierError_PropagatesError(t *testing.T) {
	// Use the ErrorNotifier stub to simulate a notification failure.
	clock := newStubClock(time.Unix(0, 0))
	notifier := &ErrorNotifier{err: fmt.Errorf("smtp: connection refused")}
	mgr := NewAlertManager(clock, notifier, 90.0, time.Minute)

	err := mgr.Record("cpu", 95.0, "user-1")
	if err == nil {
		t.Fatal("expected error when notifier fails, got nil")
	}

	// Check error wrapping is preserved — caller can inspect cause
	if !errors.Is(err, notifier.err) {
		t.Errorf("error chain should include original notifier error\ngot: %v", err)
	}

	// Alert should NOT be recorded if notification failed
	if len(mgr.AlertsSent()) != 0 {
		t.Error("expected no alerts recorded when notifier fails")
	}
}

// ============================================================================
// Table-driven test combining test doubles with multiple scenarios
// ============================================================================

func TestAlertManager_Table(t *testing.T) {
	epoch := time.Unix(1_000_000, 0)

	tests := []struct {
		name       string
		threshold  float64
		cooldown   time.Duration
		records    []record
		wantAlerts int
	}{
		{
			name:      "no records",
			threshold: 90,
			cooldown:  time.Minute,
			records:   nil,
			wantAlerts: 0,
		},
		{
			name:      "all below threshold",
			threshold: 90,
			cooldown:  time.Minute,
			records: []record{
				{metric: "cpu", value: 50, user: "u1", advance: 0},
				{metric: "cpu", value: 80, user: "u1", advance: 0},
			},
			wantAlerts: 0,
		},
		{
			name:      "one above threshold",
			threshold: 90,
			cooldown:  time.Minute,
			records: []record{
				{metric: "cpu", value: 95, user: "u1", advance: 0},
			},
			wantAlerts: 1,
		},
		{
			name:      "above threshold twice in cooldown",
			threshold: 90,
			cooldown:  time.Minute,
			records: []record{
				{metric: "cpu", value: 95, user: "u1", advance: 0},
				{metric: "cpu", value: 98, user: "u1", advance: 30 * time.Second},
			},
			wantAlerts: 1, // second suppressed by cooldown
		},
		{
			name:      "above threshold twice after cooldown",
			threshold: 90,
			cooldown:  time.Minute,
			records: []record{
				{metric: "cpu", value: 95, user: "u1", advance: 0},
				{metric: "cpu", value: 98, user: "u1", advance: 2 * time.Minute},
			},
			wantAlerts: 2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mgr, clock, notifier := newTestManager(t, epoch, tt.threshold, tt.cooldown)

			for _, r := range tt.records {
				if r.advance > 0 {
					clock.Advance(r.advance)
				}
				if err := mgr.Record(r.metric, r.value, r.user); err != nil {
					t.Fatalf("Record(%q, %f, %q): %v", r.metric, r.value, r.user, err)
				}
			}

			if got := notifier.Count(); got != tt.wantAlerts {
				t.Errorf("alerts fired = %d, want %d", got, tt.wantAlerts)
			}
		})
	}
}

type record struct {
	metric  string
	value   float64
	user    string
	advance time.Duration
}
