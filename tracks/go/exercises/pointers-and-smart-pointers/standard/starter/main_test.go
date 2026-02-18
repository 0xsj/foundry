package connpool

import (
	"testing"
	"time"
)

// ============================================================================
// CONNECTION TESTS
// ============================================================================

func TestNewConnection(t *testing.T) {
	conn := NewConnection("conn-1", "db.example.com", 5432)

	if conn == nil {
		t.Fatal("NewConnection returned nil")
	}
	if conn.ID != "conn-1" {
		t.Errorf("ID = %q, want %q", conn.ID, "conn-1")
	}
	if conn.Host != "db.example.com" {
		t.Errorf("Host = %q, want %q", conn.Host, "db.example.com")
	}
	if conn.Port != 5432 {
		t.Errorf("Port = %d, want %d", conn.Port, 5432)
	}
	if conn.CreatedAt.IsZero() {
		t.Error("CreatedAt should be set to time.Now(), not zero")
	}
	if conn.LastUsedAt != nil {
		t.Error("LastUsedAt should be nil initially")
	}
	if conn.IdleTimeoutSecs != nil {
		t.Error("IdleTimeoutSecs should be nil initially")
	}
}

// ============================================================================
// POOL CONSTRUCTOR TESTS
// ============================================================================

func TestNewConnectionPoolRejectsInvalidSize(t *testing.T) {
	_, err := NewConnectionPool(0)
	if err == nil {
		t.Error("NewConnectionPool(0) should return an error")
	}

	_, err = NewConnectionPool(-1)
	if err == nil {
		t.Error("NewConnectionPool(-1) should return an error")
	}
}

func TestNewConnectionPoolValidSize(t *testing.T) {
	p, err := NewConnectionPool(5)
	if err != nil {
		t.Fatalf("NewConnectionPool(5) returned error: %v", err)
	}
	if p == nil {
		t.Fatal("NewConnectionPool returned nil pool")
	}

	stats := p.Stats()
	if stats.MaxSize != 5 {
		t.Errorf("MaxSize = %d, want %d", stats.MaxSize, 5)
	}
	if stats.IdleCount != 0 {
		t.Errorf("IdleCount = %d, want 0", stats.IdleCount)
	}
	if stats.ActiveCount != 0 {
		t.Errorf("ActiveCount = %d, want 0", stats.ActiveCount)
	}
}

// ============================================================================
// SEED TESTS
// ============================================================================

func TestSeedAddsToIdlePool(t *testing.T) {
	p, _ := NewConnectionPool(3)
	conn := NewConnection("conn-1", "localhost", 5432)

	if err := p.Seed(conn); err != nil {
		t.Fatalf("Seed returned error: %v", err)
	}

	stats := p.Stats()
	if stats.IdleCount != 1 {
		t.Errorf("IdleCount = %d, want 1", stats.IdleCount)
	}
}

func TestSeedRejectsNilConnection(t *testing.T) {
	p, _ := NewConnectionPool(3)

	if err := p.Seed(nil); err == nil {
		t.Error("Seed(nil) should return an error")
	}
}

func TestSeedRejectsWhenAtCapacity(t *testing.T) {
	p, _ := NewConnectionPool(2)
	p.Seed(NewConnection("conn-1", "localhost", 5432))
	p.Seed(NewConnection("conn-2", "localhost", 5432))

	err := p.Seed(NewConnection("conn-3", "localhost", 5432))
	if err == nil {
		t.Error("Seed should return error when pool is at capacity")
	}
}

func TestSeedCountsActiveTowardCapacity(t *testing.T) {
	p, _ := NewConnectionPool(2)
	conn1 := NewConnection("conn-1", "localhost", 5432)
	conn2 := NewConnection("conn-2", "localhost", 5432)

	p.Seed(conn1)
	p.Seed(conn2)
	p.Borrow() // move conn1 to active

	// idle=1, active=1 => total=2 = maxSize
	err := p.Seed(NewConnection("conn-3", "localhost", 5432))
	if err == nil {
		t.Error("Seed should return error — pool at capacity (1 active + 1 idle = maxSize)")
	}
}

// ============================================================================
// BORROW TESTS
// ============================================================================

func TestBorrowMovesConnectionToActive(t *testing.T) {
	p, _ := NewConnectionPool(3)
	conn := NewConnection("conn-1", "localhost", 5432)
	p.Seed(conn)

	borrowed, err := p.Borrow()
	if err != nil {
		t.Fatalf("Borrow returned error: %v", err)
	}
	if borrowed == nil {
		t.Fatal("Borrow returned nil connection")
	}
	if borrowed != conn {
		t.Error("Borrow should return the seeded connection (pointer equality)")
	}

	stats := p.Stats()
	if stats.IdleCount != 0 {
		t.Errorf("IdleCount = %d, want 0 after Borrow", stats.IdleCount)
	}
	if stats.ActiveCount != 1 {
		t.Errorf("ActiveCount = %d, want 1 after Borrow", stats.ActiveCount)
	}
}

func TestBorrowSetsLastUsedAt(t *testing.T) {
	p, _ := NewConnectionPool(3)
	conn := NewConnection("conn-1", "localhost", 5432)
	p.Seed(conn)

	if conn.LastUsedAt != nil {
		t.Error("LastUsedAt should be nil before Borrow")
	}

	before := time.Now()
	p.Borrow()
	after := time.Now()

	if conn.LastUsedAt == nil {
		t.Fatal("LastUsedAt should be set after Borrow")
	}
	if conn.LastUsedAt.Before(before) || conn.LastUsedAt.After(after) {
		t.Errorf("LastUsedAt = %v, want between %v and %v", *conn.LastUsedAt, before, after)
	}
}

func TestBorrowErrorWhenEmpty(t *testing.T) {
	p, _ := NewConnectionPool(3)

	_, err := p.Borrow()
	if err == nil {
		t.Error("Borrow should return an error when pool is empty")
	}
}

// ============================================================================
// RETURN TESTS
// ============================================================================

func TestReturnMovesConnectionBackToIdle(t *testing.T) {
	p, _ := NewConnectionPool(3)
	conn := NewConnection("conn-1", "localhost", 5432)
	p.Seed(conn)

	borrowed, _ := p.Borrow()

	if err := p.Return(borrowed); err != nil {
		t.Fatalf("Return returned error: %v", err)
	}

	stats := p.Stats()
	if stats.IdleCount != 1 {
		t.Errorf("IdleCount = %d, want 1 after Return", stats.IdleCount)
	}
	if stats.ActiveCount != 0 {
		t.Errorf("ActiveCount = %d, want 0 after Return", stats.ActiveCount)
	}
}

func TestReturnErrorOnNil(t *testing.T) {
	p, _ := NewConnectionPool(3)

	if err := p.Return(nil); err == nil {
		t.Error("Return(nil) should return an error")
	}
}

func TestReturnErrorForUnknownConnection(t *testing.T) {
	p, _ := NewConnectionPool(3)
	foreignConn := NewConnection("foreign", "other.host", 5432)

	err := p.Return(foreignConn)
	if err == nil {
		t.Error("Return should error for a connection not in the active list")
	}
}

// ============================================================================
// BORROW / RETURN CYCLE
// ============================================================================

func TestBorrowReturnCycle(t *testing.T) {
	p, _ := NewConnectionPool(2)
	p.Seed(NewConnection("conn-1", "localhost", 5432))
	p.Seed(NewConnection("conn-2", "localhost", 5432))

	// Borrow both
	c1, _ := p.Borrow()
	c2, _ := p.Borrow()

	stats := p.Stats()
	if stats.IdleCount != 0 || stats.ActiveCount != 2 {
		t.Errorf("after borrowing both: idle=%d active=%d, want idle=0 active=2",
			stats.IdleCount, stats.ActiveCount)
	}

	// Third borrow should fail
	_, err := p.Borrow()
	if err == nil {
		t.Error("Borrow should fail when all connections are active")
	}

	// Return one and borrow again
	p.Return(c1)
	c3, err := p.Borrow()
	if err != nil {
		t.Fatalf("Borrow after Return failed: %v", err)
	}
	if c3 != c1 {
		t.Error("should get back the returned connection")
	}

	// Clean up
	p.Return(c2)
	p.Return(c3)

	finalStats := p.Stats()
	if finalStats.IdleCount != 2 || finalStats.ActiveCount != 0 {
		t.Errorf("after returning all: idle=%d active=%d, want idle=2 active=0",
			finalStats.IdleCount, finalStats.ActiveCount)
	}
}

// ============================================================================
// IsExpired TESTS
// ============================================================================

func TestIsExpiredNilTimeout(t *testing.T) {
	conn := NewConnection("conn-1", "localhost", 5432)
	// IdleTimeoutSecs is nil — no timeout configured
	if IsExpired(conn) {
		t.Error("IsExpired should return false when IdleTimeoutSecs is nil")
	}
}

func TestIsExpiredNilLastUsedAt(t *testing.T) {
	conn := NewConnection("conn-1", "localhost", 5432)
	timeout := 5
	conn.IdleTimeoutSecs = &timeout
	// LastUsedAt is still nil — never borrowed

	if IsExpired(conn) {
		t.Error("IsExpired should return false when LastUsedAt is nil")
	}
}

func TestIsExpiredNotYetExpired(t *testing.T) {
	conn := NewConnection("conn-1", "localhost", 5432)
	timeout := 3600 // 1 hour — should not expire immediately
	conn.IdleTimeoutSecs = &timeout

	now := time.Now()
	conn.LastUsedAt = &now

	if IsExpired(conn) {
		t.Error("IsExpired should return false when idle time is within timeout")
	}
}

func TestIsExpiredPastTimeout(t *testing.T) {
	conn := NewConnection("conn-1", "localhost", 5432)
	timeout := 1 // 1 second
	conn.IdleTimeoutSecs = &timeout

	pastTime := time.Now().Add(-2 * time.Second) // 2 seconds ago
	conn.LastUsedAt = &pastTime

	if !IsExpired(conn) {
		t.Error("IsExpired should return true when idle time exceeds timeout")
	}
}
