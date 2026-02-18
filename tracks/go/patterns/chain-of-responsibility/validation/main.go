// Chain of Responsibility: Request Validation Chain
//
// Demonstrates a chain of validators for API request processing.
// Each validator checks one aspect: schema, auth token, rate limit, business rules.
// The first validator to reject stops the chain.
//
// Run: go run ./validation/
package main

import (
	"context"
	"fmt"
	"strings"
	"time"
)

// --- Domain Types ---

// Request represents an incoming API request
type Request struct {
	Method      string
	Path        string
	ContentType string
	Body        string
	AuthToken   string
	ClientIP    string
	Timestamp   time.Time
}

// ValidationError holds details about why validation failed
type ValidationError struct {
	Validator string
	Field     string
	Message   string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("[%s] %s: %s", e.Validator, e.Field, e.Message)
}

// --- Validator Chain ---

// Validator is a single step in the validation chain.
// Returns nil to pass, or an error to reject.
type Validator func(ctx context.Context, req *Request) error

// ValidateChain runs validators in order. First failure stops the chain.
func ValidateChain(ctx context.Context, req *Request, validators ...Validator) error {
	for _, v := range validators {
		if err := v(ctx, req); err != nil {
			return err
		}
	}
	return nil
}

// --- Concrete Validators ---

// SchemaValidator checks that the request has required fields and correct format.
func SchemaValidator() Validator {
	return func(ctx context.Context, req *Request) error {
		if req.Method == "" {
			return &ValidationError{
				Validator: "schema",
				Field:     "method",
				Message:   "HTTP method is required",
			}
		}

		validMethods := map[string]bool{
			"GET": true, "POST": true, "PUT": true, "DELETE": true, "PATCH": true,
		}
		if !validMethods[req.Method] {
			return &ValidationError{
				Validator: "schema",
				Field:     "method",
				Message:   fmt.Sprintf("unsupported method: %s", req.Method),
			}
		}

		if req.Path == "" {
			return &ValidationError{
				Validator: "schema",
				Field:     "path",
				Message:   "request path is required",
			}
		}

		// POST/PUT/PATCH require content type and body
		if req.Method == "POST" || req.Method == "PUT" || req.Method == "PATCH" {
			if req.ContentType == "" {
				return &ValidationError{
					Validator: "schema",
					Field:     "content_type",
					Message:   "Content-Type header required for write operations",
				}
			}
			if req.Body == "" {
				return &ValidationError{
					Validator: "schema",
					Field:     "body",
					Message:   "request body required for write operations",
				}
			}
		}

		fmt.Println("  [schema] passed")
		return nil
	}
}

// AuthTokenValidator checks that the request has a valid auth token.
func AuthTokenValidator(validTokens map[string]string) Validator {
	return func(ctx context.Context, req *Request) error {
		if req.AuthToken == "" {
			return &ValidationError{
				Validator: "auth",
				Field:     "token",
				Message:   "authorization token is required",
			}
		}

		// Strip "Bearer " prefix if present
		token := req.AuthToken
		if strings.HasPrefix(token, "Bearer ") {
			token = strings.TrimPrefix(token, "Bearer ")
		}

		if _, ok := validTokens[token]; !ok {
			return &ValidationError{
				Validator: "auth",
				Field:     "token",
				Message:   "invalid or expired token",
			}
		}

		fmt.Printf("  [auth] passed (user: %s)\n", validTokens[token])
		return nil
	}
}

// RateLimitValidator checks that the client hasn't exceeded their rate limit.
// Uses a simple in-memory counter (production would use Redis or similar).
func RateLimitValidator(maxRequests int, window time.Duration) Validator {
	// Per-IP request tracking
	type clientState struct {
		count     int
		windowEnd time.Time
	}
	clients := make(map[string]*clientState)

	return func(ctx context.Context, req *Request) error {
		now := time.Now()
		state, exists := clients[req.ClientIP]

		if !exists || now.After(state.windowEnd) {
			// New window
			clients[req.ClientIP] = &clientState{
				count:     1,
				windowEnd: now.Add(window),
			}
			fmt.Printf("  [rate-limit] passed (1/%d in window)\n", maxRequests)
			return nil
		}

		state.count++
		if state.count > maxRequests {
			return &ValidationError{
				Validator: "rate-limit",
				Field:     "client_ip",
				Message: fmt.Sprintf(
					"rate limit exceeded: %d requests in %v (max %d)",
					state.count, window, maxRequests,
				),
			}
		}

		fmt.Printf("  [rate-limit] passed (%d/%d in window)\n", state.count, maxRequests)
		return nil
	}
}

// BusinessRuleValidator checks domain-specific rules.
func BusinessRuleValidator() Validator {
	return func(ctx context.Context, req *Request) error {
		// Rule: requests to /admin paths require special handling
		if strings.HasPrefix(req.Path, "/admin") && req.Method != "GET" {
			return &ValidationError{
				Validator: "business-rules",
				Field:     "path",
				Message:   "write operations on /admin paths require elevated privileges",
			}
		}

		// Rule: request body size limit (simulated via string length)
		if len(req.Body) > 1024 {
			return &ValidationError{
				Validator: "business-rules",
				Field:     "body",
				Message:   fmt.Sprintf("request body too large: %d bytes (max 1024)", len(req.Body)),
			}
		}

		// Rule: reject requests with timestamps too far in the future
		if !req.Timestamp.IsZero() && req.Timestamp.After(time.Now().Add(5*time.Minute)) {
			return &ValidationError{
				Validator: "business-rules",
				Field:     "timestamp",
				Message:   "request timestamp is too far in the future",
			}
		}

		fmt.Println("  [business-rules] passed")
		return nil
	}
}

// --- Main ---

func main() {
	fmt.Println("=== Chain of Responsibility: Validation Chain ===\n")

	// Set up the validator chain
	validTokens := map[string]string{
		"tok_abc123": "alice",
		"tok_def456": "bob",
	}

	chain := []Validator{
		SchemaValidator(),
		AuthTokenValidator(validTokens),
		RateLimitValidator(3, time.Minute),
		BusinessRuleValidator(),
	}

	// --- Test Cases ---

	// Test 1: Valid GET request
	fmt.Println("Test 1: Valid GET request")
	req := &Request{
		Method:    "GET",
		Path:      "/api/users",
		AuthToken: "Bearer tok_abc123",
		ClientIP:  "192.168.1.1",
		Timestamp: time.Now(),
	}
	if err := ValidateChain(context.Background(), req, chain...); err != nil {
		fmt.Printf("  REJECTED: %v\n", err)
	} else {
		fmt.Println("  ACCEPTED")
	}

	// Test 2: Missing auth token -- chain stops at auth validator
	fmt.Println("\nTest 2: Missing auth token")
	req = &Request{
		Method:   "GET",
		Path:     "/api/users",
		ClientIP: "192.168.1.2",
	}
	if err := ValidateChain(context.Background(), req, chain...); err != nil {
		fmt.Printf("  REJECTED: %v\n", err)
	} else {
		fmt.Println("  ACCEPTED")
	}

	// Test 3: Invalid method -- chain stops at schema validator
	fmt.Println("\nTest 3: Invalid method")
	req = &Request{
		Method:    "PURGE",
		Path:      "/api/cache",
		AuthToken: "Bearer tok_abc123",
		ClientIP:  "192.168.1.1",
	}
	if err := ValidateChain(context.Background(), req, chain...); err != nil {
		fmt.Printf("  REJECTED: %v\n", err)
	} else {
		fmt.Println("  ACCEPTED")
	}

	// Test 4: POST to /admin -- passes schema, auth, rate limit, but fails business rules
	fmt.Println("\nTest 4: POST to /admin (business rule violation)")
	req = &Request{
		Method:      "POST",
		Path:        "/admin/settings",
		ContentType: "application/json",
		Body:        `{"theme": "dark"}`,
		AuthToken:   "Bearer tok_abc123",
		ClientIP:    "192.168.1.1",
	}
	if err := ValidateChain(context.Background(), req, chain...); err != nil {
		fmt.Printf("  REJECTED: %v\n", err)
	} else {
		fmt.Println("  ACCEPTED")
	}

	// Test 5: Rate limit exceeded -- same IP sends too many requests
	fmt.Println("\nTest 5: Rate limit exceeded")
	for i := 0; i < 4; i++ {
		req = &Request{
			Method:    "GET",
			Path:      "/api/data",
			AuthToken: "Bearer tok_def456",
			ClientIP:  "10.0.0.1",
		}
		fmt.Printf("  Request %d: ", i+1)
		if err := ValidateChain(context.Background(), req, chain...); err != nil {
			fmt.Printf("REJECTED: %v\n", err)
		} else {
			fmt.Println("ACCEPTED")
		}
	}
}
