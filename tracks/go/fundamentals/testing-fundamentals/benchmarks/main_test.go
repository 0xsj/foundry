// Package benchmarks demonstrates Go benchmark patterns.
//
// Key patterns shown:
//  1. Basic benchmark (b.N loop)
//  2. b.ResetTimer() to exclude setup
//  3. b.ReportAllocs() for explicit allocation tracking
//  4. Comparing two implementations side-by-side
//  5. b.SetBytes() for throughput measurements
//  6. Sub-benchmarks with b.Run
//
// Run:              go test -bench=. -benchmem ./benchmarks/
// Single benchmark: go test -bench=BenchmarkBuildMessage -benchmem ./benchmarks/
// Longer run:       go test -bench=. -benchmem -benchtime=5s ./benchmarks/
package benchmarks

import (
	"strings"
	"testing"
)

// ============================================================================
// Correctness tests (always run, even without -bench)
// ============================================================================

func TestBuildMessage(t *testing.T) {
	want := "[ERROR] payments: transaction failed (code=500)"
	if got := BuildMessageConcat("ERROR", "payments", "transaction failed", 500); got != want {
		t.Errorf("Concat: got %q, want %q", got, want)
	}
	if got := BuildMessageBuilder("ERROR", "payments", "transaction failed", 500); got != want {
		t.Errorf("Builder: got %q, want %q", got, want)
	}
}

// ============================================================================
// Example 1: Basic benchmark
//
// b.N is set automatically by the benchmark runner.
// Start small, increases until timing stabilizes.
// NEVER set b.N yourself.
// ============================================================================

func BenchmarkBuildMessageConcat(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = BuildMessageConcat("ERROR", "payments", "transaction failed", 500)
	}
}

func BenchmarkBuildMessageBuilder(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = BuildMessageBuilder("ERROR", "payments", "transaction failed", 500)
	}
}

// ============================================================================
// Example 2: b.ResetTimer() to exclude setup cost
//
// Without ResetTimer(), the time to build `inputs` is included in the result.
// That would make you think the function is slower than it is.
// ============================================================================

func BenchmarkCountWords(b *testing.B) {
	// Setup: build a large string (we don't want to measure this)
	words := strings.Repeat("hello world ", 1000)

	b.ResetTimer() // start timing HERE — after setup

	for i := 0; i < b.N; i++ {
		_ = CountWords(words)
	}
}

// ============================================================================
// Example 3: b.ReportAllocs() — force allocation reporting
//
// Equivalent to -benchmem for this specific benchmark.
// Use when you're explicitly optimizing for zero allocations.
// ============================================================================

func BenchmarkBuildMessageBuilderAllocs(b *testing.B) {
	b.ReportAllocs() // show allocs even without -benchmem flag

	for i := 0; i < b.N; i++ {
		_ = BuildMessageBuilder("ERROR", "payments", "transaction failed", 500)
	}
}

// ============================================================================
// Example 4: b.SetBytes() for throughput (MB/s) measurements
//
// When processing variable-length data, bytes/op and MB/s are more meaningful
// than ns/op alone. SetBytes tells the runner how many bytes each op processes.
// ============================================================================

func BenchmarkContainsAny(b *testing.B) {
	text := strings.Repeat("the quick brown fox jumps over the lazy dog ", 100)
	keywords := []string{"error", "fail", "panic", "fatal", "critical"}

	b.SetBytes(int64(len(text))) // report MB/s based on input size
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_ = ContainsAny(text, keywords)
	}
}

// ============================================================================
// Example 5: Sub-benchmarks with b.Run
//
// Like t.Run for tests — creates named sub-benchmarks, selectable with -bench.
// Great for comparing the same function with different inputs.
//
// Run just these: go test -bench=BenchmarkBuildMessageInput -benchmem ./benchmarks/
// ============================================================================

func BenchmarkBuildMessageInput(b *testing.B) {
	cases := []struct {
		name    string
		level   string
		service string
		msg     string
		code    int
	}{
		{"short", "INFO", "api", "ok", 200},
		{"medium", "ERROR", "payments", "transaction failed", 500},
		{"long", "WARNING", "notification-service", "rate limit exceeded for user, retrying", 429},
	}

	for _, tc := range cases {
		tc := tc // Go < 1.22 loop variable capture
		b.Run(tc.name, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				_ = BuildMessageBuilder(tc.level, tc.service, tc.msg, tc.code)
			}
		})
	}
}

// ============================================================================
// Example 6: StopTimer/StartTimer for per-iteration cleanup
//
// Use when you need to reset state between iterations but don't want to count
// the reset time. More granular than ResetTimer (which only works once).
// ============================================================================

func BenchmarkWithReset(b *testing.B) {
	// Imagine we're benchmarking something that mutates a shared buffer
	buf := make([]byte, 0, 1024)

	for i := 0; i < b.N; i++ {
		b.StopTimer()
		buf = buf[:0] // reset buffer — don't count this
		b.StartTimer()

		// Only time the actual work
		buf = append(buf, []byte("log entry: transaction processed")...)
		_ = buf
	}
}
