// Package cache provides a simple file-based response cache.
// Cache entries are stored as files in a base directory.
// Entries expire after a configurable TTL.
package cache

import (
	"fmt"
	"os"
	"path"   // BUG: should be path/filepath for OS paths
	"time"
)

// FileCache caches string values on disk with TTL-based expiration.
type FileCache struct {
	baseDir string
	ttl     time.Duration
}

// NewFileCache creates a FileCache that stores entries under baseDir.
// Entries expire after ttl duration.
func NewFileCache(baseDir string, ttl time.Duration) *FileCache {
	// BUG: baseDir is never created — if it doesn't exist, all Sets fail
	return &FileCache{baseDir: baseDir, ttl: ttl}
}

// keyToPath converts a cache key to a file path.
// Keys may contain colons (e.g. "user:8821") which are valid in filenames on Unix.
// BUG: uses path.Join (URL paths) not filepath.Join (OS paths)
// BUG: colon in key is invalid in Windows filenames
func (c *FileCache) keyToPath(key string) string {
	return path.Join(c.baseDir, key+".cache")
}

// Get retrieves the cached value for key, or returns an error if missing or expired.
// Returns os.ErrNotExist if the key is not in the cache.
func (c *FileCache) Get(key string) (string, error) {
	p := c.keyToPath(key)

	info, err := os.Stat(p)
	if err != nil {
		// BUG: wraps error — os.IsNotExist(err) won't work for callers
		// who check for cache miss using os.IsNotExist or errors.Is(err, os.ErrNotExist)
		return "", fmt.Errorf("cache miss for %q: %w", key, err)
	}

	// Check TTL
	if time.Since(info.ModTime()) > c.ttl {
		os.Remove(p) // best-effort removal of expired entry
		return "", fmt.Errorf("cache expired for %q", key)
	}

	// BUG: reads the entire file into memory — fine for small values,
	// but this cache has no size limit, so a large cached response loads entirely
	data, err := os.ReadFile(p)
	if err != nil {
		return "", err  // BUG: error not wrapped with context
	}

	return string(data), nil
}

// Set stores value in the cache under key.
// Overwrites any existing entry.
func (c *FileCache) Set(key, value string) error {
	p := c.keyToPath(key)

	// BUG: writes directly to the cache file — if the process dies mid-write,
	// the file is left in a partially written state. Next Get will return corrupt data.
	f, err := os.Create(p)
	if err != nil {
		return err  // BUG: no context on the error
	}
	defer f.Close()

	// BUG: unbuffered Write in a loop would be catastrophic.
	// This is a single Write, so it's technically fine, but the pattern is
	// inconsistent with how the lesson teaches write patterns.
	// For large values, a buffered writer and explicit Flush would be appropriate.
	_, err = f.Write([]byte(value))
	if err != nil {
		return err
	}

	// BUG: f.Close() is deferred but its error is silently discarded.
	// Close flushes OS buffers — a failed Close can mean data didn't reach disk.
	return nil
}

// Delete removes a cache entry if it exists.
// Returns nil if the key was not in the cache.
func (c *FileCache) Delete(key string) error {
	p := c.keyToPath(key)
	err := os.Remove(p)
	if os.IsNotExist(err) {
		return nil // not an error — idempotent delete
	}
	return err  // BUG: error not wrapped
}

// Purge removes all expired entries from the cache directory.
// Intended to be called periodically (e.g., from a background goroutine).
func (c *FileCache) Purge() error {
	entries, err := os.ReadDir(c.baseDir)
	if err != nil {
		return err  // BUG: error not wrapped
	}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		info, err := entry.Info()
		if err != nil {
			continue // skip entries we can't stat
		}
		if time.Since(info.ModTime()) > c.ttl {
			// BUG: error ignored — silently fails if file can't be removed
			os.Remove(path.Join(c.baseDir, entry.Name()))
		}
	}
	return nil
}
