// Package ratelimiter — write your tests here.
//
// Your task: write a comprehensive test suite for RateLimiter.
//
// Required:
//   - At least one table-driven test with t.Run
//   - A StubClock test double (no time.Sleep!)
//   - Test for refill after window elapses
//   - Concurrency test (verify with go test -race ./...)
//   - At least one t.Helper() assertion helper
//   - A BenchmarkAllow with b.ResetTimer() and b.ReportAllocs()
//
// Run tests:       go test -v ./...
// Run with race:   go test -race ./...
// Run benchmarks:  go test -bench=. -benchmem ./...
package ratelimiter

import "testing"

// TODO: Define StubClock here

// TODO: Write your tests below

// Placeholder so the file compiles before you add real tests.
// Delete this when you start writing.
func TestPlaceholder(t *testing.T) {
	t.Skip("replace with real tests")
}
