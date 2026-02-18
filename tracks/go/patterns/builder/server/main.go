// Builder Pattern: HTTP Server with Functional Options
//
// Demonstrates the canonical functional options pattern for configuring
// an HTTP server. Options like WithTimeout, WithTLS, WithMiddleware,
// and WithLogger are composable, self-documenting, and backward-compatible.
//
// Run: go run ./server/

package main

import (
	"crypto/tls"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"time"
)

// --- Config and Option Types ---

// config holds all server settings. Fields are unexported -- only
// With* functions can set them. This prevents callers from bypassing
// validation or creating inconsistent configurations.
type config struct {
	readTimeout  time.Duration
	writeTimeout time.Duration
	idleTimeout  time.Duration
	maxConns     int
	tlsCert      string
	tlsKey       string
	logger       *log.Logger
	middleware   []Middleware
	enablePprof  bool
}

// Middleware is a standard HTTP middleware function.
type Middleware func(http.Handler) http.Handler

// Option configures a Server. Each With* function returns an Option.
type Option func(*config) error

// --- Option Constructors ---

// WithReadTimeout sets the maximum duration for reading the request.
func WithReadTimeout(d time.Duration) Option {
	return func(c *config) error {
		if d <= 0 {
			return fmt.Errorf("read timeout must be positive, got %v", d)
		}
		c.readTimeout = d
		return nil
	}
}

// WithWriteTimeout sets the maximum duration for writing the response.
func WithWriteTimeout(d time.Duration) Option {
	return func(c *config) error {
		if d <= 0 {
			return fmt.Errorf("write timeout must be positive, got %v", d)
		}
		c.writeTimeout = d
		return nil
	}
}

// WithIdleTimeout sets the maximum duration for idle keep-alive connections.
func WithIdleTimeout(d time.Duration) Option {
	return func(c *config) error {
		if d <= 0 {
			return fmt.Errorf("idle timeout must be positive, got %v", d)
		}
		c.idleTimeout = d
		return nil
	}
}

// WithMaxConns sets the maximum number of concurrent connections.
func WithMaxConns(n int) Option {
	return func(c *config) error {
		if n < 1 {
			return fmt.Errorf("max connections must be >= 1, got %d", n)
		}
		c.maxConns = n
		return nil
	}
}

// WithTLS enables TLS with the given certificate and key files.
func WithTLS(certFile, keyFile string) Option {
	return func(c *config) error {
		if certFile == "" || keyFile == "" {
			return fmt.Errorf("TLS requires both cert and key files")
		}
		c.tlsCert = certFile
		c.tlsKey = keyFile
		return nil
	}
}

// WithLogger sets a custom logger for the server.
func WithLogger(l *log.Logger) Option {
	return func(c *config) error {
		if l == nil {
			return fmt.Errorf("logger cannot be nil")
		}
		c.logger = l
		return nil
	}
}

// WithMiddleware appends middleware to the server's middleware chain.
// Middleware is applied in the order it is added.
func WithMiddleware(mw ...Middleware) Option {
	return func(c *config) error {
		c.middleware = append(c.middleware, mw...)
		return nil
	}
}

// WithPprof enables the pprof debug endpoints.
func WithPprof() Option {
	return func(c *config) error {
		c.enablePprof = true
		return nil
	}
}

// --- Preset Bundles ---

// ProductionDefaults returns an Option that applies production-safe settings.
// Callers can override individual settings after applying the bundle.
func ProductionDefaults() Option {
	return func(c *config) error {
		c.readTimeout = 15 * time.Second
		c.writeTimeout = 15 * time.Second
		c.idleTimeout = 60 * time.Second
		c.maxConns = 10000
		c.enablePprof = false
		return nil
	}
}

// DevelopmentDefaults returns an Option with relaxed settings for local dev.
func DevelopmentDefaults() Option {
	return func(c *config) error {
		c.readTimeout = 60 * time.Second
		c.writeTimeout = 60 * time.Second
		c.idleTimeout = 120 * time.Second
		c.maxConns = 100
		c.enablePprof = true
		return nil
	}
}

// --- Server ---

// Server wraps http.Server with configuration applied via functional options.
type Server struct {
	addr   string
	config config
	http   *http.Server
}

// NewServer creates a Server with the given address and options.
// Required parameters (addr) come first; optional settings follow as Options.
func NewServer(addr string, handler http.Handler, opts ...Option) (*Server, error) {
	// Start with sensible defaults
	cfg := config{
		readTimeout:  30 * time.Second,
		writeTimeout: 30 * time.Second,
		idleTimeout:  120 * time.Second,
		maxConns:     1000,
		logger:       log.New(os.Stdout, "[server] ", log.LstdFlags|log.Lshortfile),
	}

	// Apply each option, failing on first error
	for _, opt := range opts {
		if err := opt(&cfg); err != nil {
			return nil, fmt.Errorf("server option error: %w", err)
		}
	}

	// Apply middleware chain (outermost first)
	wrapped := handler
	for i := len(cfg.middleware) - 1; i >= 0; i-- {
		wrapped = cfg.middleware[i](wrapped)
	}

	srv := &http.Server{
		Addr:         addr,
		Handler:      wrapped,
		ReadTimeout:  cfg.readTimeout,
		WriteTimeout: cfg.writeTimeout,
		IdleTimeout:  cfg.idleTimeout,
	}

	// Configure TLS if cert/key provided
	if cfg.tlsCert != "" && cfg.tlsKey != "" {
		srv.TLSConfig = &tls.Config{
			MinVersion: tls.VersionTLS12,
		}
	}

	return &Server{
		addr:   addr,
		config: cfg,
		http:   srv,
	}, nil
}

// String returns a human-readable description of the server configuration.
func (s *Server) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "Server[%s]", s.addr)
	fmt.Fprintf(&b, "\n  Read Timeout:  %v", s.config.readTimeout)
	fmt.Fprintf(&b, "\n  Write Timeout: %v", s.config.writeTimeout)
	fmt.Fprintf(&b, "\n  Idle Timeout:  %v", s.config.idleTimeout)
	fmt.Fprintf(&b, "\n  Max Conns:     %d", s.config.maxConns)
	if s.config.tlsCert != "" {
		fmt.Fprintf(&b, "\n  TLS:           %s", s.config.tlsCert)
	} else {
		fmt.Fprintf(&b, "\n  TLS:           disabled")
	}
	fmt.Fprintf(&b, "\n  Middleware:     %d registered", len(s.config.middleware))
	fmt.Fprintf(&b, "\n  Pprof:         %v", s.config.enablePprof)
	return b.String()
}

// --- Example Middleware ---

// loggingMiddleware logs each request.
func loggingMiddleware(logger *log.Logger) Middleware {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			start := time.Now()
			logger.Printf("-> %s %s", r.Method, r.URL.Path)
			next.ServeHTTP(w, r)
			logger.Printf("<- %s %s (%v)", r.Method, r.URL.Path, time.Since(start))
		})
	}
}

// recoveryMiddleware catches panics and returns 500.
func recoveryMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if err := recover(); err != nil {
				http.Error(w, "Internal Server Error", http.StatusInternalServerError)
			}
		}()
		next.ServeHTTP(w, r)
	})
}

// --- Main ---

func main() {
	fmt.Println("=== HTTP Server Builder with Functional Options ===")
	fmt.Println(strings.Repeat("-", 55))

	// Application handler
	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		fmt.Fprintln(w, `{"status":"ok"}`)
	})

	logger := log.New(os.Stdout, "[api] ", log.LstdFlags)

	// --- Example 1: Production server ---
	fmt.Println("\n1. Production Server (preset + overrides):")
	prodSrv, err := NewServer(":8080", mux,
		ProductionDefaults(),                    // Start with production preset
		WithTLS("cert.pem", "key.pem"),          // Add TLS
		WithLogger(logger),                      // Custom logger
		WithMiddleware(                           // Middleware chain
			recoveryMiddleware,
			loggingMiddleware(logger),
		),
		WithMaxConns(50000),                     // Override preset's maxConns
	)
	if err != nil {
		log.Fatalf("Failed to create production server: %v", err)
	}
	fmt.Println(prodSrv)

	// --- Example 2: Development server ---
	fmt.Println("\n2. Development Server (relaxed defaults + pprof):")
	devSrv, err := NewServer(":3000", mux,
		DevelopmentDefaults(),
		WithMiddleware(loggingMiddleware(logger)),
	)
	if err != nil {
		log.Fatalf("Failed to create dev server: %v", err)
	}
	fmt.Println(devSrv)

	// --- Example 3: Minimal server (just defaults) ---
	fmt.Println("\n3. Minimal Server (defaults only):")
	minSrv, err := NewServer(":9090", mux)
	if err != nil {
		log.Fatalf("Failed to create minimal server: %v", err)
	}
	fmt.Println(minSrv)

	// --- Example 4: Invalid option (demonstrates error handling) ---
	fmt.Println("\n4. Invalid Option (negative timeout):")
	_, err = NewServer(":8080", mux, WithReadTimeout(-5*time.Second))
	if err != nil {
		fmt.Printf("   Caught: %v\n", err)
	}

	fmt.Println(strings.Repeat("-", 55))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Required params (addr, handler) are positional")
	fmt.Println("- Optional config uses With* options")
	fmt.Println("- Presets bundle common configurations")
	fmt.Println("- Options can be combined and overridden")
	fmt.Println("- Invalid options fail at construction, not at runtime")
	fmt.Println("- Adding new options never breaks existing callers")
}
