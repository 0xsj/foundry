// Package middleware implements a composable HTTP-like request pipeline.
// Run tests: go test -v ./...
package middleware

import (
	"fmt"
	"log"
	"time"
)

// ============================================================================
// TYPES — provided, do not change
// ============================================================================

// Request represents an incoming HTTP-like request.
type Request struct {
	Method   string
	Path     string
	Headers  map[string]string
	Body     string
	ClientIP string // used by RateLimit to track per-client counts
}

// Response represents an outgoing response.
type Response struct {
	Status  int
	Body    string
	Headers map[string]string
}

// Handler is a function that processes a Request and returns a Response.
// This is the core type of the pipeline.
type Handler func(Request) Response

// Middleware is a function that wraps a Handler, adding behavior before/after.
// It receives the next Handler in the chain and returns a new Handler.
type Middleware func(Handler) Handler

// ============================================================================
// CHAIN
// Compose multiple middlewares into one. The first middleware in the list
// runs outermost (first on request, last on response).
//
// Example: Chain(Logger, Auth("token"), RateLimit(60)) wraps the final handler
// so Logger runs first, then Auth, then RateLimit, then the handler.
// ============================================================================

// Chain composes middlewares into a single Middleware.
// Chain(m1, m2, m3)(handler) is equivalent to m1(m2(m3(handler))).
func Chain(middlewares ...Middleware) Middleware {
	// TODO: Implement
	// Tip: iterate middlewares in reverse order, wrapping the handler each time.
	_ = middlewares
	return func(next Handler) Handler {
		return next
	}
}

// ============================================================================
// LOGGER MIDDLEWARE
// Logs the request method, path, response status, and elapsed time.
// Format: "METHOD /path → STATUS (elapsed)"
// ============================================================================

// Logger wraps a handler and logs each request.
func Logger(next Handler) Handler {
	// TODO: Implement
	// Use time.Now() before calling next, time.Since() after.
	// Use log.Printf for output.
	_ = log.Printf
	_ = time.Now
	return next
}

// ============================================================================
// AUTH MIDDLEWARE
// Auth returns a Middleware that checks the Authorization header.
// If the header value doesn't match token, return a 401 response immediately.
// The token is captured by the closure — not stored as a global.
// ============================================================================

// Auth returns a middleware that enforces a static bearer token.
func Auth(token string) Middleware {
	// TODO: Implement
	// The returned Middleware captures `token` via closure.
	// Check r.Headers["Authorization"] against token.
	_ = token
	return func(next Handler) Handler {
		return next
	}
}

// ============================================================================
// RATELIMIT MIDDLEWARE
// RateLimit returns a Middleware that limits requests per client IP per minute.
// After maxPerMin calls from the same IP in the current minute window, return 429.
// State (counts, window) lives in a closure — not global variables.
// ============================================================================

// RateLimit returns a middleware that enforces per-IP rate limits.
func RateLimit(maxPerMin int) Middleware {
	// TODO: Implement
	// State hint: you need to track:
	//   - counts per IP for the current minute
	//   - the current minute window (so you can reset counts each minute)
	//
	// time.Now().Truncate(time.Minute) gives you the current minute as a Time value.
	_ = maxPerMin
	_ = fmt.Sprintf
	_ = time.Minute
	return func(next Handler) Handler {
		return next
	}
}

// ============================================================================
// RECOVER MIDDLEWARE
// Recover catches any panic from the inner handler and returns a 500 response
// instead of crashing the process. Log the panic value.
// ============================================================================

// Recover wraps a handler and catches panics, returning a 500 response.
func Recover(next Handler) Handler {
	// TODO: Implement
	// Reminder: recover() only works directly inside a deferred function.
	// You may need to introduce an inner function to make defer + recover work.
	return next
}
