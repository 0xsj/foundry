package scheduler

import (
	"testing"
)

// TestWorkerPoolStats verifies that RecordCompletion updates the pool's stats.
// FAILS: value receiver on RecordCompletion — increments a copy, original unchanged.
// Also PANICS if reached: JobStats is nil in NewWorkerPool.
func TestWorkerPoolStats(t *testing.T) {
	pool := NewWorkerPool(5)

	// This panics because pool.JobStats is nil (BUG 2 may surface here first).
	// Once BUG 2 is fixed, BUG 1 will cause CompletedCount to remain 0.
	pool.RecordCompletion(150)
	pool.RecordCompletion(200)
	pool.RecordFailure()

	stats := pool.Stats()

	if stats.CompletedCount != 2 {
		t.Errorf("CompletedCount = %d, want 2 (RecordCompletion has wrong receiver type)", stats.CompletedCount)
	}
	if stats.FailureCount != 1 {
		t.Errorf("FailureCount = %d, want 1", stats.FailureCount)
	}
	if stats.TotalRuntimeMs != 350 {
		t.Errorf("TotalRuntimeMs = %d, want 350", stats.TotalRuntimeMs)
	}
}

// TestJobUpdate verifies RecordCompletion updates are reflected in the pool.
// This test isolates BUG 1 (value receiver) once BUG 2 is fixed.
func TestJobUpdate(t *testing.T) {
	stats := &JobStats{}

	stats.RecordCompletion(100)
	stats.RecordCompletion(200)

	if stats.CompletedCount != 2 {
		t.Errorf("CompletedCount = %d after 2 calls, want 2 (value receiver discards changes)", stats.CompletedCount)
	}
	if stats.TotalRuntimeMs != 300 {
		t.Errorf("TotalRuntimeMs = %d, want 300", stats.TotalRuntimeMs)
	}
}

// TestWorkerReferences verifies that refs returned by RegisterAll point to
// distinct workers with distinct IDs.
// FAILS: all refs[i] point to the same loop variable — last worker's address.
func TestWorkerReferences(t *testing.T) {
	workers := []Worker{
		{ID: "worker-a", Capacity: 10},
		{ID: "worker-b", Capacity: 20},
		{ID: "worker-c", Capacity: 30},
	}
	pool := NewWorkerPool(10)
	// Fix BUG 2 first so pool doesn't nil-panic during AddWorker
	refs := RegisterAll(workers, pool)

	if len(refs) != 3 {
		t.Fatalf("expected 3 refs, got %d", len(refs))
	}

	expectedIDs := []string{"worker-a", "worker-b", "worker-c"}
	for i, ref := range refs {
		if ref.ID != expectedIDs[i] {
			t.Errorf("refs[%d].ID = %q, want %q (pointer to loop variable bug — all point to last element)", i, ref.ID, expectedIDs[i])
		}
	}

	// Also verify they're distinct pointers
	if refs[0] == refs[1] || refs[1] == refs[2] {
		t.Error("refs should point to distinct workers, but some are equal (loop variable capture bug)")
	}
}

// TestHealthCheckStatus verifies that a ServiceHealth wrapping a nil *ConcreteHealthHandler
// reports unhealthy rather than panicking.
// FAILS: checker != nil (non-nil interface wrapping nil pointer), but calling
// IsHealthy() on the nil pointer panics.
func TestHealthCheckStatus(t *testing.T) {
	var handler *ConcreteHealthHandler = nil // typed nil pointer

	// Assigning a nil *ConcreteHealthHandler to a HealthChecker interface
	// produces a non-nil interface value. The nil check in Healthy() passes,
	// but calling IsHealthy() dereferences the nil pointer — panic.
	sh := NewServiceHealth("database", handler)

	// This should return false (not healthy), not panic
	healthy := sh.Healthy()
	if healthy {
		t.Error("expected unhealthy for nil handler, got healthy=true")
	}
}

// TestHealthCheckHealthy verifies that a properly initialized handler reports correctly.
func TestHealthCheckHealthy(t *testing.T) {
	handler := NewConcreteHealthHandler(true)
	sh := NewServiceHealth("cache", handler)

	if !sh.Healthy() {
		t.Error("expected healthy=true for an initialized healthy handler")
	}
}

// TestHealthCheckUnhealthy verifies that a handler configured as unhealthy reports correctly.
func TestHealthCheckUnhealthy(t *testing.T) {
	handler := NewConcreteHealthHandler(false)
	sh := NewServiceHealth("queue", handler)

	if sh.Healthy() {
		t.Error("expected healthy=false for handler configured as unhealthy")
	}
}
