package main

import (
	"context"
	"io"
	"net/http"
	"time"
)

// --- Types ---

// Middleware is a function that wraps an http.Handler with additional behavior.
type Middleware func(http.Handler) http.Handler

// --- Context Keys ---

// Use unexported types for context keys to prevent collisions across packages.
type contextKey struct{ name string }

var (
	requestIDKey = &contextKey{"request-id"}
	clientIDKey  = &contextKey{"client-id"}
)

// RequestIDFromCtx extracts the request ID from the context.
func RequestIDFromCtx(ctx context.Context) string {
	if id, ok := ctx.Value(requestIDKey).(string); ok {
		return id
	}
	return ""
}

// ClientIDFromCtx extracts the authenticated client ID from the context.
func ClientIDFromCtx(ctx context.Context) string {
	if id, ok := ctx.Value(clientIDKey).(string); ok {
		return id
	}
	return ""
}

// --- Chain ---

// Chain composes middlewares so the first middleware in the list is the outermost
// (executes first on the request path, last on the response path).
//
// Usage:
//
//	handler := Chain(finalHandler,
//	    WithRequestID,    // executes 1st
//	    WithLogging(os.Stdout), // executes 2nd
//	    WithAuth(validate),     // executes 3rd
//	)
func Chain(handler http.Handler, middlewares ...Middleware) http.Handler {
	// TODO: Apply middlewares in the correct order.
	// The first middleware in the list should be the outermost wrapper.
	return handler
}

// --- Response Recorder ---

// responseRecorder wraps http.ResponseWriter to capture the status code.
// This is needed by logging middleware to record what status was sent.
type responseRecorder struct {
	http.ResponseWriter
	statusCode int
	written    bool
}

// TODO: Implement WriteHeader on responseRecorder.
// Remember: Write() implicitly calls WriteHeader(200) if not yet called.

// TODO: Implement Write on responseRecorder.

// --- Middleware: Request ID ---

// WithRequestID assigns a unique request ID to each request.
// If the request already has an X-Request-ID header, reuse it.
// Otherwise generate a unique ID.
// Store the ID in the request context and set it on the response header.
func WithRequestID(next http.Handler) http.Handler {
	// TODO: Implement request ID middleware.
	// - Check for existing X-Request-ID header
	// - Generate unique ID if missing
	// - Store in context via requestIDKey
	// - Set X-Request-ID response header
	return next
}

// --- Middleware: Logging ---

// WithLogging logs each request after completion.
// Format: "<method> <path> <status> <duration> [<request-id>]"
//
// The io.Writer parameter makes this testable -- pass os.Stdout for production,
// bytes.Buffer for tests.
func WithLogging(out io.Writer) Middleware {
	return func(next http.Handler) http.Handler {
		// TODO: Implement logging middleware.
		// - Create a responseRecorder to capture status code
		// - Record start time
		// - Call next handler
		// - Log method, path, status, duration, request ID
		return next
	}
}

// --- Middleware: Authentication ---

// AuthValidator validates a bearer token and returns the client ID.
// Returns an error if the token is invalid.
type AuthValidator func(token string) (clientID string, err error)

// WithAuth validates the Authorization header using the provided validator.
// Returns 401 for missing header, 403 for invalid token.
// On success, stores the client ID in the request context.
func WithAuth(validate AuthValidator) Middleware {
	return func(next http.Handler) http.Handler {
		// TODO: Implement auth middleware.
		// - Read Authorization header
		// - Strip "Bearer " prefix
		// - Validate token using the validator function
		// - Store client ID in context via clientIDKey
		// - Return appropriate HTTP errors for missing/invalid tokens
		return next
	}
}

// --- Middleware: Rate Limiting ---

// WithRateLimit enforces a per-client request limit within a sliding window.
// Clients are identified by their context client ID (from auth middleware),
// falling back to RemoteAddr.
func WithRateLimit(requestsPerWindow int, window time.Duration) Middleware {
	// TODO: Implement rate limiting middleware.
	// - Track request counts per client ID
	// - Reset counts when window expires
	// - Return 429 with Retry-After header when limit exceeded
	// - Must be safe for concurrent access (use sync.Mutex)
	return func(next http.Handler) http.Handler {
		return next
	}
}

// --- Middleware: Panic Recovery ---

// WithRecovery catches panics from downstream handlers and returns 500.
// Logs the panic value to the provided writer.
func WithRecovery(out io.Writer) Middleware {
	return func(next http.Handler) http.Handler {
		// TODO: Implement panic recovery middleware.
		// - Use defer/recover to catch panics
		// - Write 500 response with JSON error body
		// - Log panic value and stack trace
		// - Do NOT re-panic
		return next
	}
}

// --- Middleware: Response Compression ---

// WithCompression applies gzip compression to responses when the client
// supports it (Accept-Encoding: gzip).
func WithCompression(next http.Handler) http.Handler {
	// TODO: Implement compression middleware.
	// - Check Accept-Encoding for "gzip"
	// - If supported, wrap ResponseWriter with gzip writer
	// - Set Content-Encoding: gzip, remove Content-Length
	// - Close gzip writer after handler completes
	// - If not supported, pass through unchanged
	return next
}

// --- Main (for manual testing) ---

func main() {
	// This main is for manual testing with curl.
	// Run the test suite for automated verification.
	//
	// Example:
	//   go run .
	//   curl -H "Authorization: Bearer valid-token" http://localhost:8080/api/data
}
