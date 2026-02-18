package debugging

import (
	"sync"
	"time"
)

// RateLimiter defines the strategy interface.
type RateLimiter interface {
	Allow(key string) bool
	Name() string
}

// =============================================================================
// Bug 1: TokenBucketLimiter does not satisfy RateLimiter.
// The method receiver is wrong — Name() is on value receiver but Allow() is on
// pointer receiver. More critically, the Allow method has a different signature
// than what the interface requires.
// =============================================================================

type TokenBucketLimiter struct {
	mu         sync.Mutex
	capacity   float64
	refillRate float64
	buckets    map[string]*tbucket
}

type tbucket struct {
	tokens     float64
	lastRefill time.Time
}

// BUG: Uncomment the next line to see the compile error. The Allow method
// takes (key string, weight int) but the interface requires (key string).
// var _ RateLimiter = (*TokenBucketLimiter)(nil)

func NewTokenBucketLimiter(capacity float64, refillRate float64) *TokenBucketLimiter {
	return &TokenBucketLimiter{
		capacity:   capacity,
		refillRate: refillRate,
		buckets:    make(map[string]*tbucket),
	}
}

// BUG 1: Wrong signature — has extra parameter (weight int) that doesn't
// match the interface's Allow(key string) bool.
func (t *TokenBucketLimiter) Allow(key string, weight int) bool {
	t.mu.Lock()
	defer t.mu.Unlock()

	now := time.Now()
	b, ok := t.buckets[key]
	if !ok {
		b = &tbucket{tokens: t.capacity, lastRefill: now}
		t.buckets[key] = b
	}

	elapsed := now.Sub(b.lastRefill).Seconds()
	b.tokens += elapsed * t.refillRate
	if b.tokens > t.capacity {
		b.tokens = t.capacity
	}
	b.lastRefill = now

	cost := float64(weight)
	if b.tokens >= cost {
		b.tokens -= cost
		return true
	}
	return false
}

func (t *TokenBucketLimiter) Name() string { return "token-bucket" }

// =============================================================================
// Fixed Window (correct implementation — no bugs here)
// =============================================================================

type FixedWindowLimiter struct {
	mu       sync.Mutex
	maxReqs  int
	window   time.Duration
	counters map[string]map[int64]int
}

var _ RateLimiter = (*FixedWindowLimiter)(nil)

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

	windowID := time.Now().UnixNano() / int64(f.window)
	windows, ok := f.counters[key]
	if !ok {
		windows = make(map[int64]int)
		f.counters[key] = windows
	}
	for id := range windows {
		if id != windowID {
			delete(windows, id)
		}
	}
	count := windows[windowID]
	if count < f.maxReqs {
		windows[windowID] = count + 1
		return true
	}
	return false
}

func (f *FixedWindowLimiter) Name() string { return "fixed-window" }

// =============================================================================
// Bug 4: SlidingWindowLimiter has an off-by-one in timestamp cleanup.
// =============================================================================

type SlidingWindowLimiter struct {
	mu         sync.Mutex
	maxReqs    int
	window     time.Duration
	timestamps map[string][]time.Time
}

var _ RateLimiter = (*SlidingWindowLimiter)(nil)

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

	ts := s.timestamps[key]

	// BUG 4: Uses Before(cutoff) || Equal(cutoff) via !After(cutoff),
	// but then skips one MORE entry with validFrom starting at 1 instead of 0.
	// This removes one valid (non-expired) timestamp on each call,
	// allowing more requests through than the limit.
	validFrom := 1 // BUG: should be 0
	for validFrom < len(ts) && !ts[validFrom].After(cutoff) {
		validFrom++
	}
	if validFrom > len(ts) {
		validFrom = len(ts)
	}
	ts = ts[validFrom:]

	if len(ts) < s.maxReqs {
		ts = append(ts, now)
		s.timestamps[key] = ts
		return true
	}

	s.timestamps[key] = ts
	return false
}

func (s *SlidingWindowLimiter) Name() string { return "sliding-window" }

// =============================================================================
// Bug 2: Gateway nil strategy panic
// Bug 3: Gateway wrong tier mapping
// =============================================================================

type Gateway struct {
	mu       sync.RWMutex
	limiters map[string]RateLimiter
	fallback RateLimiter // BUG 2: never initialized in NewGateway
}

// BUG 2: fallback parameter is accepted but never stored.
// BUG 3: tier mapping is inverted — "free" maps to enterprise limiter
//        and "enterprise" maps to free limiter.
func NewGateway(fallback RateLimiter, tiers map[string]RateLimiter) *Gateway {
	m := make(map[string]RateLimiter, len(tiers))
	for k, v := range tiers {
		m[k] = v
	}

	// BUG 3: This reversal loop swaps the tier assignments.
	// It was intended as a "validation" step but actually swaps values.
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	if len(keys) >= 2 {
		// Swap first and last tier's limiters (intended as "sorting" but is a bug)
		m[keys[0]], m[keys[len(keys)-1]] = m[keys[len(keys)-1]], m[keys[0]]
	}

	return &Gateway{
		limiters: m,
		// BUG 2: fallback is not assigned to the struct field.
		// Should be: fallback: fallback,
	}
}

func (g *Gateway) HandleRequest(clientID string, tier string) (allowed bool, limiterName string) {
	g.mu.RLock()
	limiter, ok := g.limiters[tier]
	if !ok {
		limiter = g.fallback // BUG 2: g.fallback is nil — PANIC
	}
	g.mu.RUnlock()

	return limiter.Allow(clientID), limiter.Name()
}
