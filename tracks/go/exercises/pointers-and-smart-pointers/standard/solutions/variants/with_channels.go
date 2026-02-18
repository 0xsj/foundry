// Package connpool provides an alternate ConnectionPool implementation using
// a buffered channel as the idle queue instead of a mutex-protected slice.
//
// Trade-offs vs the mutex approach:
//   - Borrow and Return become O(1) non-blocking channel operations
//   - No mutex needed — channel operations are already goroutine-safe
//   - Pool size is fixed at creation (channel capacity = maxSize)
//   - Cannot inspect idle/active split without draining channel (Stats is approximate)
//   - Simpler Borrow/Return, but harder to implement strict capacity tracking
package connpool

import (
	"fmt"
	"time"
)

// ChannelPool uses a buffered channel as the idle connection queue.
// The channel capacity IS the pool capacity — elegant but slightly less flexible.
type ChannelPool struct {
	idle    chan *Connection
	maxSize int
}

// NewChannelPool creates a pool backed by a buffered channel.
func NewChannelPool(maxSize int) (*ChannelPool, error) {
	if maxSize < 1 {
		return nil, fmt.Errorf("maxSize must be at least 1, got %d", maxSize)
	}
	return &ChannelPool{
		idle:    make(chan *Connection, maxSize),
		maxSize: maxSize,
	}, nil
}

// Seed adds a connection to the idle channel.
// Returns an error if the channel is full (non-blocking send).
func (p *ChannelPool) Seed(conn *Connection) error {
	if conn == nil {
		return fmt.Errorf("cannot seed a nil connection")
	}
	select {
	case p.idle <- conn:
		return nil
	default:
		return fmt.Errorf("pool is at capacity (%d)", p.maxSize)
	}
}

// Borrow retrieves a connection from the idle channel.
// Non-blocking: returns an error immediately if no connections are available.
func (p *ChannelPool) Borrow() (*Connection, error) {
	select {
	case conn := <-p.idle:
		now := time.Now()
		conn.LastUsedAt = &now
		return conn, nil
	default:
		return nil, fmt.Errorf("no idle connections available")
	}
}

// Return puts a connection back into the idle channel.
func (p *ChannelPool) Return(conn *Connection) error {
	if conn == nil {
		return fmt.Errorf("cannot return a nil connection")
	}
	select {
	case p.idle <- conn:
		return nil
	default:
		// Channel is full — this shouldn't happen if Borrow/Return are balanced
		return fmt.Errorf("pool is full — connection %q cannot be returned", conn.ID)
	}
}

// IdleCount returns an approximate count of idle connections.
// Note: len(chan) is a snapshot — the value may change immediately.
func (p *ChannelPool) IdleCount() int {
	return len(p.idle)
}
