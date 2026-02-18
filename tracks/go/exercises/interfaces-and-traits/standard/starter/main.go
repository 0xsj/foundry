// Package storage implements a pluggable key-value storage backend.
// It demonstrates interface definition, composition, implicit satisfaction,
// interface guards, and the decorator pattern.
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
// Using a concrete error type (not fmt.Errorf) allows callers to use errors.As
// to inspect the missing key.
type NotFoundError struct {
	Key string
}

func (e *NotFoundError) Error() string {
	// TODO: return a descriptive message, e.g. `key "foo" not found`
	return ""
}

// ============================================================================
// INTERFACES
// ============================================================================

// Store is the base storage interface.
// All implementations must satisfy this interface.
type Store interface {
	// Get returns the value for key, or *NotFoundError if the key does not exist.
	Get(key string) ([]byte, error)

	// Set stores value under key, overwriting any existing value.
	Set(key string, value []byte) error

	// Delete removes the key. No error if the key doesn't exist.
	Delete(key string) error

	// List returns all keys that start with prefix, in any order.
	// Returns an empty slice (not nil) if no keys match.
	List(prefix string) ([]string, error)
}

// Transaction represents a set of buffered writes that can be committed or
// rolled back atomically.
type Transaction interface {
	// Set stages a write. The write is not visible until Commit.
	Set(key string, value []byte) error

	// Delete stages a deletion. The deletion is not visible until Commit.
	Delete(key string) error

	// Commit applies all staged writes and deletions atomically.
	// Returns an error if the commit fails; the transaction is not retryable.
	Commit() error

	// Rollback discards all staged writes and deletions.
	Rollback() error
}

// TransactionalStore is a Store that also supports transactions.
type TransactionalStore interface {
	Store

	// Begin starts a new transaction.
	// Changes made through the transaction are not visible until Commit.
	Begin() (Transaction, error)
}

// ============================================================================
// MEMORY STORE
// ============================================================================

// MemoryStore is a thread-safe, in-memory implementation of Store.
type MemoryStore struct {
	mu   sync.RWMutex
	data map[string][]byte
}

// NewMemoryStore returns an initialized MemoryStore.
func NewMemoryStore() *MemoryStore {
	// TODO: return an initialized *MemoryStore with a non-nil data map
	return nil
}

// TODO: implement Get, Set, Delete, List with pointer receivers
// - Get: return *NotFoundError for missing keys
// - Set: write under lock
// - Delete: remove key (no error if not found)
// - List: collect all keys with the given prefix

// Compile-time assertion: *MemoryStore must satisfy Store.
// If a method is missing, this line fails to compile.
// TODO: uncomment after implementing all four methods
// var _ Store = (*MemoryStore)(nil)

// ============================================================================
// FILE STORE
// ============================================================================

// FileStore is a Store backed by a JSON file on disk.
// The file format is map[string]string (values stored as base64 strings).
// The file is read on every Get/List, and written on every Set/Delete.
// (Not optimized — for learning purposes.)
type FileStore struct {
	path string
	mu   sync.RWMutex
}

// NewFileStore returns a FileStore that persists data at path.
// Creates the file if it does not exist.
func NewFileStore(path string) (*FileStore, error) {
	// TODO: create a FileStore. If the file doesn't exist, that's OK —
	// treat it as an empty store. Return an error only for unexpected I/O errors.
	return nil, fmt.Errorf("not implemented")
}

// load reads the JSON file and returns the data map.
// Returns an empty map if the file doesn't exist.
func (f *FileStore) load() (map[string]string, error) {
	// TODO: read f.path, unmarshal JSON into map[string]string.
	// If the file doesn't exist (os.IsNotExist), return an empty map, nil.
	return nil, fmt.Errorf("not implemented")
}

// save writes the data map to the JSON file.
func (f *FileStore) save(data map[string]string) error {
	// TODO: marshal data to JSON and write to f.path using os.WriteFile.
	return fmt.Errorf("not implemented")
}

// TODO: implement Get, Set, Delete, List
// - Get: load the file, look up the key, return *NotFoundError if missing
// - Set: load, update, save
// - Delete: load, delete, save (no error if key not found)
// - List: load, collect keys with prefix

// TODO: uncomment after implementing all methods
// var _ Store = (*FileStore)(nil)

// ============================================================================
// MEMORY TX STORE (TRANSACTIONAL)
// ============================================================================

// MemoryTxStore extends MemoryStore with transaction support.
// Uncommitted changes are not visible through Get or List.
type MemoryTxStore struct {
	MemoryStore // embedded by value — promotes Get, Set, Delete, List
}

// NewMemoryTxStore returns an initialized MemoryTxStore.
func NewMemoryTxStore() *MemoryTxStore {
	// TODO: return an initialized *MemoryTxStore
	return nil
}

// Begin starts a new transaction on the store.
// TODO: Implement. Return a *memoryTx that holds a reference to this store.
func (s *MemoryTxStore) Begin() (Transaction, error) {
	return nil, fmt.Errorf("not implemented")
}

// TODO: uncomment after implementing Begin
// var _ TransactionalStore = (*MemoryTxStore)(nil)

// memoryTx is the transaction implementation for MemoryTxStore.
// It buffers writes and deletions until Commit.
type memoryTx struct {
	parent  *MemoryTxStore
	pending map[string][]byte   // staged writes: key → value
	deleted map[string]struct{} // staged deletions
}

// TODO: implement Set, Delete, Commit, Rollback on *memoryTx
// - Set: add to pending, remove from deleted
// - Delete: add to deleted, remove from pending
// - Commit: acquire parent's lock, apply pending and deleted to parent.data
// - Rollback: clear pending and deleted (discard staged changes)

// ============================================================================
// METRICS STORE (DECORATOR)
// ============================================================================

// StoreMetrics holds counters for storage operations.
type StoreMetrics struct {
	ReadCount   int
	WriteCount  int
	DeleteCount int
}

// MetricsStore wraps any Store and tracks operation counts.
// It is itself a Store — callers don't need to know it's a wrapper.
type MetricsStore struct {
	backend Store
	mu      sync.Mutex
	counts  StoreMetrics
}

// NewMetricsStore wraps backend with metrics tracking.
func NewMetricsStore(backend Store) *MetricsStore {
	// TODO: return a *MetricsStore wrapping backend
	return nil
}

// Metrics returns a copy of the current operation counts.
func (m *MetricsStore) Metrics() StoreMetrics {
	// TODO: return a copy (not a pointer) of m.counts
	return StoreMetrics{}
}

// TODO: implement Get, Set, Delete, List
// Each method should:
//   1. Increment the appropriate counter (under m.mu)
//   2. Delegate to m.backend
//   3. Return the backend's result

// TODO: uncomment after implementing all methods
// var _ Store = (*MetricsStore)(nil)

// Ensure the unused imports don't cause compile errors while you work.
var (
	_ = json.Marshal
	_ = os.ReadFile
	_ = strings.HasPrefix
)
