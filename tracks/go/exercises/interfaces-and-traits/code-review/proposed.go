// Package gateway — proposed changes (PR under review)
//
// This file adds LoggingMiddleware: a Handler wrapper that logs every request
// and response. Three design issues are present. The code compiles and works
// correctly, but the interface design will cause friction as the codebase grows.
//
// Context: see codebase/gateway.go for the existing types (Request, Response,
// Handler, Logger, Router). This file builds on top of that foundation.
//
// For review purposes, the relevant existing types are reproduced below.
package gateway

import (
	"fmt"
	"time"
)

// ============================================================================
// EXISTING TYPES (from codebase/gateway.go — reproduced for context)
// ============================================================================

// Request represents an inbound API request.
type Request struct {
	ID         string
	Method     string
	Path       string
	Headers    map[string]string
	Body       []byte
	ReceivedAt time.Time
}

// Response represents an outbound API response.
type Response struct {
	StatusCode int
	Headers    map[string]string
	Body       []byte
	Latency    time.Duration
}

// Handler processes a request and returns a response.
type Handler interface {
	Handle(req Request) (Response, error)
}

// Logger is an interface for structured logging.
type Logger interface {
	Log(level, message string, fields map[string]any)
	Flush() error
	Close() error
}

// Router dispatches requests to registered handlers by path prefix.
type Router struct {
	routes map[string]Handler
}

func NewRouter() *Router {
	return &Router{routes: make(map[string]Handler)}
}

func (r *Router) Register(path string, h Handler) {
	r.routes[path] = h
}

func (r *Router) Handle(req Request) (Response, error) {
	h, ok := r.routes[req.Path]
	if !ok {
		return Response{StatusCode: 404}, fmt.Errorf("no route for %s", req.Path)
	}
	return h.Handle(req)
}

// ============================================================================
// PROPOSED CHANGES — code under review begins here
// ============================================================================

// ============================================================================
// ISSUE 1: Logger interface is too large.
//
// LoggingMiddleware only calls Log() — it never calls Flush() or Close().
// But it accepts the full Logger interface (3 methods), so any type passed
// to NewLoggingMiddleware must implement all three methods.
//
// A test double for LoggingMiddleware needs to implement Flush() and Close()
// even though the middleware never calls them. A simple function-based
// logger (func(level, msg string, fields map[string]any)) can't be used.
//
// Fix: define a minimal interface with only what's needed.
// ============================================================================

// LoggingMiddleware wraps a Handler and logs every request.
type LoggingMiddleware struct {
	next   Handler // the handler being wrapped
	logger Logger  // ISSUE 1: accepts the full Logger (3 methods); only calls Log()
}

// NewLoggingMiddleware creates a middleware that logs requests using logger.
// ISSUE 1: parameter type should be a smaller interface (just the Log method),
// or a function type. Accepting the full Logger forces callers to provide
// Flush() and Close() implementations that are never used.
func NewLoggingMiddleware(next Handler, logger Logger) *LoggingMiddleware {
	return &LoggingMiddleware{next: next, logger: logger}
}

// Handle logs the request, calls the next handler, logs the response.
func (lm *LoggingMiddleware) Handle(req Request) (Response, error) {
	start := time.Now()

	lm.logger.Log("info", "request received", map[string]any{
		"id":     req.ID,
		"method": req.Method,
		"path":   req.Path,
	})

	resp, err := lm.next.Handle(req)
	latency := time.Since(start)

	if err != nil {
		lm.logger.Log("error", "request failed", map[string]any{
			"id":      req.ID,
			"latency": latency.String(),
			"error":   err.Error(),
		})
		return resp, err
	}

	lm.logger.Log("info", "request completed", map[string]any{
		"id":      req.ID,
		"status":  resp.StatusCode,
		"latency": latency.String(),
	})

	return resp, nil
}

// ============================================================================
// ISSUE 2: RegisterHandler accepts a concrete type instead of an interface.
//
// RegisterHandler takes a *Router as its first parameter.
// This means you can only call RegisterHandler with a *Router — nothing else.
// It's not testable with a mock router, and it can't be used with a future
// Router2 or TestRouter that satisfies the same behavior.
//
// The existing codebase already has a Handler interface. The function should
// accept something like a "Registrar" interface with just the Register method.
//
// Fix: define a minimal Registrar interface and accept that instead.
// ============================================================================

// RegisterHandler adds a handler to the router with logging middleware applied.
// ISSUE 2: accepts *Router (concrete type) instead of a Registrar interface.
// Cannot be called with anything other than a *Router.
func RegisterHandler(r *Router, path string, h Handler, logger Logger) {
	wrapped := NewLoggingMiddleware(h, logger)
	r.Register(path, wrapped)
}

// ============================================================================
// ISSUE 3: MetricsCollector is a single-method interface where a function
// type would be cleaner and more composable.
//
// MetricsCollector has exactly one method: RecordLatency.
// There's no state to bundle, no reason to make it an interface.
// Using a function type is simpler: callers can pass any func(string, time.Duration).
//
// Fix: type RecordLatencyFunc func(route string, d time.Duration)
// ============================================================================

// MetricsCollector records latency metrics.
// ISSUE 3: single-method interface — a function type would be simpler.
type MetricsCollector interface {
	RecordLatency(route string, d time.Duration)
}

// MetricsMiddleware wraps a Handler and records latency.
type MetricsMiddleware struct {
	next    Handler
	metrics MetricsCollector // ISSUE 3: could be a function type instead
}

func NewMetricsMiddleware(next Handler, m MetricsCollector) *MetricsMiddleware {
	return &MetricsMiddleware{next: next, metrics: m}
}

func (mm *MetricsMiddleware) Handle(req Request) (Response, error) {
	start := time.Now()
	resp, err := mm.next.Handle(req)
	mm.metrics.RecordLatency(req.Path, time.Since(start))
	return resp, err
}

// String implements fmt.Stringer — not an issue, just a utility.
func (lm *LoggingMiddleware) String() string {
	return fmt.Sprintf("LoggingMiddleware{next: %T}", lm.next)
}
