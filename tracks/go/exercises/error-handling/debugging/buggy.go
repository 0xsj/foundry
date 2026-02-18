// Package dbloader loads and validates database connection configs.
// There are 4 bugs in this file. Find them.
//
// Run tests: go test -v ./...
package dbloader

import (
	"errors"
	"fmt"
	"strings"
	"time"
)

// ErrTimeout is the sentinel for timeout conditions.
// Callers should use errors.Is(err, ErrTimeout) to detect timeouts.
var ErrTimeout = errors.New("timeout")

// ============================================================================
// BUG 1 LIVES HERE
// loadConfig builds an error when the connection times out.
// The test expects errors.Is(err, ErrTimeout) to return true,
// but it's always returning false.
// ============================================================================

// ConnConfig holds the connection parameters.
type ConnConfig struct {
	Host     string
	Port     int
	Database string
	Timeout  time.Duration
}

// loadConfig simulates loading a config, returning a timeout error if
// the timeout is too short.
func loadConfig(cfg ConnConfig) error {
	if cfg.Timeout < 100*time.Millisecond {
		// BUG 1: should use %w to wrap ErrTimeout, not %v
		return fmt.Errorf("connection to %s:%d timed out: %v", cfg.Host, cfg.Port, ErrTimeout)
	}
	return nil
}

// ============================================================================
// BUG 2 LIVES HERE
// parseError wraps an error from a lower-level library call.
// The test calls errors.Is with a sentinel — but something in the chain breaks it.
// ============================================================================

var ErrConnectionRefused = errors.New("connection refused")

// dial simulates a low-level dial that can fail with ErrConnectionRefused.
func dial(address string) error {
	if strings.Contains(address, "down") {
		// Returns the sentinel error — this is correct
		return ErrConnectionRefused
	}
	return nil
}

// connect wraps dial errors with context.
func connect(address string) error {
	if err := dial(address); err != nil {
		// BUG 2: uses %v instead of %w — severs the error chain
		return fmt.Errorf("connect to %s: %v", address, err)
	}
	return nil
}

// ============================================================================
// BUG 3 LIVES HERE
// ConnectWithRetry should return an error when address is empty,
// but it panics instead. Library code should never panic for bad input.
// ============================================================================

// ConnectWithRetry attempts to connect to address, retrying up to maxRetries times.
// Returns an error if all retries fail or if address is empty.
func ConnectWithRetry(address string, maxRetries int) error {
	// BUG 3: panics on empty address instead of returning an error
	if address == "" {
		panic("address cannot be empty")
	}

	var lastErr error
	for i := 0; i < maxRetries; i++ {
		if err := connect(address); err != nil {
			lastErr = err
			time.Sleep(time.Millisecond) // small delay between retries
			continue
		}
		return nil
	}
	return fmt.Errorf("ConnectWithRetry: all %d attempts failed: %w", maxRetries, lastErr)
}

// ============================================================================
// BUG 4 LIVES HERE
// openPool creates a connection pool and validates it by pinging.
// If ping fails, the pool is not usable — the function should return the error.
// But it silently succeeds instead.
// ============================================================================

// Pool represents a database connection pool.
type Pool struct {
	Address string
	Size    int
}

// ping simulates a health-check on an established pool connection.
func (p *Pool) ping() error {
	if p.Address == "dead-host:5432" {
		return fmt.Errorf("ping: no route to host")
	}
	return nil
}

// openPool creates a Pool and validates it is reachable.
// Returns an error if the pool cannot be established or the ping fails.
func openPool(address string, size int) (*Pool, error) {
	if address == "" {
		return nil, errors.New("address is required")
	}
	if size <= 0 {
		return nil, fmt.Errorf("pool size must be positive, got %d", size)
	}

	pool := &Pool{Address: address, Size: size}

	// BUG 4: err is checked (not ignored with _) but the error is never returned
	if err := pool.ping(); err != nil {
		// forgot: return nil, fmt.Errorf("openPool: %w", err)
		_ = err // this silences the compiler but swallows the error
	}

	return pool, nil
}
