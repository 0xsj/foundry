// Package storage implements a pluggable key-value storage backend.
//
// Key design decisions:
//
//  1. Store interface is minimal (4 methods) — easy to implement and test.
//  2. TransactionalStore composes Store — any transactional store is also a Store.
//  3. All constructors return concrete types, not interfaces.
//     Callers get the full API; they can always assign to an interface themselves.
//  4. Interface guards immediately below each type — compile-time satisfaction checks.
//  5. MetricsStore demonstrates the Decorator pattern: wraps any Store without
//     changing its interface. The caller never needs to know.
//  6. NotFoundError is a concrete type — callers can use errors.As to inspect the key.
//
// See solutions/README.md for a comparison of design variants.
package storage

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"sync"
)

// ============================================================================
// ERRORS
// ============================================================================

// NotFoundError is returned when a key does not exist in the store.
// Using a concrete type (rather than fmt.Errorf) allows callers to use errors.As
// to inspect the missing key programmatically.
type NotFoundError struct {
	Key string
}

func (e *NotFoundError) Error() string {
	return fmt.Sprintf("key %q not found", e.Key)
}

// ============================================================================
// INTERFACES
// ============================================================================

// Store is the base storage interface.
// Minimal by design: 4 methods cover everything callers actually need.
// Functions that only read should accept Reader (defined below for composition demo).
type Store interface {
	Get(key string) ([]byte, error)
	Set(key string, value []byte) error
	Delete(key string) error
	List(prefix string) ([]string, error)
}

// Transaction represents a set of buffered writes applied atomically on Commit.
type Transaction interface {
	Set(key string, value []byte) error
	Delete(key string) error
	Commit() error
	Rollback() error
}

// TransactionalStore is a Store that also supports transactions.
// Embedding Store means any TransactionalStore is usable as a plain Store.
type TransactionalStore interface {
	Store
	Begin() (Transaction, error)
}

// ============================================================================
// MEMORY STORE
// ============================================================================

// MemoryStore is a thread-safe, in-memory Store implementation.
// The zero value is not useful (nil map panics on write) — use NewMemoryStore.
type MemoryStore struct {
	mu   sync.RWMutex
	data map[string][]byte
}

// NewMemoryStore returns an initialized MemoryStore ready for use.
func NewMemoryStore() *MemoryStore {
	return &MemoryStore{data: make(map[string][]byte)}
}

// Get returns the value for key, or *NotFoundError if the key doesn't exist.
// Uses RLock — concurrent reads don't block each other.
func (m *MemoryStore) Get(key string) ([]byte, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	v, ok := m.data[key]
	if !ok {
		return nil, &NotFoundError{Key: key}
	}
	// Return a copy — callers shouldn't be able to mutate the stored slice.
	result := make([]byte, len(v))
	copy(result, v)
	return result, nil
}

// Set stores value under key. Uses full Lock — writes are exclusive.
func (m *MemoryStore) Set(key string, value []byte) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	// Store a copy — callers shouldn't affect stored data by mutating their slice.
	stored := make([]byte, len(value))
	copy(stored, value)
	m.data[key] = stored
	return nil
}

// Delete removes key from the store. No error if the key doesn't exist.
func (m *MemoryStore) Delete(key string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.data, key)
	return nil
}

// List returns all keys with the given prefix, in arbitrary order.
// Returns an empty (non-nil) slice if no keys match.
func (m *MemoryStore) List(prefix string) ([]string, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	keys := make([]string, 0) // non-nil empty slice
	for k := range m.data {
		if strings.HasPrefix(k, prefix) {
			keys = append(keys, k)
		}
	}
	return keys, nil
}

// Interface guard: verify *MemoryStore satisfies Store at compile time.
// If any method is missing, this line fails with a clear error.
var _ Store = (*MemoryStore)(nil)

// ============================================================================
// FILE STORE
// ============================================================================

// FileStore is a Store backed by a JSON file.
// Data is persisted across process restarts. Not optimized for high throughput —
// every operation reads or writes the entire file.
//
// File format: JSON object with string keys and base64-encoded string values.
// We store []byte values as hex strings for human readability.
type FileStore struct {
	path string
	mu   sync.RWMutex
}

// NewFileStore returns a FileStore that persists to path.
// If the file doesn't exist, it's treated as an empty store (not an error).
func NewFileStore(path string) (*FileStore, error) {
	f := &FileStore{path: path}
	// Validate we can create/access the file by doing a test load.
	_, err := f.load()
	if err != nil {
		return nil, fmt.Errorf("initializing file store at %q: %w", path, err)
	}
	return f, nil
}

// load reads the JSON file and returns the data map.
// Returns an empty map (not an error) if the file doesn't exist.
func (f *FileStore) load() (map[string]string, error) {
	data, err := os.ReadFile(f.path)
	if err != nil {
		if os.IsNotExist(err) {
			return make(map[string]string), nil
		}
		return nil, fmt.Errorf("reading store file: %w", err)
	}
	var m map[string]string
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, fmt.Errorf("parsing store file: %w", err)
	}
	return m, nil
}

// save marshals the data map and writes it to the file.
func (f *FileStore) save(data map[string]string) error {
	b, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return fmt.Errorf("marshaling store data: %w", err)
	}
	if err := os.WriteFile(f.path, b, 0600); err != nil {
		return fmt.Errorf("writing store file: %w", err)
	}
	return nil
}

func (f *FileStore) Get(key string) ([]byte, error) {
	f.mu.RLock()
	defer f.mu.RUnlock()
	m, err := f.load()
	if err != nil {
		return nil, err
	}
	v, ok := m[key]
	if !ok {
		return nil, &NotFoundError{Key: key}
	}
	return []byte(v), nil
}

func (f *FileStore) Set(key string, value []byte) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	m, err := f.load()
	if err != nil {
		return err
	}
	m[key] = string(value)
	return f.save(m)
}

func (f *FileStore) Delete(key string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	m, err := f.load()
	if err != nil {
		return err
	}
	delete(m, key)
	return f.save(m)
}

func (f *FileStore) List(prefix string) ([]string, error) {
	f.mu.RLock()
	defer f.mu.RUnlock()
	m, err := f.load()
	if err != nil {
		return nil, err
	}
	keys := make([]string, 0)
	for k := range m {
		if strings.HasPrefix(k, prefix) {
			keys = append(keys, k)
		}
	}
	return keys, nil
}

var _ Store = (*FileStore)(nil)

// ============================================================================
// MEMORY TX STORE (TRANSACTIONAL)
//
// MemoryTxStore embeds MemoryStore (by value) to inherit its Get/Set/Delete/List.
// Begin() adds the transactional capability on top.
//
// Transaction semantics:
// - Staged writes/deletes are held in the memoryTx struct (not the parent store).
// - Reads (Get, List) on the parent store see only committed data.
// - Commit acquires the parent's lock and applies all staged changes atomically.
// - Rollback just discards the memoryTx.
// ============================================================================

// MemoryTxStore extends MemoryStore with transaction support.
type MemoryTxStore struct {
	MemoryStore // embedded by value — promotes Get, Set, Delete, List
}

func NewMemoryTxStore() *MemoryTxStore {
	return &MemoryTxStore{MemoryStore: MemoryStore{data: make(map[string][]byte)}}
}

// Begin starts a new transaction. The transaction buffers writes until Commit.
func (s *MemoryTxStore) Begin() (Transaction, error) {
	return &memoryTx{
		parent:  s,
		pending: make(map[string][]byte),
		deleted: make(map[string]struct{}),
	}, nil
}

var _ TransactionalStore = (*MemoryTxStore)(nil)

// memoryTx buffers writes until Commit.
type memoryTx struct {
	parent  *MemoryTxStore
	pending map[string][]byte
	deleted map[string]struct{}
}

// Set stages a write. The change is not visible in the parent store until Commit.
func (t *memoryTx) Set(key string, value []byte) error {
	stored := make([]byte, len(value))
	copy(stored, value)
	t.pending[key] = stored
	delete(t.deleted, key) // un-stage any previous deletion of this key
	return nil
}

// Delete stages a deletion.
func (t *memoryTx) Delete(key string) error {
	t.deleted[key] = struct{}{}
	delete(t.pending, key) // un-stage any pending write for this key
	return nil
}

// Commit applies all staged writes and deletions to the parent store atomically.
// After Commit, the transaction is unusable.
func (t *memoryTx) Commit() error {
	// Acquire the parent's lock for the full apply — atomic from callers' perspective.
	t.parent.mu.Lock()
	defer t.parent.mu.Unlock()

	for k, v := range t.pending {
		t.parent.data[k] = v
	}
	for k := range t.deleted {
		delete(t.parent.data, k)
	}

	// Clear staged changes (make transaction unusable after commit)
	t.pending = nil
	t.deleted = nil
	return nil
}

// Rollback discards all staged changes.
func (t *memoryTx) Rollback() error {
	t.pending = nil
	t.deleted = nil
	return nil
}

// ============================================================================
// METRICS STORE (DECORATOR)
//
// MetricsStore wraps any Store and counts operations.
// It is itself a Store — callers don't see the wrapper.
//
// This is the Decorator pattern: same interface, added behavior (counting),
// transparent to callers. The backend could be MemoryStore, FileStore,
// another MetricsStore, or any future Store implementation.
// ============================================================================

// StoreMetrics holds operation counters.
type StoreMetrics struct {
	ReadCount   int
	WriteCount  int
	DeleteCount int
}

// MetricsStore wraps any Store with operation counting.
type MetricsStore struct {
	backend Store
	mu      sync.Mutex
	counts  StoreMetrics
}

// NewMetricsStore wraps backend with metrics tracking.
// Returns *MetricsStore (concrete type) — callers get Metrics() in addition to Store.
func NewMetricsStore(backend Store) *MetricsStore {
	return &MetricsStore{backend: backend}
}

// Metrics returns a copy of the current operation counts.
// Returns a copy (not pointer) — safe for concurrent reads without locking.
func (m *MetricsStore) Metrics() StoreMetrics {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.counts // copy
}

func (m *MetricsStore) Get(key string) ([]byte, error) {
	m.mu.Lock()
	m.counts.ReadCount++
	m.mu.Unlock()
	return m.backend.Get(key)
}

func (m *MetricsStore) Set(key string, value []byte) error {
	m.mu.Lock()
	m.counts.WriteCount++
	m.mu.Unlock()
	return m.backend.Set(key, value)
}

func (m *MetricsStore) Delete(key string) error {
	m.mu.Lock()
	m.counts.DeleteCount++
	m.mu.Unlock()
	return m.backend.Delete(key)
}

func (m *MetricsStore) List(prefix string) ([]string, error) {
	// Note: List is a read operation but is not counted in ReadCount.
	// Whether to count List reads is a design choice — here we don't
	// to keep ReadCount aligned with single-key Get operations.
	return m.backend.List(prefix)
}

var _ Store = (*MetricsStore)(nil)
