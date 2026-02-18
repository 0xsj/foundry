package main

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"testing"
	"time"
)

// ============================================================
// Client Tests
// ============================================================

func TestNewClient_ValidBaseURL(t *testing.T) {
	c, err := NewClient("https://api.example.com")
	if err != nil {
		t.Fatalf("expected no error, got: %v", err)
	}
	if c == nil {
		t.Fatal("expected non-nil client")
	}
}

func TestNewClient_EmptyBaseURL(t *testing.T) {
	_, err := NewClient("")
	if err == nil {
		t.Fatal("expected error for empty base URL")
	}
}

func TestNewClient_WithTimeout(t *testing.T) {
	c, err := NewClient("https://api.example.com", WithTimeout(5*time.Second))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if c.timeout != 5*time.Second {
		t.Errorf("expected timeout 5s, got %v", c.timeout)
	}
}

func TestNewClient_WithTimeout_Negative(t *testing.T) {
	_, err := NewClient("https://api.example.com", WithTimeout(-1*time.Second))
	if err == nil {
		t.Fatal("expected error for negative timeout")
	}
}

func TestNewClient_WithRetries(t *testing.T) {
	c, err := NewClient("https://api.example.com", WithRetries(3))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if c.retries != 3 {
		t.Errorf("expected retries 3, got %d", c.retries)
	}
}

func TestNewClient_WithRetries_Negative(t *testing.T) {
	_, err := NewClient("https://api.example.com", WithRetries(-1))
	if err == nil {
		t.Fatal("expected error for negative retries")
	}
}

func TestNewClient_WithBearerToken(t *testing.T) {
	c, err := NewClient("https://api.example.com", WithBearerToken("my-token"))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := c.headers.Get("Authorization")
	want := "Bearer my-token"
	if got != want {
		t.Errorf("expected Authorization %q, got %q", want, got)
	}
}

func TestNewClient_WithBearerToken_Empty(t *testing.T) {
	_, err := NewClient("https://api.example.com", WithBearerToken(""))
	if err == nil {
		t.Fatal("expected error for empty bearer token")
	}
}

func TestNewClient_WithDefaultHeader(t *testing.T) {
	c, err := NewClient("https://api.example.com",
		WithDefaultHeader("X-Custom", "value1"),
		WithDefaultHeader("X-Another", "value2"),
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got := c.headers.Get("X-Custom"); got != "value1" {
		t.Errorf("expected X-Custom=value1, got %q", got)
	}
	if got := c.headers.Get("X-Another"); got != "value2" {
		t.Errorf("expected X-Another=value2, got %q", got)
	}
}

func TestNewClient_WithUserAgent(t *testing.T) {
	c, err := NewClient("https://api.example.com", WithUserAgent("test-sdk/1.0"))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if got := c.headers.Get("User-Agent"); got != "test-sdk/1.0" {
		t.Errorf("expected User-Agent=test-sdk/1.0, got %q", got)
	}
}

func TestNewClient_MultipleOptions(t *testing.T) {
	c, err := NewClient("https://api.example.com",
		WithTimeout(5*time.Second),
		WithRetries(2),
		WithBearerToken("tok"),
		WithUserAgent("sdk/1.0"),
		WithDefaultHeader("X-Trace", "abc123"),
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if c.timeout != 5*time.Second {
		t.Errorf("timeout: got %v, want 5s", c.timeout)
	}
	if c.retries != 2 {
		t.Errorf("retries: got %d, want 2", c.retries)
	}
	if got := c.headers.Get("Authorization"); got != "Bearer tok" {
		t.Errorf("Authorization: got %q, want 'Bearer tok'", got)
	}
	if got := c.headers.Get("User-Agent"); got != "sdk/1.0" {
		t.Errorf("User-Agent: got %q, want 'sdk/1.0'", got)
	}
	if got := c.headers.Get("X-Trace"); got != "abc123" {
		t.Errorf("X-Trace: got %q, want 'abc123'", got)
	}
}

// ============================================================
// RequestBuilder Tests — HTTP Methods
// ============================================================

func mustClient(t *testing.T) *Client {
	t.Helper()
	c, err := NewClient("https://api.example.com")
	if err != nil {
		t.Fatalf("failed to create client: %v", err)
	}
	return c
}

func TestRequestBuilder_Get(t *testing.T) {
	c := mustClient(t)
	req, err := c.Request().
		Get("/users").
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if req.Method != http.MethodGet {
		t.Errorf("expected GET, got %s", req.Method)
	}
	if req.URL.Path != "/users" {
		t.Errorf("expected path /users, got %s", req.URL.Path)
	}
}

func TestRequestBuilder_Post(t *testing.T) {
	c := mustClient(t)
	body := []byte(`{"name":"test"}`)
	req, err := c.Request().
		Post("/users").
		Body(body).
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if req.Method != http.MethodPost {
		t.Errorf("expected POST, got %s", req.Method)
	}
}

func TestRequestBuilder_Put(t *testing.T) {
	c := mustClient(t)
	body := []byte(`{"name":"updated"}`)
	req, err := c.Request().
		Put("/users/123").
		Body(body).
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if req.Method != http.MethodPut {
		t.Errorf("expected PUT, got %s", req.Method)
	}
	if req.URL.Path != "/users/123" {
		t.Errorf("expected path /users/123, got %s", req.URL.Path)
	}
}

func TestRequestBuilder_Delete(t *testing.T) {
	c := mustClient(t)
	req, err := c.Request().
		Delete("/users/123").
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if req.Method != http.MethodDelete {
		t.Errorf("expected DELETE, got %s", req.Method)
	}
}

// ============================================================
// RequestBuilder Tests — Headers and Query Params
// ============================================================

func TestRequestBuilder_QueryParams(t *testing.T) {
	c := mustClient(t)
	req, err := c.Request().
		Get("/search").
		QueryParam("q", "golang builder").
		QueryParam("page", "2").
		QueryParam("limit", "25").
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	q := req.URL.Query()
	if got := q.Get("q"); got != "golang builder" {
		t.Errorf("q param: got %q, want 'golang builder'", got)
	}
	if got := q.Get("page"); got != "2" {
		t.Errorf("page param: got %q, want '2'", got)
	}
	if got := q.Get("limit"); got != "25" {
		t.Errorf("limit param: got %q, want '25'", got)
	}
}

func TestRequestBuilder_Headers_MergeWithClientDefaults(t *testing.T) {
	c, err := NewClient("https://api.example.com",
		WithBearerToken("client-token"),
		WithDefaultHeader("X-Client-ID", "sdk"),
	)
	if err != nil {
		t.Fatalf("client error: %v", err)
	}

	req, err := c.Request().
		Get("/data").
		Header("X-Request-ID", "req-123").
		Build(context.Background())

	if err != nil {
		t.Fatalf("build error: %v", err)
	}

	// Client default should be present
	if got := req.Header.Get("Authorization"); got != "Bearer client-token" {
		t.Errorf("Authorization: got %q, want 'Bearer client-token'", got)
	}
	if got := req.Header.Get("X-Client-ID"); got != "sdk" {
		t.Errorf("X-Client-ID: got %q, want 'sdk'", got)
	}
	// Request-specific header should be present
	if got := req.Header.Get("X-Request-ID"); got != "req-123" {
		t.Errorf("X-Request-ID: got %q, want 'req-123'", got)
	}
}

func TestRequestBuilder_Headers_OverrideClientDefault(t *testing.T) {
	c, err := NewClient("https://api.example.com",
		WithDefaultHeader("Accept", "text/plain"),
	)
	if err != nil {
		t.Fatalf("client error: %v", err)
	}

	req, err := c.Request().
		Get("/data").
		Header("Accept", "application/json"). // Override client default
		Build(context.Background())

	if err != nil {
		t.Fatalf("build error: %v", err)
	}
	if got := req.Header.Get("Accept"); got != "application/json" {
		t.Errorf("Accept: got %q, want 'application/json'", got)
	}
}

// ============================================================
// RequestBuilder Tests — Body
// ============================================================

func TestRequestBuilder_RawBody(t *testing.T) {
	c := mustClient(t)
	payload := []byte(`{"key":"value"}`)
	req, err := c.Request().
		Post("/data").
		Body(payload).
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	body, _ := io.ReadAll(req.Body)
	if string(body) != `{"key":"value"}` {
		t.Errorf("body: got %q, want '{\"key\":\"value\"}'", string(body))
	}
}

func TestRequestBuilder_JSONBody(t *testing.T) {
	c := mustClient(t)

	type User struct {
		Name  string `json:"name"`
		Email string `json:"email"`
	}

	req, err := c.Request().
		Post("/users").
		JSONBody(User{Name: "Alice", Email: "alice@example.com"}).
		Build(context.Background())

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Check Content-Type header
	if got := req.Header.Get("Content-Type"); got != "application/json" {
		t.Errorf("Content-Type: got %q, want 'application/json'", got)
	}

	// Check body is valid JSON
	body, _ := io.ReadAll(req.Body)
	var decoded User
	if err := json.Unmarshal(body, &decoded); err != nil {
		t.Fatalf("body is not valid JSON: %v", err)
	}
	if decoded.Name != "Alice" || decoded.Email != "alice@example.com" {
		t.Errorf("decoded body: got %+v, want {Alice alice@example.com}", decoded)
	}
}

// ============================================================
// RequestBuilder Tests — Validation
// ============================================================

func TestRequestBuilder_NoMethod(t *testing.T) {
	c := mustClient(t)
	_, err := c.Request().
		Build(context.Background())

	if err == nil {
		t.Fatal("expected error for missing method")
	}
}

func TestRequestBuilder_EmptyPath(t *testing.T) {
	c := mustClient(t)
	_, err := c.Request().
		Get("").
		Build(context.Background())

	if err == nil {
		t.Fatal("expected error for empty path")
	}
}

func TestRequestBuilder_GetWithBody(t *testing.T) {
	c := mustClient(t)
	_, err := c.Request().
		Get("/users").
		Body([]byte(`{"bad":"request"}`)).
		Build(context.Background())

	if err == nil {
		t.Fatal("expected error for GET request with body")
	}
}

func TestRequestBuilder_DeleteWithBody(t *testing.T) {
	c := mustClient(t)
	_, err := c.Request().
		Delete("/users/123").
		Body([]byte(`{"bad":"request"}`)).
		Build(context.Background())

	if err == nil {
		t.Fatal("expected error for DELETE request with body")
	}
}

// ============================================================
// RequestBuilder Tests — URL Construction
// ============================================================

func TestRequestBuilder_URLConstruction(t *testing.T) {
	c, err := NewClient("https://api.example.com")
	if err != nil {
		t.Fatalf("client error: %v", err)
	}

	req, err := c.Request().
		Get("/v2/users").
		QueryParam("active", "true").
		Build(context.Background())

	if err != nil {
		t.Fatalf("build error: %v", err)
	}

	want := "https://api.example.com/v2/users?active=true"
	if got := req.URL.String(); got != want {
		t.Errorf("URL: got %q, want %q", got, want)
	}
}

func TestRequestBuilder_URLConstruction_TrailingSlash(t *testing.T) {
	// Base URL has trailing slash, path has leading slash
	c, err := NewClient("https://api.example.com/")
	if err != nil {
		t.Fatalf("client error: %v", err)
	}

	req, err := c.Request().
		Get("/users").
		Build(context.Background())

	if err != nil {
		t.Fatalf("build error: %v", err)
	}

	// Should not have double slash
	got := req.URL.String()
	if got != "https://api.example.com/users" {
		t.Errorf("URL: got %q, want 'https://api.example.com/users'", got)
	}
}
