package debugging

import (
	"testing"
	"time"
)

// --- Bug 1: Interface Satisfaction ---

func TestTokenBucket_SatisfiesInterface(t *testing.T) {
	// This test verifies that *TokenBucketLimiter satisfies RateLimiter.
	// Currently, it does NOT because Allow has the wrong signature.
	// Fix the Allow method to match the interface.
	var limiter RateLimiter = NewTokenBucketLimiter(5, 1)

	if !limiter.Allow("client-1") {
		t.Error("expected first request to be allowed")
	}

	if limiter.Name() != "token-bucket" {
		t.Errorf("expected name %q, got %q", "token-bucket", limiter.Name())
	}
}

func TestTokenBucket_BasicFunctionality(t *testing.T) {
	limiter := NewTokenBucketLimiter(3, 1)

	// Should allow 3 requests (capacity)
	for i := 0; i < 3; i++ {
		if !limiter.Allow("client-1") {
			t.Errorf("expected request %d to be allowed", i+1)
		}
	}

	// 4th should be denied
	if limiter.Allow("client-1") {
		t.Error("expected 4th request to be denied")
	}
}

// --- Bug 2: Nil Fallback Panic ---

func TestGateway_FallbackDoesNotPanic(t *testing.T) {
	// Create a gateway with a fallback and one tier.
	// Requesting an unknown tier should use the fallback, not panic.
	fallback := NewFixedWindowLimiter(10, 1*time.Second)

	gw := NewGateway(fallback, map[string]RateLimiter{
		"paid": NewFixedWindowLimiter(100, 1*time.Second),
	})

	// This should NOT panic — it should use the fallback limiter
	allowed, name := gw.HandleRequest("client-1", "unknown-tier")
	if !allowed {
		t.Error("expected fallback to allow request")
	}
	if name != "fixed-window" {
		t.Errorf("expected fallback limiter name %q, got %q", "fixed-window", name)
	}
}

func TestGateway_FallbackIsSet(t *testing.T) {
	fallback := NewFixedWindowLimiter(1, 1*time.Second)
	gw := NewGateway(fallback, map[string]RateLimiter{})

	// With no tiers configured, everything should go to fallback
	allowed, _ := gw.HandleRequest("client-1", "any-tier")
	if !allowed {
		t.Error("expected first request to fallback to be allowed")
	}

	// Second request should be denied (fallback limit is 1)
	allowed, _ = gw.HandleRequest("client-1", "any-tier")
	if allowed {
		t.Error("expected second request to be denied by fallback limiter")
	}
}

// --- Bug 3: Wrong Tier Mapping ---

func TestGateway_CorrectTierMapping(t *testing.T) {
	// Each tier should map to the correct limiter.
	// The bug swaps tier assignments.
	freeLimiter := NewFixedWindowLimiter(5, 1*time.Second)
	paidLimiter := NewFixedWindowLimiter(50, 1*time.Second)
	enterpriseLimiter := NewSlidingWindowLimiter(500, 1*time.Second)

	gw := NewGateway(freeLimiter, map[string]RateLimiter{
		"free":       freeLimiter,
		"paid":       paidLimiter,
		"enterprise": enterpriseLimiter,
	})

	// Verify each tier gets the right limiter by checking the name
	tests := []struct {
		tier     string
		wantName string
	}{
		{"free", "fixed-window"},
		{"enterprise", "sliding-window"},
	}

	for _, tt := range tests {
		_, name := gw.HandleRequest("test-client", tt.tier)
		if name != tt.wantName {
			t.Errorf("tier %q: expected limiter %q, got %q (tier mapping may be swapped)",
				tt.tier, tt.wantName, name)
		}
	}
}

func TestGateway_EnterpriseTierGetsHigherLimit(t *testing.T) {
	// Enterprise should allow 100 requests; free should allow 5.
	// If tiers are swapped, enterprise gets 5 and free gets 100.
	gw := NewGateway(
		NewFixedWindowLimiter(1, 1*time.Second),
		map[string]RateLimiter{
			"free":       NewFixedWindowLimiter(5, 1*time.Second),
			"enterprise": NewFixedWindowLimiter(100, 1*time.Second),
		},
	)

	// Enterprise client should be able to make 100 requests
	enterpriseAllowed := 0
	for i := 0; i < 100; i++ {
		allowed, _ := gw.HandleRequest("ent-client", "enterprise")
		if allowed {
			enterpriseAllowed++
		}
	}

	if enterpriseAllowed != 100 {
		t.Errorf("enterprise tier: expected 100 allowed, got %d (tier mapping may be wrong)",
			enterpriseAllowed)
	}

	// Free client should be limited to 5
	freeAllowed := 0
	for i := 0; i < 20; i++ {
		allowed, _ := gw.HandleRequest("free-client", "free")
		if allowed {
			freeAllowed++
		}
	}

	if freeAllowed != 5 {
		t.Errorf("free tier: expected 5 allowed, got %d (tier mapping may be wrong)",
			freeAllowed)
	}
}

// --- Bug 4: Sliding Window Off-by-One ---

func TestSlidingWindow_ExactLimit(t *testing.T) {
	// With maxReqs=3, exactly 3 requests should be allowed, then denied.
	limiter := NewSlidingWindowLimiter(3, 1*time.Second)

	allowed := 0
	for i := 0; i < 10; i++ {
		if limiter.Allow("client-1") {
			allowed++
		}
	}

	if allowed != 3 {
		t.Errorf("expected exactly 3 requests allowed (maxReqs=3), got %d (off-by-one in cleanup?)",
			allowed)
	}
}

func TestSlidingWindow_DoesNotLeakExtraRequests(t *testing.T) {
	// The off-by-one bug causes the limiter to allow more requests
	// than configured because it drops a valid timestamp on each call.
	limiter := NewSlidingWindowLimiter(5, 500*time.Millisecond)

	// Make 5 requests (should all be allowed)
	for i := 0; i < 5; i++ {
		if !limiter.Allow("client-1") {
			t.Errorf("expected request %d to be allowed", i+1)
		}
	}

	// 6th request should be denied immediately (no time has passed)
	if limiter.Allow("client-1") {
		t.Error("expected 6th request to be denied (limit is 5)")
	}

	// 7th request should also be denied
	if limiter.Allow("client-1") {
		t.Error("expected 7th request to be denied (limit is 5)")
	}
}

func TestSlidingWindow_StrictCountAfterExpiry(t *testing.T) {
	limiter := NewSlidingWindowLimiter(2, 100*time.Millisecond)

	// Fill up
	limiter.Allow("client-1")
	limiter.Allow("client-1")

	// Should be denied
	if limiter.Allow("client-1") {
		t.Error("expected denial at capacity")
	}

	// Wait for expiry
	time.Sleep(150 * time.Millisecond)

	// Should allow exactly 2 more
	count := 0
	for i := 0; i < 5; i++ {
		if limiter.Allow("client-1") {
			count++
		}
	}

	if count != 2 {
		t.Errorf("after expiry, expected exactly 2 allowed, got %d", count)
	}
}
