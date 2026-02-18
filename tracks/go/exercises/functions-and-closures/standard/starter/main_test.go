package middleware

import (
	"strings"
	"testing"
	"time"
)

// ============================================================================
// TEST HELPERS
// ============================================================================

// okHandler is a simple handler that always returns 200.
var okHandler Handler = func(r Request) Response {
	return Response{Status: 200, Body: "OK"}
}

// echoHandler returns the request path and body in the response.
var echoHandler Handler = func(r Request) Response {
	return Response{Status: 200, Body: r.Method + " " + r.Path + ": " + r.Body}
}

// panicHandler always panics — used to test the Recover middleware.
var panicHandler Handler = func(r Request) Response {
	panic("intentional panic for testing")
}

func makeRequest(method, path string) Request {
	return Request{
		Method:   method,
		Path:     path,
		Headers:  make(map[string]string),
		ClientIP: "127.0.0.1",
	}
}

func makeAuthedRequest(method, path, token string) Request {
	r := makeRequest(method, path)
	r.Headers["Authorization"] = token
	return r
}

// ============================================================================
// CHAIN TESTS
// ============================================================================

func TestChain_NoMiddleware(t *testing.T) {
	chain := Chain()(okHandler)
	resp := chain(makeRequest("GET", "/health"))
	if resp.Status != 200 {
		t.Errorf("expected 200, got %d", resp.Status)
	}
}

func TestChain_SingleMiddleware(t *testing.T) {
	var called bool
	tracer := func(next Handler) Handler {
		return func(r Request) Response {
			called = true
			return next(r)
		}
	}

	chain := Chain(tracer)(okHandler)
	chain(makeRequest("GET", "/"))
	if !called {
		t.Error("middleware was not called")
	}
}

func TestChain_OrderIsPreserved(t *testing.T) {
	// Each middleware appends its name to a shared log.
	// We verify the execution order matches registration order.
	var log []string

	makeTracer := func(name string) Middleware {
		return func(next Handler) Handler {
			return func(r Request) Response {
				log = append(log, name+":before")
				resp := next(r)
				log = append(log, name+":after")
				return resp
			}
		}
	}

	chain := Chain(makeTracer("m1"), makeTracer("m2"), makeTracer("m3"))(okHandler)
	chain(makeRequest("GET", "/"))

	expected := []string{
		"m1:before", "m2:before", "m3:before",
		"m3:after", "m2:after", "m1:after",
	}
	if len(log) != len(expected) {
		t.Fatalf("expected %d log entries, got %d: %v", len(expected), len(log), log)
	}
	for i, want := range expected {
		if log[i] != want {
			t.Errorf("step %d: want %q, got %q", i, want, log[i])
		}
	}
}

func TestChain_MultipleMiddlewaresPassRequest(t *testing.T) {
	chain := Chain(
		func(next Handler) Handler {
			return func(r Request) Response { return next(r) }
		},
		func(next Handler) Handler {
			return func(r Request) Response { return next(r) }
		},
	)(echoHandler)

	resp := chain(makeRequest("POST", "/api/data"))
	if resp.Status != 200 {
		t.Errorf("expected 200, got %d", resp.Status)
	}
	if !strings.Contains(resp.Body, "/api/data") {
		t.Errorf("expected body to contain path, got %q", resp.Body)
	}
}

// ============================================================================
// AUTH TESTS
// ============================================================================

func TestAuth_ValidToken(t *testing.T) {
	handler := Auth("secret-token")(okHandler)
	resp := handler(makeAuthedRequest("GET", "/", "secret-token"))
	if resp.Status != 200 {
		t.Errorf("expected 200 with valid token, got %d", resp.Status)
	}
}

func TestAuth_MissingHeader(t *testing.T) {
	handler := Auth("secret-token")(okHandler)
	resp := handler(makeRequest("GET", "/"))
	if resp.Status != 401 {
		t.Errorf("expected 401 with missing token, got %d", resp.Status)
	}
}

func TestAuth_WrongToken(t *testing.T) {
	handler := Auth("secret-token")(okHandler)
	resp := handler(makeAuthedRequest("GET", "/", "wrong-token"))
	if resp.Status != 401 {
		t.Errorf("expected 401 with wrong token, got %d", resp.Status)
	}
}

func TestAuth_ShortCircuits(t *testing.T) {
	// If auth fails, the inner handler must NOT be called.
	var innerCalled bool
	inner := Handler(func(r Request) Response {
		innerCalled = true
		return Response{Status: 200}
	})

	handler := Auth("token")(inner)
	handler(makeAuthedRequest("GET", "/", "wrong"))

	if innerCalled {
		t.Error("inner handler was called despite auth failure — should have short-circuited")
	}
}

func TestAuth_CapturesTokenPerInstance(t *testing.T) {
	// Each Auth call should capture its own token.
	authA := Auth("token-a")(okHandler)
	authB := Auth("token-b")(okHandler)

	if authA(makeAuthedRequest("GET", "/", "token-a")).Status != 200 {
		t.Error("authA should accept token-a")
	}
	if authA(makeAuthedRequest("GET", "/", "token-b")).Status != 401 {
		t.Error("authA should reject token-b")
	}
	if authB(makeAuthedRequest("GET", "/", "token-b")).Status != 200 {
		t.Error("authB should accept token-b")
	}
}

// ============================================================================
// RATELIMIT TESTS
// ============================================================================

func TestRateLimit_UnderLimit(t *testing.T) {
	handler := RateLimit(5)(okHandler)
	r := makeRequest("GET", "/")

	for i := 0; i < 5; i++ {
		resp := handler(r)
		if resp.Status != 200 {
			t.Errorf("request %d: expected 200, got %d", i+1, resp.Status)
		}
	}
}

func TestRateLimit_ExceedsLimit(t *testing.T) {
	handler := RateLimit(3)(okHandler)
	r := makeRequest("GET", "/")

	for i := 0; i < 3; i++ {
		handler(r) // consume the quota
	}

	resp := handler(r) // 4th request — should be rate limited
	if resp.Status != 429 {
		t.Errorf("expected 429 after exceeding limit, got %d", resp.Status)
	}
}

func TestRateLimit_PerClientIP(t *testing.T) {
	handler := RateLimit(2)(okHandler)

	r1 := makeRequest("GET", "/")
	r1.ClientIP = "10.0.0.1"

	r2 := makeRequest("GET", "/")
	r2.ClientIP = "10.0.0.2"

	// Exhaust r1's quota
	handler(r1)
	handler(r1)
	resp := handler(r1)
	if resp.Status != 429 {
		t.Errorf("r1: expected 429 after limit, got %d", resp.Status)
	}

	// r2 should still have its own quota
	resp = handler(r2)
	if resp.Status != 200 {
		t.Errorf("r2: expected 200 (independent quota), got %d", resp.Status)
	}
}

func TestRateLimit_IndependentInstances(t *testing.T) {
	// Two RateLimit instances should have independent state.
	rl1 := RateLimit(1)(okHandler)
	rl2 := RateLimit(1)(okHandler)

	r := makeRequest("GET", "/")

	rl1(r) // consume rl1's quota
	if rl1(r).Status != 429 {
		t.Error("rl1: expected 429")
	}
	// rl2's quota should be independent
	if rl2(r).Status != 200 {
		t.Error("rl2: expected 200 — independent instance")
	}
}

// ============================================================================
// RECOVER TESTS
// ============================================================================

func TestRecover_NoPanic(t *testing.T) {
	handler := Recover(okHandler)
	resp := handler(makeRequest("GET", "/"))
	if resp.Status != 200 {
		t.Errorf("expected 200 with no panic, got %d", resp.Status)
	}
}

func TestRecover_CatchesPanic(t *testing.T) {
	handler := Recover(panicHandler)
	// This should NOT cause the test to panic
	resp := handler(makeRequest("GET", "/"))
	if resp.Status != 500 {
		t.Errorf("expected 500 after catching panic, got %d", resp.Status)
	}
}

func TestRecover_DoesNotAffectNormalHandlers(t *testing.T) {
	handler := Recover(echoHandler)
	resp := handler(makeRequest("DELETE", "/resource/42"))
	if resp.Status != 200 || !strings.Contains(resp.Body, "/resource/42") {
		t.Errorf("unexpected response: %+v", resp)
	}
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

func TestIntegration_FullChain(t *testing.T) {
	chain := Chain(
		Recover,
		Logger,
		Auth("my-token"),
		RateLimit(10),
	)(echoHandler)

	// Valid request
	r := makeAuthedRequest("GET", "/api/users", "my-token")
	r.ClientIP = "10.0.0.1"
	resp := chain(r)
	if resp.Status != 200 {
		t.Errorf("valid request: expected 200, got %d", resp.Status)
	}

	// Invalid auth — short-circuits before rate limiter and handler
	r2 := makeRequest("GET", "/api/users")
	r2.ClientIP = "10.0.0.2"
	resp = chain(r2)
	if resp.Status != 401 {
		t.Errorf("unauthed request: expected 401, got %d", resp.Status)
	}
}

func TestIntegration_RecoverInChain(t *testing.T) {
	chain := Chain(Recover, Auth("token"))(panicHandler)

	r := makeAuthedRequest("GET", "/", "token")
	resp := chain(r)
	if resp.Status != 500 {
		t.Errorf("panic should be recovered: expected 500, got %d", resp.Status)
	}
}

func TestIntegration_RateLimitAfterAuth(t *testing.T) {
	const maxPerMin = 3
	chain := Chain(Auth("secret"), RateLimit(maxPerMin))(okHandler)

	r := makeAuthedRequest("GET", "/", "secret")
	r.ClientIP = "1.2.3.4"

	for i := 0; i < maxPerMin; i++ {
		if resp := chain(r); resp.Status != 200 {
			t.Errorf("request %d: expected 200, got %d", i+1, resp.Status)
		}
	}

	// Next request should be rate limited
	if resp := chain(r); resp.Status != 429 {
		t.Errorf("expected 429 after %d requests, got %d", maxPerMin, resp.Status)
	}

	// But wrong token should still get 401, not 429
	badAuth := makeAuthedRequest("GET", "/", "wrong")
	badAuth.ClientIP = "1.2.3.4"
	if resp := chain(badAuth); resp.Status != 401 {
		t.Errorf("wrong token after rate limit: expected 401, got %d", resp.Status)
	}
}

// TestRateLimit_WindowReset verifies that counts reset after the minute window.
// This test is slow (waits ~2 minutes) and is skipped in short mode.
func TestRateLimit_WindowReset(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping window reset test in short mode (takes ~2 min)")
	}

	handler := RateLimit(2)(okHandler)
	r := makeRequest("GET", "/")

	// Exhaust quota
	handler(r)
	handler(r)
	if handler(r).Status != 429 {
		t.Fatal("expected 429 after exhausting quota")
	}

	// Wait for the next minute window
	now := time.Now()
	nextMinute := now.Truncate(time.Minute).Add(time.Minute)
	time.Sleep(time.Until(nextMinute) + 100*time.Millisecond)

	// Should be allowed again
	if resp := handler(r); resp.Status != 200 {
		t.Errorf("after window reset: expected 200, got %d", resp.Status)
	}
}
