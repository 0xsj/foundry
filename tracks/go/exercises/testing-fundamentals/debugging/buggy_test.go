// Package webhookprocessor — test suite with 4 bugs. Find them.
//
// Symptoms:
//   - TestProcessEvent_Formats always passes even when FormatEventSummary is broken
//   - TestProcessEvent_Parallel is flaky (panics or wrong values) when run with -count=5
//   - BenchmarkProcessEvent shows inflated ns/op (includes build time)
//   - TestProcessEvent_Concurrent fails with go test -race
//
// Run tests:       go test -v ./...
// Race detector:   go test -race -count=3 ./...
// Benchmarks:      go test -bench=BenchmarkProcessEvent -benchmem ./...
package webhookprocessor

import (
	"fmt"
	"strings"
	"sync"
	"testing"
)

// ============================================================================
// BUG 1: Silent assertion — the test passes without actually checking anything
// ============================================================================

func TestProcessEvent_Formats(t *testing.T) {
	tests := []struct {
		name    string
		raw     string
		wantFmt string
	}{
		{name: "payment succeeded", raw: "payment.succeeded:evt-001:amount=100", wantFmt: "[payment.succeeded] evt-001"},
		{name: "subscription created", raw: "subscription.created:evt-002:", wantFmt: "[subscription.created] evt-002"},
		{name: "ping", raw: "webhook.ping:evt-003", wantFmt: "[webhook.ping] evt-003"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := ProcessEvent(tt.raw)
			if err != nil {
				t.Fatalf("ProcessEvent(%q): %v", tt.raw, err)
			}

			got := FormatEventSummary(event)

			// BUG 1 IS HERE
			// This compares got to itself, not to tt.wantFmt.
			// The test always passes regardless of what FormatEventSummary returns.
			if got != got {
				t.Errorf("FormatEventSummary() = %q, want %q", got, tt.wantFmt)
			}
		})
	}
}

// ============================================================================
// BUG 2: Parallel subtest captures the loop variable by reference
// ============================================================================

func TestProcessEvent_Parallel(t *testing.T) {
	tests := []struct {
		name    string
		raw     string
		wantID  string
		wantType string
	}{
		{name: "payment event", raw: "payment.succeeded:evt-001", wantID: "evt-001", wantType: "payment.succeeded"},
		{name: "subscription event", raw: "subscription.created:evt-002", wantID: "evt-002", wantType: "subscription.created"},
		{name: "ping event", raw: "webhook.ping:evt-003", wantID: "evt-003", wantType: "webhook.ping"},
	}

	for _, tt := range tests {
		// BUG 2 IS HERE
		// tt is not captured — all goroutines share the same tt variable.
		// By the time they run, the loop has finished and tt holds the last case.
		// Fix: add `tt := tt` before t.Run, or rely on Go 1.22+ loop semantics.
		t.Run(tt.name, func(t *testing.T) {
			t.Parallel()

			event, err := ProcessEvent(tt.raw)
			if err != nil {
				t.Fatalf("ProcessEvent(%q): %v", tt.raw, err)
			}
			if event.ID != tt.wantID {
				t.Errorf("ID = %q, want %q", event.ID, tt.wantID)
			}
			if event.Type != tt.wantType {
				t.Errorf("Type = %q, want %q", event.Type, tt.wantType)
			}
		})
	}
}

// ============================================================================
// BUG 3: Benchmark includes setup cost — b.ResetTimer() missing
// ============================================================================

func BenchmarkProcessEvent(b *testing.B) {
	// BUG 3 IS HERE
	// The setup below builds 1000 raw event strings.
	// This happens before the loop but after the timer starts.
	// Without b.ResetTimer(), the build time is included in the benchmark result,
	// making the ns/op artificially high.

	events := make([]string, 1000)
	for i := 0; i < 1000; i++ {
		events[i] = fmt.Sprintf("payment.succeeded:evt-%04d:amount=%d,currency=USD", i, i*100)
	}

	// Missing: b.ResetTimer()

	for i := 0; i < b.N; i++ {
		raw := events[i%len(events)]
		_, _ = ProcessEvent(raw)
	}
}

// ============================================================================
// BUG 4: Shared mutable state accessed concurrently without synchronization
//
// This test will fail under go test -race because `results` is written
// by multiple goroutines without a mutex or atomic.
// ============================================================================

func TestProcessEvent_Concurrent(t *testing.T) {
	raws := []string{
		"payment.succeeded:evt-001:amount=100",
		"payment.failed:evt-002:amount=200",
		"subscription.created:evt-003:plan=premium",
	}

	// BUG 4 IS HERE
	// results is a plain slice. Multiple goroutines append to it concurrently.
	// append is not safe for concurrent use — this is a data race.
	// Fix: use a mutex, sync.Map, or a channel to collect results.
	var results []Event

	var wg sync.WaitGroup
	for _, raw := range raws {
		raw := raw
		wg.Add(1)
		go func() {
			defer wg.Done()
			event, err := ProcessEvent(raw)
			if err != nil {
				t.Errorf("ProcessEvent(%q): %v", raw, err)
				return
			}
			results = append(results, event) // DATA RACE: concurrent unsynchronized write
		}()
	}
	wg.Wait()

	if len(results) != len(raws) {
		t.Errorf("processed %d events, want %d", len(results), len(raws))
	}

	// Verify all events have non-empty IDs
	for _, e := range results {
		if strings.TrimSpace(e.ID) == "" {
			t.Errorf("event %q has empty ID", e.Type)
		}
	}
}
