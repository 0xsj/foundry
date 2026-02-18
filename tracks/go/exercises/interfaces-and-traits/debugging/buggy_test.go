package cache

import (
	"testing"
)

// ============================================================================
// BUG 1 TEST: nil interface trap
// newBackend(false, "") should return a nil Storer — not a non-nil interface
// wrapping a nil *MemoryBackend.
// ============================================================================

func TestNewBackendReturnsNilWhenNotConfigured(t *testing.T) {
	backend := newBackend(false, "")

	// This should be nil — no backend configured means no backend.
	if backend != nil {
		t.Errorf("newBackend(false) should return nil, got %T (non-nil interface)", backend)
	}
}

func TestNewBackendHealthCheckOnTypedNil(t *testing.T) {
	// If newBackend returns a non-nil interface with a nil concrete value,
	// calling IsHealthy will panic when it tries to call Ping() on the nil pointer.
	backend := newBackend(false, "")

	if backend == nil {
		// Correct behavior — skip the panic test
		t.Log("newBackend correctly returned nil — Bug 1 is fixed")
		return
	}

	// If we get here, backend is a non-nil interface wrapping nil.
	// Creating a CacheManager with it and calling IsHealthy will panic.
	t.Log("Bug 1 present: backend is non-nil interface wrapping nil *MemoryBackend")
	// (don't actually call IsHealthy here — it would panic and fail the test suite)
}

// ============================================================================
// BUG 2 TEST: value receiver — stats not updated
// ============================================================================

func TestCacheManagerSetCount(t *testing.T) {
	m := NewCacheManager(NewMemoryBackend())

	_ = m.Set("key1", []byte("value1"))
	_ = m.Set("key2", []byte("value2"))
	_ = m.Set("key3", []byte("value3"))

	stats := m.Stats()
	if stats.SetCount != 3 {
		t.Errorf("SetCount = %d, want 3 (value receiver discards the increment)", stats.SetCount)
	}
}

func TestCacheManagerGetCount(t *testing.T) {
	m := NewCacheManager(NewMemoryBackend())
	_ = m.Set("key", []byte("value"))

	_, _ = m.Get("key")
	_, _ = m.Get("key")

	stats := m.Stats()
	if stats.GetCount != 2 {
		t.Errorf("GetCount = %d, want 2", stats.GetCount)
	}
}

// ============================================================================
// BUG 3 TEST: type assertion without comma-ok
// InspectBackend should handle unknown backends gracefully — not panic.
// ============================================================================

func TestCacheManagerInspectMemory(t *testing.T) {
	m := NewCacheManager(NewMemoryBackend())
	got := m.InspectBackend()
	if got != "memory backend (no config)" {
		t.Errorf("InspectBackend = %q, want %q", got, "memory backend (no config)")
	}
}

func TestCacheManagerInspectFile(t *testing.T) {
	m := NewCacheManager(NewFileBackend("/data/store.db"))
	got := m.InspectBackend()
	expected := "file backend at /data/store.db"
	if got != expected {
		t.Errorf("InspectBackend = %q, want %q", got, expected)
	}
}

// This test verifies that InspectBackend uses the safe comma-ok form.
// A wrappedBackend implements Storer but is neither *MemoryBackend nor *FileBackend.
// The type switch will hit default — that's fine. But if the code used unsafe
// assertions elsewhere in the function, this test would expose them.
func TestCacheManagerInspectUnknownBackend(t *testing.T) {
	m := NewCacheManager(&wrappedBackend{})
	// Should not panic; should return a description with the type name.
	got := m.InspectBackend()
	if got == "" {
		t.Error("InspectBackend should return a non-empty string for unknown backend")
	}
}

// ============================================================================
// BUG 2 (ADDITIONAL): mutation in Set should persist
// ============================================================================

func TestCacheManagerSetActuallyStores(t *testing.T) {
	m := NewCacheManager(NewMemoryBackend())
	_ = m.Set("greeting", []byte("hello"))

	v, err := m.Get("greeting")
	if err != nil {
		t.Fatalf("Get after Set: %v", err)
	}
	if string(v) != "hello" {
		t.Errorf("Get = %q, want %q", v, "hello")
	}
}

// ============================================================================
// ADDITIONAL TESTS: IsHealthy
// ============================================================================

func TestCacheManagerIsHealthyWithHealthyBackend(t *testing.T) {
	m := NewCacheManager(NewMemoryBackend())
	healthy, err := m.IsHealthy()
	if err != nil {
		t.Errorf("IsHealthy unexpected error: %v", err)
	}
	if !healthy {
		t.Error("MemoryBackend should be healthy")
	}
}

func TestCacheManagerIsHealthyWithUnhealthyBackend(t *testing.T) {
	m := NewCacheManager(NewFileBackend("/nonexistent/path"))
	healthy, err := m.IsHealthy()
	if err == nil {
		t.Error("FileBackend.Ping should return an error")
	}
	if healthy {
		t.Error("should not be healthy when Ping returns error")
	}
}

func TestCacheManagerIsHealthyWithNoHealthChecker(t *testing.T) {
	// wrappedBackend implements Storer but not HealthChecker
	m := NewCacheManager(&wrappedBackend{})
	healthy, err := m.IsHealthy()
	if err != nil {
		t.Errorf("unexpected error for backend without HealthChecker: %v", err)
	}
	if healthy {
		t.Error("should return false (not healthy) when backend has no HealthChecker")
	}
}

// ============================================================================
// TEST HELPER
// ============================================================================

// wrappedBackend is a minimal Storer with no HealthChecker.
// Used to test that InspectBackend and IsHealthy handle unknown types gracefully.
type wrappedBackend struct{}

func (w *wrappedBackend) Get(key string) ([]byte, error) { return nil, nil }
func (w *wrappedBackend) Set(key string, value []byte) error { return nil }
