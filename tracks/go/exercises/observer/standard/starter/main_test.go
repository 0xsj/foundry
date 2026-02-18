package healthmonitor

import (
	"errors"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// --- Test doubles ---

// recordingObserver records all status changes it receives.
type recordingObserver struct {
	mu      sync.Mutex
	changes []StatusChange
}

func (o *recordingObserver) OnStatusChange(change StatusChange) error {
	o.mu.Lock()
	defer o.mu.Unlock()
	o.changes = append(o.changes, change)
	return nil
}

func (o *recordingObserver) Changes() []StatusChange {
	o.mu.Lock()
	defer o.mu.Unlock()
	out := make([]StatusChange, len(o.changes))
	copy(out, o.changes)
	return out
}

// errorObserver always returns an error.
type errorObserver struct {
	err error
}

func (o *errorObserver) OnStatusChange(_ StatusChange) error {
	return o.err
}

// panicObserver panics on every notification.
type panicObserver struct{}

func (o *panicObserver) OnStatusChange(_ StatusChange) error {
	panic("observer exploded")
}

// slowObserver simulates a slow observer.
type slowObserver struct {
	delay time.Duration
	mu    sync.Mutex
	count int
}

func (o *slowObserver) OnStatusChange(_ StatusChange) error {
	time.Sleep(o.delay)
	o.mu.Lock()
	o.count++
	o.mu.Unlock()
	return nil
}

func (o *slowObserver) Count() int {
	o.mu.Lock()
	defer o.mu.Unlock()
	return o.count
}

// countingObserver counts how many notifications it received.
type countingObserver struct {
	count atomic.Int64
}

func (o *countingObserver) OnStatusChange(_ StatusChange) error {
	o.count.Add(1)
	return nil
}

// --- Tests ---

func TestNewHealthMonitor(t *testing.T) {
	m := NewHealthMonitor()
	if m == nil {
		t.Fatal("NewHealthMonitor returned nil")
	}
}

func TestReportStatus_NotifiesOnChange(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}
	m.Register(obs)

	// First report: Unknown -> Healthy (should notify)
	errs := m.ReportStatus("api-gateway", StatusHealthy, nil)
	if len(errs) > 0 {
		t.Fatalf("unexpected errors: %v", errs)
	}

	changes := obs.Changes()
	if len(changes) != 1 {
		t.Fatalf("expected 1 notification, got %d", len(changes))
	}

	change := changes[0]
	if change.Service != "api-gateway" {
		t.Errorf("service = %q, want %q", change.Service, "api-gateway")
	}
	if change.Previous != StatusUnknown {
		t.Errorf("previous = %v, want %v", change.Previous, StatusUnknown)
	}
	if change.Current != StatusHealthy {
		t.Errorf("current = %v, want %v", change.Current, StatusHealthy)
	}
	if change.Timestamp.IsZero() {
		t.Error("timestamp should not be zero")
	}
}

func TestReportStatus_DoesNotNotifyWhenStatusUnchanged(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}
	m.Register(obs)

	m.ReportStatus("api-gateway", StatusHealthy, nil)
	m.ReportStatus("api-gateway", StatusHealthy, nil) // same status
	m.ReportStatus("api-gateway", StatusHealthy, nil) // same status

	changes := obs.Changes()
	if len(changes) != 1 {
		t.Fatalf("expected 1 notification (only the first change), got %d", len(changes))
	}
}

func TestReportStatus_NotifiesOnEachTransition(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}
	m.Register(obs)

	m.ReportStatus("db", StatusHealthy, nil)
	m.ReportStatus("db", StatusDegraded, map[string]string{"latency": "500ms"})
	m.ReportStatus("db", StatusUnhealthy, map[string]string{"error": "connection refused"})
	m.ReportStatus("db", StatusHealthy, nil)

	changes := obs.Changes()
	if len(changes) != 4 {
		t.Fatalf("expected 4 transitions, got %d", len(changes))
	}

	// Verify the chain of transitions
	expected := []struct {
		prev, curr ServiceStatus
	}{
		{StatusUnknown, StatusHealthy},
		{StatusHealthy, StatusDegraded},
		{StatusDegraded, StatusUnhealthy},
		{StatusUnhealthy, StatusHealthy},
	}
	for i, exp := range expected {
		if changes[i].Previous != exp.prev || changes[i].Current != exp.curr {
			t.Errorf("transition %d: got %v->%v, want %v->%v",
				i, changes[i].Previous, changes[i].Current, exp.prev, exp.curr)
		}
	}

	// Verify metadata on the degraded transition
	if changes[1].Metadata["latency"] != "500ms" {
		t.Errorf("metadata not propagated: %v", changes[1].Metadata)
	}
}

func TestReportStatus_MultipleObservers(t *testing.T) {
	m := NewHealthMonitor()
	obs1 := &recordingObserver{}
	obs2 := &recordingObserver{}
	obs3 := &recordingObserver{}
	m.Register(obs1)
	m.Register(obs2)
	m.Register(obs3)

	m.ReportStatus("cache", StatusUnhealthy, nil)

	for i, obs := range []*recordingObserver{obs1, obs2, obs3} {
		changes := obs.Changes()
		if len(changes) != 1 {
			t.Errorf("observer %d: expected 1 notification, got %d", i, len(changes))
		}
	}
}

func TestUnregister(t *testing.T) {
	m := NewHealthMonitor()
	obs1 := &recordingObserver{}
	obs2 := &recordingObserver{}
	m.Register(obs1)
	m.Register(obs2)

	m.ReportStatus("svc", StatusHealthy, nil)

	// Both should have received the notification
	if len(obs1.Changes()) != 1 || len(obs2.Changes()) != 1 {
		t.Fatal("both observers should have been notified")
	}

	// Unregister obs1
	m.Unregister(obs1)

	m.ReportStatus("svc", StatusUnhealthy, nil)

	// obs1 should NOT have received the second notification
	if len(obs1.Changes()) != 1 {
		t.Errorf("obs1 should have 1 notification after unregister, got %d", len(obs1.Changes()))
	}
	// obs2 should have received both
	if len(obs2.Changes()) != 2 {
		t.Errorf("obs2 should have 2 notifications, got %d", len(obs2.Changes()))
	}
}

func TestUnregister_NonExistent(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}

	// Should not panic
	m.Unregister(obs)
}

func TestCurrentStatus(t *testing.T) {
	m := NewHealthMonitor()

	// Unknown service returns StatusUnknown
	if s := m.CurrentStatus("nonexistent"); s != StatusUnknown {
		t.Errorf("unknown service status = %v, want %v", s, StatusUnknown)
	}

	m.ReportStatus("api", StatusHealthy, nil)
	if s := m.CurrentStatus("api"); s != StatusHealthy {
		t.Errorf("status = %v, want %v", s, StatusHealthy)
	}

	m.ReportStatus("api", StatusDegraded, nil)
	if s := m.CurrentStatus("api"); s != StatusDegraded {
		t.Errorf("status = %v, want %v", s, StatusDegraded)
	}
}

func TestAllStatuses(t *testing.T) {
	m := NewHealthMonitor()
	m.ReportStatus("api", StatusHealthy, nil)
	m.ReportStatus("db", StatusDegraded, nil)
	m.ReportStatus("cache", StatusUnhealthy, nil)

	statuses := m.AllStatuses()
	if len(statuses) != 3 {
		t.Fatalf("expected 3 services, got %d", len(statuses))
	}

	expected := map[string]ServiceStatus{
		"api":   StatusHealthy,
		"db":    StatusDegraded,
		"cache": StatusUnhealthy,
	}
	for svc, want := range expected {
		got, ok := statuses[svc]
		if !ok {
			t.Errorf("service %q not in statuses", svc)
			continue
		}
		if got != want {
			t.Errorf("service %q: status = %v, want %v", svc, got, want)
		}
	}

	// Verify it's a copy: modifying returned map should not affect monitor
	statuses["api"] = StatusUnhealthy
	if m.CurrentStatus("api") != StatusHealthy {
		t.Error("AllStatuses returned internal map reference, not a copy")
	}
}

func TestObserverError_ContinuesNotifying(t *testing.T) {
	m := NewHealthMonitor()
	obs1 := &recordingObserver{}
	errObs := &errorObserver{err: errors.New("notification failed")}
	obs2 := &recordingObserver{}

	m.Register(obs1)
	m.Register(errObs) // this will error
	m.Register(obs2)

	errs := m.ReportStatus("svc", StatusUnhealthy, nil)

	// Error should be collected
	if len(errs) != 1 {
		t.Fatalf("expected 1 error, got %d", len(errs))
	}
	if errs[0].Error() != "notification failed" {
		t.Errorf("error = %q, want %q", errs[0].Error(), "notification failed")
	}

	// Both recording observers should still have been notified
	if len(obs1.Changes()) != 1 {
		t.Error("obs1 should have been notified despite obs2 error")
	}
	if len(obs2.Changes()) != 1 {
		t.Error("obs2 should have been notified despite errObs error")
	}
}

func TestObserverPanic_RecoveredAndContinues(t *testing.T) {
	m := NewHealthMonitor()
	obs1 := &recordingObserver{}
	panicObs := &panicObserver{}
	obs2 := &recordingObserver{}

	m.Register(obs1)
	m.Register(panicObs)
	m.Register(obs2)

	// Should not panic the caller
	errs := m.ReportStatus("svc", StatusUnhealthy, nil)

	// The panic should be recovered and reported as an error
	if len(errs) != 1 {
		t.Fatalf("expected 1 error from recovered panic, got %d: %v", len(errs), errs)
	}

	// Both non-panicking observers should have been notified
	if len(obs1.Changes()) != 1 {
		t.Error("obs1 should have been notified despite panic in another observer")
	}
	if len(obs2.Changes()) != 1 {
		t.Error("obs2 should have been notified despite panic in another observer")
	}
}

func TestConcurrentReportStatus(t *testing.T) {
	m := NewHealthMonitor()
	obs := &countingObserver{}
	m.Register(obs)

	services := []string{"api", "db", "cache", "queue", "auth"}
	statuses := []ServiceStatus{StatusHealthy, StatusDegraded, StatusUnhealthy}

	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func(n int) {
			defer wg.Done()
			svc := services[n%len(services)]
			status := statuses[n%len(statuses)]
			m.ReportStatus(svc, status, nil)
		}(i)
	}
	wg.Wait()

	// We can't predict the exact number of notifications (depends on ordering),
	// but there should be at least one per service (the first report is always
	// a change from Unknown) and the observer count should be > 0.
	count := obs.count.Load()
	if count == 0 {
		t.Error("expected at least some notifications from concurrent reports")
	}
}

func TestConcurrentRegisterAndReport(t *testing.T) {
	m := NewHealthMonitor()

	var wg sync.WaitGroup

	// Concurrently register observers while reporting statuses
	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			obs := &recordingObserver{}
			m.Register(obs)
		}()
	}

	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func(n int) {
			defer wg.Done()
			svc := fmt.Sprintf("svc-%d", n%5)
			m.ReportStatus(svc, StatusHealthy, nil)
		}(i)
	}

	wg.Wait()
	// Test passes if no race condition detected (run with -race flag)
}

func TestMultipleServices_IndependentTracking(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}
	m.Register(obs)

	// Report different statuses for different services
	m.ReportStatus("api", StatusHealthy, nil)
	m.ReportStatus("db", StatusDegraded, nil)

	// Report same status again for api (should NOT notify)
	m.ReportStatus("api", StatusHealthy, nil)
	// Report same status again for db (should NOT notify)
	m.ReportStatus("db", StatusDegraded, nil)

	changes := obs.Changes()
	if len(changes) != 2 {
		t.Fatalf("expected 2 notifications (one per service), got %d", len(changes))
	}
}

func TestStatusChange_TimestampIsSet(t *testing.T) {
	m := NewHealthMonitor()
	obs := &recordingObserver{}
	m.Register(obs)

	before := time.Now()
	m.ReportStatus("svc", StatusHealthy, nil)
	after := time.Now()

	changes := obs.Changes()
	if len(changes) != 1 {
		t.Fatal("expected 1 notification")
	}

	ts := changes[0].Timestamp
	if ts.Before(before) || ts.After(after) {
		t.Errorf("timestamp %v not between %v and %v", ts, before, after)
	}
}
