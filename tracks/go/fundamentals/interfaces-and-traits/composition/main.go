// Package main demonstrates interface composition in Go:
// - Embedding interfaces to build capability hierarchies
// - io.ReadWriter and similar stdlib composed interfaces
// - Interface upgrades: checking for optional capabilities at runtime
// - Small interfaces that compose into larger ones
// - The "accept the minimum you need" design principle
//
// Run with: go run ./composition/
package main

import (
	"fmt"
	"strings"
)

// ============================================================================
// INTERFACE COMPOSITION
//
// Build a storage interface hierarchy from small, composable pieces.
// Each piece can be used independently — functions express exactly what they need.
// ============================================================================

// Reader is the read side of a storage backend.
type Reader interface {
	Get(key string) ([]byte, error)
	List(prefix string) ([]string, error)
}

// Writer is the write side of a storage backend.
type Writer interface {
	Set(key string, value []byte) error
	Delete(key string) error
}

// ReadWriter composes Reader and Writer.
// Any type satisfying both Reader and Writer automatically satisfies ReadWriter.
// This mirrors io.ReadWriter in the standard library.
type ReadWriter interface {
	Reader
	Writer
}

// Snapshotter can export/import a complete snapshot of its state.
type Snapshotter interface {
	Snapshot() ([]byte, error)
	Restore(data []byte) error
}

// FullStore composes all capabilities.
// Only backends that support everything (read, write, snapshot) satisfy this.
type FullStore interface {
	ReadWriter
	Snapshotter
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

// MemoryStore is a simple in-memory key-value store.
// It satisfies FullStore (Reader + Writer + Snapshotter).
type MemoryStore struct {
	data map[string][]byte
}

func NewMemoryStore() *MemoryStore {
	return &MemoryStore{data: make(map[string][]byte)}
}

func (m *MemoryStore) Get(key string) ([]byte, error) {
	v, ok := m.data[key]
	if !ok {
		return nil, fmt.Errorf("key %q not found", key)
	}
	return v, nil
}

func (m *MemoryStore) List(prefix string) ([]string, error) {
	var keys []string
	for k := range m.data {
		if strings.HasPrefix(k, prefix) {
			keys = append(keys, k)
		}
	}
	return keys, nil
}

func (m *MemoryStore) Set(key string, value []byte) error {
	m.data[key] = value
	return nil
}

func (m *MemoryStore) Delete(key string) error {
	delete(m.data, key)
	return nil
}

// Snapshot serializes the store to a naive key=value format.
func (m *MemoryStore) Snapshot() ([]byte, error) {
	var sb strings.Builder
	for k, v := range m.data {
		fmt.Fprintf(&sb, "%s=%s\n", k, string(v))
	}
	return []byte(sb.String()), nil
}

func (m *MemoryStore) Restore(data []byte) error {
	m.data = make(map[string][]byte)
	for _, line := range strings.Split(strings.TrimSpace(string(data)), "\n") {
		if line == "" {
			continue
		}
		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			return fmt.Errorf("invalid snapshot line: %q", line)
		}
		m.data[parts[0]] = []byte(parts[1])
	}
	return nil
}

// Compile-time assertions: *MemoryStore satisfies all composed interfaces.
var _ Reader = (*MemoryStore)(nil)
var _ Writer = (*MemoryStore)(nil)
var _ ReadWriter = (*MemoryStore)(nil)
var _ FullStore = (*MemoryStore)(nil)

// ReadOnlyStore wraps a Reader — only exposes the read interface.
// Useful for providing read-only access to a read-write store.
type ReadOnlyStore struct {
	backend Reader
}

func NewReadOnlyStore(r Reader) *ReadOnlyStore {
	return &ReadOnlyStore{backend: r}
}

func (r *ReadOnlyStore) Get(key string) ([]byte, error)      { return r.backend.Get(key) }
func (r *ReadOnlyStore) List(prefix string) ([]string, error) { return r.backend.List(prefix) }

var _ Reader = (*ReadOnlyStore)(nil)

// ============================================================================
// FUNCTIONS THAT ACCEPT THE MINIMUM NEEDED
//
// By accepting a small interface, these functions work with any implementation
// that satisfies it — including future implementations we haven't written yet.
// ============================================================================

// lookupUserProfile reads user profile data — needs only a Reader.
// Works with MemoryStore, FileStore, RedisStore, or any other Reader.
func lookupUserProfile(store Reader, userID string) ([]byte, error) {
	return store.Get("user:" + userID)
}

// storeEvent writes an event — needs only a Writer.
func storeEvent(store Writer, eventKey string, payload []byte) error {
	return store.Set("event:"+eventKey, payload)
}

// populateCache writes a batch of key-value pairs.
// Accepts ReadWriter because it both reads (to check existence) and writes.
func populateCache(store ReadWriter, pairs map[string][]byte) error {
	for key, value := range pairs {
		// Check if the key already exists before overwriting
		existing, err := store.Get(key)
		if err == nil && len(existing) > 0 {
			fmt.Printf("  skipping existing key: %s\n", key)
			continue
		}
		if err := store.Set(key, value); err != nil {
			return fmt.Errorf("populating cache: %w", err)
		}
	}
	return nil
}

// ============================================================================
// INTERFACE UPGRADES
//
// Accept a small interface, but check at runtime whether the value satisfies
// a larger interface. Use the extended capability if available, fall back
// to the basic capability if not.
//
// This is how io.Copy checks for io.WriterTo and io.ReaderFrom for efficiency.
// ============================================================================

// BulkWriter can write multiple keys atomically — an optional capability.
type BulkWriter interface {
	Writer
	SetBulk(pairs map[string][]byte) error
}

// MemoryStore happens to support bulk writes efficiently.
func (m *MemoryStore) SetBulk(pairs map[string][]byte) error {
	for k, v := range pairs {
		m.data[k] = v
	}
	fmt.Printf("  [bulk] wrote %d keys atomically\n", len(pairs))
	return nil
}

var _ BulkWriter = (*MemoryStore)(nil)

// syncConfig writes a configuration map to the store.
// If the store supports bulk writes, it uses them. Otherwise, writes one by one.
// The caller just passes a Writer — they don't need to know about BulkWriter.
func syncConfig(store Writer, config map[string][]byte) error {
	fmt.Printf("syncing %d config keys to store...\n", len(config))

	// Interface upgrade: check if the concrete type supports bulk writes.
	if bulk, ok := store.(BulkWriter); ok {
		fmt.Println("  store supports bulk writes — using SetBulk")
		return bulk.SetBulk(config)
	}

	// Fall back to individual writes.
	fmt.Println("  store does not support bulk writes — writing individually")
	for k, v := range config {
		if err := store.Set(k, v); err != nil {
			return err
		}
	}
	return nil
}

// ============================================================================
// STANDARD LIBRARY EXAMPLE: io.ReadWriter
//
// The standard library's io package is the canonical example of interface
// composition. io.ReadWriter is defined as:
//
//   type ReadWriter interface {
//       Reader
//       Writer
//   }
//
// This is identical to the pattern we've used above with storage interfaces.
// ============================================================================

// processPayload reads from src, transforms, and writes to dst.
// Accepts *strings.Reader and *strings.Builder for this demo.
// In real code you'd accept io.Reader and io.Writer for maximum flexibility.
func processPayload(src *strings.Reader, dst *strings.Builder, transform func([]byte) []byte) error {
	buf := make([]byte, 1024)
	n, err := src.Read(buf)
	if err != nil && n == 0 {
		return err
	}
	transformed := transform(buf[:n])
	_, err = dst.Write(transformed)
	return err
}

// ============================================================================
// MAIN
// ============================================================================

func main() {
	fmt.Println("=== Interface Composition ===")
	fmt.Println()

	store := NewMemoryStore()

	// --- Using the narrow interfaces ---
	fmt.Println("--- Narrow interfaces ---")
	_ = storeEvent(store, "user.signup.u001", []byte(`{"user":"alice"}`))
	profile, _ := lookupUserProfile(store, "u001") // no profile yet — error ignored
	fmt.Printf("profile lookup (empty): %q\n", profile)
	fmt.Println()

	// --- Interface upgrade: BulkWriter ---
	fmt.Println("--- Interface upgrade: BulkWriter ---")
	config := map[string][]byte{
		"config:db_host":  []byte("postgres.internal"),
		"config:db_port":  []byte("5432"),
		"config:log_level": []byte("info"),
	}
	_ = syncConfig(store, config) // MemoryStore supports SetBulk — uses it
	fmt.Println()

	// Demonstrate fallback: ReadOnlyStore doesn't have SetBulk
	roStore := NewReadOnlyStore(store)
	_ = roStore // ReadOnlyStore satisfies Reader only — not Writer
	fmt.Println("ReadOnlyStore only satisfies Reader (not Writer)")
	fmt.Println()

	// --- populateCache: ReadWriter ---
	fmt.Println("--- populateCache: ReadWriter ---")
	// Pre-populate some keys
	_ = store.Set("config:db_host", []byte("postgres.internal"))
	seed := map[string][]byte{
		"config:db_host":  []byte("overwrite-attempt"), // should be skipped
		"config:timeout":  []byte("30s"),               // new key — written
	}
	_ = populateCache(store, seed)
	fmt.Println()

	// --- Snapshot / Restore: FullStore ---
	fmt.Println("--- Snapshot / Restore ---")
	snap, _ := store.Snapshot()
	fmt.Printf("snapshot (%d bytes):\n%s\n", len(snap), string(snap))

	restored := NewMemoryStore()
	_ = restored.Restore(snap)
	v, _ := restored.Get("config:db_host")
	fmt.Printf("restored config:db_host = %q\n", string(v))
	fmt.Println()

	// --- Demonstrating the minimum-needed principle ---
	fmt.Println("--- Passing ReadOnlyStore to a Reader function ---")
	// ReadOnlyStore satisfies Reader — lookupUserProfile accepts it
	_ = roStore.Get // demonstrating the type exists
	fmt.Println("lookupUserProfile accepts any Reader — MemoryStore, ReadOnlyStore, FileStore...")
}
