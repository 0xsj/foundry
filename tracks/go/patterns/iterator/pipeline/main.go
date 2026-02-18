// Iterator Pipeline: Lazy Data Processing
//
// Demonstrates building a data processing pipeline using Go 1.23
// range-over-function iterators. Reads structured data, filters,
// transforms, and aggregates -- all lazily, no intermediate slices.
//
// Scenario: Process server access log entries. Count requests by
// status code, find slow requests, and compute percentile latencies --
// all in a single pass through the data using composable iterators.
//
// Run: go run ./pipeline/

package main

import (
	"fmt"
	"iter"
	"math"
	"slices"
	"sort"
	"strings"
	"time"
)

// --- Domain Types ---

// AccessLogEntry represents a parsed HTTP access log entry.
type AccessLogEntry struct {
	Timestamp  time.Time
	Method     string
	Path       string
	StatusCode int
	LatencyMs  float64
	BytesSent  int
	UserAgent  string
}

// --- Iterator Combinators ---

func filter[T any](pred func(T) bool, seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		for v := range seq {
			if pred(v) {
				if !yield(v) {
					return
				}
			}
		}
	}
}

func mapIter[T, U any](f func(T) U, seq iter.Seq[T]) iter.Seq[U] {
	return func(yield func(U) bool) {
		for v := range seq {
			if !yield(f(v)) {
				return
			}
		}
	}
}

func take[T any](n int, seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		i := 0
		for v := range seq {
			if i >= n {
				return
			}
			if !yield(v) {
				return
			}
			i++
		}
	}
}

// flatMap maps each element to an iterator and flattens the results.
func flatMap[T, U any](f func(T) iter.Seq[U], seq iter.Seq[T]) iter.Seq[U] {
	return func(yield func(U) bool) {
		for v := range seq {
			for u := range f(v) {
				if !yield(u) {
					return
				}
			}
		}
	}
}

// tee sends each element to a side-effect function while passing it through.
// Useful for counting, logging, or accumulating stats during pipeline execution.
func tee[T any](sideEffect func(T), seq iter.Seq[T]) iter.Seq[T] {
	return func(yield func(T) bool) {
		for v := range seq {
			sideEffect(v)
			if !yield(v) {
				return
			}
		}
	}
}

// groupBy collects elements by key. This is a terminal operation (must consume all elements).
func groupBy[T any, K comparable](keyFn func(T) K, seq iter.Seq[T]) map[K][]T {
	groups := make(map[K][]T)
	for v := range seq {
		key := keyFn(v)
		groups[key] = append(groups[key], v)
	}
	return groups
}

// collect gathers all elements from an iterator into a slice.
func collect[T any](seq iter.Seq[T]) []T {
	return slices.Collect(seq)
}

// --- Data Source: Simulated Log Lines ---

// rawLogLines simulates reading lines from a log file.
// In production, this would wrap a bufio.Scanner over an io.Reader.
func rawLogLines() iter.Seq[string] {
	lines := []string{
		"2024-03-15T10:00:01Z GET /api/v1/users 200 12.3ms 4521 Mozilla/5.0",
		"2024-03-15T10:00:01Z POST /api/v1/users 201 45.7ms 312 curl/8.4.0",
		"2024-03-15T10:00:02Z GET /api/v1/users/123 200 8.1ms 892 Mozilla/5.0",
		"2024-03-15T10:00:02Z GET /api/v1/health 200 1.2ms 28 kube-probe/1.28",
		"2024-03-15T10:00:03Z DELETE /api/v1/users/456 404 3.4ms 142 curl/8.4.0",
		"2024-03-15T10:00:03Z GET /api/v1/users 200 250.8ms 45210 Mozilla/5.0",
		"2024-03-15T10:00:04Z POST /api/v1/login 200 89.2ms 1024 Mozilla/5.0",
		"2024-03-15T10:00:04Z GET /api/v1/users/789 500 1502.3ms 0 Mozilla/5.0",
		"2024-03-15T10:00:05Z PUT /api/v1/users/123 200 34.5ms 892 curl/8.4.0",
		"2024-03-15T10:00:05Z GET /api/v1/metrics 200 5.6ms 15234 prometheus/2.48",
		"2024-03-15T10:00:06Z POST /api/v1/webhooks 202 156.7ms 64 internal-service/1.0",
		"2024-03-15T10:00:06Z GET /api/v1/users 200 11.9ms 4521 Mozilla/5.0",
		"2024-03-15T10:00:07Z GET /api/v1/users/999 404 2.8ms 142 Mozilla/5.0",
		"2024-03-15T10:00:07Z POST /api/v1/login 401 15.3ms 89 Mozilla/5.0",
		"2024-03-15T10:00:08Z GET /api/v1/users 200 310.4ms 45210 Mozilla/5.0",
		"2024-03-15T10:00:08Z GET /static/app.js 200 0.8ms 125000 Mozilla/5.0",
		"2024-03-15T10:00:09Z GET /api/v1/users/123/orders 200 78.9ms 8234 Mozilla/5.0",
		"2024-03-15T10:00:09Z POST /api/v1/orders 201 234.5ms 512 Mozilla/5.0",
		"2024-03-15T10:00:10Z GET /api/v1/health 200 1.1ms 28 kube-probe/1.28",
		"2024-03-15T10:00:10Z GET /api/v1/users 500 5023.1ms 0 Mozilla/5.0",
	}
	return slices.Values(lines)
}

// parseLogEntry parses a raw log line into a structured entry.
// Returns the entry and whether parsing succeeded.
func parseLogEntry(line string) (AccessLogEntry, bool) {
	// Format: timestamp method path status latency bytes useragent
	parts := strings.SplitN(line, " ", 7)
	if len(parts) < 7 {
		return AccessLogEntry{}, false
	}

	ts, err := time.Parse(time.RFC3339, parts[0])
	if err != nil {
		return AccessLogEntry{}, false
	}

	var status int
	fmt.Sscanf(parts[3], "%d", &status)

	var latency float64
	latencyStr := strings.TrimSuffix(parts[4], "ms")
	fmt.Sscanf(latencyStr, "%f", &latency)

	var bytes int
	fmt.Sscanf(parts[5], "%d", &bytes)

	return AccessLogEntry{
		Timestamp:  ts,
		Method:     parts[1],
		Path:       parts[2],
		StatusCode: status,
		LatencyMs:  latency,
		BytesSent:  bytes,
		UserAgent:  parts[6],
	}, true
}

// --- Pipeline Stages ---

// parsedEntries converts raw log lines into structured entries, skipping unparseable lines.
func parsedEntries(lines iter.Seq[string]) iter.Seq[AccessLogEntry] {
	return func(yield func(AccessLogEntry) bool) {
		for line := range lines {
			if entry, ok := parseLogEntry(line); ok {
				if !yield(entry) {
					return
				}
			}
		}
	}
}

// apiOnly filters to only API requests (excludes static files, health checks).
func apiOnly(entries iter.Seq[AccessLogEntry]) iter.Seq[AccessLogEntry] {
	return filter(func(e AccessLogEntry) bool {
		return strings.HasPrefix(e.Path, "/api/v1/") &&
			e.Path != "/api/v1/health" &&
			e.Path != "/api/v1/metrics"
	}, entries)
}

// slowRequests filters to requests above a latency threshold.
func slowRequests(thresholdMs float64, entries iter.Seq[AccessLogEntry]) iter.Seq[AccessLogEntry] {
	return filter(func(e AccessLogEntry) bool {
		return e.LatencyMs > thresholdMs
	}, entries)
}

// errorRequests filters to 4xx and 5xx responses.
func errorRequests(entries iter.Seq[AccessLogEntry]) iter.Seq[AccessLogEntry] {
	return filter(func(e AccessLogEntry) bool {
		return e.StatusCode >= 400
	}, entries)
}

// --- Aggregation ---

// StatusStats tracks request counts and latencies per status code.
type StatusStats struct {
	Count      int
	TotalMs    float64
	MaxMs      float64
	TotalBytes int
}

func computeStatusStats(entries iter.Seq[AccessLogEntry]) map[int]*StatusStats {
	stats := make(map[int]*StatusStats)
	for e := range entries {
		s, ok := stats[e.StatusCode]
		if !ok {
			s = &StatusStats{}
			stats[e.StatusCode] = s
		}
		s.Count++
		s.TotalMs += e.LatencyMs
		if e.LatencyMs > s.MaxMs {
			s.MaxMs = e.LatencyMs
		}
		s.TotalBytes += e.BytesSent
	}
	return stats
}

// percentile computes the p-th percentile of latencies (0-100).
func percentile(latencies []float64, p float64) float64 {
	if len(latencies) == 0 {
		return 0
	}
	sort.Float64s(latencies)
	rank := p / 100.0 * float64(len(latencies)-1)
	lower := int(math.Floor(rank))
	upper := int(math.Ceil(rank))
	if lower == upper {
		return latencies[lower]
	}
	frac := rank - float64(lower)
	return latencies[lower]*(1-frac) + latencies[upper]*frac
}

func main() {
	fmt.Println("=== Access Log Processing Pipeline ===")
	fmt.Println()

	// --- Pipeline 1: Status code breakdown ---
	fmt.Println("--- Status Code Breakdown (API requests only) ---")
	allEntries := parsedEntries(rawLogLines())
	apiEntries := apiOnly(allEntries)
	stats := computeStatusStats(apiEntries)

	// Sort status codes for consistent output
	codes := make([]int, 0, len(stats))
	for code := range stats {
		codes = append(codes, code)
	}
	sort.Ints(codes)

	totalRequests := 0
	for _, code := range codes {
		s := stats[code]
		totalRequests += s.Count
		avgMs := s.TotalMs / float64(s.Count)
		fmt.Printf("  %d: %d requests, avg %.1fms, max %.1fms, %d bytes total\n",
			code, s.Count, avgMs, s.MaxMs, s.TotalBytes)
	}
	fmt.Printf("  total API requests: %d\n", totalRequests)
	fmt.Println()

	// --- Pipeline 2: Slow request report ---
	fmt.Println("--- Slow Requests (>100ms) ---")
	slowEntries := slowRequests(100.0, apiOnly(parsedEntries(rawLogLines())))
	for e := range slowEntries {
		fmt.Printf("  [%.0fms] %s %s -> %d\n", e.LatencyMs, e.Method, e.Path, e.StatusCode)
	}
	fmt.Println()

	// --- Pipeline 3: Error details ---
	fmt.Println("--- Error Responses (4xx/5xx) ---")
	errors := errorRequests(apiOnly(parsedEntries(rawLogLines())))
	for e := range errors {
		fmt.Printf("  %s %s %s -> %d (%.1fms)\n",
			e.Timestamp.Format("15:04:05"), e.Method, e.Path, e.StatusCode, e.LatencyMs)
	}
	fmt.Println()

	// --- Pipeline 4: Latency percentiles using tee ---
	fmt.Println("--- Latency Percentiles (API requests) ---")
	var latencies []float64
	counted := tee(func(e AccessLogEntry) {
		latencies = append(latencies, e.LatencyMs)
	}, apiOnly(parsedEntries(rawLogLines())))

	// Consume the iterator to populate latencies
	entryCount := 0
	for range counted {
		entryCount++
	}

	fmt.Printf("  requests analyzed: %d\n", entryCount)
	fmt.Printf("  p50: %.1fms\n", percentile(latencies, 50))
	fmt.Printf("  p90: %.1fms\n", percentile(latencies, 90))
	fmt.Printf("  p95: %.1fms\n", percentile(latencies, 95))
	fmt.Printf("  p99: %.1fms\n", percentile(latencies, 99))
	fmt.Println()

	// --- Pipeline 5: Top N slowest with take ---
	fmt.Println("--- Top 3 Slowest API Requests ---")
	// Collect, sort, then iterate top N
	allAPI := collect(apiOnly(parsedEntries(rawLogLines())))
	sort.Slice(allAPI, func(i, j int) bool {
		return allAPI[i].LatencyMs > allAPI[j].LatencyMs
	})
	for e := range take(3, slices.Values(allAPI)) {
		fmt.Printf("  %.0fms  %s %s -> %d\n", e.LatencyMs, e.Method, e.Path, e.StatusCode)
	}
	fmt.Println()

	// --- Pipeline 6: Group by path ---
	fmt.Println("--- Requests Per Path ---")
	groups := groupBy(func(e AccessLogEntry) string { return e.Path },
		apiOnly(parsedEntries(rawLogLines())))

	// Sort paths for consistent output
	paths := make([]string, 0, len(groups))
	for p := range groups {
		paths = append(paths, p)
	}
	sort.Strings(paths)

	for _, path := range paths {
		entries := groups[path]
		var totalLatency float64
		for _, e := range entries {
			totalLatency += e.LatencyMs
		}
		avg := totalLatency / float64(len(entries))
		fmt.Printf("  %-35s %d reqs, avg %.1fms\n", path, len(entries), avg)
	}
	fmt.Println()

	// --- Pipeline 7: FlatMap demonstration ---
	fmt.Println("--- FlatMap: Expand Each Entry Into Tags ---")
	tagGenerator := func(e AccessLogEntry) iter.Seq[string] {
		return func(yield func(string) bool) {
			// Generate tags for this entry
			yield(fmt.Sprintf("method:%s", e.Method))
			yield(fmt.Sprintf("status:%d", e.StatusCode))
			if e.LatencyMs > 100 {
				yield("latency:slow")
			} else {
				yield("latency:fast")
			}
		}
	}
	tags := flatMap(tagGenerator, take(3, apiOnly(parsedEntries(rawLogLines()))))
	fmt.Print("  tags from first 3 API entries: ")
	for tag := range tags {
		fmt.Printf("[%s] ", tag)
	}
	fmt.Println()

	fmt.Println()
	fmt.Println("Key takeaways:")
	fmt.Println("- Each pipeline stage is a small, reusable function")
	fmt.Println("- No intermediate slices between filter/map stages")
	fmt.Println("- The same raw data is re-iterated for each pipeline (like re-reading a file)")
	fmt.Println("- Terminal operations (groupBy, percentile) consume the iterator")
	fmt.Println("- tee enables side-effects (counting, accumulating) without breaking the pipeline")
	fmt.Println("- flatMap enables one-to-many transformations")
}
