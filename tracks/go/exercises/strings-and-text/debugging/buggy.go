// Package textutil provides text-processing utilities for the log aggregation pipeline.
package textutil

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

// ============================================================================
// Bug 1: String index gives a byte, not a rune
//
// CapitalizeDisplayName takes a user's display name and returns it with the
// first character uppercased. It should work correctly for all UTF-8 names.
//
// Examples:
//   "alice"    → "Alice"
//   "ångström" → "Ångström"
//   "josé"     → "José"
// ============================================================================

func CapitalizeDisplayName(name string) string {
	if name == "" {
		return name
	}
	// BUG: s[0] is a byte, not a character.
	// For a name starting with a multibyte character (e.g., 'å' = 2 bytes),
	// s[0] gives the first byte of the UTF-8 encoding — not the character.
	// Uppercasing a raw byte produces a nonsense result.
	first := unicode.ToUpper(rune(name[0]))
	return string(first) + name[1:]
}

// ============================================================================
// Bug 2: String concatenation in a loop — O(n²) allocations
//
// BuildMetricLabels takes a map of metric label key-value pairs and returns
// them formatted as a sorted, comma-separated string:
//   "env=production,region=us-east-1,service=payment-svc"
//
// The function is correct but unacceptably slow for large label sets.
// ============================================================================

func BuildMetricLabels(labels map[string]string) string {
	// Collect and sort the keys for deterministic output
	keys := make([]string, 0, len(labels))
	for k := range labels {
		keys = append(keys, k)
	}
	// sort.Strings not used here to keep imports minimal for the bug demo
	// (the test doesn't require sorted output — it requires fast output)

	// BUG: string concatenation with + inside a loop.
	// Each iteration allocates a new string and copies all previous content.
	// For 10,000 labels, this is ~50M bytes of copying.
	result := ""
	for i, k := range keys {
		if i > 0 {
			result += ","
		}
		result += k + "=" + labels[k]
	}
	return result
}

// ============================================================================
// Bug 3: regexp.MustCompile called inside a hot loop
//
// FilterErrorLines takes a slice of log lines and returns only those that
// contain an ERROR-level entry. Uses a regexp for accurate word-boundary
// matching (to avoid matching "NO_ERROR" or "ERRORS").
//
// Functionally correct, but causes serious performance degradation at scale.
// ============================================================================

func FilterErrorLines(lines []string) []string {
	result := make([]string, 0)
	for _, line := range lines {
		// BUG: regexp.MustCompile is called on every iteration.
		// It builds a finite automaton from scratch each time.
		// For 50,000 log lines, this compiles the same pattern 50,000 times.
		re := regexp.MustCompile(`\bERROR\b`)
		if re.MatchString(line) {
			result = append(result, line)
		}
	}
	return result
}

// ============================================================================
// Bug 4: Unnecessary []byte → string conversion allocates a copy
//
// NormalizeHeaderValue takes a raw HTTP header value as []byte, trims
// whitespace, and returns it as a string.
//
// The function produces correct output but makes an extra allocation in the
// hot path by converting []byte to string before calling strings.TrimSpace,
// then converting back — when the bytes package has a direct equivalent.
// ============================================================================

func NormalizeHeaderValue(raw []byte) string {
	// BUG: string(raw) allocates a new string (copies all bytes).
	// Then strings.TrimSpace returns a sub-slice of the allocated string.
	// Then string(...) on a []byte conversion at return allocates again.
	//
	// The bytes package has bytes.TrimSpace which operates on []byte directly
	// and returns a sub-slice — zero allocation.
	trimmed := strings.TrimSpace(string(raw))
	return fmt.Sprintf("%s", trimmed)
}
