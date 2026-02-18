// Package cache implements a cache manager with pluggable storage backends
// and health checking. There are three interface-related bugs in this file.
package cache

import (
	"fmt"
)

// ============================================================================
// INTERFACES
// ============================================================================

// Storer is the storage backend interface.
type Storer interface {
	Get(key string) ([]byte, error)
	Set(key string, value []byte) error
}

// HealthChecker can report whether a backend is reachable.
type HealthChecker interface {
	Ping() error
}

// ============================================================================
// BACKENDS
// ============================================================================

// MemoryBackend is an in-memory Storer.
type MemoryBackend struct {
	data map[string][]byte
}

func NewMemoryBackend() *MemoryBackend {
	return &MemoryBackend{data: make(map[string][]byte)}
}

func (m *MemoryBackend) Get(key string) ([]byte, error) {
	v, ok := m.data[key]
	if !ok {
		return nil, fmt.Errorf("key not found: %s", key)
	}
	return v, nil
}

func (m *MemoryBackend) Set(key string, value []byte) error {
	m.data[key] = value
	return nil
}

// Ping always succeeds for in-memory backend.
func (m *MemoryBackend) Ping() error {
	return nil
}

// FileBackend is a (stub) file-backed Storer.
type FileBackend struct {
	Path string
}

func NewFileBackend(path string) *FileBackend {
	return &FileBackend{Path: path}
}

func (f *FileBackend) Get(key string) ([]byte, error) {
	return nil, fmt.Errorf("not implemented")
}

func (f *FileBackend) Set(key string, value []byte) error {
	return fmt.Errorf("not implemented")
}

func (f *FileBackend) Ping() error {
	return fmt.Errorf("file backend: path %q not reachable", f.Path)
}

// ============================================================================
// CACHE MANAGER
// ============================================================================

// CacheStats tracks cache operations.
type CacheStats struct {
	SetCount int
	GetCount int
}

// CacheManager wraps a Storer and tracks usage statistics.
type CacheManager struct {
	backend Storer
	stats   CacheStats
}

func NewCacheManager(backend Storer) *CacheManager {
	return &CacheManager{backend: backend}
}

// BUG 2: value receiver — stats is a copy, increment is discarded.
// Fix: change to pointer receiver (m *CacheManager).
func (m CacheManager) Set(key string, value []byte) error {
	m.stats.SetCount++
	return m.backend.Set(key, value)
}

// BUG 2 (same): value receiver on Get.
func (m CacheManager) Get(key string) ([]byte, error) {
	m.stats.GetCount++
	return m.backend.Get(key)
}

// Stats returns the current operation counts.
func (m *CacheManager) Stats() CacheStats {
	return m.stats
}

// IsHealthy reports whether the backend supports health checking and is reachable.
// Returns (false, nil) if the backend doesn't implement HealthChecker.
// Returns (false, err) if the backend is reachable but reports an error.
// Returns (true, nil) if the backend is healthy.
func (m *CacheManager) IsHealthy() (bool, error) {
	hc, ok := m.backend.(HealthChecker)
	if !ok {
		return false, nil // backend doesn't support health checks
	}
	err := hc.Ping()
	if err != nil {
		return false, err
	}
	return true, nil
}

// ============================================================================
// BUG 1: nil interface trap
//
// newBackend is supposed to return nil when useDisk is false
// (meaning "use no backend" — caller should handle the nil case).
// Instead it returns a non-nil interface holding a nil *MemoryBackend.
// The caller's nil check passes, but any method call will panic.
// ============================================================================

// newBackend constructs the appropriate backend.
// Returns nil if neither disk nor memory backend is requested.
// BUG 1: returns a typed nil (*MemoryBackend)(nil) as Storer interface —
//
//	this is a non-nil interface with a nil concrete value.
func newBackend(useDisk bool, diskPath string) Storer {
	var mem *MemoryBackend // nil *MemoryBackend

	if useDisk {
		return NewFileBackend(diskPath)
	}

	// BUG: returning a nil *MemoryBackend as Storer
	// creates interface{type: *MemoryBackend, value: nil}
	// which is NOT nil. The caller's "if backend == nil" check will fail.
	return mem
}

// ============================================================================
// BUG 3: type assertion without comma-ok
//
// InspectBackend tries to access backend-specific fields.
// If the backend is not a *FileBackend, the assertion panics.
// The comment says it's safe because of the type switch, but the
// actual assertion in the FileBackend case uses the unsafe single-return form.
// ============================================================================

// InspectBackend returns a description of the backend's configuration.
func (m *CacheManager) InspectBackend() string {
	switch m.backend.(type) {
	case *MemoryBackend:
		return "memory backend (no config)"
	case *FileBackend:
		// BUG 3: single-return type assertion — panics if m.backend is
		// somehow a different type by the time we get here.
		// In this specific code path it looks safe, but the pattern is wrong:
		// use comma-ok in all non-switch contexts.
		//
		// More importantly: if InspectBackend is ever called from a context
		// where the backend was swapped between the type switch and this assertion
		// (e.g., in a concurrent scenario), it would panic.
		//
		// Fix: use fb, ok := m.backend.(*FileBackend); if ok { ... }
		fb := m.backend.(*FileBackend)
		return fmt.Sprintf("file backend at %s", fb.Path)
	default:
		return fmt.Sprintf("unknown backend: %T", m.backend)
	}
}
