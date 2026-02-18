// Package ratelimiter implements a token-bucket rate limiter.
//
// The token-bucket algorithm works like this:
//   - The bucket holds up to `rate` tokens
//   - Each request consumes one token
//   - Tokens refill to full capacity when the `window` duration elapses
//   - If the bucket is empty, requests are denied
//
// This is a simplified (non-sliding-window) version:
// the bucket refills all at once when the window expires.
package ratelimiter

import (
	"sync"
	"time"
)

// Clock is the time abstraction.
// Production code uses RealClock; tests inject a StubClock.
type Clock interface {
	Now() time.Time
}

// RealClock delegates to the standard library. Use in production.
type RealClock struct{}

func (RealClock) Now() time.Time { return time.Now() }

// RateLimiter is a token-bucket rate limiter.
// Safe for concurrent use.
type RateLimiter struct {
	mu          sync.Mutex
	clock       Clock
	rate        int           // max tokens (capacity)
	window      time.Duration // refill period
	tokens      int           // current token count
	windowStart time.Time     // when the current window started
}

// New creates a RateLimiter that allows `rate` requests per `window`.
// The clock parameter controls time — inject RealClock{} for production,
// a StubClock for tests.
func New(rate int, window time.Duration, clock Clock) *RateLimiter {
	return &RateLimiter{
		clock:       clock,
		rate:        rate,
		window:      window,
		tokens:      rate,
		windowStart: clock.Now(),
	}
}

// Allow reports whether the incoming request is within the rate limit.
// Returns true and consumes one token if a token is available.
// Returns false without consuming a token if the bucket is empty.
func (rl *RateLimiter) Allow() bool {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	now := rl.clock.Now()

	// Refill: if the window has elapsed, reset tokens to full capacity
	if now.Sub(rl.windowStart) >= rl.window {
		rl.tokens = rl.rate
		rl.windowStart = now
	}

	if rl.tokens <= 0 {
		return false
	}

	rl.tokens--
	return true
}

// Tokens returns the current number of available tokens.
// Primarily useful for testing and observability.
func (rl *RateLimiter) Tokens() int {
	rl.mu.Lock()
	defer rl.mu.Unlock()
	return rl.tokens
}
