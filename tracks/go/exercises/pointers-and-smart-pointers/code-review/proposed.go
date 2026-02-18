package resources

import (
	"bytes"
	"fmt"
	"sync"
	"time"
)

// ResourceKind identifies what type of resource is being tracked.
type ResourceKind int

const (
	KindServer    ResourceKind = iota // 0
	KindContainer                     // 1
	KindConfig                        // 2
)

// ResourceMeta holds metadata about a single tracked resource.
// Small struct: ID (string header: 16B), Kind (8B), AllocatedAt (24B),
// ExpiresAt (24B), Tags (slice header: 24B) = ~96 bytes total.
type ResourceMeta struct {
	ID          string
	Kind        ResourceKind
	AllocatedAt time.Time
	ExpiresAt   *time.Time // nil = no expiry
	Tags        *[]string  // ISSUE 1: pointer to slice — unnecessary
}

// ResourceTracker tracks active resource allocations.
type ResourceTracker struct {
	*sync.Mutex                         // ISSUE 2: embedded pointer to sync.Mutex — should embed by value
	resources   map[string]*ResourceMeta
	pool        *sync.Pool               // ISSUE 3: sync.Pool misuse — stores *ResourceMeta, never resets
	stats       *TrackerStats            // ISSUE 4: pointer to small stats struct — unnecessary
}

// TrackerStats holds aggregate statistics for the tracker.
// Small struct: 3 int fields = 24 bytes.
type TrackerStats struct {
	TotalAllocated int
	TotalReleased  int
	CurrentActive  int
}

// NewResourceTracker creates an initialized ResourceTracker.
func NewResourceTracker() *ResourceTracker {
	return &ResourceTracker{
		Mutex:     &sync.Mutex{},          // ISSUE 2: &sync.Mutex{} — embed by value instead
		resources: make(map[string]*ResourceMeta),
		pool: &sync.Pool{
			New: func() any {
				return &ResourceMeta{}
			},
		},
		stats: &TrackerStats{},
	}
}

// Allocate registers a new resource with the tracker.
func (t *ResourceTracker) Allocate(id string, kind ResourceKind, tags []string) error {
	t.Lock()
	defer t.Unlock()

	if _, exists := t.resources[id]; exists {
		return fmt.Errorf("resource %q is already allocated", id)
	}

	meta := t.pool.Get().(*ResourceMeta)
	// ISSUE 3: missing reset — pool.Get() may return a dirty *ResourceMeta
	// from a previous Release() call. Fields from the previous allocation
	// are still set.
	meta.ID = id
	meta.Kind = kind
	meta.AllocatedAt = time.Now()
	meta.Tags = &tags  // ISSUE 1: &tags captures the parameter slice — caller's slice, not a copy

	t.resources[id] = meta
	t.stats.TotalAllocated++
	t.stats.CurrentActive++
	return nil
}

// Release removes a resource from the tracker and returns it to the pool.
func (t *ResourceTracker) Release(id string) error {
	t.Lock()
	defer t.Unlock()

	meta, ok := t.resources[id]
	if !ok {
		return fmt.Errorf("resource %q not found", id)
	}

	delete(t.resources, id)
	t.pool.Put(meta) // returns to pool without resetting — next Get returns dirty object
	t.stats.TotalReleased++
	t.stats.CurrentActive--
	return nil
}

// SetExpiry sets an expiration time for a resource.
func (t *ResourceTracker) SetExpiry(id string, expiry time.Time) error {
	t.Lock()
	defer t.Unlock()

	meta, ok := t.resources[id]
	if !ok {
		return fmt.Errorf("resource %q not found", id)
	}
	meta.ExpiresAt = &expiry  // this is fine — takes address of local copy of the parameter
	return nil
}

// GetTags returns the tags for a resource.
// ISSUE 1 consequence: caller receives a pointer to the original parameter slice.
func (t *ResourceTracker) GetTags(id string) (*[]string, bool) { // ISSUE 1: returns *[]string
	t.Lock()
	defer t.Unlock()

	meta, ok := t.resources[id]
	if !ok {
		return nil, false
	}
	return meta.Tags, true
}

// IsExpired reports whether a resource has passed its expiry time.
func (t *ResourceTracker) IsExpired(id string) bool {
	t.Lock()
	defer t.Unlock()

	meta, ok := t.resources[id]
	if !ok {
		return false
	}
	if meta.ExpiresAt == nil {
		return false
	}
	return time.Now().After(*meta.ExpiresAt)
}

// Stats returns a copy of the current statistics.
// ISSUE 4 consequence: dereferencing *TrackerStats to return a copy is correct,
// but the pointer field itself is unnecessary — TrackerStats is 24 bytes.
func (t *ResourceTracker) Stats() TrackerStats {
	t.Lock()
	defer t.Unlock()

	return *t.stats
}

// PurgeExpired removes all expired resources from the tracker.
func (t *ResourceTracker) PurgeExpired() int {
	t.Lock()
	defer t.Unlock()

	count := 0
	for id, meta := range t.resources {
		if meta.ExpiresAt != nil && time.Now().After(*meta.ExpiresAt) {
			delete(t.resources, id)
			t.pool.Put(meta)
			count++
		}
	}
	t.stats.TotalReleased += count
	t.stats.CurrentActive -= count
	return count
}

// ============================================================================
// ReportBuilder: demonstrates sync.Pool for short-lived buffer reuse
// (This part is correct — shown for contrast with the pool misuse above)
// ============================================================================

var reportBufPool = sync.Pool{
	New: func() any { return &bytes.Buffer{} },
}

// BuildReport assembles a text report of current allocations.
func (t *ResourceTracker) BuildReport() string {
	t.Lock()
	defer t.Unlock()

	buf := reportBufPool.Get().(*bytes.Buffer)
	buf.Reset() // correct: reset before use
	defer func() {
		reportBufPool.Put(buf) // correct: return after use
	}()

	fmt.Fprintf(buf, "Active resources: %d\n", len(t.resources))
	for id, meta := range t.resources {
		fmt.Fprintf(buf, "  %s: kind=%d allocated=%s\n",
			id, meta.Kind, meta.AllocatedAt.Format(time.RFC3339))
	}
	return buf.String()
}
