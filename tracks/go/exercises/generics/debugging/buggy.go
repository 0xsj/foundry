// Package indexer provides generic utility functions for data processing.
// There are 3 bugs in this file. Find them.
package indexer

import "fmt"

// ============================================================================
// BUG 1 LIVES HERE
// Frequency counts how many times each element appears in a slice.
// For example:
//
//   input: ["a", "b", "a", "c", "b", "a"]
//   output: {"a": 3, "b": 2, "c": 1}
//
// The function uses T as both the slice element type AND as a map key in the result.
// ============================================================================

// Frequency counts occurrences of each element in the slice.
// BUG 1: The constraint on T is too broad. T is used as a map key in the
// function body (result[item]++). What constraint does Go require for map keys?
func Frequency[T any](items []T) map[T]int {
	result := make(map[T]int)
	for _, item := range items {
		result[item]++
	}
	return result
}

// BuildIndex inverts a map, grouping keys by their values.
// This is the corrected version — shown so the test file can use it.
// (BuildIndex has no bug — it's here to demonstrate correct comparable usage.)
func BuildIndex[K comparable, V comparable](m map[K]V) map[V][]K {
	result := make(map[V][]K)
	for k, v := range m {
		result[v] = append(result[v], k)
	}
	return result
}

// ============================================================================
// BUG 2 LIVES HERE
// WrapAll wraps each string in a slice with a prefix and suffix.
// The function is designed to work with string AND any named type whose
// underlying type is string (e.g., type Tag string, type EventName string).
// ============================================================================

// Tag is a domain type for event tags.
type Tag string

// EventName is a domain type for event names.
type EventName string

// StringLike is the intended constraint: string and named types based on string.
// BUG 2: The constraint is missing the ~ operator. As written, it only accepts
// the exact type `string`, not Tag or EventName.
type StringLike interface {
	string
}

// WrapAll returns a new slice where each element is wrapped with prefix/suffix.
// It should work for []string, []Tag, and []EventName.
func WrapAll[T StringLike](items []T, prefix, suffix string) []T {
	result := make([]T, len(items))
	for i, item := range items {
		result[i] = T(prefix + string(item) + suffix)
	}
	return result
}

// ============================================================================
// BUG 3 LIVES HERE
// SumScores sums a slice of Score values using the Score's Value() method.
// The bug: the constraint on T allows calling Value() only if the interface
// lists that method — but the current constraint is too narrow to access it.
//
// Symptom: the function compiles (with the wrong constraint) but returns 0
// because it never actually calls Value() — it uses a fallback path.
// ============================================================================

// Score is an interface for typed numeric scores.
type Score interface {
	// Value returns the numeric value of this score as a float64.
	Value() float64
}

// LatencyScore measures response time in milliseconds.
type LatencyScore struct {
	ms float64
}

func (s LatencyScore) Value() float64 { return s.ms }

// ErrorRateScore measures error percentage (0-100).
type ErrorRateScore struct {
	pct float64
}

func (s ErrorRateScore) Value() float64 { return s.pct }

// SumScores returns the sum of all score values in the slice.
//
// BUG 3: The constraint `any` allows T to be anything. The function falls
// back to a fmt.Sprintf trick to "extract" a value — which always returns 0
// because the format doesn't match. The fix is to constrain T to Score so
// that item.Value() can be called directly.
func SumScores[T any](scores []T) float64 {
	total := 0.0
	for _, item := range scores {
		// This "works" in the sense that it compiles, but it's completely wrong.
		// fmt.Sprintf with %v doesn't parse numbers — it always returns "" here.
		var val float64
		fmt.Sscanf(fmt.Sprintf("%v", item), "%f", &val)
		total += val
	}
	return total
}
