# Expert Review: File-Based Response Cache

## Critical Issues

### 1. `Set`: No atomic write — corrupt cache entries on crash or concurrent write

**Location:** `Set`, `os.Create(p)` + `f.Write(...)`

The file is written directly in place. If the process crashes or is killed between `os.Create` and `f.Write`, the cache file exists but is empty. The next `Get` call will return an empty string with no error — silently serving corrupt data.

Worse: if two goroutines call `Set` for the same key concurrently, their writes interleave. One `os.Create` truncates the file while the other is mid-write. Result: garbled data.

**Fix:** Write to a temp file in the same directory, then rename. `os.Rename` is atomic on POSIX systems (same filesystem). A crash between write and rename leaves the original file intact; a successful rename replaces it atomically:

```go
func (c *FileCache) Set(key, value string) (err error) {
    p := c.keyToPath(key)
    dir := filepath.Dir(p)

    tmp, err := os.CreateTemp(dir, ".cache-*")
    if err != nil {
        return fmt.Errorf("set %q: create temp: %w", key, err)
    }
    tmpName := tmp.Name()

    defer func() {
        if err != nil {
            os.Remove(tmpName)
        }
    }()

    bw := bufio.NewWriter(tmp)
    if _, err = bw.WriteString(value); err != nil {
        tmp.Close()
        return fmt.Errorf("set %q: write: %w", key, err)
    }
    if err = bw.Flush(); err != nil {
        tmp.Close()
        return fmt.Errorf("set %q: flush: %w", key, err)
    }
    if err = tmp.Close(); err != nil {
        return fmt.Errorf("set %q: close: %w", key, err)
    }
    if err = os.Rename(tmpName, p); err != nil {
        return fmt.Errorf("set %q: rename: %w", key, err)
    }
    return nil
}
```

**Concept:** The temp-file-then-rename pattern is the standard idiom for safe file writes. It eliminates partial-write corruption and provides a poor-man's atomic swap. The temp file must be on the same filesystem as the target (same directory) for rename to be atomic.

---

### 2. `NewFileCache`: Base directory never created — all Sets fail silently

**Location:** `NewFileCache`, the constructor

`NewFileCache` returns a cache without ensuring `baseDir` exists. The first call to `Set` will fail with a `*PathError` ("no such file or directory"), and that error is returned unwrapped without context about why.

**Fix:** Create the directory in the constructor, or return an error:

```go
func NewFileCache(baseDir string, ttl time.Duration) (*FileCache, error) {
    if err := os.MkdirAll(baseDir, 0700); err != nil {
        return nil, fmt.Errorf("init cache dir %q: %w", baseDir, err)
    }
    return &FileCache{baseDir: baseDir, ttl: ttl}, nil
}
```

`0700` (owner read/write/execute, no group or other access) is appropriate for a cache that may contain sensitive API responses.

Alternatively, create the directory lazily in `Set` — but then errors in `Set` become confusing ("directory not found" rather than "cache not initialized").

**Concept:** Constructors should fail fast and early if their preconditions aren't met. A cache that silently fails on every write is worse than one that fails at construction time with a clear error.

---

### 3. `Set`: `defer f.Close()` discards the Close error

**Location:** `Set`, `defer f.Close()`

`f.Close()` on a writable file flushes OS kernel buffers to the filesystem. A failed `Close` can mean data didn't reach disk — but `defer f.Close()` discards the return value.

This is already partially addressed in Fix #1 (explicit `tmp.Close()` with error check), but the pattern is worth calling out explicitly:

```go
// Wrong — discards Close error:
defer f.Close()

// Correct for write-critical paths:
if err := f.Close(); err != nil {
    return fmt.Errorf("close: %w", err)
}
```

For read-only files (`os.Open`), discarding Close error is acceptable — there's nothing to flush. For writable files (`os.Create`, `os.OpenFile` with write flags), capturing the Close error is required for correctness.

**Concept:** See [[io-and-files#defer-fclose-on-writes]].

---

## Major Concerns

### 4. `import "path"` instead of `"path/filepath"` for OS file paths

**Location:** `import` block, `keyToPath`, `Purge`

`path` (from `path` package) always uses `/` as the separator — it's designed for URLs and import paths. `path/filepath` uses the OS-appropriate separator (`\` on Windows, `/` on Unix).

Using `path.Join` for OS file paths breaks on Windows.

**Fix:** Replace all `path.Join(...)` with `filepath.Join(...)` and change the import:

```go
import "path/filepath"  // not "path"

func (c *FileCache) keyToPath(key string) string {
    return filepath.Join(c.baseDir, key+".cache")
}
```

**Related:** The cache key `"user:8821"` contains a colon. Colons are valid in Unix filenames but **invalid in Windows filenames**. A real implementation would sanitize keys (replace `:` with `_`, hash the key, or URL-encode it):

```go
func sanitizeKey(key string) string {
    var sb strings.Builder
    for _, r := range key {
        if r == ':' || r == '/' || r == '\\' {
            sb.WriteRune('_')
        } else {
            sb.WriteRune(r)
        }
    }
    return sb.String()
}
```

---

### 5. `Get`: Error wrapping breaks `os.IsNotExist` / `errors.Is` for cache-miss detection

**Location:** `Get`, the Stat error path

```go
return "", fmt.Errorf("cache miss for %q: %w", key, err)
```

The author uses `%w` (wrap), which is correct — `errors.Is(err, os.ErrNotExist)` will work through the wrapping. So this is actually fine.

However, the **expiry path** returns a plain `fmt.Errorf` without `%w`:

```go
return "", fmt.Errorf("cache expired for %q", key)
```

Callers can't programmatically distinguish "key not found" from "key expired" from "I/O error". For a cache, the usual contract is: return a sentinel or typed error for misses, so callers can decide whether to fetch fresh data.

**Fix:** Define a sentinel error type:

```go
var ErrCacheMiss = errors.New("cache miss")
var ErrCacheExpired = errors.New("cache expired")

// In Get:
if os.IsNotExist(err) {
    return "", fmt.Errorf("%w: %s", ErrCacheMiss, key)
}
// ...
return "", fmt.Errorf("%w: %s", ErrCacheExpired, key)
```

Callers can then: `errors.Is(err, cache.ErrCacheMiss)` or `errors.Is(err, cache.ErrCacheExpired)`.

---

### 6. No concurrency safety — concurrent access to same key is a race

**Location:** Entire implementation

`Get` and `Set` are not concurrency safe. Two goroutines calling `Set` for the same key simultaneously: one truncates the file (via `os.Create`), the other is mid-write. Result: partial content.

Two goroutines calling `Get` then `Set`: both see an expired entry, both call the upstream API, both write the result. Wasted work (thundering herd on cache expiry).

**Minimum fix for a single-process cache:** A `sync.Mutex` per key (or a single mutex for the whole cache):

```go
type FileCache struct {
    baseDir string
    ttl     time.Duration
    mu      sync.Mutex  // protects all cache operations
}
```

A global mutex serializes all cache operations — simple and correct, but a bottleneck under high concurrency. For production, use a per-key lock (`sync.Map` with per-key mutexes) or an existing battle-tested cache library.

---

## Minor Suggestions

### 7. `Purge`: remove errors are silently ignored

**Location:** `Purge`, the `os.Remove` call

```go
os.Remove(path.Join(c.baseDir, entry.Name()))  // error ignored
```

If removal fails (permissions change, NFS mount timeout), the expired entry stays indefinitely and Purge gives no indication. At minimum, log the error or collect them:

```go
var errs []error
for _, entry := range entries {
    if expired {
        if err := os.Remove(filepath.Join(c.baseDir, entry.Name())); err != nil {
            errs = append(errs, err)
        }
    }
}
return errors.Join(errs...)  // Go 1.20+
```

---

### 8. `Delete`: wrapped error inconsistency

**Location:** `Delete`

`Delete` returns `err` unwrapped on failure, while `Get` and `Set` wrap with context. Consistent error wrapping across all methods makes debugging easier:

```go
if err := os.Remove(p); err != nil && !os.IsNotExist(err) {
    return fmt.Errorf("delete %q: %w", key, err)
}
```

---

## Positive Feedback

- The TTL check using `info.ModTime()` is elegant — it uses the filesystem's own timestamp rather than storing metadata separately. Simple and correct for single-host caches.
- `Delete` handles the "key not found" case correctly with `os.IsNotExist(err)` — idempotent deletes are the right API choice.
- The overall structure (`Get`/`Set`/`Delete`/`Purge`) is a clean, minimal API for a file cache.
- `os.Remove(p)` in `Get` after detecting expiry is a good touch — expired entries are cleaned up lazily rather than accumulating.

---

## Summary

| # | Severity | Location | Issue |
|---|----------|----------|-------|
| 1 | Critical | `Set` | Direct write — no atomic rename — corrupt entries on crash/concurrent access |
| 2 | Critical | `NewFileCache` | Base directory never created — all Sets silently fail if dir absent |
| 3 | Critical | `Set` | `defer f.Close()` discards flush error — data may not reach disk |
| 4 | Major | `keyToPath`, `Purge` | `import "path"` instead of `"path/filepath"` — broken on Windows |
| 5 | Major | `Get` | No typed error for cache miss vs expired — callers can't distinguish |
| 6 | Major | All methods | No concurrency safety — race conditions under concurrent access |
| 7 | Minor | `Purge` | Remove errors silently ignored |
| 8 | Minor | `Delete` | Inconsistent error wrapping |
