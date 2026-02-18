package dbloader

import (
	"errors"
	"testing"
	"time"
)

// Test for Bug 1: %v instead of %w breaks errors.Is for ErrTimeout
func TestLoadConfig_TimeoutDetectable(t *testing.T) {
	cfg := ConnConfig{
		Host:     "db.prod.internal",
		Port:     5432,
		Database: "payments",
		Timeout:  10 * time.Millisecond, // too short — should produce timeout error
	}
	err := loadConfig(cfg)
	if err == nil {
		t.Fatal("loadConfig() error = nil, want timeout error")
	}
	if !errors.Is(err, ErrTimeout) {
		t.Errorf("errors.Is(err, ErrTimeout) = false, want true\nerr = %v", err)
	}
}

// Test for Bug 2: %v instead of %w breaks errors.Is for ErrConnectionRefused
func TestConnect_SentinelPreservedThroughWrap(t *testing.T) {
	err := connect("down.db.internal:5432")
	if err == nil {
		t.Fatal("connect() error = nil, want error")
	}
	if !errors.Is(err, ErrConnectionRefused) {
		t.Errorf("errors.Is(err, ErrConnectionRefused) = false, want true\nerr = %v", err)
	}
}

// Test for Bug 3: panic instead of error for empty address
func TestConnectWithRetry_EmptyAddressReturnsError(t *testing.T) {
	defer func() {
		if r := recover(); r != nil {
			t.Errorf("ConnectWithRetry panicked with %v, want error return", r)
		}
	}()

	err := ConnectWithRetry("", 3)
	if err == nil {
		t.Fatal("ConnectWithRetry(\"\") error = nil, want error")
	}
}

// Test for Bug 4: swallowed ping error causes silent success
func TestOpenPool_PingFailureReturnsError(t *testing.T) {
	pool, err := openPool("dead-host:5432", 5)
	if err == nil {
		t.Fatal("openPool() error = nil, want ping error")
	}
	if pool != nil {
		t.Error("openPool() returned non-nil pool despite ping failure")
	}
}

// Sanity checks — these should pass even before bug fixes
func TestConnect_Success(t *testing.T) {
	if err := connect("live.db.internal:5432"); err != nil {
		t.Errorf("connect(good address) error = %v, want nil", err)
	}
}

func TestConnectWithRetry_Success(t *testing.T) {
	if err := ConnectWithRetry("live.db.internal:5432", 3); err != nil {
		t.Errorf("ConnectWithRetry(good address) error = %v, want nil", err)
	}
}

func TestOpenPool_Success(t *testing.T) {
	pool, err := openPool("live.db.internal:5432", 5)
	if err != nil {
		t.Fatalf("openPool(good address) error = %v, want nil", err)
	}
	if pool == nil {
		t.Error("openPool() returned nil pool")
	}
}
