package strategy

import (
	"fmt"
	"sync"
	"testing"
	"time"
)

// --- Token Bucket Tests ---

func TestTokenBucket_AllowsBurstsUpToCapacity(t *testing.T) {
	limiter := NewTokenBucketLimiter(5, 1) // capacity=5, refill=1/sec

	allowed := 0
	for i := 0; i < 5; i++ {
		if limiter.Allow("client-1") {
			allowed++
		}
	}

	if allowed != 5 {
		t.Errorf("expected 5 requests allowed (burst capacity), got %d", allowed)
	}

	// 6th request should be denied (bucket empty)
	if limiter.Allow("client-1") {
		t.Error("expected 6th request to be denied (bucket empty)")
	}
}

func TestTokenBucket_RefillsOverTime(t *testing.T) {
	limiter := NewTokenBucketLimiter(3, 10) // capacity=3, refill=10/sec

	// Drain the bucket
	for i := 0; i < 3; i++ {
		limiter.Allow("client-1")
	}

	// Should be empty
	if limiter.Allow("client-1") {
		t.Error("expected bucket to be empty after draining")
	}

	// Wait for refill (at 10/sec, 150ms should give us at least 1 token)
	time.Sleep(150 * time.Millisecond)

	if !limiter.Allow("client-1") {
		t.Error("expected request to be allowed after refill period")
	}
}

func TestTokenBucket_IsolatesKeys(t *testing.T) {
	limiter := NewTokenBucketLimiter(2, 1)

	// Drain client-1
	limiter.Allow("client-1")
	limiter.Allow("client-1")

	// client-2 should still have tokens
	if !limiter.Allow("client-2") {
		t.Error("expected client-2 to have tokens (independent bucket)")
	}
}

func TestTokenBucket_DoesNotExceedCapacity(t *testing.T) {
	limiter := NewTokenBucketLimiter(3, 100) // high refill rate

	// Wait for potential over-refill
	time.Sleep(100 * time.Millisecond)

	allowed := 0
	for i := 0; i < 10; i++ {
		if limiter.Allow("client-1") {
			allowed++
		}
	}

	if allowed > 3 {
		t.Errorf("expected at most 3 requests (capacity), got %d", allowed)
	}
}

func TestTokenBucket_Name(t *testing.T) {
	limiter := NewTokenBucketLimiter(5, 1)
	if limiter.Name() != "token-bucket" {
		t.Errorf("expected name %q, got %q", "token-bucket", limiter.Name())
	}
}

// --- Fixed Window Tests ---

func TestFixedWindow_AllowsUpToMax(t *testing.T) {
	limiter := NewFixedWindowLimiter(3, 1*time.Second)

	allowed := 0
	for i := 0; i < 3; i++ {
		if limiter.Allow("client-1") {
			allowed++
		}
	}

	if allowed != 3 {
		t.Errorf("expected 3 requests allowed, got %d", allowed)
	}

	if limiter.Allow("client-1") {
		t.Error("expected 4th request to be denied")
	}
}

func TestFixedWindow_ResetsAfterWindow(t *testing.T) {
	limiter := NewFixedWindowLimiter(2, 100*time.Millisecond)

	// Use up the window
	limiter.Allow("client-1")
	limiter.Allow("client-1")

	if limiter.Allow("client-1") {
		t.Error("expected request to be denied in current window")
	}

	// Wait for next window
	time.Sleep(150 * time.Millisecond)

	if !limiter.Allow("client-1") {
		t.Error("expected request to be allowed in new window")
	}
}

func TestFixedWindow_IsolatesKeys(t *testing.T) {
	limiter := NewFixedWindowLimiter(1, 1*time.Second)

	limiter.Allow("client-1") // uses up client-1's window

	if limiter.Allow("client-1") {
		t.Error("expected client-1 to be denied")
	}

	if !limiter.Allow("client-2") {
		t.Error("expected client-2 to be allowed (separate counter)")
	}
}

func TestFixedWindow_Name(t *testing.T) {
	limiter := NewFixedWindowLimiter(5, 1*time.Second)
	if limiter.Name() != "fixed-window" {
		t.Errorf("expected name %q, got %q", "fixed-window", limiter.Name())
	}
}

// --- Sliding Window Tests ---

func TestSlidingWindow_AllowsUpToMax(t *testing.T) {
	limiter := NewSlidingWindowLimiter(3, 1*time.Second)

	allowed := 0
	for i := 0; i < 3; i++ {
		if limiter.Allow("client-1") {
			allowed++
		}
	}

	if allowed != 3 {
		t.Errorf("expected 3 requests allowed, got %d", allowed)
	}

	if limiter.Allow("client-1") {
		t.Error("expected 4th request to be denied")
	}
}

func TestSlidingWindow_SlidesOverTime(t *testing.T) {
	limiter := NewSlidingWindowLimiter(2, 200*time.Millisecond)

	// T=0: two requests
	limiter.Allow("client-1")
	limiter.Allow("client-1")

	// T=0: should be denied
	if limiter.Allow("client-1") {
		t.Error("expected denial at capacity")
	}

	// T=250ms: first two requests should have expired
	time.Sleep(250 * time.Millisecond)

	if !limiter.Allow("client-1") {
		t.Error("expected request to be allowed after old timestamps expired")
	}
}

func TestSlidingWindow_IsolatesKeys(t *testing.T) {
	limiter := NewSlidingWindowLimiter(1, 1*time.Second)

	limiter.Allow("client-1")

	if limiter.Allow("client-1") {
		t.Error("expected client-1 to be denied")
	}

	if !limiter.Allow("client-2") {
		t.Error("expected client-2 to be allowed")
	}
}

func TestSlidingWindow_Name(t *testing.T) {
	limiter := NewSlidingWindowLimiter(5, 1*time.Second)
	if limiter.Name() != "sliding-window" {
		t.Errorf("expected name %q, got %q", "sliding-window", limiter.Name())
	}
}

// --- Gateway Tests ---

func TestGateway_RoutesToCorrectLimiter(t *testing.T) {
	tokenBucket := NewTokenBucketLimiter(100, 100)
	fixedWindow := NewFixedWindowLimiter(100, 1*time.Second)
	slidingWindow := NewSlidingWindowLimiter(100, 1*time.Second)

	gw := NewGateway(fixedWindow, map[string]RateLimiter{
		"free":       fixedWindow,
		"paid":       tokenBucket,
		"enterprise": slidingWindow,
	})

	tests := []struct {
		tier         string
		expectedName string
	}{
		{"free", "fixed-window"},
		{"paid", "token-bucket"},
		{"enterprise", "sliding-window"},
	}

	for _, tt := range tests {
		allowed, name := gw.HandleRequest("client-1", tt.tier)
		if !allowed {
			t.Errorf("tier %q: expected request to be allowed", tt.tier)
		}
		if name != tt.expectedName {
			t.Errorf("tier %q: expected limiter %q, got %q", tt.tier, tt.expectedName, name)
		}
	}
}

func TestGateway_FallsBackForUnknownTier(t *testing.T) {
	fallback := NewFixedWindowLimiter(10, 1*time.Second)

	gw := NewGateway(fallback, map[string]RateLimiter{
		"paid": NewTokenBucketLimiter(100, 100),
	})

	allowed, name := gw.HandleRequest("client-1", "unknown-tier")
	if !allowed {
		t.Error("expected fallback to allow request")
	}
	if name != "fixed-window" {
		t.Errorf("expected fallback limiter name %q, got %q", "fixed-window", name)
	}
}

func TestGateway_EmptyTierUsesFallback(t *testing.T) {
	fallback := NewFixedWindowLimiter(10, 1*time.Second)

	gw := NewGateway(fallback, map[string]RateLimiter{})

	allowed, name := gw.HandleRequest("client-1", "")
	if !allowed {
		t.Error("expected fallback to allow request")
	}
	if name != "fixed-window" {
		t.Errorf("expected fallback limiter name %q, got %q", "fixed-window", name)
	}
}

// --- Concurrency Tests ---

func TestTokenBucket_ConcurrentAccess(t *testing.T) {
	limiter := NewTokenBucketLimiter(100, 1000) // 100 burst, 1000/sec refill

	var wg sync.WaitGroup
	allowed := make([]int, 10)

	for g := 0; g < 10; g++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			for i := 0; i < 50; i++ {
				if limiter.Allow(fmt.Sprintf("client-%d", id%3)) {
					allowed[id]++
				}
			}
		}(g)
	}

	wg.Wait()

	total := 0
	for _, a := range allowed {
		total += a
	}

	// With 3 clients, 100 tokens each = 300 max tokens.
	// Some will refill during the test, but total should be reasonable.
	if total == 0 {
		t.Error("expected some requests to be allowed under concurrent access")
	}
}

func TestFixedWindow_ConcurrentAccess(t *testing.T) {
	limiter := NewFixedWindowLimiter(50, 1*time.Second)

	var wg sync.WaitGroup
	var mu sync.Mutex
	totalAllowed := 0

	for g := 0; g < 10; g++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			localAllowed := 0
			for i := 0; i < 20; i++ {
				if limiter.Allow("shared-client") {
					localAllowed++
				}
			}
			mu.Lock()
			totalAllowed += localAllowed
			mu.Unlock()
		}()
	}

	wg.Wait()

	// 10 goroutines * 20 requests = 200 total attempts for one client
	// Fixed window allows 50, so we should see exactly 50
	if totalAllowed != 50 {
		t.Errorf("expected exactly 50 allowed (fixed window limit), got %d", totalAllowed)
	}
}

func TestSlidingWindow_ConcurrentAccess(t *testing.T) {
	limiter := NewSlidingWindowLimiter(50, 1*time.Second)

	var wg sync.WaitGroup
	var mu sync.Mutex
	totalAllowed := 0

	for g := 0; g < 10; g++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			localAllowed := 0
			for i := 0; i < 20; i++ {
				if limiter.Allow("shared-client") {
					localAllowed++
				}
			}
			mu.Lock()
			totalAllowed += localAllowed
			mu.Unlock()
		}()
	}

	wg.Wait()

	if totalAllowed != 50 {
		t.Errorf("expected exactly 50 allowed (sliding window limit), got %d", totalAllowed)
	}
}

func TestGateway_ConcurrentAccess(t *testing.T) {
	gw := NewGateway(
		NewFixedWindowLimiter(1000, 1*time.Second),
		map[string]RateLimiter{
			"free":       NewFixedWindowLimiter(10, 1*time.Second),
			"paid":       NewTokenBucketLimiter(50, 100),
			"enterprise": NewSlidingWindowLimiter(100, 1*time.Second),
		},
	)

	var wg sync.WaitGroup
	tiers := []string{"free", "paid", "enterprise", "unknown"}

	for g := 0; g < 20; g++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			tier := tiers[id%len(tiers)]
			clientID := fmt.Sprintf("client-%d", id)
			for i := 0; i < 100; i++ {
				gw.HandleRequest(clientID, tier)
			}
		}(g)
	}

	wg.Wait()
	// Test passes if no data races (run with -race flag)
}

// --- Interface Compliance Tests ---

func TestRateLimiterInterface(t *testing.T) {
	// Verify all types satisfy the interface at test time as well
	var _ RateLimiter = (*TokenBucketLimiter)(nil)
	var _ RateLimiter = (*FixedWindowLimiter)(nil)
	var _ RateLimiter = (*SlidingWindowLimiter)(nil)
}

// --- Strategy Swapping Tests ---

func TestGateway_DifferentStrategiesSameClient(t *testing.T) {
	// Same client ID but different tier should use different limiters
	gw := NewGateway(
		NewFixedWindowLimiter(1, 1*time.Second),
		map[string]RateLimiter{
			"free": NewFixedWindowLimiter(1, 1*time.Second),
			"paid": NewTokenBucketLimiter(10, 10),
		},
	)

	// client-1 as free tier: should be limited to 1
	allowed1, name1 := gw.HandleRequest("client-1", "free")
	if !allowed1 || name1 != "fixed-window" {
		t.Errorf("expected allowed=true, name=fixed-window; got allowed=%v, name=%s", allowed1, name1)
	}

	allowed2, _ := gw.HandleRequest("client-1", "free")
	if allowed2 {
		t.Error("expected client-1 free tier second request to be denied")
	}

	// client-1 as paid tier: should still have capacity (different limiter instance)
	allowed3, name3 := gw.HandleRequest("client-1", "paid")
	if !allowed3 || name3 != "token-bucket" {
		t.Errorf("expected allowed=true, name=token-bucket; got allowed=%v, name=%s", allowed3, name3)
	}
}
