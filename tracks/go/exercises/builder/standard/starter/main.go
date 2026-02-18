package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
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
// The base URL is required; options configure defaults for all requests.
func NewClient(baseURL string, opts ...ClientOption) (*Client, error) {
	// TODO: Implement
	// 1. Validate baseURL is non-empty
	// 2. Create client with sensible defaults
	// 3. Apply each option, returning first error
	// 4. Return configured client
	return nil, fmt.Errorf("not implemented")
}

// WithDefaultHeader adds a header that will be sent on every request.
func WithDefaultHeader(key, value string) ClientOption {
	// TODO: Implement
	return func(c *Client) error {
		return fmt.Errorf("not implemented")
	}
}

// WithBearerToken sets the Authorization header with a Bearer token.
func WithBearerToken(token string) ClientOption {
	// TODO: Implement
	return func(c *Client) error {
		return fmt.Errorf("not implemented")
	}
}

// WithTimeout sets the default timeout for all requests.
func WithTimeout(d time.Duration) ClientOption {
	// TODO: Implement
	return func(c *Client) error {
		return fmt.Errorf("not implemented")
	}
}

// WithRetries sets the default number of retries (0 = no retries).
func WithRetries(n int) ClientOption {
	// TODO: Implement
	return func(c *Client) error {
		return fmt.Errorf("not implemented")
	}
}

// WithUserAgent sets the User-Agent header.
func WithUserAgent(ua string) ClientOption {
	// TODO: Implement
	return func(c *Client) error {
		return fmt.Errorf("not implemented")
	}
}

// Request creates a new RequestBuilder with this client's defaults.
func (c *Client) Request() *RequestBuilder {
	// TODO: Implement
	// Create a RequestBuilder pre-loaded with client defaults
	return nil
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

// Get sets the HTTP method to GET and the request path.
func (rb *RequestBuilder) Get(path string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Post sets the HTTP method to POST and the request path.
func (rb *RequestBuilder) Post(path string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Put sets the HTTP method to PUT and the request path.
func (rb *RequestBuilder) Put(path string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Delete sets the HTTP method to DELETE and the request path.
func (rb *RequestBuilder) Delete(path string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Header adds a header to this request. Overrides client defaults for this key.
func (rb *RequestBuilder) Header(key, value string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// QueryParam adds a URL query parameter.
func (rb *RequestBuilder) QueryParam(key, value string) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Body sets the raw request body.
func (rb *RequestBuilder) Body(data []byte) *RequestBuilder {
	// TODO: Implement
	return rb
}

// JSONBody marshals v as JSON and sets it as the request body.
// Also sets Content-Type to application/json.
func (rb *RequestBuilder) JSONBody(v any) *RequestBuilder {
	// TODO: Implement
	// 1. Marshal v to JSON
	// 2. Set rb.body
	// 3. Set Content-Type header
	return rb
}

// Timeout overrides the client's default timeout for this request.
func (rb *RequestBuilder) Timeout(d time.Duration) *RequestBuilder {
	// TODO: Implement
	return rb
}

// Build validates the request and returns an *http.Request.
// Returns an error if the request is invalid.
func (rb *RequestBuilder) Build(ctx context.Context) (*http.Request, error) {
	// TODO: Implement
	// 1. Check for accumulated errors (rb.err)
	// 2. Validate method is set
	// 3. Validate path is non-empty
	// 4. Validate GET/DELETE have no body
	// 5. Construct full URL (baseURL + path + query params)
	// 6. Create http.Request with method, URL, and body
	// 7. Merge headers (client defaults + request overrides)
	// 8. Return the request
	return nil, fmt.Errorf("not implemented")
}

// ============================================================
// Main — demo usage
// ============================================================

func main() {
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

	// Build a GET request
	req, err := client.Request().
		Get("/users").
		QueryParam("page", "1").
		QueryParam("limit", "25").
		Header("Accept", "application/json").
		Build(context.Background())

	if err != nil {
		fmt.Printf("GET error: %v\n", err)
	} else {
		fmt.Printf("GET %s\n", req.URL)
		fmt.Printf("Headers: %v\n", req.Header)
	}

	// Build a POST request with JSON body
	payload := map[string]string{
		"name":  "Alice",
		"email": "alice@example.com",
	}

	req, err = client.Request().
		Post("/users").
		JSONBody(payload).
		Build(context.Background())

	if err != nil {
		fmt.Printf("POST error: %v\n", err)
	} else {
		fmt.Printf("\nPOST %s\n", req.URL)
		fmt.Printf("Content-Type: %s\n", req.Header.Get("Content-Type"))
	}

	// Suppress unused import warnings
	_ = json.Marshal
	_ = url.Values{}
}
