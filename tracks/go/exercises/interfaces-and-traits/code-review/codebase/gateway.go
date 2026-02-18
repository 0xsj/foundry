// Package gatewayctx provides context for the code review exercise.
// This represents the existing codebase that proposed.go builds on top of.
// Read this file to understand the types and patterns in use before reviewing proposed.go.
// This is NOT the file under review — do not apply review comments here.
package gatewayctx

import (
	"fmt"
	"time"
)

// Request represents an inbound API request.
type Request struct {
	ID       string
	Method   string
	Path     string
	Headers  map[string]string
	Body     []byte
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
// This is the core abstraction — all backend calls go through a Handler.
type Handler interface {
	Handle(req Request) (Response, error)
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

// Logger is an interface for structured logging.
// Used by middleware and handlers to emit log entries.
type Logger interface {
	Log(level, message string, fields map[string]any)
	Flush() error
	Close() error
}

// StdoutLogger is a Logger that writes to stdout.
type StdoutLogger struct{}

func (l *StdoutLogger) Log(level, message string, fields map[string]any) {
	fmt.Printf("[%s] %s %v\n", level, message, fields)
}

func (l *StdoutLogger) Flush() error { return nil }
func (l *StdoutLogger) Close() error { return nil }
