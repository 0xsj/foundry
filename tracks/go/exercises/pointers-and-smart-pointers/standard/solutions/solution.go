// Package connpool implements a connection pool manager.
// Demonstrates: constructor returning *T, *T optional fields,
// nil checks before dereference, pointer receivers for mutation,
// pointer equality for identity, sync.Mutex for shared state.
package connpool

import (
	"fmt"
	"sync"
	"time"
)

// ============================================================================
// CONNECTION
// ============================================================================

// Connection represents a single reusable database connection.
// LastUsedAt and IdleTimeoutSecs are *T fields — nil means "not set yet"
// and "no timeout configured" respectively. This is the *T-as-optional pattern.
type Connection struct {
	ID              string
	Host            string
	Port            int
	CreatedAt       time.Time
	LastUsedAt      *time.Time // nil until first borrow
	IdleTimeoutSecs *int       // nil means no idle timeout
}

// NewConnection creates a Connection with CreatedAt set to now.
// Returns *Connection because:
//   - The pool stores []*Connection — all borrows return the same pointer
//   - Mutations (setting LastUsedAt) need to be visible everywhere the connection is held
func NewConnection(id, host string, port int) *Connection {
	return &Connection{
		ID:        id,
		Host:      host,
		Port:      port,
		CreatedAt: time.Now(),
		// LastUsedAt:      nil — intentionally left unset (pointer to optional)
		// IdleTimeoutSecs: nil — intentionally left unset (pointer to optional)
	}
}

// ============================================================================
// POOL STATS
// Returned by value — it's a snapshot, not live state.
// Callers get their own copy; mutations don't affect the pool.
// ============================================================================

// PoolStats is a point-in-time snapshot of pool occupancy.
type PoolStats struct {
	IdleCount   int
	ActiveCount int
	MaxSize     int
}

// ============================================================================
// CONNECTION POOL
// ============================================================================

// ConnectionPool manages a bounded set of reusable database connections.
// Connections live in either the idle list (available) or active list (in use).
// All mutations are protected by a sync.Mutex — safe for concurrent use.
type ConnectionPool struct {
	maxSize int
	idle    []*Connection // available for borrowing
	active  []*Connection // currently borrowed by callers
	mu      sync.Mutex
}

// NewConnectionPool creates a pool that can hold at most maxSize connections total.
// Returns an error for invalid sizes rather than panicking — callers decide how to
// handle configuration errors.
func NewConnectionPool(maxSize int) (*ConnectionPool, error) {
	if maxSize < 1 {
		return nil, fmt.Errorf("maxSize must be at least 1, got %d", maxSize)
	}
	return &ConnectionPool{
		maxSize: maxSize,
		// idle and active start as nil slices — valid Go, len == 0
		// No need to pre-allocate with make unless we know the capacity ahead of time
	}, nil
}

// Seed adds a pre-built connection to the idle pool.
// Capacity check: idle + active must be < maxSize (total connections, not just idle).
// A nil connection is rejected — this would cause panics on later dereference.
func (p *ConnectionPool) Seed(conn *Connection) error {
	if conn == nil {
		return fmt.Errorf("cannot seed a nil connection")
	}

	p.mu.Lock()
	defer p.mu.Unlock()

	// Count both idle and active toward pool capacity.
	// A borrowed connection is still "owned" by the pool — it's just in use.
	if len(p.idle)+len(p.active) >= p.maxSize {
		return fmt.Errorf("pool is at capacity (%d connections)", p.maxSize)
	}

	p.idle = append(p.idle, conn)
	return nil
}

// Borrow removes a connection from idle and marks it active.
// Updates LastUsedAt — this is why *time.Time was chosen over time.Time for
// that field: nil means "never used", distinct from any real timestamp.
func (p *ConnectionPool) Borrow() (*Connection, error) {
	p.mu.Lock()
	defer p.mu.Unlock()

	if len(p.idle) == 0 {
		return nil, fmt.Errorf("no idle connections available (active=%d, max=%d)",
			len(p.active), p.maxSize)
	}

	// Take from the front of the idle list (FIFO — use oldest connection first
	// to avoid some connections sitting idle forever).
	conn := p.idle[0]
	p.idle = p.idle[1:]
	p.active = append(p.active, conn)

	// Set LastUsedAt. Must assign to variable first — can't take address of time.Now() directly.
	// time.Now() returns a value, and you can only take addresses of addressable things (variables).
	now := time.Now()
	conn.LastUsedAt = &now // sets the field on the actual Connection in memory

	return conn, nil
}

// Return moves a connection from active back to idle.
// Uses pointer equality (c == conn) to find the connection — not value equality.
// Two *Connection values are equal only if they point to the exact same Connection.
func (p *ConnectionPool) Return(conn *Connection) error {
	if conn == nil {
		return fmt.Errorf("cannot return a nil connection")
	}

	p.mu.Lock()
	defer p.mu.Unlock()

	// Find and remove from active list.
	// We compare pointers (identities), not struct values.
	found := false
	for i, c := range p.active {
		if c == conn { // pointer equality — same memory address, same Connection
			// Remove element i: copy tail over it and truncate
			p.active = append(p.active[:i], p.active[i+1:]...)
			found = true
			break
		}
	}

	if !found {
		return fmt.Errorf("connection %q is not in the active list", conn.ID)
	}

	p.idle = append(p.idle, conn)
	return nil
}

// Stats returns a snapshot of pool occupancy.
// Returns PoolStats by value — the caller gets a copy that won't change
// if the pool state changes after this call returns.
func (p *ConnectionPool) Stats() PoolStats {
	p.mu.Lock()
	defer p.mu.Unlock()

	return PoolStats{
		IdleCount:   len(p.idle),
		ActiveCount: len(p.active),
		MaxSize:     p.maxSize,
	}
}

// ============================================================================
// IsExpired
// Pure function — reads only from the Connection, no pool state needed.
// Demonstrates *T nil checks before dereference.
// ============================================================================

// IsExpired reports whether conn has been idle longer than its IdleTimeoutSecs.
//
// Returns false if:
//   - IdleTimeoutSecs is nil — no timeout configured
//   - LastUsedAt is nil — never borrowed, so "idle since creation"; we treat
//     un-borrowed connections as not expired (they haven't been used yet)
func IsExpired(conn *Connection) bool {
	// Guard: if no timeout is configured, the connection never expires
	if conn.IdleTimeoutSecs == nil {
		return false
	}

	// Guard: if never used, we don't consider it expired
	// (it's fresh — expiry tracks idle time after last use, not time since creation)
	if conn.LastUsedAt == nil {
		return false
	}

	// Both pointers are non-nil — safe to dereference
	idleDuration := time.Since(*conn.LastUsedAt)
	timeout := time.Duration(*conn.IdleTimeoutSecs) * time.Second
	return idleDuration > timeout
}
