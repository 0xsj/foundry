package storage

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

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

func TestNotFoundError(t *testing.T) {
	err := &NotFoundError{Key: "user:42"}
	if err.Error() == "" {
		t.Error("NotFoundError.Error() should return a non-empty string")
	}
	if got := err.Error(); !contains(got, "user:42") {
		t.Errorf("NotFoundError.Error() = %q, want it to contain key %q", got, "user:42")
	}
}

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
		t.Fatal("expected error, got nil")
	}
	var nfe *NotFoundError
	if !errors.As(err, &nfe) {
		t.Errorf("expected *NotFoundError, got %T", err)
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
		t.Error("expected error after delete")
	}
}

func TestMemoryStoreDeleteNonExistent(t *testing.T) {
	s := NewMemoryStore()
	if err := s.Delete("does:not:exist"); err != nil {
		t.Errorf("Delete non-existent: %v", err)
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
		t.Errorf("List returned %d keys, want 3: %v", len(keys), keys)
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
	s1, _ := NewFileStore(path)
	mustSet(t, s1, "persist:key", []byte("persisted-value"))

	s2, _ := NewFileStore(path)
	v := mustGet(t, s2, "persist:key")
	if string(v) != "persisted-value" {
		t.Errorf("persisted Get = %q, want %q", v, "persisted-value")
	}
}

func TestFileStoreGetNotFound(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, _ := NewFileStore(path)
	_, err := s.Get("missing:key")
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	var nfe *NotFoundError
	if !errors.As(err, &nfe) {
		t.Errorf("expected *NotFoundError, got %T", err)
	}
}

func TestFileStoreNonExistentFile(t *testing.T) {
	path := filepath.Join(t.TempDir(), "new_store.json")
	s, err := NewFileStore(path)
	if err != nil {
		t.Fatalf("NewFileStore with non-existent file: %v", err)
	}
	keys, err := s.List("")
	if err != nil {
		t.Fatalf("List: %v", err)
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
		t.Error("expected error after delete")
	}
}

func TestFileStoreFileCreated(t *testing.T) {
	path := filepath.Join(t.TempDir(), "store.json")
	s, _ := NewFileStore(path)
	mustSet(t, s, "key", []byte("value"))
	if _, err := os.Stat(path); err != nil {
		t.Errorf("expected file at %s: %v", path, err)
	}
}

func TestMemoryTxStoreCommit(t *testing.T) {
	s := NewMemoryTxStore()
	tx, err := s.Begin()
	if err != nil {
		t.Fatalf("Begin: %v", err)
	}
	_ = tx.Set("config:feature_x", []byte("enabled"))
	_ = tx.Set("config:feature_y", []byte("disabled"))

	_, err = s.Get("config:feature_x")
	if err == nil {
		t.Error("uncommitted write should not be visible before Commit")
	}

	if err := tx.Commit(); err != nil {
		t.Fatalf("Commit: %v", err)
	}
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
	_ = tx.Rollback()

	v := mustGet(t, s, "config:stable")
	if string(v) != "original" {
		t.Errorf("after Rollback, config:stable = %q, want %q", v, "original")
	}
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

	_, err := s.Get("session:abc")
	if err != nil {
		t.Error("before Commit, staged Delete should not affect Get")
	}

	_ = tx.Commit()
	_, err = s.Get("session:abc")
	if err == nil {
		t.Error("after Commit, deleted key should not be found")
	}
}

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
		t.Errorf("WriteCount = %d, want 0 (writes happened on backend directly)", metrics.WriteCount)
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
	mustDelete(t, m, "key")

	metrics := m.Metrics()
	if metrics.DeleteCount != 2 {
		t.Errorf("DeleteCount = %d, want 2", metrics.DeleteCount)
	}
}

func TestMetricsStoreDelegates(t *testing.T) {
	m := NewMetricsStore(NewMemoryStore())
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
		t.Errorf("List = %d keys, want 2", len(keys))
	}
}

func TestMetricsStoreWrapsAnyStore(t *testing.T) {
	inner := NewMetricsStore(NewMemoryStore())
	outer := NewMetricsStore(inner)

	mustSet(t, outer, "key", []byte("value"))
	_ = mustGet(t, outer, "key")

	if outer.Metrics().WriteCount != 1 {
		t.Errorf("outer WriteCount = %d, want 1", outer.Metrics().WriteCount)
	}
	if inner.Metrics().WriteCount != 1 {
		t.Errorf("inner WriteCount = %d, want 1", inner.Metrics().WriteCount)
	}
}

func contains(s, substr string) bool {
	return len(substr) == 0 || (len(s) >= len(substr) && func() bool {
		for i := 0; i <= len(s)-len(substr); i++ {
			if s[i:i+len(substr)] == substr {
				return true
			}
		}
		return false
	}())
}
