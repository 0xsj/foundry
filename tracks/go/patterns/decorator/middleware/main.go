// Decorator Pattern: HTTP Middleware Chain
//
// Demonstrates the most common form of decorator in Go: HTTP middleware.
// Each middleware wraps http.Handler and returns http.Handler, adding
// cross-cutting behavior (logging, auth, CORS, rate limiting, panic recovery).
// A Chain() helper composes them in readable order.
//
// Run: go run ./middleware/

package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"
)

// --- Middleware Type ---

// Middleware is a function that wraps an http.Handler with additional behavior.
// This is the canonical Go middleware signature.
type Middleware func(http.Handler) http.Handler

// Chain applies middlewares to a handler in the order listed.
// The first middleware in the list is the outermost (runs first on request, last on response).
func Chain(handler http.Handler, middlewares ...Middleware) http.Handler {
	for i := len(middlewares) - 1; i >= 0; i-- {
		handler = middlewares[i](handler)
	}
	return handler
}

// --- Context Keys ---

// unexported type for context keys prevents collisions between packages
type contextKey struct{ name string }

var (
	requestIDKey = &contextKey{"request-id"}
	clientIPKey  = &contextKey{"client-ip"}
)

// RequestID extracts the request ID from context.
func RequestID(ctx context.Context) string {
	if id, ok := ctx.Value(requestIDKey).(string); ok {
		return id
	}
	return ""
}

// --- Status Recorder ---

// statusRecorder wraps http.ResponseWriter to capture the status code.
// This is a decorator on ResponseWriter itself -- decorators all the way down.
type statusRecorder struct {
	http.ResponseWriter
	statusCode int
	written    bool
}

func (sr *statusRecorder) WriteHeader(code int) {
	if !sr.written {
		sr.statusCode = code
		sr.written = true
	}
	sr.ResponseWriter.WriteHeader(code)
}

func (sr *statusRecorder) Write(b []byte) (int, error) {
	if !sr.written {
		sr.statusCode = http.StatusOK
		sr.written = true
	}
	return sr.ResponseWriter.Write(b)
}

// --- Middleware: Request ID ---

// WithRequestID injects a unique request ID into the context and response headers.
// If the incoming request already has an X-Request-ID header, it's reused (for tracing
// across services). Otherwise a new one is generated.
func WithRequestID(next http.Handler) http.Handler {
	var counter uint64
	var mu sync.Mutex

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id := r.Header.Get("X-Request-ID")
		if id == "" {
			mu.Lock()
			counter++
			id = fmt.Sprintf("req-%d-%d", time.Now().UnixMilli(), counter)
			mu.Unlock()
		}

		ctx := context.WithValue(r.Context(), requestIDKey, id)
		w.Header().Set("X-Request-ID", id)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// --- Middleware: Logging ---

// WithLogging logs each request with method, path, status code, and duration.
// It uses the statusRecorder decorator to capture the response status.
func WithLogging(logger *log.Logger) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			recorder := &statusRecorder{ResponseWriter: w, statusCode: http.StatusOK}

			// Log before (request received)
			reqID := RequestID(r.Context())
			logger.Printf("[%s] --> %s %s", reqID, r.Method, r.URL.Path)

			next.ServeHTTP(recorder, r)

			// Log after (response sent)
			logger.Printf("[%s] <-- %s %s %d (%v)",
				reqID, r.Method, r.URL.Path, recorder.statusCode, time.Since(start).Round(time.Microsecond))
		})
	}
}

// --- Middleware: Authentication ---

// WithAuth validates API keys from the Authorization header.
// In production, this would check a database, cache, or JWT validator.
// This demonstrates a configurable middleware (the validKeys map).
func WithAuth(validKeys map[string]string) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authHeader := r.Header.Get("Authorization")
			if authHeader == "" {
				http.Error(w, `{"error": "missing Authorization header"}`, http.StatusUnauthorized)
				return
			}

			key := strings.TrimPrefix(authHeader, "Bearer ")
			clientName, valid := validKeys[key]
			if !valid {
				http.Error(w, `{"error": "invalid API key"}`, http.StatusForbidden)
				return
			}

			// Inject client identity into context for downstream handlers
			ctx := context.WithValue(r.Context(), clientIPKey, clientName)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// --- Middleware: CORS ---

// WithCORS adds CORS headers and handles preflight OPTIONS requests.
// The allowedOrigin parameter configures which origin is allowed.
func WithCORS(allowedOrigin string) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Access-Control-Allow-Origin", allowedOrigin)
			w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Request-ID")
			w.Header().Set("Access-Control-Max-Age", "86400")

			// Handle preflight: respond immediately without forwarding to handler
			if r.Method == http.MethodOptions {
				w.WriteHeader(http.StatusNoContent)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// --- Middleware: Rate Limiting ---

// WithRateLimit enforces a simple per-client rate limit using a token bucket.
// In production, you'd use a distributed rate limiter (Redis, etc.).
func WithRateLimit(requestsPerSecond int) Middleware {
	type client struct {
		tokens    int
		lastReset time.Time
	}

	var (
		mu      sync.Mutex
		clients = make(map[string]*client)
	)

	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Use client identity from auth middleware, fall back to remote addr
			clientID := r.RemoteAddr
			if name, ok := r.Context().Value(clientIPKey).(string); ok {
				clientID = name
			}

			mu.Lock()
			c, exists := clients[clientID]
			if !exists {
				c = &client{tokens: requestsPerSecond, lastReset: time.Now()}
				clients[clientID] = c
			}

			// Reset tokens if a second has passed
			if time.Since(c.lastReset) > time.Second {
				c.tokens = requestsPerSecond
				c.lastReset = time.Now()
			}

			if c.tokens <= 0 {
				mu.Unlock()
				w.Header().Set("Retry-After", "1")
				http.Error(w, `{"error": "rate limit exceeded"}`, http.StatusTooManyRequests)
				return
			}

			c.tokens--
			mu.Unlock()

			next.ServeHTTP(w, r)
		})
	}
}

// --- Middleware: Panic Recovery ---

// WithRecovery catches panics from downstream handlers and converts them to
// 500 Internal Server Error responses. Without this, a panic would crash the
// goroutine and the client would get a connection reset.
func WithRecovery(logger *log.Logger) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			defer func() {
				if rec := recover(); rec != nil {
					reqID := RequestID(r.Context())
					logger.Printf("[%s] PANIC: %v", reqID, rec)
					http.Error(w, `{"error": "internal server error"}`, http.StatusInternalServerError)
				}
			}()

			next.ServeHTTP(w, r)
		})
	}
}

// --- Application Handlers ---

func handleGetUsers(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	fmt.Fprintf(w, `{"users": ["alice", "bob", "charlie"]}`)
}

func handlePanic(w http.ResponseWriter, r *http.Request) {
	// This handler deliberately panics to demonstrate recovery middleware
	panic("something went terribly wrong in the handler")
}

// --- Main ---

func main() {
	logger := log.New(os.Stdout, "", log.LstdFlags|log.Lmicroseconds)

	// Valid API keys (in production: from a database or vault)
	validKeys := map[string]string{
		"key-alice-123": "alice",
		"key-bob-456":   "bob",
	}

	// Build the middleware chain.
	// Read top-to-bottom: this is the order they execute on each request.
	mux := http.NewServeMux()
	mux.HandleFunc("/api/users", handleGetUsers)
	mux.HandleFunc("/api/panic", handlePanic)

	server := Chain(mux,
		WithRecovery(logger),       // 1st: catch panics (outermost, so it catches everything)
		WithRequestID,              // 2nd: assign request ID
		WithLogging(logger),        // 3rd: log requests with request ID
		WithCORS("*"),              // 4th: handle CORS
		WithAuth(validKeys),        // 5th: authenticate
		WithRateLimit(10),          // 6th: rate limit (innermost)
	)

	// Print the middleware stack for clarity
	fmt.Println("=== HTTP Middleware Chain Demo ===")
	fmt.Println()
	fmt.Println("Middleware stack (execution order):")
	fmt.Println("  1. Recovery    -- catches panics, returns 500")
	fmt.Println("  2. Request ID  -- injects X-Request-ID")
	fmt.Println("  3. Logging     -- logs method, path, status, duration")
	fmt.Println("  4. CORS        -- adds CORS headers, handles preflight")
	fmt.Println("  5. Auth        -- validates Bearer token")
	fmt.Println("  6. Rate Limit  -- token bucket per client")
	fmt.Println("  7. Handler     -- actual business logic")
	fmt.Println()
	fmt.Println("Starting server on :8080")
	fmt.Println()
	fmt.Println("Test with:")
	fmt.Println("  curl -H 'Authorization: Bearer key-alice-123' http://localhost:8080/api/users")
	fmt.Println("  curl http://localhost:8080/api/users  # → 401 Unauthorized")
	fmt.Println("  curl -H 'Authorization: Bearer key-alice-123' http://localhost:8080/api/panic  # → 500")
	fmt.Println()

	if err := http.ListenAndServe(":8080", server); err != nil {
		logger.Fatalf("server error: %v", err)
	}
}
