// Storage Backend Factory
//
// Demonstrates: factory function + registry pattern
//
// A storage layer that supports multiple backends (in-memory, filesystem, S3-like).
// The registry allows new backends to be added without modifying the factory code.
//
// Run: go run .
package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// ---------------------------------------------------------------------
// Product interface -- what the factory creates
// ---------------------------------------------------------------------

// Store defines the interface every storage backend must satisfy.
type Store interface {
	Get(key string) ([]byte, error)
	Put(key string, data []byte) error
	Delete(key string) error
	List(prefix string) ([]string, error)
}

// StoreConfig holds backend-agnostic configuration.
type StoreConfig struct {
	Type   string            // "memory", "file", "s3"
	Params map[string]string // backend-specific params
}

// ---------------------------------------------------------------------
// Registry -- central factory registry
// ---------------------------------------------------------------------

// FactoryFunc creates a Store from configuration parameters.
type FactoryFunc func(params map[string]string) (Store, error)

var (
	registryMu sync.RWMutex
	registry   = make(map[string]FactoryFunc)
)

// Register adds a storage backend factory. Panics on duplicate registration.
func Register(name string, factory FactoryFunc) {
	registryMu.Lock()
	defer registryMu.Unlock()
	if _, exists := registry[name]; exists {
		panic(fmt.Sprintf("storage: duplicate registration: %s", name))
	}
	registry[name] = factory
}

// NewStore looks up the registered factory and creates a Store.
func NewStore(cfg StoreConfig) (Store, error) {
	registryMu.RLock()
	factory, exists := registry[cfg.Type]
	registryMu.RUnlock()

	if !exists {
		available := make([]string, 0, len(registry))
		registryMu.RLock()
		for name := range registry {
			available = append(available, name)
		}
		registryMu.RUnlock()
		return nil, fmt.Errorf("storage: unknown backend %q (available: %s)",
			cfg.Type, strings.Join(available, ", "))
	}
	return factory(cfg.Params)
}

// ListDrivers returns all registered backend names.
func ListDrivers() []string {
	registryMu.RLock()
	defer registryMu.RUnlock()
	names := make([]string, 0, len(registry))
	for name := range registry {
		names = append(names, name)
	}
	return names
}

// ---------------------------------------------------------------------
// Implementation 1: In-Memory Store
// ---------------------------------------------------------------------

type memoryStore struct {
	mu   sync.RWMutex
	data map[string][]byte
}

func newMemoryStore(_ map[string]string) (Store, error) {
	return &memoryStore{data: make(map[string][]byte)}, nil
}

func (s *memoryStore) Get(key string) ([]byte, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	data, ok := s.data[key]
	if !ok {
		return nil, fmt.Errorf("key not found: %s", key)
	}
	// Return a copy to prevent mutation
	result := make([]byte, len(data))
	copy(result, data)
	return result, nil
}

func (s *memoryStore) Put(key string, data []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	stored := make([]byte, len(data))
	copy(stored, data)
	s.data[key] = stored
	return nil
}

func (s *memoryStore) Delete(key string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.data[key]; !ok {
		return fmt.Errorf("key not found: %s", key)
	}
	delete(s.data, key)
	return nil
}

func (s *memoryStore) List(prefix string) ([]string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var keys []string
	for key := range s.data {
		if strings.HasPrefix(key, prefix) {
			keys = append(keys, key)
		}
	}
	return keys, nil
}

// ---------------------------------------------------------------------
// Implementation 2: Filesystem Store
// ---------------------------------------------------------------------

type fileStore struct {
	baseDir string
}

func newFileStore(params map[string]string) (Store, error) {
	dir, ok := params["directory"]
	if !ok {
		return nil, fmt.Errorf("file store requires 'directory' parameter")
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, fmt.Errorf("creating base directory: %w", err)
	}
	return &fileStore{baseDir: dir}, nil
}

func (s *fileStore) keyPath(key string) string {
	// Sanitize key to prevent path traversal
	safe := filepath.Clean(key)
	return filepath.Join(s.baseDir, safe)
}

func (s *fileStore) Get(key string) ([]byte, error) {
	data, err := os.ReadFile(s.keyPath(key))
	if os.IsNotExist(err) {
		return nil, fmt.Errorf("key not found: %s", key)
	}
	return data, err
}

func (s *fileStore) Put(key string, data []byte) error {
	path := s.keyPath(key)
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("creating directory: %w", err)
	}
	return os.WriteFile(path, data, 0o644)
}

func (s *fileStore) Delete(key string) error {
	err := os.Remove(s.keyPath(key))
	if os.IsNotExist(err) {
		return fmt.Errorf("key not found: %s", key)
	}
	return err
}

func (s *fileStore) List(prefix string) ([]string, error) {
	var keys []string
	err := filepath.Walk(s.baseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		rel, _ := filepath.Rel(s.baseDir, path)
		if strings.HasPrefix(rel, prefix) {
			keys = append(keys, rel)
		}
		return nil
	})
	return keys, err
}

// ---------------------------------------------------------------------
// Implementation 3: S3-Like Store (simulated)
// ---------------------------------------------------------------------

// s3Store simulates an S3-compatible storage backend.
// In production, this would use the AWS SDK.
type s3Store struct {
	bucket   string
	region   string
	endpoint string
	mu       sync.RWMutex
	objects  map[string][]byte // simulated storage
}

func newS3Store(params map[string]string) (Store, error) {
	bucket, ok := params["bucket"]
	if !ok {
		return nil, fmt.Errorf("s3 store requires 'bucket' parameter")
	}
	region := params["region"]
	if region == "" {
		region = "us-east-1"
	}
	endpoint := params["endpoint"] // optional: for MinIO / localstack

	return &s3Store{
		bucket:   bucket,
		region:   region,
		endpoint: endpoint,
		objects:  make(map[string][]byte),
	}, nil
}

func (s *s3Store) Get(key string) ([]byte, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	data, ok := s.objects[key]
	if !ok {
		return nil, fmt.Errorf("s3://%s/%s: key not found", s.bucket, key)
	}
	result := make([]byte, len(data))
	copy(result, data)
	return result, nil
}

func (s *s3Store) Put(key string, data []byte) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	stored := make([]byte, len(data))
	copy(stored, data)
	s.objects[key] = stored
	fmt.Printf("  [S3] PUT s3://%s/%s (%d bytes, region=%s)\n",
		s.bucket, key, len(data), s.region)
	return nil
}

func (s *s3Store) Delete(key string) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.objects[key]; !ok {
		return fmt.Errorf("s3://%s/%s: key not found", s.bucket, key)
	}
	delete(s.objects, key)
	return nil
}

func (s *s3Store) List(prefix string) ([]string, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	var keys []string
	for key := range s.objects {
		if strings.HasPrefix(key, prefix) {
			keys = append(keys, key)
		}
	}
	return keys, nil
}

// ---------------------------------------------------------------------
// Self-registration -- each backend registers itself
// In a real codebase, these would be in separate packages with init().
// Here we register in init() to demonstrate the pattern in a single file.
// ---------------------------------------------------------------------

func init() {
	Register("memory", newMemoryStore)
	Register("file", newFileStore)
	Register("s3", newS3Store)
}

// ---------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------

func main() {
	fmt.Println("=== Storage Backend Factory Demo ===")
	fmt.Println()

	// Show registered drivers
	fmt.Printf("Registered backends: %v\n\n", ListDrivers())

	// Create stores from configuration -- the caller never imports
	// implementation packages directly.
	configs := []StoreConfig{
		{
			Type:   "memory",
			Params: nil,
		},
		{
			Type: "s3",
			Params: map[string]string{
				"bucket": "user-uploads",
				"region": "eu-west-1",
			},
		},
	}

	for _, cfg := range configs {
		fmt.Printf("--- Backend: %s ---\n", cfg.Type)

		store, err := NewStore(cfg)
		if err != nil {
			fmt.Printf("  Error creating store: %v\n\n", err)
			continue
		}

		// Use the store through the interface -- same code regardless of backend
		if err := store.Put("config/app.json", []byte(`{"debug": true}`)); err != nil {
			fmt.Printf("  Put error: %v\n", err)
			continue
		}

		data, err := store.Get("config/app.json")
		if err != nil {
			fmt.Printf("  Get error: %v\n", err)
			continue
		}
		fmt.Printf("  Retrieved: %s\n", string(data))

		keys, _ := store.List("config/")
		fmt.Printf("  Keys with prefix 'config/': %v\n\n", keys)
	}

	// Demonstrate error handling for unknown backend
	fmt.Println("--- Unknown backend ---")
	_, err := NewStore(StoreConfig{Type: "dynamodb"})
	if err != nil {
		fmt.Printf("  Expected error: %v\n", err)
	}
}
