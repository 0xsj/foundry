package cache

import "testing"

func TestCache_SetAndGet(t *testing.T) {
	cache := NewCache()

	// This will panic: assignment to entry in nil map
	cache.Set("user:123", "Alice")

	val, ok := cache.Get("user:123")
	if !ok {
		t.Error("expected key to exist")
	}
	if val != "Alice" {
		t.Errorf("got %q, want %q", val, "Alice")
	}
}

func TestCache_GetMissing(t *testing.T) {
	cache := NewCache()

	_, ok := cache.Get("nonexistent")
	if ok {
		t.Error("expected key to not exist")
	}
}
