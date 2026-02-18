// Package ratelimiter — reference test suite.
//
// Patterns demonstrated:
//  1. StubClock test double for deterministic time control
//  2. t.Helper() assertion helpers (mustAllow, mustDeny)
//  3. Table-driven tests with t.Run
//  4. Concurrency test with goroutines + race detector
//  5. BenchmarkAllow with b.ResetTimer and b.ReportAllocs
//
// Run tests:       go test -v ./...
// Race detector:   go test -race ./...
// Benchmarks:      go test -bench=. -benchmem ./...
package ratelimiter

import (
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// ============================================================================
// StubClock — test double that controls time
// ============================================================================

// StubClock is a controllable clock for testing.
// Define it in the test file — it's a test artifact, not production code.
type StubClock struct {
	mu  sync.Mutex
	now time.Time
}

func newStubClock() *StubClock {
	return &StubClock{now: time.Unix(0, 0)}
}

func (c *StubClock) Now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.now
}

// Advance moves the clock forward by d.
func (c *StubClock) Advance(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.now = c.now.Add(d)
}

// ============================================================================
// Test helpers with t.Helper()
// ============================================================================

// mustAllow asserts that Allow() returns true.
// t.Helper() makes failures report at the call site.
func mustAllow(t *testing.T, rl *RateLimiter) {
	t.Helper()
	if !rl.Allow() {
		t.Errorf("Allow() = false, want true (tokens = %d)", rl.Tokens())
	}
}

// mustDeny asserts that Allow() returns false.
func mustDeny(t *testing.T, rl *RateLimiter) {
	t.Helper()
	if rl.Allow() {
		t.Errorf("Allow() = true, want false (tokens = %d)", rl.Tokens())
	}
}

// ============================================================================
// Tests
// ============================================================================

func TestAllow_WithinLimit(t *testing.T) {
	rl := New(3, time.Minute, newStubClock())

	mustAllow(t, rl)
	mustAllow(t, rl)
	mustAllow(t, rl)
}

func TestAllow_ExceedsLimit(t *testing.T) {
	rl := New(2, time.Minute, newStubClock())

	mustAllow(t, rl)
	mustAllow(t, rl)
	mustDeny(t, rl)  // bucket exhausted
	mustDeny(t, rl)  // still denied — no refill yet
}

func TestAllow_RefillAfterWindow(t *testing.T) {
	// This test demonstrates WHY you need a stub clock.
	// Without it, you'd need time.Sleep(time.Minute) — a 60s test.
	// With the stub, the test runs in microseconds.
	clock := newStubClock()
	rl := New(3, time.Minute, clock)

	// Exhaust the bucket
	mustAllow(t, rl)
	mustAllow(t, rl)
	mustAllow(t, rl)
	mustDeny(t, rl)

	// Advance past the window — tokens should refill
	clock.Advance(time.Minute)

	// Now requests should be allowed again
	mustAllow(t, rl)
	mustAllow(t, rl)
	mustAllow(t, rl)
	mustDeny(t, rl)
}

func TestAllow_WindowNotYetElapsed_NoRefill(t *testing.T) {
	clock := newStubClock()
	rl := New(2, time.Minute, clock)

	mustAllow(t, rl)
	mustAllow(t, rl)
	mustDeny(t, rl)

	// Advance only halfway — no refill yet
	clock.Advance(59 * time.Second)
	mustDeny(t, rl)

	// Advance past the window — refill now
	clock.Advance(2 * time.Second)
	mustAllow(t, rl)
}

func TestTokens_DecreasesOnAllow(t *testing.T) {
	rl := New(5, time.Minute, newStubClock())

	if got := rl.Tokens(); got != 5 {
		t.Errorf("initial tokens = %d, want 5", got)
	}

	rl.Allow()
	if got := rl.Tokens(); got != 4 {
		t.Errorf("after 1 Allow: tokens = %d, want 4", got)
	}

	rl.Allow()
	rl.Allow()
	if got := rl.Tokens(); got != 2 {
		t.Errorf("after 3 Allows: tokens = %d, want 2", got)
	}
}

// ============================================================================
// Table-driven test — covers multiple rate configurations
// ============================================================================

func TestAllow_TableDriven(t *testing.T) {
	tests := []struct {
		name        string
		rate        int
		window      time.Duration
		requests    int  // total requests to fire
		wantAllowed int  // how many should be allowed
		wantDenied  int  // how many should be denied
	}{
		{
			name:        "rate 1 per minute, 3 requests",
			rate:        1,
			window:      time.Minute,
			requests:    3,
			wantAllowed: 1,
			wantDenied:  2,
		},
		{
			name:        "rate 5 per minute, 5 requests — exactly at limit",
			rate:        5,
			window:      time.Minute,
			requests:    5,
			wantAllowed: 5,
			wantDenied:  0,
		},
		{
			name:        "rate 5 per minute, 6 requests — one over limit",
			rate:        5,
			window:      time.Minute,
			requests:    6,
			wantAllowed: 5,
			wantDenied:  1,
		},
		{
			name:        "rate 10, 0 requests",
			rate:        10,
			window:      time.Minute,
			requests:    0,
			wantAllowed: 0,
			wantDenied:  0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rl := New(tt.rate, tt.window, newStubClock())

			var allowed, denied int
			for i := 0; i < tt.requests; i++ {
				if rl.Allow() {
					allowed++
				} else {
					denied++
				}
			}

			if allowed != tt.wantAllowed {
				t.Errorf("allowed = %d, want %d", allowed, tt.wantAllowed)
			}
			if denied != tt.wantDenied {
				t.Errorf("denied = %d, want %d", denied, tt.wantDenied)
			}
		})
	}
}

// ============================================================================
// Concurrency test — run with go test -race ./...
// ============================================================================

func TestAllow_Concurrent(t *testing.T) {
	const goroutines = 50
	const rate = 10

	rl := New(rate, time.Minute, newStubClock())

	var wg sync.WaitGroup
	var allowed atomic.Int32

	wg.Add(goroutines)
	for i := 0; i < goroutines; i++ {
		go func() {
			defer wg.Done()
			if rl.Allow() {
				allowed.Add(1)
			}
		}()
	}
	wg.Wait()

	// Exactly `rate` requests should have been allowed.
	// The race detector (go test -race) will catch any data race.
	if got := int(allowed.Load()); got != rate {
		t.Errorf("concurrent: allowed = %d, want %d", got, rate)
	}
}

// ============================================================================
// Benchmark
// ============================================================================

func BenchmarkAllow(b *testing.B) {
	// Set a very high rate so Allow() never denies during the benchmark.
	// We're measuring the cost of Allow() on the hot (allowed) path.
	rl := New(b.N+1, time.Hour, RealClock{})

	b.ResetTimer()  // don't count New() time
	b.ReportAllocs() // should see 0 allocs/op — no heap allocation in Allow()

	for i := 0; i < b.N; i++ {
		rl.Allow()
	}
}

// BenchmarkAllow_Denied measures the deny path — may differ from allow path.
func BenchmarkAllow_Denied(b *testing.B) {
	rl := New(0, time.Hour, RealClock{}) // rate 0 — always denies

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		rl.Allow()
	}
}
