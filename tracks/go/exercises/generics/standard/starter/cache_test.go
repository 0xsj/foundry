package cache

import (
	"sync"
	"testing"
	"time"
)

// ============================================================================
// Basic operations
// ============================================================================

func TestSetAndGet(t *testing.T) {
	c := New[string, int](Options{})

	c.Set("requests", 42)
	v, ok := c.Get("requests")
	if !ok {
		t.Fatal("expected key to exist")
	}
	if v != 42 {
		t.Fatalf("got %d, want 42", v)
	}
}

func TestGetMissing(t *testing.T) {
	c := New[string, string](Options{})

	v, ok := c.Get("nonexistent")
	if ok {
		t.Fatal("expected ok=false for missing key")
	}
	if v != "" {
		t.Fatalf("expected zero value, got %q", v)
	}
}

func TestSetOverwrite(t *testing.T) {
	c := New[string, int](Options{})

	c.Set("count", 1)
	c.Set("count", 99)

	v, ok := c.Get("count")
	if !ok {
		t.Fatal("expected key to exist after overwrite")
	}
	if v != 99 {
		t.Fatalf("got %d, want 99", v)
	}
}

func TestDelete(t *testing.T) {
	c := New[string, bool](Options{})
	c.Set("feature_x", true)
	c.Delete("feature_x")

	_, ok := c.Get("feature_x")
	if ok {
		t.Fatal("expected key to be absent after Delete")
	}
}

func TestDeleteMissing(t *testing.T) {
	c := New[string, bool](Options{})
	// Should not panic
	c.Delete("nonexistent")
}

func TestLen(t *testing.T) {
	c := New[int, string](Options{})
	if c.Len() != 0 {
		t.Fatalf("empty cache: got Len=%d, want 0", c.Len())
	}

	c.Set(1, "one")
	c.Set(2, "two")
	c.Set(3, "three")

	if c.Len() != 3 {
		t.Fatalf("got Len=%d, want 3", c.Len())
	}

	c.Delete(2)
	if c.Len() != 2 {
		t.Fatalf("after delete: got Len=%d, want 2", c.Len())
	}
}

func TestFlush(t *testing.T) {
	c := New[string, int](Options{})
	c.Set("a", 1)
	c.Set("b", 2)
	c.Flush()

	if c.Len() != 0 {
		t.Fatalf("after Flush: got Len=%d, want 0", c.Len())
	}
	_, ok := c.Get("a")
	if ok {
		t.Fatal("expected key 'a' absent after Flush")
	}
}

// ============================================================================
// TTL expiration
// ============================================================================

func TestTTLExpiration(t *testing.T) {
	c := New[string, string](Options{TTL: 50 * time.Millisecond})

	c.Set("session", "user-123")

	// Entry should exist immediately
	v, ok := c.Get("session")
	if !ok || v != "user-123" {
		t.Fatalf("expected hit immediately after Set, got ok=%v v=%q", ok, v)
	}

	// Wait for TTL to elapse
	time.Sleep(60 * time.Millisecond)

	_, ok = c.Get("session")
	if ok {
		t.Fatal("expected entry to be expired after TTL elapsed")
	}
}

func TestTTLExpiredNotCountedInLen(t *testing.T) {
	c := New[string, int](Options{TTL: 30 * time.Millisecond})

	c.Set("a", 1)
	c.Set("b", 2)

	time.Sleep(40 * time.Millisecond)

	// Expired entries should not count toward Len
	if n := c.Len(); n != 0 {
		t.Fatalf("expected Len=0 after TTL, got %d", n)
	}
}

func TestNoTTLNeverExpires(t *testing.T) {
	c := New[string, int](Options{}) // zero TTL = no expiration

	c.Set("permanent", 1)
	time.Sleep(10 * time.Millisecond)

	_, ok := c.Get("permanent")
	if !ok {
		t.Fatal("entry with no TTL should not expire")
	}
}

// ============================================================================
// LRU eviction
// ============================================================================

func TestLRUEviction(t *testing.T) {
	c := New[string, int](Options{MaxSize: 3})

	c.Set("a", 1)
	c.Set("b", 2)
	c.Set("c", 3)

	// Cache is full. Next Set should evict the LRU entry.
	// "a" was inserted first and not accessed since — it is LRU.
	c.Set("d", 4)

	_, ok := c.Get("a")
	if ok {
		t.Fatal("expected 'a' to be evicted (LRU)")
	}

	// b, c, d should still be present
	for _, key := range []string{"b", "c", "d"} {
		if _, ok := c.Get(key); !ok {
			t.Fatalf("expected key %q to still be present", key)
		}
	}
}

func TestLRUAccessUpdatesOrder(t *testing.T) {
	c := New[string, int](Options{MaxSize: 3})

	c.Set("a", 1)
	c.Set("b", 2)
	c.Set("c", 3)

	// Access "a" — makes it the most recently used.
	// Now the LRU order (least to most recent) is: b, c, a
	c.Get("a")

	// Adding "d" should evict "b" (now the LRU), not "a"
	c.Set("d", 4)

	_, aOK := c.Get("a")
	_, bOK := c.Get("b")

	if !aOK {
		t.Fatal("'a' should still be present — it was accessed recently")
	}
	if bOK {
		t.Fatal("'b' should have been evicted — it was the LRU")
	}
}

func TestLRUSetUpdatesOrder(t *testing.T) {
	c := New[string, int](Options{MaxSize: 3})

	c.Set("a", 1)
	c.Set("b", 2)
	c.Set("c", 3)

	// Re-setting "a" should move it to MRU position.
	// LRU order after: b, c, a
	c.Set("a", 99)

	// Adding "d" should evict "b"
	c.Set("d", 4)

	v, aOK := c.Get("a")
	_, bOK := c.Get("b")

	if !aOK || v != 99 {
		t.Fatalf("'a' should still be present with updated value, got ok=%v v=%d", aOK, v)
	}
	if bOK {
		t.Fatal("'b' should have been evicted")
	}
}

func TestLenRespectsMaxSize(t *testing.T) {
	c := New[string, int](Options{MaxSize: 5})
	for i := 0; i < 10; i++ {
		c.Set(string(rune('a'+i)), i)
	}
	if n := c.Len(); n > 5 {
		t.Fatalf("Len=%d exceeds MaxSize=5", n)
	}
}

// ============================================================================
// Type safety — same logic, different K and V types
// ============================================================================

func TestTypeSafetyIntKey(t *testing.T) {
	c := New[int, string](Options{})
	c.Set(1001, "user-alice")
	c.Set(1002, "user-bob")

	v, ok := c.Get(1001)
	if !ok || v != "user-alice" {
		t.Fatalf("got ok=%v v=%q, want ok=true v=user-alice", ok, v)
	}
}

func TestTypeSafetyStructValue(t *testing.T) {
	type Session struct {
		UserID string
		Token  string
	}

	c := New[string, Session](Options{})
	c.Set("sess-abc", Session{UserID: "user-1", Token: "tok-xyz"})

	sess, ok := c.Get("sess-abc")
	if !ok {
		t.Fatal("expected session to exist")
	}
	if sess.UserID != "user-1" || sess.Token != "tok-xyz" {
		t.Fatalf("unexpected session: %+v", sess)
	}
}

// ============================================================================
// Concurrency — run with: go test -race ./...
// ============================================================================

func TestConcurrentAccess(t *testing.T) {
	c := New[int, int](Options{MaxSize: 100})

	var wg sync.WaitGroup
	for i := 0; i < 100; i++ {
		wg.Add(2)
		go func(i int) {
			defer wg.Done()
			c.Set(i, i*10)
		}(i)
		go func(i int) {
			defer wg.Done()
			c.Get(i)
		}(i)
	}
	wg.Wait()
	// No data race and no panic = pass
}

func TestConcurrentSetAndDelete(t *testing.T) {
	c := New[string, int](Options{})
	var wg sync.WaitGroup

	for i := 0; i < 50; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			key := string(rune('a' + i%26))
			c.Set(key, i)
			c.Delete(key)
		}(i)
	}
	wg.Wait()
}
