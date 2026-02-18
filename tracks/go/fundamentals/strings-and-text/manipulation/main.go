// String manipulation — strings package, Builder, fmt formatting.
//
// Run with: go run ./manipulation/
package main

import (
	"fmt"
	"strings"
)

func main() {
	searchAndTest()
	splitAndJoin()
	trimAndReplace()
	builderVsConcat()
	fmtFormatting()
}

// ============================================================================
// Search and Test
// ============================================================================

func searchAndTest() {
	fmt.Println("=== Search and Test ===")

	// Imagining we're processing HTTP headers
	contentType := "application/json; charset=utf-8"

	fmt.Println("strings.Contains:", strings.Contains(contentType, "json"))
	fmt.Println("strings.HasPrefix:", strings.HasPrefix(contentType, "application/"))
	fmt.Println("strings.HasSuffix:", strings.HasSuffix(contentType, "utf-8"))
	fmt.Println("strings.Count:", strings.Count(contentType, ";"))

	// Index: find where a substring starts (-1 if missing)
	i := strings.Index(contentType, ";")
	fmt.Printf("strings.Index(\";\") = %d → mediaType=%q\n", i, contentType[:i])

	// Cut: cleaner way to split on first occurrence (Go 1.18+)
	mediaType, params, found := strings.Cut(contentType, "; ")
	fmt.Printf("strings.Cut: mediaType=%q params=%q found=%v\n", mediaType, params, found)

	// Case-insensitive comparison
	fmt.Println("EqualFold:", strings.EqualFold("Content-Type", "content-type"))

	fmt.Println()
}

// ============================================================================
// Split and Join
// ============================================================================

func splitAndJoin() {
	fmt.Println("=== Split and Join ===")

	// Parse a comma-separated tag list (common in config, webhooks, etc.)
	tagList := "error,critical,auth-service,retry"
	tags := strings.Split(tagList, ",")
	fmt.Printf("Split: %v (%d tags)\n", tags, len(tags))

	// SplitN: limit the number of parts (useful for "key=value=morevalue" patterns)
	kv := "database.host=db.prod.internal=primary"
	parts := strings.SplitN(kv, "=", 2)
	fmt.Printf("SplitN: key=%q value=%q\n", parts[0], parts[1])

	// Fields: whitespace-aware split — handles tabs, multiple spaces, newlines
	// Typical use: parsing log lines, user input, CLI arguments
	logLine := "  2024-01-15   ERROR   auth-service   connection refused  "
	fields := strings.Fields(logLine)
	fmt.Printf("Fields: %v (%d fields)\n", fields, len(fields))

	// Join: reconstruct from parts
	normalized := strings.Join(fields, " ")
	fmt.Printf("Join after Fields: %q\n", normalized)

	// Join with custom separator: building CSV, query strings, log messages
	filters := []string{"status=active", "region=us-east", "tier=premium"}
	query := strings.Join(filters, "&")
	fmt.Printf("Query string: %q\n", query)

	fmt.Println()
}

// ============================================================================
// Trim and Replace
// ============================================================================

func trimAndReplace() {
	fmt.Println("=== Trim and Replace ===")

	// TrimSpace: the most common trim — remove leading/trailing whitespace
	raw := "  \t user-agent: Mozilla/5.0 \n"
	fmt.Printf("TrimSpace: %q\n", strings.TrimSpace(raw))

	// TrimPrefix / TrimSuffix: useful for stripping protocol prefixes, file extensions
	url := "https://api.example.com/v1/users"
	stripped := strings.TrimPrefix(url, "https://")
	fmt.Printf("TrimPrefix: %q\n", stripped)

	filename := "report-2024.csv.gz"
	base := strings.TrimSuffix(filename, ".gz")
	fmt.Printf("TrimSuffix: %q\n", base)

	// Trim with cutset: remove any of the characters in the cutset from both ends
	// Useful for stripping punctuation, brackets, quotes
	raw2 := `"[error: connection refused]"`
	clean := strings.Trim(raw2, `"[]`)
	fmt.Printf("Trim cutset: %q\n", clean)

	// Replace: targeted substitution
	template := "service={{SERVICE}} region={{REGION}} env={{ENV}}"
	result := strings.ReplaceAll(template, "{{SERVICE}}", "payment-svc")
	result = strings.ReplaceAll(result, "{{REGION}}", "us-east-1")
	result = strings.ReplaceAll(result, "{{ENV}}", "production")
	fmt.Printf("ReplaceAll: %q\n", result)

	// Replace with n=-1 vs ReplaceAll
	s := "aaabbbccc"
	fmt.Printf("Replace first 2 a→X: %q\n", strings.Replace(s, "a", "X", 2))
	fmt.Printf("ReplaceAll a→X:       %q\n", strings.ReplaceAll(s, "a", "X"))

	fmt.Println()
}

// ============================================================================
// Builder vs Concatenation
// ============================================================================

func builderVsConcat() {
	fmt.Println("=== strings.Builder vs + Concatenation ===")

	headers := []struct{ key, value string }{
		{"Content-Type", "application/json"},
		{"Authorization", "Bearer tok_abc123"},
		{"X-Request-ID", "req-42"},
		{"X-Trace-ID", "trace-99"},
	}

	// BAD approach: + concatenation in a loop
	// Each iteration creates a new string — O(n²) allocations.
	// Shown here for comparison; avoid in production.
	badResult := ""
	for _, h := range headers {
		badResult += h.key + ": " + h.value + "\r\n"
	}
	fmt.Printf("concat result (first 40 chars): %q...\n", badResult[:40])

	// GOOD approach: strings.Builder
	// Maintains a []byte internally. WriteString appends bytes.
	// Single allocation at String() call.
	var b strings.Builder
	// Grow pre-allocates capacity hint — avoids internal resizing
	b.Grow(128)
	for _, h := range headers {
		b.WriteString(h.key)
		b.WriteString(": ")
		b.WriteString(h.value)
		b.WriteString("\r\n")
	}
	builderResult := b.String()
	fmt.Printf("builder result (first 40 chars): %q...\n", builderResult[:40])

	// They produce the same output
	fmt.Printf("results equal: %v\n", badResult == builderResult)

	// When to use what:
	// + or fmt.Sprintf  → 2-3 strings, not in a loop
	// strings.Builder   → loop, many appends
	// strings.Join      → you already have a []string
	fmt.Println("\nWhen to use what:")
	one := "prefix" + ":" + "value"              // fine — not in a loop
	two := fmt.Sprintf("%s=%d", "count", 42)      // fine — mixed types
	three := strings.Join([]string{"a", "b", "c"}, ", ") // fine — known slice
	fmt.Println(" +:", one)
	fmt.Println(" Sprintf:", two)
	fmt.Println(" Join:", three)

	fmt.Println()
}

// ============================================================================
// fmt Formatting
// ============================================================================

func fmtFormatting() {
	fmt.Println("=== fmt Formatting ===")

	// %v — default format
	// %+v — with struct field names
	// %#v — Go syntax (useful for debugging)
	type Config struct {
		Host string
		Port int
		TLS  bool
	}
	cfg := Config{"api.example.com", 443, true}
	fmt.Printf("%%v:  %v\n", cfg)
	fmt.Printf("%%+v: %+v\n", cfg)
	fmt.Printf("%%#v: %#v\n", cfg)
	fmt.Printf("%%T:  %T\n", cfg)

	// %s, %q — strings
	msg := "hello\tworld\n"
	fmt.Printf("%%s: %s", msg) // prints the actual tab and newline
	fmt.Printf("%%q: %q\n", msg) // escape sequences visible

	// Integer verbs
	n := 255
	fmt.Printf("\nn=%d: %%d=%d  %%b=%b  %%o=%o  %%x=%x  %%X=%X\n", n, n, n, n, n, n)

	// Float formatting
	pi := 3.14159265358979
	fmt.Printf("\npi: %%f=%f  %%.2f=%.2f  %%e=%e  %%g=%g\n", pi, pi, pi, pi)

	// Width and alignment — useful for report generation
	fmt.Println("\nReport column alignment:")
	rows := []struct {
		service string
		count   int
		rate    float64
	}{
		{"payment-svc", 12400, 0.0024},
		{"auth-svc", 88200, 0.00082},
		{"inventory", 3300, 0.0091},
	}
	fmt.Printf("%-15s %8s %10s\n", "Service", "Requests", "Error Rate")
	fmt.Printf("%-15s %8s %10s\n", strings.Repeat("-", 15), strings.Repeat("-", 8), strings.Repeat("-", 10))
	for _, row := range rows {
		fmt.Printf("%-15s %8d %9.4f%%\n", row.service, row.count, row.rate*100)
	}

	// Sprintf for building strings (not printing)
	requestID := fmt.Sprintf("req-%06d", 42) // zero-padded to 6 digits
	fmt.Printf("\nrequest ID: %q\n", requestID)

	// Fprintf: write to any io.Writer (file, HTTP response, etc.)
	var sb strings.Builder
	fmt.Fprintf(&sb, `{"service":%q,"count":%d}`, "auth-svc", 88200)
	fmt.Printf("Fprintf to Builder: %s\n", sb.String())

	fmt.Println()
}
