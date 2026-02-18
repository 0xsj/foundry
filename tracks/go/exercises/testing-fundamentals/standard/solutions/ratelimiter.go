// Package ratelimiter implements a token-bucket rate limiter.
package ratelimiter

import (
	"sync"
	"time"
)

// Clock is the time abstraction.
type Clock interface {
	Now() time.Time
}

// RealClock delegates to the standard library.
type RealClock struct{}

func (RealClock) Now() time.Time { return time.Now() }

// RateLimiter is a token-bucket rate limiter. Safe for concurrent use.
type RateLimiter struct {
	mu          sync.Mutex
	clock       Clock
	rate        int
	window      time.Duration
	tokens      int
	windowStart time.Time
}

// New creates a RateLimiter that allows `rate` requests per `window`.
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
func (rl *RateLimiter) Allow() bool {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	now := rl.clock.Now()
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
func (rl *RateLimiter) Tokens() int {
	rl.mu.Lock()
	defer rl.mu.Unlock()
	return rl.tokens
}
