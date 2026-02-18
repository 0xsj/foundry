// Package middleware implements a composable HTTP-like request pipeline.
// Run tests: go test -v ./...
package middleware

import (
	"fmt"
	"log"
	"sync"
	"time"
)

// ============================================================================
// TYPES
// ============================================================================

// Request represents an incoming HTTP-like request.
type Request struct {
	Method   string
	Path     string
	Headers  map[string]string
	Body     string
	ClientIP string
}

// Response represents an outgoing response.
type Response struct {
	Status  int
	Body    string
	Headers map[string]string
}

// Handler processes a Request and returns a Response.
// This is a function type — functions are first-class values in Go.
type Handler func(Request) Response

// Middleware wraps a Handler, adding behavior before/after.
// A Middleware is itself a function that takes a Handler and returns a Handler.
// This is the core of the composition pattern.
type Middleware func(Handler) Handler

// ============================================================================
// CHAIN
// ============================================================================

// Chain composes multiple middlewares into a single Middleware.
// The first argument wraps outermost — it runs first on request, last on response.
//
// Chain(m1, m2, m3)(handler) produces: m1(m2(m3(handler)))
//
// How it works:
//   - We start with the final handler
//   - We iterate middlewares in reverse order (innermost first)
//   - Each middleware wraps the previous result
//   - Result: m1 wraps m2 wraps m3 wraps handler
func Chain(middlewares ...Middleware) Middleware {
	return func(final Handler) Handler {
		// Iterate in reverse: innermost middleware applied first
		for i := len(middlewares) - 1; i >= 0; i-- {
			final = middlewares[i](final)
		}
		return final
	}
}

// ============================================================================
// LOGGER MIDDLEWARE
// ============================================================================

// Logger wraps a handler and logs each request with method, path, status, and duration.
//
// This demonstrates the simplest middleware pattern: run code before and after next().
// The elapsed time calculation requires capturing start time before calling next.
func Logger(next Handler) Handler {
	return func(r Request) Response {
		start := time.Now()
		resp := next(r)
		log.Printf("%s %s → %d (%s)", r.Method, r.Path, resp.Status, time.Since(start))
		return resp
	}
}

// ============================================================================
// AUTH MIDDLEWARE
// ============================================================================

// Auth returns a Middleware that enforces a static bearer token.
//
// Key design point: Auth is a function that RETURNS a Middleware.
// The token is captured in the closure — each call to Auth creates an
// independent middleware with its own captured token. This is the
// "factory function" pattern using closures.
//
// Contrast: Logger is already a Middleware (func(Handler) Handler).
// Auth is a function that produces a Middleware — one extra layer.
func Auth(token string) Middleware {
	// token is captured here — in the enclosing scope of the returned Middleware
	return func(next Handler) Handler {
		return func(r Request) Response {
			if r.Headers["Authorization"] != token {
				// Short-circuit: don't call next at all
				return Response{
					Status: 401,
					Body:   "unauthorized",
				}
			}
			// Token matches — proceed to next handler
			return next(r)
		}
	}
}

// ============================================================================
// RATELIMIT MIDDLEWARE
// ============================================================================

// rateLimitState tracks request counts per IP within a minute window.
// It's a separate struct to keep the closure variables organized.
type rateLimitState struct {
	mu      sync.Mutex
	counts  map[string]int // IP → count in current window
	window  time.Time      // the current minute window
}

// RateLimit returns a Middleware that limits requests per client IP per minute.
//
// The state (counts map and window) lives entirely inside the closure.
// Each call to RateLimit creates an independent rate limiter — there's no
// global state. This is a key benefit of the closure-as-state pattern.
//
// Design tradeoff: using sync.Mutex makes this goroutine-safe, which matters
// in any real HTTP server where requests are concurrent. Without the mutex,
// concurrent requests could race on the counts map (map writes are not safe
// for concurrent use without synchronization).
func RateLimit(maxPerMin int) Middleware {
	state := &rateLimitState{
		counts: make(map[string]int),
		window: time.Now().Truncate(time.Minute),
	}

	return func(next Handler) Handler {
		return func(r Request) Response {
			state.mu.Lock()
			defer state.mu.Unlock()

			// Check if we've moved into a new minute window
			now := time.Now().Truncate(time.Minute)
			if now.After(state.window) {
				// New minute: reset all counts
				state.counts = make(map[string]int)
				state.window = now
			}

			// Check this IP's count
			if state.counts[r.ClientIP] >= maxPerMin {
				return Response{
					Status: 429,
					Body:   fmt.Sprintf("rate limit exceeded: max %d requests per minute", maxPerMin),
				}
			}

			// Increment count and proceed
			state.counts[r.ClientIP]++
			return next(r)
		}
	}
}

// ============================================================================
// RECOVER MIDDLEWARE
// ============================================================================

// Recover wraps a handler and catches any panic from the inner handler,
// returning a 500 response instead of crashing the process.
//
// The tricky part: recover() only works when called DIRECTLY from a deferred
// function. It does NOT work if called from a function called by a defer.
// We use an immediately-invoked inner function to create the right scope.
//
// Why Recover is outermost in a chain:
//   Chain(Recover, Logger, Auth(...))(handler)
//
// If Logger panics, Recover catches it. If we put Logger outside Recover,
// a panic in Logger would escape. Put Recover first (outermost) to catch
// panics from every layer inside it.
func Recover(next Handler) Handler {
	return func(r Request) Response {
		var resp Response

		// Immediately-invoked function creates the scope for defer+recover
		func() {
			defer func() {
				if rec := recover(); rec != nil {
					log.Printf("PANIC recovered: %v", rec)
					resp = Response{Status: 500, Body: "internal server error"}
				}
			}()
			resp = next(r) // if this panics, defer+recover catches it
		}()

		return resp
	}
}
