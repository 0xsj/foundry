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

// --- Token Bucket Strategy ---

// TokenBucketLimiter implements rate limiting using the token bucket algorithm.
// It allows bursts up to the bucket capacity and refills tokens at a steady rate.
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

// Compile-time interface guard
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

	// TODO: Implement token bucket algorithm
	// 1. Get or create a bucket for this key
	// 2. Calculate elapsed time since last refill
	// 3. Add tokens based on elapsed time and refill rate (cap at capacity)
	// 4. If tokens >= 1, consume one token and return true
	// 5. Otherwise return false

	_ = key // remove once implemented
	return false
}

func (t *TokenBucketLimiter) Name() string {
	return "token-bucket"
}

// --- Fixed Window Strategy ---

// FixedWindowLimiter implements rate limiting using fixed time windows.
// Requests are counted within discrete time windows that reset at boundaries.
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

	// TODO: Implement fixed window algorithm
	// 1. Calculate the current window ID: time.Now().UnixNano() / int64(f.window)
	// 2. Get or create a counter map for this key
	// 3. Clean up old windows (any windowID != current)
	// 4. Check if current window count < maxReqs
	// 5. If yes, increment and return true. If no, return false.

	_ = key // remove once implemented
	return false
}

func (f *FixedWindowLimiter) Name() string {
	return "fixed-window"
}

// --- Sliding Window Strategy ---

// SlidingWindowLimiter implements rate limiting using a sliding time window.
// It tracks individual request timestamps and counts requests within a rolling window.
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

	// TODO: Implement sliding window algorithm
	// 1. Get the current time
	// 2. Get or create a timestamp slice for this key
	// 3. Remove all timestamps older than (now - window)
	// 4. If len(remaining timestamps) < maxReqs, add current timestamp and return true
	// 5. Otherwise return false

	_ = key // remove once implemented
	return false
}

func (s *SlidingWindowLimiter) Name() string {
	return "sliding-window"
}

// --- Gateway ---

// Gateway routes requests through the appropriate rate limiter based on client tier.
type Gateway struct {
	mu       sync.RWMutex
	limiters map[string]RateLimiter // tier -> limiter
	fallback RateLimiter
}

// NewGateway creates a gateway with a fallback limiter and tier-specific limiters.
func NewGateway(fallback RateLimiter, tierLimiters map[string]RateLimiter) *Gateway {
	// TODO: Implement Gateway constructor
	// 1. Copy the tierLimiters map (don't store the caller's map directly)
	// 2. Store the fallback
	// 3. Return the Gateway

	_ = fallback     // remove once implemented
	_ = tierLimiters // remove once implemented
	return nil
}

// HandleRequest checks rate limiting for a client request.
// Returns whether the request is allowed and which limiter was used.
func (g *Gateway) HandleRequest(clientID string, tier string) (allowed bool, limiterName string) {
	// TODO: Implement request handling
	// 1. Look up the limiter for the given tier
	// 2. If not found, use the fallback limiter
	// 3. Call Allow with the clientID
	// 4. Return the result and the limiter's Name()

	_ = clientID // remove once implemented
	_ = tier     // remove once implemented
	return false, ""
}
