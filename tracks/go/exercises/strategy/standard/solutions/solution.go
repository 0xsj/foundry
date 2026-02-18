package strategy

import (
	"sync"
	"time"
)

// RateLimiter defines the strategy interface for rate limiting.
// All rate-limiting algorithms must satisfy this interface.
type RateLimiter interface {
	// Allow returns true if the request for the given key should be permitted.
	Allow(key string) bool
	// Name returns the name of this rate-limiting strategy.
	Name() string
}

// =============================================================================
// Token Bucket Strategy
// =============================================================================

// TokenBucketLimiter implements rate limiting using the token bucket algorithm.
// It allows bursts up to the bucket capacity and refills tokens at a steady rate.
//
// How it works:
//   - Each key gets its own bucket with a fixed capacity (max tokens).
//   - Tokens are consumed on each Allow() call (1 token per request).
//   - Tokens refill at a steady rate (refillRate tokens/second).
//   - Refill is "lazy" — calculated from elapsed time, not a background goroutine.
//   - If tokens >= 1, the request is allowed. Otherwise, denied.
//
// Trade-offs:
//   - Allows bursts (up to capacity) — good for bursty traffic patterns.
//   - Smooth refill — doesn't create cliff edges like fixed windows.
//   - Slightly more complex than fixed window.
type TokenBucketLimiter struct {
	mu         sync.Mutex
	capacity   float64
	refillRate float64 // tokens per second
	buckets    map[string]*bucket
}

type bucket struct {
	tokens     float64
	lastRefill time.Time
}

// Compile-time interface guard: *TokenBucketLimiter must satisfy RateLimiter.
var _ RateLimiter = (*TokenBucketLimiter)(nil)

// NewTokenBucketLimiter creates a token bucket limiter.
// capacity is the maximum tokens (burst size).
// refillRate is how many tokens are added per second.
func NewTokenBucketLimiter(capacity float64, refillRate float64) *TokenBucketLimiter {
	return &TokenBucketLimiter{
		capacity:   capacity,
		refillRate: refillRate,
		buckets:    make(map[string]*bucket),
	}
}

func (t *TokenBucketLimiter) Allow(key string) bool {
	t.mu.Lock()
	defer t.mu.Unlock()

	now := time.Now()

	// Get or create bucket for this key
	b, ok := t.buckets[key]
	if !ok {
		// New key: start with a full bucket
		b = &bucket{
			tokens:     t.capacity,
			lastRefill: now,
		}
		t.buckets[key] = b
	}

	// Lazy refill: calculate tokens earned since last refill
	elapsed := now.Sub(b.lastRefill).Seconds()
	b.tokens += elapsed * t.refillRate
	if b.tokens > t.capacity {
		b.tokens = t.capacity // cap at capacity
	}
	b.lastRefill = now

	// Try to consume a token
	if b.tokens >= 1 {
		b.tokens--
		return true
	}

	return false
}

func (t *TokenBucketLimiter) Name() string {
	return "token-bucket"
}

// =============================================================================
// Fixed Window Strategy
// =============================================================================

// FixedWindowLimiter implements rate limiting using fixed time windows.
// Requests are counted within discrete time windows that reset at boundaries.
//
// How it works:
//   - Time is divided into fixed-duration windows (e.g., 1-second chunks).
//   - Each window has an independent request counter per key.
//   - When a new window starts, the counter resets to zero.
//   - If the counter < maxReqs, the request is allowed. Otherwise, denied.
//
// Trade-offs:
//   - Simple and efficient (O(1) per request).
//   - Has a boundary problem: a burst at the end of one window + start of next
//     can exceed the intended rate (2x max in a short period).
//   - Good enough for most use cases; use sliding window for stricter accuracy.
type FixedWindowLimiter struct {
	mu       sync.Mutex
	maxReqs  int
	window   time.Duration
	counters map[string]map[int64]int // key -> windowID -> count
}

var _ RateLimiter = (*FixedWindowLimiter)(nil)

// NewFixedWindowLimiter creates a fixed window limiter.
// maxReqs is the maximum requests per window.
// window is the duration of each window.
func NewFixedWindowLimiter(maxReqs int, window time.Duration) *FixedWindowLimiter {
	return &FixedWindowLimiter{
		maxReqs:  maxReqs,
		window:   window,
		counters: make(map[string]map[int64]int),
	}
}

func (f *FixedWindowLimiter) Allow(key string) bool {
	f.mu.Lock()
	defer f.mu.Unlock()

	// Calculate current window ID
	windowID := time.Now().UnixNano() / int64(f.window)

	// Get or create counter map for this key
	windows, ok := f.counters[key]
	if !ok {
		windows = make(map[int64]int)
		f.counters[key] = windows
	}

	// Clean up old windows to prevent memory leaks.
	// Only the current window matters; delete everything else.
	for id := range windows {
		if id != windowID {
			delete(windows, id)
		}
	}

	// Check and increment
	count := windows[windowID]
	if count < f.maxReqs {
		windows[windowID] = count + 1
		return true
	}

	return false
}

func (f *FixedWindowLimiter) Name() string {
	return "fixed-window"
}

// =============================================================================
// Sliding Window Strategy
// =============================================================================

// SlidingWindowLimiter implements rate limiting using a sliding time window.
// It tracks individual request timestamps and counts requests within a rolling window.
//
// How it works:
//   - Each request timestamp is stored per key.
//   - On each Allow() call, timestamps older than (now - window) are removed.
//   - If remaining timestamp count < maxReqs, the request is allowed and recorded.
//   - Otherwise, denied.
//
// Trade-offs:
//   - Most accurate — no boundary problem like fixed window.
//   - Higher memory usage — stores individual timestamps.
//   - O(n) cleanup per call (where n is timestamps in window).
//   - Best for strict rate limiting where accuracy matters.
type SlidingWindowLimiter struct {
	mu         sync.Mutex
	maxReqs    int
	window     time.Duration
	timestamps map[string][]time.Time // key -> sorted timestamps of recent requests
}

var _ RateLimiter = (*SlidingWindowLimiter)(nil)

// NewSlidingWindowLimiter creates a sliding window limiter.
// maxReqs is the maximum requests within the sliding window.
// window is the duration of the sliding window.
func NewSlidingWindowLimiter(maxReqs int, window time.Duration) *SlidingWindowLimiter {
	return &SlidingWindowLimiter{
		maxReqs:    maxReqs,
		window:     window,
		timestamps: make(map[string][]time.Time),
	}
}

func (s *SlidingWindowLimiter) Allow(key string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()

	now := time.Now()
	cutoff := now.Add(-s.window)

	// Get timestamps for this key
	ts := s.timestamps[key]

	// Remove expired timestamps.
	// Since timestamps are appended in order, find the first valid index.
	validFrom := 0
	for validFrom < len(ts) && ts[validFrom].Before(cutoff) {
		validFrom++
	}
	ts = ts[validFrom:]

	// Check if under the limit
	if len(ts) < s.maxReqs {
		ts = append(ts, now)
		s.timestamps[key] = ts
		return true
	}

	// Store the cleaned slice even when denied (frees expired entries)
	s.timestamps[key] = ts
	return false
}

func (s *SlidingWindowLimiter) Name() string {
	return "sliding-window"
}

// =============================================================================
// Gateway
// =============================================================================

// Gateway routes requests through the appropriate rate limiter based on client tier.
// It acts as the "context" in the Strategy pattern — it holds a reference to the
// current strategy and delegates to it.
type Gateway struct {
	mu       sync.RWMutex
	limiters map[string]RateLimiter // tier -> limiter
	fallback RateLimiter
}

// NewGateway creates a gateway with a fallback limiter and tier-specific limiters.
func NewGateway(fallback RateLimiter, tierLimiters map[string]RateLimiter) *Gateway {
	// Copy the map to avoid the caller mutating our internal state
	m := make(map[string]RateLimiter, len(tierLimiters))
	for k, v := range tierLimiters {
		m[k] = v
	}

	return &Gateway{
		limiters: m,
		fallback: fallback,
	}
}

// HandleRequest checks rate limiting for a client request.
// Returns whether the request is allowed and which limiter was used.
func (g *Gateway) HandleRequest(clientID string, tier string) (allowed bool, limiterName string) {
	g.mu.RLock()
	limiter, ok := g.limiters[tier]
	if !ok {
		limiter = g.fallback
	}
	g.mu.RUnlock()

	return limiter.Allow(clientID), limiter.Name()
}
