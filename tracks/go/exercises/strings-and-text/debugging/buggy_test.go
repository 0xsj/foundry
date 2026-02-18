package textutil

import (
	"strings"
	"testing"
)

// ============================================================================
// Bug 1: CapitalizeDisplayName
// ============================================================================

func TestCapitalizeDisplayName_ASCII(t *testing.T) {
	tests := []struct{ input, want string }{
		{"alice", "Alice"},
		{"bob", "Bob"},
		{"charlie123", "Charlie123"},
	}
	for _, tc := range tests {
		got := CapitalizeDisplayName(tc.input)
		if got != tc.want {
			t.Errorf("CapitalizeDisplayName(%q) = %q, want %q", tc.input, got, tc.want)
		}
	}
}

func TestCapitalizeDisplayName_Unicode(t *testing.T) {
	// These will FAIL with the buggy implementation.
	// The first character is multibyte; byte indexing corrupts it.
	tests := []struct{ input, want string }{
		{"ångström", "Ångström"},
		{"josé", "José"},
		{"αlpha", "Αlpha"},   // Greek lowercase alpha → uppercase
		{"über", "Über"},
	}
	for _, tc := range tests {
		got := CapitalizeDisplayName(tc.input)
		if got != tc.want {
			t.Errorf("CapitalizeDisplayName(%q) = %q, want %q", tc.input, got, tc.want)
		}
	}
}

func TestCapitalizeDisplayName_Empty(t *testing.T) {
	if got := CapitalizeDisplayName(""); got != "" {
		t.Errorf("CapitalizeDisplayName(\"\") = %q, want %q", got, "")
	}
}

func TestCapitalizeDisplayName_AlreadyCapital(t *testing.T) {
	if got := CapitalizeDisplayName("Alice"); got != "Alice" {
		t.Errorf("CapitalizeDisplayName(\"Alice\") = %q, want %q", got, "Alice")
	}
}

// ============================================================================
// Bug 2: BuildMetricLabels
// ============================================================================

func TestBuildMetricLabels_Basic(t *testing.T) {
	labels := map[string]string{
		"service": "payment-svc",
		"env":     "production",
	}
	got := BuildMetricLabels(labels)
	// Output must contain both key=value pairs separated by comma
	if !strings.Contains(got, "service=payment-svc") {
		t.Errorf("result %q missing service=payment-svc", got)
	}
	if !strings.Contains(got, "env=production") {
		t.Errorf("result %q missing env=production", got)
	}
	if strings.Count(got, ",") != len(labels)-1 {
		t.Errorf("result %q should have %d commas, format: k=v,k=v", got, len(labels)-1)
	}
}

func TestBuildMetricLabels_Performance(t *testing.T) {
	// Build 10,000 labels. With + concatenation this takes several seconds.
	// With strings.Builder it completes in milliseconds.
	labels := make(map[string]string, 10000)
	for i := 0; i < 10000; i++ {
		labels[strings.Repeat("k", i%20+1)] = strings.Repeat("v", i%30+1)
	}

	// This test is intentionally not using t.Skip or timing assertions —
	// the O(n²) behavior makes it visibly slow in CI. Once fixed with
	// strings.Builder, it should complete essentially instantly.
	result := BuildMetricLabels(labels)
	if result == "" {
		t.Error("BuildMetricLabels returned empty string for 10,000 labels")
	}
}

func TestBuildMetricLabels_Empty(t *testing.T) {
	got := BuildMetricLabels(map[string]string{})
	if got != "" {
		t.Errorf("BuildMetricLabels(empty) = %q, want %q", got, "")
	}
}

// ============================================================================
// Bug 3: FilterErrorLines
// ============================================================================

func TestFilterErrorLines_Basic(t *testing.T) {
	lines := []string{
		"2024-01-15T14:30:00Z INFO  svc: all good",
		"2024-01-15T14:30:01Z ERROR svc: something failed",
		"2024-01-15T14:30:02Z WARN  svc: elevated rate",
		"2024-01-15T14:30:03Z ERROR svc: another failure",
		"2024-01-15T14:30:04Z DEBUG svc: reconnecting",
	}
	got := FilterErrorLines(lines)
	if len(got) != 2 {
		t.Errorf("FilterErrorLines: got %d results, want 2", len(got))
	}
}

func TestFilterErrorLines_WordBoundary(t *testing.T) {
	// The pattern uses \bERROR\b — should NOT match "NO_ERROR" or "ERRORS"
	lines := []string{
		"status: NO_ERROR",
		"2024-01-15T14:30:01Z ERROR svc: real error",
		"ERRORS: 0",
	}
	got := FilterErrorLines(lines)
	if len(got) != 1 {
		t.Errorf("FilterErrorLines: got %v, want only the real ERROR line", got)
	}
}

func TestFilterErrorLines_Performance(t *testing.T) {
	// 50,000 lines — compiling the regexp inside the loop causes severe slowdown.
	// After the fix (package-level var), this should be fast.
	const n = 50000
	lines := make([]string, n)
	for i := 0; i < n; i++ {
		if i%100 == 0 {
			lines[i] = "2024-01-15T14:30:00Z ERROR svc: failure"
		} else {
			lines[i] = "2024-01-15T14:30:00Z INFO  svc: ok"
		}
	}
	got := FilterErrorLines(lines)
	if len(got) != n/100 {
		t.Errorf("FilterErrorLines: got %d results, want %d", len(got), n/100)
	}
}

// ============================================================================
// Bug 4: NormalizeHeaderValue
// ============================================================================

func TestNormalizeHeaderValue_Basic(t *testing.T) {
	tests := []struct {
		input []byte
		want  string
	}{
		{[]byte("  application/json  "), "application/json"},
		{[]byte("\t  Bearer tok123 \n"), "Bearer tok123"},
		{[]byte("no-whitespace"), "no-whitespace"},
		{[]byte(""), ""},
	}
	for _, tc := range tests {
		got := NormalizeHeaderValue(tc.input)
		if got != tc.want {
			t.Errorf("NormalizeHeaderValue(%q) = %q, want %q", tc.input, got, tc.want)
		}
	}
}

func BenchmarkNormalizeHeaderValue(b *testing.B) {
	raw := []byte("  application/json; charset=utf-8  ")
	for i := 0; i < b.N; i++ {
		_ = NormalizeHeaderValue(raw)
	}
}
