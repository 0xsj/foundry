package main

import (
	"bytes"
	"compress/gzip"
	"context"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"
	"time"
)

// --- Test helpers ---

func okHandler(body string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(body))
	})
}

func panicHandler(msg string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		panic(msg)
	})
}

func statusHandler(code int) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(code)
	})
}

func contextCheckHandler(t *testing.T, key *contextKey, expected string) http.Handler {
	t.Helper()
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		val, ok := r.Context().Value(key).(string)
		if !ok || val != expected {
			t.Errorf("expected context value %q for key %s, got %q (ok=%v)", expected, key.name, val, ok)
		}
		w.WriteHeader(http.StatusOK)
	})
}

// --- Chain Tests ---

func TestChain_ExecutionOrder(t *testing.T) {
	var order []string

	mwA := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			order = append(order, "A-before")
			next.ServeHTTP(w, r)
			order = append(order, "A-after")
		})
	}

	mwB := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			order = append(order, "B-before")
			next.ServeHTTP(w, r)
			order = append(order, "B-after")
		})
	}

	mwC := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			order = append(order, "C-before")
			next.ServeHTTP(w, r)
			order = append(order, "C-after")
		})
	}

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		order = append(order, "handler")
	})

	chain := Chain(handler, mwA, mwB, mwC)

	req := httptest.NewRequest("GET", "/test", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	expected := []string{"A-before", "B-before", "C-before", "handler", "C-after", "B-after", "A-after"}
	if len(order) != len(expected) {
		t.Fatalf("expected %d entries, got %d: %v", len(expected), len(order), order)
	}
	for i, v := range expected {
		if order[i] != v {
			t.Errorf("position %d: expected %q, got %q (full order: %v)", i, v, order[i], order)
		}
	}
}

func TestChain_EmptyMiddleware(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := Chain(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rr.Code)
	}
}

// --- Request ID Tests ---

func TestWithRequestID_GeneratesID(t *testing.T) {
	var capturedID string
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		capturedID = RequestIDFromCtx(r.Context())
		w.WriteHeader(http.StatusOK)
	})

	chain := WithRequestID(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if capturedID == "" {
		t.Error("expected request ID in context, got empty string")
	}

	responseID := rr.Header().Get("X-Request-ID")
	if responseID == "" {
		t.Error("expected X-Request-ID in response header")
	}

	if capturedID != responseID {
		t.Errorf("context ID %q does not match response header ID %q", capturedID, responseID)
	}
}

func TestWithRequestID_ReusesExisting(t *testing.T) {
	existingID := "trace-abc-123"
	var capturedID string

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		capturedID = RequestIDFromCtx(r.Context())
		w.WriteHeader(http.StatusOK)
	})

	chain := WithRequestID(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("X-Request-ID", existingID)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if capturedID != existingID {
		t.Errorf("expected reused ID %q, got %q", existingID, capturedID)
	}

	if rr.Header().Get("X-Request-ID") != existingID {
		t.Errorf("expected response header to echo %q", existingID)
	}
}

func TestWithRequestID_UniqueAcrossConcurrentRequests(t *testing.T) {
	ids := make([]string, 100)
	var wg sync.WaitGroup

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	chain := WithRequestID(handler)

	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func(idx int) {
			defer wg.Done()
			req := httptest.NewRequest("GET", "/", nil)
			rr := httptest.NewRecorder()
			chain.ServeHTTP(rr, req)
			ids[idx] = rr.Header().Get("X-Request-ID")
		}(i)
	}
	wg.Wait()

	seen := make(map[string]bool)
	for i, id := range ids {
		if id == "" {
			t.Errorf("request %d: got empty ID", i)
			continue
		}
		if seen[id] {
			t.Errorf("duplicate request ID: %q", id)
		}
		seen[id] = true
	}
}

// --- Logging Tests ---

func TestWithLogging_OutputFormat(t *testing.T) {
	var buf bytes.Buffer

	handler := okHandler(`{"users":[]}`)
	chain := Chain(handler, WithRequestID, WithLogging(&buf))

	req := httptest.NewRequest("GET", "/api/users", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	logOutput := buf.String()

	if !strings.Contains(logOutput, "GET") {
		t.Errorf("log should contain method GET: %s", logOutput)
	}
	if !strings.Contains(logOutput, "/api/users") {
		t.Errorf("log should contain path /api/users: %s", logOutput)
	}
	if !strings.Contains(logOutput, "200") {
		t.Errorf("log should contain status 200: %s", logOutput)
	}
}

func TestWithLogging_CapturesNon200Status(t *testing.T) {
	var buf bytes.Buffer

	handler := statusHandler(http.StatusNotFound)
	chain := Chain(handler, WithLogging(&buf))

	req := httptest.NewRequest("GET", "/missing", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	_ = rr // suppress unused
	logOutput := buf.String()

	if !strings.Contains(logOutput, "404") {
		t.Errorf("log should contain status 404: %s", logOutput)
	}
}

// --- Auth Tests ---

func testValidator(token string) (string, error) {
	validTokens := map[string]string{
		"valid-token-alice": "alice",
		"valid-token-bob":   "bob",
	}
	if clientID, ok := validTokens[token]; ok {
		return clientID, nil
	}
	return "", errors.New("invalid token")
}

func TestWithAuth_ValidToken(t *testing.T) {
	handler := contextCheckHandler(t, clientIDKey, "alice")
	chain := WithAuth(testValidator)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer valid-token-alice")
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rr.Code)
	}
}

func TestWithAuth_MissingHeader(t *testing.T) {
	handler := okHandler("should not reach")
	chain := WithAuth(testValidator)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d", rr.Code)
	}
}

func TestWithAuth_InvalidToken(t *testing.T) {
	handler := okHandler("should not reach")
	chain := WithAuth(testValidator)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer bad-token")
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusForbidden {
		t.Errorf("expected 403, got %d", rr.Code)
	}
}

func TestWithAuth_StoresClientIDInContext(t *testing.T) {
	var gotClientID string
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotClientID = ClientIDFromCtx(r.Context())
		w.WriteHeader(http.StatusOK)
	})
	chain := WithAuth(testValidator)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Authorization", "Bearer valid-token-bob")
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if gotClientID != "bob" {
		t.Errorf("expected client ID 'bob', got %q", gotClientID)
	}
	_ = rr
}

// --- Rate Limit Tests ---

func TestWithRateLimit_AllowsUnderLimit(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(5, time.Second)(handler)

	for i := 0; i < 5; i++ {
		req := httptest.NewRequest("GET", "/", nil)
		req.RemoteAddr = "192.168.1.1:1234"
		rr := httptest.NewRecorder()
		chain.ServeHTTP(rr, req)

		if rr.Code != http.StatusOK {
			t.Errorf("request %d: expected 200, got %d", i+1, rr.Code)
		}
	}
}

func TestWithRateLimit_BlocksOverLimit(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(3, time.Second)(handler)

	for i := 0; i < 5; i++ {
		req := httptest.NewRequest("GET", "/", nil)
		req.RemoteAddr = "192.168.1.1:1234"
		rr := httptest.NewRecorder()
		chain.ServeHTTP(rr, req)

		if i < 3 && rr.Code != http.StatusOK {
			t.Errorf("request %d: expected 200, got %d", i+1, rr.Code)
		}
		if i >= 3 && rr.Code != http.StatusTooManyRequests {
			t.Errorf("request %d: expected 429, got %d", i+1, rr.Code)
		}
	}
}

func TestWithRateLimit_SetsRetryAfterHeader(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(1, time.Second)(handler)

	// First request: OK
	req := httptest.NewRequest("GET", "/", nil)
	req.RemoteAddr = "10.0.0.1:5000"
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	// Second request: rate limited
	req = httptest.NewRequest("GET", "/", nil)
	req.RemoteAddr = "10.0.0.1:5000"
	rr = httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Header().Get("Retry-After") == "" {
		t.Error("expected Retry-After header on 429 response")
	}
}

func TestWithRateLimit_PerClientIsolation(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(2, time.Second)(handler)

	// Client A: 2 requests (should all succeed)
	for i := 0; i < 2; i++ {
		req := httptest.NewRequest("GET", "/", nil)
		req.RemoteAddr = "client-a:1234"
		rr := httptest.NewRecorder()
		chain.ServeHTTP(rr, req)
		if rr.Code != http.StatusOK {
			t.Errorf("client A request %d: expected 200, got %d", i+1, rr.Code)
		}
	}

	// Client B: should have its own quota
	req := httptest.NewRequest("GET", "/", nil)
	req.RemoteAddr = "client-b:1234"
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("client B: expected 200, got %d", rr.Code)
	}
}

func TestWithRateLimit_UsesClientIDFromContext(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(1, time.Second)(handler)

	// Inject client ID via context (simulating auth middleware upstream)
	req := httptest.NewRequest("GET", "/", nil)
	ctx := context.WithValue(req.Context(), clientIDKey, "authenticated-user")
	req = req.WithContext(ctx)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("first request: expected 200, got %d", rr.Code)
	}

	// Second request with same client ID: should be rate limited
	req = httptest.NewRequest("GET", "/", nil)
	req.RemoteAddr = "different-ip:9999" // different IP, same client ID
	ctx = context.WithValue(req.Context(), clientIDKey, "authenticated-user")
	req = req.WithContext(ctx)
	rr = httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusTooManyRequests {
		t.Errorf("second request: expected 429, got %d", rr.Code)
	}
}

func TestWithRateLimit_ConcurrentSafety(t *testing.T) {
	handler := okHandler(`{"ok":true}`)
	chain := WithRateLimit(100, time.Second)(handler)

	var wg sync.WaitGroup
	for i := 0; i < 200; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			req := httptest.NewRequest("GET", "/", nil)
			req.RemoteAddr = "concurrent-client:1234"
			rr := httptest.NewRecorder()
			chain.ServeHTTP(rr, req)
			// Just verify no panic -- concurrency safety test
			if rr.Code != http.StatusOK && rr.Code != http.StatusTooManyRequests {
				t.Errorf("unexpected status: %d", rr.Code)
			}
		}()
	}
	wg.Wait()
}

// --- Recovery Tests ---

func TestWithRecovery_CatchesPanic(t *testing.T) {
	var logBuf bytes.Buffer
	handler := panicHandler("test panic value")
	chain := WithRecovery(&logBuf)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()

	// Should not panic
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusInternalServerError {
		t.Errorf("expected 500, got %d", rr.Code)
	}

	body := rr.Body.String()
	if !strings.Contains(body, "internal server error") {
		t.Errorf("expected error message in body, got: %s", body)
	}
}

func TestWithRecovery_LogsPanicValue(t *testing.T) {
	var logBuf bytes.Buffer
	handler := panicHandler("database connection lost")
	chain := WithRecovery(&logBuf)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	_ = rr
	logOutput := logBuf.String()
	if !strings.Contains(logOutput, "database connection lost") {
		t.Errorf("expected panic value in log, got: %s", logOutput)
	}
}

func TestWithRecovery_PassesThroughNormalRequests(t *testing.T) {
	var logBuf bytes.Buffer
	handler := okHandler(`{"status":"healthy"}`)
	chain := WithRecovery(&logBuf)(handler)

	req := httptest.NewRequest("GET", "/", nil)
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rr.Code)
	}
	if rr.Body.String() != `{"status":"healthy"}` {
		t.Errorf("expected body unchanged, got: %s", rr.Body.String())
	}
}

// --- Compression Tests ---

func TestWithCompression_CompressesWhenAccepted(t *testing.T) {
	body := strings.Repeat(`{"data": "test value"}`, 100) // large enough to benefit from compression
	handler := okHandler(body)
	chain := WithCompression(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Accept-Encoding", "gzip")
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Header().Get("Content-Encoding") != "gzip" {
		t.Error("expected Content-Encoding: gzip in response")
	}

	// Verify we can decompress the response
	reader, err := gzip.NewReader(rr.Body)
	if err != nil {
		t.Fatalf("failed to create gzip reader: %v", err)
	}
	defer reader.Close()

	decompressed, err := io.ReadAll(reader)
	if err != nil {
		t.Fatalf("failed to decompress: %v", err)
	}

	if string(decompressed) != body {
		t.Errorf("decompressed body does not match original.\nexpected length: %d\ngot length: %d", len(body), len(decompressed))
	}
}

func TestWithCompression_SkipsWhenNotAccepted(t *testing.T) {
	body := `{"data": "test"}`
	handler := okHandler(body)
	chain := WithCompression(handler)

	req := httptest.NewRequest("GET", "/", nil)
	// No Accept-Encoding header
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if rr.Header().Get("Content-Encoding") == "gzip" {
		t.Error("should not compress when client does not accept gzip")
	}

	if rr.Body.String() != body {
		t.Errorf("expected uncompressed body %q, got %q", body, rr.Body.String())
	}
}

func TestWithCompression_RemovesContentLength(t *testing.T) {
	handler := okHandler(`{"data": "test"}`)
	chain := WithCompression(handler)

	req := httptest.NewRequest("GET", "/", nil)
	req.Header.Set("Accept-Encoding", "gzip")
	rr := httptest.NewRecorder()
	chain.ServeHTTP(rr, req)

	if cl := rr.Header().Get("Content-Length"); cl != "" {
		t.Errorf("Content-Length should be removed when compressing, got: %s", cl)
	}
}

// --- Integration Test ---

func TestFullMiddlewareStack(t *testing.T) {
	var logBuf bytes.Buffer

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		clientID := ClientIDFromCtx(r.Context())
		reqID := RequestIDFromCtx(r.Context())
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"client":"` + clientID + `","request_id":"` + reqID + `"}`))
	})

	stack := Chain(handler,
		WithRecovery(&logBuf),
		WithRequestID,
		WithLogging(&logBuf),
		WithAuth(testValidator),
		WithRateLimit(10, time.Second),
	)

	req := httptest.NewRequest("GET", "/api/data", nil)
	req.Header.Set("Authorization", "Bearer valid-token-alice")
	rr := httptest.NewRecorder()
	stack.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d. Body: %s", rr.Code, rr.Body.String())
	}

	body := rr.Body.String()
	if !strings.Contains(body, `"client":"alice"`) {
		t.Errorf("expected client alice in response body: %s", body)
	}
	if !strings.Contains(body, `"request_id":`) {
		t.Errorf("expected request_id in response body: %s", body)
	}
	if rr.Header().Get("X-Request-ID") == "" {
		t.Error("expected X-Request-ID in response headers")
	}
}
