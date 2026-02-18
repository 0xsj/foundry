// Package tabletests demonstrates table-driven tests and subtests in Go.
//
// Key patterns shown:
//  1. Anonymous struct slice for test cases
//  2. t.Run for subtests (named, isolated, selectable with -run)
//  3. t.Helper() for shared assertion helpers
//  4. Parallel subtests with loop variable capture
//
// Run all tests: go test ./table-tests/
// Run verbose:   go test -v ./table-tests/
// Run one case:  go test -v -run TestParseAmount/negative_value ./table-tests/
package tabletests

import (
	"testing"
)

// ============================================================================
// Example 1: Basic table-driven test
// ============================================================================

func TestParseAmount(t *testing.T) {
	// The test table: each entry is a named case.
	// Convention: name + inputs + expected outputs.
	tests := []struct {
		name    string
		input   string
		want    int
		wantErr bool
	}{
		// Happy path
		{name: "valid zero", input: "0", want: 0},
		{name: "valid integer", input: "42", want: 42},
		{name: "valid large", input: "99999", want: 99999},
		{name: "leading whitespace", input: "  10", want: 10},
		{name: "trailing whitespace", input: "10  ", want: 10},

		// Error cases
		{name: "empty string", input: "", wantErr: true},
		{name: "whitespace only", input: "   ", wantErr: true},
		{name: "negative value", input: "-1", wantErr: true},
		{name: "non-numeric", input: "abc", wantErr: true},
		{name: "decimal", input: "1.50", wantErr: true},
		{name: "overflow", input: "99999999999999999999", wantErr: true},
	}

	for _, tt := range tests {
		// t.Run creates a subtest.
		// The name appears in failure output and is selectable with -run.
		t.Run(tt.name, func(t *testing.T) {
			got, err := ParseAmount(tt.input)

			// Check error presence matches expectation
			if (err != nil) != tt.wantErr {
				t.Errorf("ParseAmount(%q) error = %v, wantErr = %v", tt.input, err, tt.wantErr)
				return // early return — don't check value if error state is wrong
			}

			// Only check the value when no error is expected
			if !tt.wantErr && got != tt.want {
				t.Errorf("ParseAmount(%q) = %d, want %d", tt.input, got, tt.want)
			}
		})
	}
}

// ============================================================================
// Example 2: Multi-field output, shared helper with t.Helper()
// ============================================================================

func TestFormatCurrency(t *testing.T) {
	tests := []struct {
		name  string
		cents int
		want  string
	}{
		{name: "zero", cents: 0, want: "$0.00"},
		{name: "one cent", cents: 1, want: "$0.01"},
		{name: "fifty cents", cents: 50, want: "$0.50"},
		{name: "one dollar", cents: 100, want: "$1.00"},
		{name: "mixed", cents: 1050, want: "$10.50"},
		{name: "large amount", cents: 999999, want: "$9999.99"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := FormatCurrency(tt.cents)
			assertStringEqual(t, got, tt.want)
		})
	}
}

// assertStringEqual is a test helper.
//
// t.Helper() marks this as a helper function so that when it calls t.Errorf,
// the failure message points to the CALLER (the test case), not to this line.
// Try removing t.Helper() and running a failing test — you'll see the difference.
func assertStringEqual(t *testing.T, got, want string) {
	t.Helper()
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

// ============================================================================
// Example 3: Table test with multiple outputs and an error field
// ============================================================================

func TestApplyDiscount(t *testing.T) {
	tests := []struct {
		name            string
		amountCents     int
		discountPercent int
		want            int
		wantErr         bool
	}{
		{name: "zero discount", amountCents: 1000, discountPercent: 0, want: 1000},
		{name: "ten percent", amountCents: 1000, discountPercent: 10, want: 900},
		{name: "fifty percent", amountCents: 1000, discountPercent: 50, want: 500},
		{name: "full discount", amountCents: 1000, discountPercent: 100, want: 0},
		{name: "rounding truncates", amountCents: 10, discountPercent: 33, want: 6},

		// Error cases
		{name: "negative discount", amountCents: 1000, discountPercent: -1, wantErr: true},
		{name: "over 100 percent", amountCents: 1000, discountPercent: 101, wantErr: true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ApplyDiscount(tt.amountCents, tt.discountPercent)
			if (err != nil) != tt.wantErr {
				t.Errorf("ApplyDiscount(%d, %d) error = %v, wantErr = %v",
					tt.amountCents, tt.discountPercent, err, tt.wantErr)
				return
			}
			if !tt.wantErr {
				assertIntEqual(t, got, tt.want)
			}
		})
	}
}

func assertIntEqual(t *testing.T, got, want int) {
	t.Helper()
	if got != want {
		t.Errorf("got %d, want %d", got, want)
	}
}

// ============================================================================
// Example 4: Parallel subtests
//
// t.Parallel() makes subtests run concurrently. This is useful for I/O-heavy
// or slow tests. For pure computation tests it doesn't matter much.
//
// CRITICAL: Before Go 1.22, the loop variable capture `tt := tt` was required.
// After Go 1.22, each iteration gets its own variable.
// You'll see `tt := tt` in older codebases — now you know why.
// ============================================================================

func TestFormatCurrencyParallel(t *testing.T) {
	tests := []struct {
		name  string
		cents int
		want  string
	}{
		{name: "zero", cents: 0, want: "$0.00"},
		{name: "one dollar", cents: 100, want: "$1.00"},
		{name: "mixed", cents: 1050, want: "$10.50"},
	}

	for _, tt := range tests {
		// If you're on Go < 1.22, uncomment the next line:
		// tt := tt  // capture loop variable before entering goroutine

		t.Run(tt.name, func(t *testing.T) {
			t.Parallel() // this subtest runs concurrently with siblings

			got := FormatCurrency(tt.cents)
			assertStringEqual(t, got, tt.want)
		})
	}
}
