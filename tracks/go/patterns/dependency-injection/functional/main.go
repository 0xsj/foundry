// Dependency Injection: Functional DI with Injected Functions
//
// Instead of interfaces, dependencies are injected as functions. This is
// useful when a dependency has a single behavior. Closures capture external
// state, providing lightweight DI without interface boilerplate.
//
// Run: go run ./functional/

package main

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"strings"
	"time"
)

// =============================================================================
// Function types as dependencies
// =============================================================================

// Fetcher is a function that retrieves data from a URL. In production,
// this wraps http.Get. In tests, it returns canned responses.
type Fetcher func(ctx context.Context, url string) ([]byte, error)

// IDGenerator is a function that produces unique identifiers. In production,
// this generates UUIDs. In tests, it returns predictable values.
type IDGenerator func() string

// Clock is a function that returns the current time. In production,
// this is time.Now. In tests, it returns a fixed time.
type Clock func() time.Time

// =============================================================================
// HealthChecker -- uses functional DI for all external dependencies
// =============================================================================

// HealthChecker monitors the health of upstream services. Every external
// behavior is an injected function -- no interfaces needed.
type HealthChecker struct {
	fetch   Fetcher
	now     Clock
	genID   IDGenerator
	timeout time.Duration
}

// HealthResult captures the outcome of a health check.
type HealthResult struct {
	ID          string
	ServiceName string
	URL         string
	Healthy     bool
	Latency     time.Duration
	CheckedAt   time.Time
	Error       string
}

// NewHealthChecker constructs a HealthChecker with injected functions.
// Nil functions get sensible defaults.
func NewHealthChecker(fetch Fetcher, opts ...HealthCheckerOption) *HealthChecker {
	hc := &HealthChecker{
		fetch:   fetch,
		now:     time.Now,
		genID:   defaultIDGenerator,
		timeout: 5 * time.Second,
	}
	for _, opt := range opts {
		opt(hc)
	}
	return hc
}

// HealthCheckerOption configures optional behaviors.
type HealthCheckerOption func(*HealthChecker)

// WithClock injects a custom time source.
func WithClock(c Clock) HealthCheckerOption {
	return func(hc *HealthChecker) { hc.now = c }
}

// WithIDGenerator injects a custom ID generator.
func WithIDGenerator(g IDGenerator) HealthCheckerOption {
	return func(hc *HealthChecker) { hc.genID = g }
}

// WithTimeout sets the health check timeout.
func WithTimeout(d time.Duration) HealthCheckerOption {
	return func(hc *HealthChecker) { hc.timeout = d }
}

// Check performs a health check against a service endpoint.
func (hc *HealthChecker) Check(ctx context.Context, serviceName, url string) HealthResult {
	start := hc.now()
	result := HealthResult{
		ID:          hc.genID(),
		ServiceName: serviceName,
		URL:         url,
		CheckedAt:   start,
	}

	ctx, cancel := context.WithTimeout(ctx, hc.timeout)
	defer cancel()

	_, err := hc.fetch(ctx, url)
	result.Latency = hc.now().Sub(start)

	if err != nil {
		result.Healthy = false
		result.Error = err.Error()
	} else {
		result.Healthy = true
	}

	return result
}

// CheckAll checks multiple services and returns all results.
func (hc *HealthChecker) CheckAll(ctx context.Context, services map[string]string) []HealthResult {
	results := make([]HealthResult, 0, len(services))
	for name, url := range services {
		results = append(results, hc.Check(ctx, name, url))
	}
	return results
}

// =============================================================================
// Default implementations
// =============================================================================

func defaultIDGenerator() string {
	b := make([]byte, 8)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}

// =============================================================================
// Closure-based handler factory -- another functional DI pattern
// =============================================================================

// RateLimiter checks whether a request from the given key should be allowed.
type RateLimiter func(key string) (allowed bool)

// RequestProcessor processes incoming requests. Dependencies are injected
// as function fields.
type RequestProcessor struct {
	Validate    func(payload string) error
	Transform   func(payload string) string
	RateLimit   RateLimiter
	OnProcessed func(key, result string) // Callback for side effects
}

func (rp *RequestProcessor) Process(key, payload string) (string, error) {
	// Check rate limit
	if rp.RateLimit != nil && !rp.RateLimit(key) {
		return "", fmt.Errorf("rate limit exceeded for key %s", key)
	}

	// Validate
	if rp.Validate != nil {
		if err := rp.Validate(payload); err != nil {
			return "", fmt.Errorf("validation failed: %w", err)
		}
	}

	// Transform
	result := payload
	if rp.Transform != nil {
		result = rp.Transform(payload)
	}

	// Notify
	if rp.OnProcessed != nil {
		rp.OnProcessed(key, result)
	}

	return result, nil
}

// =============================================================================
// Main -- demonstrate functional DI patterns
// =============================================================================

func main() {
	fmt.Println("=== Dependency Injection: Functional DI ===")
	fmt.Println(strings.Repeat("=", 50))

	// -------------------------------------------------------------------------
	// Example 1: HealthChecker with injected functions
	// -------------------------------------------------------------------------
	fmt.Println("\n--- Example 1: HealthChecker with production functions ---")

	// Production fetcher: simulates HTTP calls
	productionFetcher := func(ctx context.Context, url string) ([]byte, error) {
		// Simulating a real HTTP GET with a small delay
		time.Sleep(10 * time.Millisecond)
		if strings.Contains(url, "down") {
			return nil, fmt.Errorf("connection refused: %s", url)
		}
		return []byte("OK"), nil
	}

	checker := NewHealthChecker(productionFetcher)

	services := map[string]string{
		"api":      "https://api.example.com/healthz",
		"database": "https://db.example.com/healthz",
		"cache":    "https://cache-down.example.com/healthz", // will fail
	}

	results := checker.CheckAll(context.Background(), services)
	for _, r := range results {
		status := "HEALTHY"
		if !r.Healthy {
			status = "UNHEALTHY"
		}
		fmt.Printf("  [%s] %s (%s) - latency: %v\n", status, r.ServiceName, r.URL, r.Latency)
		if r.Error != "" {
			fmt.Printf("         error: %s\n", r.Error)
		}
	}

	// -------------------------------------------------------------------------
	// Example 2: HealthChecker with test functions (deterministic)
	// -------------------------------------------------------------------------
	fmt.Println("\n--- Example 2: HealthChecker with test functions ---")

	fixedTime := time.Date(2026, 2, 18, 10, 0, 0, 0, time.UTC)
	callCount := 0

	testFetcher := func(ctx context.Context, url string) ([]byte, error) {
		if url == "https://broken.example.com/healthz" {
			return nil, fmt.Errorf("500 Internal Server Error")
		}
		return []byte("OK"), nil
	}

	testChecker := NewHealthChecker(
		testFetcher,
		WithClock(func() time.Time { return fixedTime }),
		WithIDGenerator(func() string {
			callCount++
			return fmt.Sprintf("check-%03d", callCount)
		}),
	)

	r := testChecker.Check(context.Background(), "api", "https://api.example.com/healthz")
	fmt.Printf("  ID: %s (predictable!)\n", r.ID)
	fmt.Printf("  Time: %v (fixed!)\n", r.CheckedAt)
	fmt.Printf("  Healthy: %v\n", r.Healthy)

	r = testChecker.Check(context.Background(), "broken", "https://broken.example.com/healthz")
	fmt.Printf("  ID: %s (sequential!)\n", r.ID)
	fmt.Printf("  Healthy: %v, Error: %s\n", r.Healthy, r.Error)

	// -------------------------------------------------------------------------
	// Example 3: RequestProcessor with function fields
	// -------------------------------------------------------------------------
	fmt.Println("\n--- Example 3: RequestProcessor with function fields ---")

	processedLog := make([]string, 0)

	processor := &RequestProcessor{
		Validate: func(payload string) error {
			if len(payload) == 0 {
				return fmt.Errorf("payload cannot be empty")
			}
			if len(payload) > 1000 {
				return fmt.Errorf("payload too large: %d bytes", len(payload))
			}
			return nil
		},
		Transform: func(payload string) string {
			return strings.ToUpper(strings.TrimSpace(payload))
		},
		RateLimit: func(key string) bool {
			// Simple: allow everything except "blocked-client"
			return key != "blocked-client"
		},
		OnProcessed: func(key, result string) {
			processedLog = append(processedLog, fmt.Sprintf("%s: %s", key, result))
		},
	}

	// Successful processing
	result, err := processor.Process("client-1", "  hello world  ")
	fmt.Printf("  Process 'hello world': result=%q, err=%v\n", result, err)

	// Rate limited
	result, err = processor.Process("blocked-client", "data")
	fmt.Printf("  Process blocked:       result=%q, err=%v\n", result, err)

	// Validation failure
	result, err = processor.Process("client-2", "")
	fmt.Printf("  Process empty:         result=%q, err=%v\n", result, err)

	fmt.Printf("  Processed log: %v\n", processedLog)

	// -------------------------------------------------------------------------
	// Example 4: Minimal processor (no optional functions)
	// -------------------------------------------------------------------------
	fmt.Println("\n--- Example 4: Minimal processor (nil functions = no-ops) ---")

	minimal := &RequestProcessor{} // All functions nil -- passthrough behavior
	result, err = minimal.Process("any-client", "raw data")
	fmt.Printf("  Minimal process: result=%q, err=%v\n", result, err)

	fmt.Println(strings.Repeat("=", 50))
	fmt.Println("\nKey takeaways:")
	fmt.Println("- Function types (Fetcher, Clock, IDGenerator) replace single-method interfaces")
	fmt.Println("- Production uses real functions; tests use deterministic ones")
	fmt.Println("- WithClock/WithIDGenerator are functional options for optional function deps")
	fmt.Println("- RequestProcessor shows struct fields as function deps (all optional)")
	fmt.Println("- Nil function checks enable graceful degradation (no-op behavior)")
	fmt.Println("- Closures capture state, so functions can have 'memory' without interfaces")
	fmt.Println("- Use this when a dependency has a single responsibility")
}
