package storage

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

// ============================================================================
// HELPERS
// ============================================================================

func mustSet(t *testing.T, s Store, key string, value []byte) {
	t.Helper()
	if err := s.Set(key, value); err != nil {
		t.Fatalf("Set(%q): %v", key, err)
	}
}

func mustGet(t *testing.T, s Store, key string) []byte {
	t.Helper()
	v, err := s.Get(key)
	if err != nil {
		t.Fatalf("Get(%q): %v", key, err)
	}
	return v
}

func mustDelete(t *testing.T, s Store, key string) {
	t.Helper()
	if err := s.Delete(key); err != nil {
		t.Fatalf("Delete(%q): %v", key, err)
	}
}

// ============================================================================
// NOT FOUND ERROR
// ============================================================================

func TestNotFoundError(t *testing.T) {
	err := &NotFoundError{Key: "user:42"}
	if err.Error() == "" {
		t.Error("NotFoundError.Error() should return a non-empty string")
	}
	if got := err.Error(); !containsStr(got, "user:42") {
		t.Errorf("NotFoundError.Error() = %q, want it to contain the key %q", got, "user:42")
	}
}

// ============================================================================
// MEMORY STORE
// ============================================================================

func TestMemoryStoreGetSet(t *testing.T) {
	s := NewMemoryStore()

	mustSet(t, s, "config:db_host", []byte("postgres.internal"))
	v := mustGet(t, s, "config:db_host")
	if string(v) != "postgres.internal" {
		t.Errorf("Get = %q, want %q", v, "postgres.internal")
	}
}

func TestMemoryStoreGetNotFound(t *testing.T) {
	s := NewMemoryStore()
	_, err := s.Get("missing:key")
	if err == nil {
		t.Fatal("expected error for missing key, got nil")
	}
	var nfe *NotFoundError
	if !errors.As(err, &nfe) {
		t.Errorf("expected *NotFoundError, got %T: %v", err, err)
	}
	if nfe.Key != "missing:key" {
		t.Errorf("NotFoundError.Key = %q, want %q", nfe.Key, "missing:key")
	}
}

func TestMemoryStoreDelete(t *testing.T) {
	s := NewMemoryStore()
	mustSet(t, s, "session:abc", []byte("token"))
	mustDelete(t, s, "session:abc")

	_, err := s.Get("session:abc")
	if err == nil {
		t.Error("expected error after deleting key, got nil")
	}
}

func TestMemoryStoreDeleteNonExistent(t *testing.T) {
	s := NewMemoryStore()
	// Deleting a non-existent key should not return an error.
	if err := s.Delete("does:not:exist"); err != nil {
		t.Errorf("Delete non-existent key returned error: %v", err)
	}
}

func TestMemoryStoreList(t *testing.T) {
	s := NewMemoryStore()
	mustSet(t, s, "config:db_host", []byte("pg"))
	mustSet(t, s, "config:db_port", []byte("5432"))
	mustSet(t, s, "config:log_level", []byte("info"))
	mustSet(t, s, "session:abc", []byte("token"))

	keys, err := s.List("config:")
	if err != nil {
		t.Fatalf("List: %v", err)
	}
	if len(keys) != 3 {
		t.Errorf("List(\"config:\") returned %d keys, want 3: %v", len(keys), keys)
	}
	for _, k := range keys {
		if !containsStr(k, "config:") {
			t.Errorf("List returned unexpected key %q", k)
		}
	}
}

func TestMemoryStoreListEmpty(t *testing.T) {
	s := NewMemoryStore()
	keys, err := s.List("nonexistent:")
	if err != nil {
		t.Fatalf("List: %v", err)
	}
	if keys == nil {
		t.Error("List should return empty slice, not nil")
	}
	if len(keys) != 0 {
		t.Errorf("List returned %d keys, want 0", len(keys))
	}
}

func TestMemoryStoreOverwrite(t *testing.T) {
	s := NewMemoryStore()
	mustSet(t, s, "key", []byte("first"))
	mustSet(t, s, "key", []byte("second"))
	v := mustGet(t, s, "key")
	if string(v) != "second" {
		t.Errorf("Get after overwrite = %q, want %q", v, "second")
	}
}

// ============================================================================
// FILE STORE
// ============================================================================

func TestFileStoreGetSet(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore: %v", err)
	}

	mustSet(t, s, "config:timeout", []byte("30s"))
	v := mustGet(t, s, "config:timeout")
	if string(v) != "30s" {
		t.Errorf("Get = %q, want %q", v, "30s")
	}
}

func TestFileStorePersistence(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")

	// Write with first instance
	s1, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore: %v", err)
	}
	mustSet(t, s1, "persist:key", []byte("persisted-value"))

	// Read with second instance (simulates restart)
	s2, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore (2nd): %v", err)
	}
	v := mustGet(t, s2, "persist:key")
	if string(v) != "persisted-value" {
		t.Errorf("persisted Get = %q, want %q", v, "persisted-value")
	}
}

func TestFileStoreGetNotFound(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore: %v", err)
	}

	_, err = s.Get("missing:key")
	if err == nil {
		t.Fatal("expected error for missing key, got nil")
	}
	var nfe *NotFoundError
	if !errors.As(err, &nfe) {
		t.Errorf("expected *NotFoundError, got %T: %v", err, err)
	}
}

func TestFileStoreNonExistentFile(t *testing.T) {
	path := filepath.Join(t.TempDir(), "new_store.json")
	// File doesn't exist yet — NewFileStore should succeed, not error.
	s, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore with non-existent file: %v", err)
	}
	// Should be readable as empty
	keys, err := s.List("")
	if err != nil {
		t.Fatalf("List on empty FileStore: %v", err)
	}
	if len(keys) != 0 {
		t.Errorf("new FileStore should have 0 keys, got %d", len(keys))
	}
}

func TestFileStoreDelete(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, _ := NewFileStore(path)
	mustSet(t, s, "tmp:key", []byte("value"))
	mustDelete(t, s, "tmp:key")

	_, err := s.Get("tmp:key")
	if err == nil {
		t.Error("expected NotFoundError after delete, got nil")
	}
}

// Verify file is actually cleaned up after the test
func TestFileStoreFileCreated(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, _ := NewFileStore(path)
	mustSet(t, s, "key", []byte("value"))

	if _, err := os.Stat(path); err != nil {
		t.Errorf("expected file to exist at %s: %v", path, err)
	}
}

// ============================================================================
// TRANSACTIONAL STORE
// ============================================================================

func TestMemoryTxStoreCommit(t *testing.T) {
	s := NewMemoryTxStore()

	tx, err := s.Begin()
	if err != nil {
		t.Fatalf("Begin: %v", err)
	}

	// Stage writes inside the transaction
	_ = tx.Set("config:feature_x", []byte("enabled"))
	_ = tx.Set("config:feature_y", []byte("disabled"))

	// Before commit: keys should NOT be visible in the store
	_, err = s.Get("config:feature_x")
	if err == nil {
		t.Error("uncommitted write should not be visible via Get before Commit")
	}

	// Commit
	if err := tx.Commit(); err != nil {
		t.Fatalf("Commit: %v", err)
	}

	// After commit: keys should be visible
	v := mustGet(t, s, "config:feature_x")
	if string(v) != "enabled" {
		t.Errorf("after Commit, Get = %q, want %q", v, "enabled")
	}
}

func TestMemoryTxStoreRollback(t *testing.T) {
	s := NewMemoryTxStore()
	mustSet(t, s, "config:stable", []byte("original"))

	tx, _ := s.Begin()
	_ = tx.Set("config:stable", []byte("modified"))
	_ = tx.Set("config:new_key", []byte("value"))

	// Rollback — all staged changes discarded
	if err := tx.Rollback(); err != nil {
		t.Fatalf("Rollback: %v", err)
	}

	// config:stable should still be original
	v := mustGet(t, s, "config:stable")
	if string(v) != "original" {
		t.Errorf("after Rollback, config:stable = %q, want %q", v, "original")
	}

	// config:new_key should not exist
	_, err := s.Get("config:new_key")
	if err == nil {
		t.Error("after Rollback, config:new_key should not exist")
	}
}

func TestMemoryTxStoreDeleteInTransaction(t *testing.T) {
	s := NewMemoryTxStore()
	mustSet(t, s, "session:abc", []byte("token"))

	tx, _ := s.Begin()
	_ = tx.Delete("session:abc")

	// Before commit: key still visible
	_, err := s.Get("session:abc")
	if err != nil {
		t.Error("before Commit, staged Delete should not affect Get")
	}

	_ = tx.Commit()

	// After commit: key deleted
	_, err = s.Get("session:abc")
	if err == nil {
		t.Error("after Commit, deleted key should not be found")
	}
}

// ============================================================================
// METRICS STORE
// ============================================================================

func TestMetricsStoreCountsReads(t *testing.T) {
	backend := NewMemoryStore()
	mustSet(t, backend, "key", []byte("value"))

	m := NewMetricsStore(backend)
	mustGet(t, m, "key")
	mustGet(t, m, "key")

	metrics := m.Metrics()
	if metrics.ReadCount != 2 {
		t.Errorf("ReadCount = %d, want 2", metrics.ReadCount)
	}
	if metrics.WriteCount != 0 {
		t.Errorf("WriteCount = %d, want 0", metrics.WriteCount)
	}
}

func TestMetricsStoreCountsWrites(t *testing.T) {
	m := NewMetricsStore(NewMemoryStore())
	mustSet(t, m, "key1", []byte("v1"))
	mustSet(t, m, "key2", []byte("v2"))
	mustSet(t, m, "key1", []byte("v1-updated"))

	metrics := m.Metrics()
	if metrics.WriteCount != 3 {
		t.Errorf("WriteCount = %d, want 3", metrics.WriteCount)
	}
}

func TestMetricsStoreCountsDeletes(t *testing.T) {
	m := NewMetricsStore(NewMemoryStore())
	mustSet(t, m, "key", []byte("v"))
	mustDelete(t, m, "key")
	mustDelete(t, m, "key") // deleting non-existent — still counts

	metrics := m.Metrics()
	if metrics.DeleteCount != 2 {
		t.Errorf("DeleteCount = %d, want 2", metrics.DeleteCount)
	}
}

func TestMetricsStoreDelegates(t *testing.T) {
	// MetricsStore should behave identically to its backend
	backend := NewMemoryStore()
	m := NewMetricsStore(backend)

	mustSet(t, m, "user:1", []byte("alice"))
	mustSet(t, m, "user:2", []byte("bob"))

	v := mustGet(t, m, "user:1")
	if string(v) != "alice" {
		t.Errorf("Get via MetricsStore = %q, want %q", v, "alice")
	}

	keys, err := m.List("user:")
	if err != nil {
		t.Fatalf("List: %v", err)
	}
	if len(keys) != 2 {
		t.Errorf("List(\"user:\") = %d keys, want 2", len(keys))
	}
}

func TestMetricsStoreWrapsAnyStore(t *testing.T) {
	// MetricsStore should wrap any Store — including another MetricsStore.
	inner := NewMetricsStore(NewMemoryStore())
	outer := NewMetricsStore(inner)

	mustSet(t, outer, "key", []byte("value"))
	_ = mustGet(t, outer, "key")

	outerMetrics := outer.Metrics()
	innerMetrics := inner.Metrics()

	if outerMetrics.WriteCount != 1 {
		t.Errorf("outer WriteCount = %d, want 1", outerMetrics.WriteCount)
	}
	if innerMetrics.WriteCount != 1 {
		t.Errorf("inner WriteCount = %d, want 1 (outer delegates to inner)", innerMetrics.WriteCount)
	}
}

// ============================================================================
// HELPERS
// ============================================================================

func containsStr(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(substr) == 0 ||
		func() bool {
			for i := 0; i <= len(s)-len(substr); i++ {
				if s[i:i+len(substr)] == substr {
					return true
				}
			}
			return false
		}())
}
