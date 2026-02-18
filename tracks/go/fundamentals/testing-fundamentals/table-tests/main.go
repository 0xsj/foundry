// Package tabletests demonstrates the code under test for table-driven test examples.
//
// Run the tests: go test ./table-tests/
package tabletests

import (
	"fmt"
	"strconv"
	"strings"
)

// ParseAmount converts a string to a non-negative integer amount in cents.
// It accepts strings like "42", "0", or "1000".
// Returns an error for negative values, non-numeric strings, or empty input.
func ParseAmount(s string) (int, error) {
	s = strings.TrimSpace(s)
	if s == "" {
		return 0, fmt.Errorf("parseAmount: empty input")
	}
	n, err := strconv.Atoi(s)
	if err != nil {
		return 0, fmt.Errorf("parseAmount: %w", err)
	}
	if n < 0 {
		return 0, fmt.Errorf("parseAmount: negative amount %d", n)
	}
	return n, nil
}

// FormatCurrency formats an integer cent amount as a currency string.
// 100 -> "$1.00", 50 -> "$0.50", 1050 -> "$10.50"
func FormatCurrency(cents int) string {
	dollars := cents / 100
	remainder := cents % 100
	return fmt.Sprintf("$%d.%02d", dollars, remainder)
}

// ApplyDiscount reduces an amount by the given percentage (0-100).
// Returns an error if the discount is out of range.
func ApplyDiscount(amountCents, discountPercent int) (int, error) {
	if discountPercent < 0 || discountPercent > 100 {
		return 0, fmt.Errorf("applyDiscount: discount %d%% out of range [0,100]", discountPercent)
	}
	discounted := amountCents * (100 - discountPercent) / 100
	return discounted, nil
}
