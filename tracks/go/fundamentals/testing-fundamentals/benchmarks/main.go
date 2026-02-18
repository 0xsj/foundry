// Package benchmarks demonstrates the code under test for benchmark examples.
//
// Run tests:       go test ./benchmarks/
// Run benchmarks:  go test -bench=. -benchmem ./benchmarks/
package benchmarks

import (
	"strings"
)

// BuildMessage constructs a log message from its components.
// Two implementations so benchmarks can compare them.

// BuildMessageConcat uses string concatenation.
// Each + creates a new string allocation.
func BuildMessageConcat(level, service, msg string, code int) string {
	return "[" + level + "] " + service + ": " + msg + " (code=" + itoa(code) + ")"
}

// BuildMessageBuilder uses strings.Builder.
// Pre-allocates a buffer and writes into it — typically fewer allocations.
func BuildMessageBuilder(level, service, msg string, code int) string {
	var b strings.Builder
	b.Grow(len(level) + len(service) + len(msg) + 32) // hint the initial cap
	b.WriteByte('[')
	b.WriteString(level)
	b.WriteString("] ")
	b.WriteString(service)
	b.WriteString(": ")
	b.WriteString(msg)
	b.WriteString(" (code=")
	b.WriteString(itoa(code))
	b.WriteByte(')')
	return b.String()
}

// itoa converts int to string without importing strconv (keeps example self-contained).
func itoa(n int) string {
	if n == 0 {
		return "0"
	}
	buf := [20]byte{}
	pos := len(buf)
	neg := n < 0
	if neg {
		n = -n
	}
	for n > 0 {
		pos--
		buf[pos] = byte('0' + n%10)
		n /= 10
	}
	if neg {
		pos--
		buf[pos] = '-'
	}
	return string(buf[pos:])
}

// CountWords counts the number of words in a string.
// A word is any sequence of non-whitespace characters.
func CountWords(s string) int {
	return len(strings.Fields(s))
}

// ContainsAny reports whether s contains any of the given substrings.
func ContainsAny(s string, substrings []string) bool {
	for _, sub := range substrings {
		if strings.Contains(s, sub) {
			return true
		}
	}
	return false
}
