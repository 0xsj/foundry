package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// ============================================================
// Client — configured via functional options
// ============================================================

// Client holds shared configuration for making HTTP requests.
// Created once with NewClient, reused across many requests.
type Client struct {
	baseURL string
	headers http.Header
	timeout time.Duration
	retries int
}

// ClientOption configures a Client. Return an error for invalid values.
type ClientOption func(*Client) error

// NewClient creates an API client with the given base URL and options.
//
// Design decisions:
// - baseURL is required (positional) because a client without a base URL is useless
// - Everything else is optional with sensible defaults
// - Options are applied in order; last one wins for conflicting settings
// - Validation happens per-option (fail fast) rather than at the end
func NewClient(baseURL string, opts ...ClientOption) (*Client, error) {
	if baseURL == "" {
		return nil, fmt.Errorf("base URL is required")
	}

	c := &Client{
		baseURL: strings.TrimRight(baseURL, "/"),
		headers: make(http.Header),
		timeout: 30 * time.Second, // Sensible default
		retries: 0,                // No retries by default
	}

	for _, opt := range opts {
		if err := opt(c); err != nil {
			return nil, fmt.Errorf("client option error: %w", err)
		}
	}

	return c, nil
}

// WithDefaultHeader adds a header that will be sent on every request.
func WithDefaultHeader(key, value string) ClientOption {
	return func(c *Client) error {
		if key == "" {
			return fmt.Errorf("header key cannot be empty")
		}
		c.headers.Set(key, value)
		return nil
	}
}

// WithBearerToken sets the Authorization header with a Bearer token.
func WithBearerToken(token string) ClientOption {
	return func(c *Client) error {
		if token == "" {
			return fmt.Errorf("bearer token cannot be empty")
		}
		c.headers.Set("Authorization", "Bearer "+token)
		return nil
	}
}

// WithTimeout sets the default timeout for all requests.
func WithTimeout(d time.Duration) ClientOption {
	return func(c *Client) error {
		if d <= 0 {
			return fmt.Errorf("timeout must be positive, got %v", d)
		}
		c.timeout = d
		return nil
	}
}

// WithRetries sets the default number of retries (0 = no retries).
func WithRetries(n int) ClientOption {
	return func(c *Client) error {
		if n < 0 {
			return fmt.Errorf("retries must be >= 0, got %d", n)
		}
		c.retries = n
		return nil
	}
}

// WithUserAgent sets the User-Agent header.
func WithUserAgent(ua string) ClientOption {
	return func(c *Client) error {
		if ua == "" {
			return fmt.Errorf("user agent cannot be empty")
		}
		c.headers.Set("User-Agent", ua)
		return nil
	}
}

// Request creates a new RequestBuilder pre-configured with this client's defaults.
// The builder inherits the client's base URL, headers, and timeout.
func (c *Client) Request() *RequestBuilder {
	return &RequestBuilder{
		client:  c,
		headers: make(http.Header),
		query:   make(url.Values),
		timeout: c.timeout,
	}
}

// ============================================================
// RequestBuilder — configured via fluent method chaining
// ============================================================

// RequestBuilder constructs a single HTTP request. Created from
// Client.Request(), configured via chained methods, finalized with Build().
type RequestBuilder struct {
	client  *Client
	method  string
	path    string
	headers http.Header
	query   url.Values
	body    []byte
	timeout time.Duration
	err     error // Captures first error for short-circuit
}

// setMethodAndPath is a helper that sets method and path with validation.
func (rb *RequestBuilder) setMethodAndPath(method, path string) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	if path == "" {
		rb.err = fmt.Errorf("path cannot be empty")
		return rb
	}
	rb.method = method
	rb.path = path
	return rb
}

// Get sets the HTTP method to GET and the request path.
func (rb *RequestBuilder) Get(path string) *RequestBuilder {
	return rb.setMethodAndPath(http.MethodGet, path)
}

// Post sets the HTTP method to POST and the request path.
func (rb *RequestBuilder) Post(path string) *RequestBuilder {
	return rb.setMethodAndPath(http.MethodPost, path)
}

// Put sets the HTTP method to PUT and the request path.
func (rb *RequestBuilder) Put(path string) *RequestBuilder {
	return rb.setMethodAndPath(http.MethodPut, path)
}

// Delete sets the HTTP method to DELETE and the request path.
func (rb *RequestBuilder) Delete(path string) *RequestBuilder {
	return rb.setMethodAndPath(http.MethodDelete, path)
}

// Header adds a header to this request. Overrides client defaults for this key.
func (rb *RequestBuilder) Header(key, value string) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	rb.headers.Set(key, value)
	return rb
}

// QueryParam adds a URL query parameter.
func (rb *RequestBuilder) QueryParam(key, value string) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	rb.query.Set(key, value)
	return rb
}

// Body sets the raw request body.
func (rb *RequestBuilder) Body(data []byte) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	rb.body = data
	return rb
}

// JSONBody marshals v as JSON and sets it as the request body.
// Also sets Content-Type to application/json.
func (rb *RequestBuilder) JSONBody(v any) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	data, err := json.Marshal(v)
	if err != nil {
		rb.err = fmt.Errorf("failed to marshal JSON body: %w", err)
		return rb
	}
	rb.body = data
	rb.headers.Set("Content-Type", "application/json")
	return rb
}

// Timeout overrides the client's default timeout for this request.
func (rb *RequestBuilder) Timeout(d time.Duration) *RequestBuilder {
	if rb.err != nil {
		return rb
	}
	if d <= 0 {
		rb.err = fmt.Errorf("timeout must be positive, got %v", d)
		return rb
	}
	rb.timeout = d
	return rb
}

// Build validates the request configuration and produces an *http.Request.
//
// Validation rules:
// - Method must be set (via Get/Post/Put/Delete)
// - Path must be non-empty
// - GET and DELETE must not have a body
// - URL must be constructable from baseURL + path + query
func (rb *RequestBuilder) Build(ctx context.Context) (*http.Request, error) {
	// Check for errors accumulated during chaining
	if rb.err != nil {
		return nil, fmt.Errorf("request build error: %w", rb.err)
	}

	// Validate method
	if rb.method == "" {
		return nil, fmt.Errorf("HTTP method is required (use Get, Post, Put, or Delete)")
	}

	// Validate path
	if rb.path == "" {
		return nil, fmt.Errorf("request path is required")
	}

	// Validate body constraints
	if len(rb.body) > 0 && (rb.method == http.MethodGet || rb.method == http.MethodDelete) {
		return nil, fmt.Errorf("%s requests must not have a body", rb.method)
	}

	// Construct URL: baseURL + path
	// Handle trailing/leading slash deduplication
	path := rb.path
	if !strings.HasPrefix(path, "/") {
		path = "/" + path
	}
	fullURL := rb.client.baseURL + path

	// Parse and add query params
	parsedURL, err := url.Parse(fullURL)
	if err != nil {
		return nil, fmt.Errorf("invalid URL %q: %w", fullURL, err)
	}
	if len(rb.query) > 0 {
		parsedURL.RawQuery = rb.query.Encode()
	}

	// Create the request
	var bodyReader *bytes.Reader
	if len(rb.body) > 0 {
		bodyReader = bytes.NewReader(rb.body)
	}

	var req *http.Request
	if bodyReader != nil {
		req, err = http.NewRequestWithContext(ctx, rb.method, parsedURL.String(), bodyReader)
	} else {
		req, err = http.NewRequestWithContext(ctx, rb.method, parsedURL.String(), nil)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Merge headers: start with client defaults, then overlay request-specific
	for key, values := range rb.client.headers {
		for _, v := range values {
			req.Header.Set(key, v)
		}
	}
	for key, values := range rb.headers {
		for _, v := range values {
			req.Header.Set(key, v)
		}
	}

	return req, nil
}

// ============================================================
// Main — demo usage
// ============================================================

func main() {
	fmt.Println("=== HTTP Request Builder Demo ===")
	fmt.Println(strings.Repeat("-", 50))

	// Create a client with functional options
	client, err := NewClient("https://api.example.com",
		WithBearerToken("sk-test-12345"),
		WithTimeout(10*time.Second),
		WithRetries(3),
		WithUserAgent("my-sdk/1.0"),
		WithDefaultHeader("X-Request-Source", "sdk"),
	)
	if err != nil {
		fmt.Printf("Client error: %v\n", err)
		return
	}

	// Build a GET request with query params
	fmt.Println("\n1. GET request:")
	req, err := client.Request().
		Get("/users").
		QueryParam("page", "1").
		QueryParam("limit", "25").
		Header("Accept", "application/json").
		Build(context.Background())

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   %s %s\n", req.Method, req.URL)
		fmt.Printf("   Authorization: %s\n", req.Header.Get("Authorization"))
		fmt.Printf("   User-Agent: %s\n", req.Header.Get("User-Agent"))
		fmt.Printf("   Accept: %s\n", req.Header.Get("Accept"))
	}

	// Build a POST request with JSON body
	fmt.Println("\n2. POST request with JSON body:")
	payload := map[string]string{
		"name":  "Alice",
		"email": "alice@example.com",
	}
	req, err = client.Request().
		Post("/users").
		JSONBody(payload).
		Build(context.Background())

	if err != nil {
		fmt.Printf("   Error: %v\n", err)
	} else {
		fmt.Printf("   %s %s\n", req.Method, req.URL)
		fmt.Printf("   Content-Type: %s\n", req.Header.Get("Content-Type"))
		body, _ := bytes.NewBuffer(nil), req.Body
		bodyBytes := make([]byte, req.ContentLength)
		req.Body.Read(bodyBytes)
		fmt.Printf("   Body: %s\n", string(bodyBytes))
	}

	// Validation: GET with body
	fmt.Println("\n3. Validation - GET with body (should fail):")
	_, err = client.Request().
		Get("/users").
		Body([]byte(`bad`)).
		Build(context.Background())
	fmt.Printf("   Error: %v\n", err)

	// Validation: missing method
	fmt.Println("\n4. Validation - missing method (should fail):")
	_, err = client.Request().
		Build(context.Background())
	fmt.Printf("   Error: %v\n", err)

	fmt.Println(strings.Repeat("-", 50))
}
