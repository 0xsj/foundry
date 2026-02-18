package connpool

import (
	"fmt"
	"sync"
	"time"
)

// ============================================================================
// CONNECTION
// A single reusable database connection.
// LastUsedAt and IdleTimeoutSecs are pointer fields — nil means "not set".
// ============================================================================

// Connection represents a single reusable database connection.
type Connection struct {
	ID              string
	Host            string
	Port            int
	CreatedAt       time.Time
	LastUsedAt      *time.Time // nil until first borrow
	IdleTimeoutSecs *int       // nil means no idle timeout
}

// NewConnection creates a new connection. CreatedAt is set to now.
// LastUsedAt and IdleTimeoutSecs start as nil — they're set later.
// TODO: Implement
func NewConnection(id, host string, port int) *Connection {
	return nil
}

// ============================================================================
// POOL STATS
// A snapshot of pool state. Returned by value — it's not live data.
// ============================================================================

// PoolStats is a snapshot of pool state at a point in time.
type PoolStats struct {
	IdleCount   int
	ActiveCount int
	MaxSize     int
}

// ============================================================================
// CONNECTION POOL
// Manages idle and active connections under a mutex.
// ============================================================================

// ConnectionPool manages a bounded set of reusable database connections.
type ConnectionPool struct {
	maxSize int
	idle    []*Connection
	active  []*Connection
	mu      sync.Mutex
}

// NewConnectionPool creates a pool with the given maximum size.
// Returns an error if maxSize < 1.
// TODO: Implement
func NewConnectionPool(maxSize int) (*ConnectionPool, error) {
	return nil, fmt.Errorf("not implemented")
}

// Seed adds a connection to the idle pool.
// Returns an error if conn is nil or the pool is at capacity.
// Think about: what counts as "at capacity"?
// TODO: Implement
func (p *ConnectionPool) Seed(conn *Connection) error {
	return fmt.Errorf("not implemented")
}

// Borrow removes a connection from idle and marks it active.
// Updates LastUsedAt on the borrowed connection.
// Returns an error (not a panic) if no idle connections are available.
// TODO: Implement
func (p *ConnectionPool) Borrow() (*Connection, error) {
	return nil, fmt.Errorf("not implemented")
}

// Return moves a connection from active back to idle.
// Returns an error if conn is nil or not found in the active list.
// TODO: Implement
func (p *ConnectionPool) Return(conn *Connection) error {
	return fmt.Errorf("not implemented")
}

// Stats returns a snapshot of the current pool state.
// TODO: Implement
func (p *ConnectionPool) Stats() PoolStats {
	return PoolStats{}
}

// ============================================================================
// IsExpired
// A pure function (no mutex needed) — reads only from the Connection struct.
// ============================================================================

// IsExpired reports whether a connection has been idle longer than its configured
// IdleTimeoutSecs. Returns false if either pointer field is nil.
// TODO: Implement
func IsExpired(conn *Connection) bool {
	return false
}
