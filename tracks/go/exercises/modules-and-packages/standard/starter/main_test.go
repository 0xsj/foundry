package main

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

// ============================================================================
// Tests for the monolithic version.
//
// After refactoring, these tests move to their respective packages:
//   internal/checker/checker_test.go   — TestCheckURL, TestCheckURL_Error
//   internal/reporter/reporter_test.go — TestPrintResults (or table-driven)
//   pkg/config/config_test.go          — TestLoadConfig
// ============================================================================

// TestCheckURL_Healthy verifies that a 200 response is correctly identified.
func TestCheckURL_Healthy(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	result := checkURL(srv.URL, 5.0)

	if result.Err != nil {
		t.Fatalf("unexpected error: %v", result.Err)
	}
	if result.StatusCode != http.StatusOK {
		t.Errorf("got status %d, want %d", result.StatusCode, http.StatusOK)
	}
	if !result.IsHealthy() {
		t.Error("expected result to be healthy")
	}
	if result.Latency <= 0 {
		t.Error("expected positive latency")
	}
}

// TestCheckURL_NotFound verifies that a 404 is captured correctly.
func TestCheckURL_NotFound(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	}))
	defer srv.Close()

	result := checkURL(srv.URL, 5.0)

	if result.Err != nil {
		t.Fatalf("unexpected error: %v", result.Err)
	}
	if result.StatusCode != http.StatusNotFound {
		t.Errorf("got status %d, want %d", result.StatusCode, http.StatusNotFound)
	}
	if result.IsHealthy() {
		t.Error("expected result to be unhealthy for 404")
	}
}

// TestCheckURL_Timeout verifies that a slow server triggers the timeout.
func TestCheckURL_Timeout(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(500 * time.Millisecond) // longer than the timeout we'll set
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	result := checkURL(srv.URL, 0.1) // 100ms timeout

	if result.Err == nil {
		t.Fatal("expected a timeout error, got nil")
	}
	if result.IsHealthy() {
		t.Error("a timed-out request should not be healthy")
	}
}

// TestCheckURL_ServerError verifies that a 500 is captured and marked unhealthy.
func TestCheckURL_ServerError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()

	result := checkURL(srv.URL, 5.0)

	if result.Err != nil {
		t.Fatalf("unexpected error: %v", result.Err)
	}
	if result.StatusCode != http.StatusInternalServerError {
		t.Errorf("got status %d, want 500", result.StatusCode)
	}
	if result.IsHealthy() {
		t.Error("expected result to be unhealthy for 500")
	}
}

// TestCheckAll verifies that checkAll returns one result per URL.
func TestCheckAll(t *testing.T) {
	// Three test servers
	makeServer := func(code int) *httptest.Server {
		return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(code)
		}))
	}

	s1 := makeServer(200)
	s2 := makeServer(404)
	s3 := makeServer(503)
	defer s1.Close()
	defer s2.Close()
	defer s3.Close()

	urls := []string{s1.URL, s2.URL, s3.URL}
	results := checkAll(urls, 5.0)

	if len(results) != 3 {
		t.Fatalf("expected 3 results, got %d", len(results))
	}

	cases := []struct {
		result  Result
		healthy bool
		code    int
	}{
		{results[0], true, 200},
		{results[1], false, 404},
		{results[2], false, 503},
	}

	for i, tc := range cases {
		if tc.result.StatusCode != tc.code {
			t.Errorf("result[%d]: got status %d, want %d", i, tc.result.StatusCode, tc.code)
		}
		if tc.result.IsHealthy() != tc.healthy {
			t.Errorf("result[%d]: IsHealthy()=%v, want %v", i, tc.result.IsHealthy(), tc.healthy)
		}
	}
}

// TestResult_IsHealthy tests the IsHealthy helper directly.
func TestResult_IsHealthy(t *testing.T) {
	tests := []struct {
		name    string
		result  Result
		healthy bool
	}{
		{"200 OK", Result{StatusCode: 200}, true},
		{"201 Created", Result{StatusCode: 201}, true},
		{"299 edge", Result{StatusCode: 299}, true},
		{"300 redirect", Result{StatusCode: 300}, false},
		{"404 not found", Result{StatusCode: 404}, false},
		{"500 server error", Result{StatusCode: 500}, false},
		{"network error", Result{Err: http.ErrHandlerTimeout}, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.result.IsHealthy()
			if got != tt.healthy {
				t.Errorf("IsHealthy() = %v, want %v", got, tt.healthy)
			}
		})
	}
}
