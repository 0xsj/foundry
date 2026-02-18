// Package main demonstrates generic functions, type inference, and basic constraints.
// Run with: go run ./basics/
package main

import (
	"fmt"
	"strings"
)

// ============================================================================
// GENERIC FUNCTIONS
// ============================================================================

// First returns the first element of a slice, or the zero value if empty.
// T is unconstrained — works for any element type.
func First[T any](slice []T) (T, bool) {
	if len(slice) == 0 {
		var zero T // zero value of whatever T is at instantiation time
		return zero, false
	}
	return slice[0], true
}

// Last mirrors First, returning the last element.
func Last[T any](slice []T) (T, bool) {
	if len(slice) == 0 {
		var zero T
		return zero, false
	}
	return slice[len(slice)-1], true
}

// Contains reports whether item is in slice.
// comparable is required because we need == to compare values.
func Contains[T comparable](slice []T, item T) bool {
	for _, v := range slice {
		if v == item {
			return true
		}
	}
	return false
}

// ============================================================================
// TYPE INFERENCE
// ============================================================================

// Map transforms a []T into a []U using a function.
// Two type parameters: T (input element), U (output element).
// Both are inferred from the arguments — you never need to write Map[string, int](...).
func Map[T, U any](slice []T, fn func(T) U) []U {
	result := make([]U, len(slice))
	for i, v := range slice {
		result[i] = fn(v)
	}
	return result
}

// Filter keeps elements where fn returns true.
func Filter[T any](slice []T, fn func(T) bool) []T {
	var result []T
	for _, v := range slice {
		if fn(v) {
			result = append(result, v)
		}
	}
	return result
}

// Reduce folds a slice into a single value.
// T: element type. U: accumulator type (can differ from T).
func Reduce[T, U any](slice []T, initial U, fn func(U, T) U) U {
	acc := initial
	for _, v := range slice {
		acc = fn(acc, v)
	}
	return acc
}

// ============================================================================
// CONSTRAINTS: comparable and Ordered
// ============================================================================

// Ordered is a constraint for all types that support < > <= >=.
// This is a simplified version — in practice use golang.org/x/exp/constraints.
type Ordered interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 | ~uintptr |
		~float32 | ~float64 |
		~string
}

// Max returns the larger of two values.
// Requires Ordered because we use >.
func Max[T Ordered](a, b T) T {
	if a > b {
		return a
	}
	return b
}

// Min returns the smaller of two values.
func Min[T Ordered](a, b T) T {
	if a < b {
		return a
	}
	return b
}

// Clamp returns val clamped to the range [lo, hi].
func Clamp[T Ordered](val, lo, hi T) T {
	if val < lo {
		return lo
	}
	if val > hi {
		return hi
	}
	return val
}

// MaxSlice returns the maximum element in a non-empty slice.
// Returns zero value and false for an empty slice.
func MaxSlice[T Ordered](slice []T) (T, bool) {
	if len(slice) == 0 {
		var zero T
		return zero, false
	}
	max := slice[0]
	for _, v := range slice[1:] {
		if v > max {
			max = v
		}
	}
	return max, true
}

// ============================================================================
// WHEN YOU NEED EXPLICIT TYPE ARGUMENTS
// ============================================================================

// Zero returns the zero value of any type T.
// T cannot be inferred because it doesn't appear in any argument.
func Zero[T any]() T {
	var z T
	return z
}

// Keys extracts the keys of a map into a slice.
// K must be comparable (all map keys are), V can be anything.
func Keys[K comparable, V any](m map[K]V) []K {
	keys := make([]K, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	return keys
}

func main() {
	// ---- First / Last ----
	v, ok := First([]int{10, 20, 30})
	fmt.Printf("First: %d, ok=%v\n", v, ok) // 10, true

	s, ok := Last([]string{"a", "b", "c"})
	fmt.Printf("Last: %q, ok=%v\n", s, ok) // "c", true

	_, ok = First([]float64{}) // empty slice
	fmt.Printf("First empty: ok=%v\n", ok) // false

	// ---- Contains ----
	fmt.Println(Contains([]string{"go", "rust", "zig"}, "rust"))   // true
	fmt.Println(Contains([]int{1, 2, 3}, 99))                      // false

	// ---- Map (type inference at work) ----
	// T=string, U=int — inferred from arguments, no annotation needed
	lengths := Map([]string{"hello", "world", "generics"}, func(s string) int {
		return len(s)
	})
	fmt.Println("Lengths:", lengths) // [5 5 8]

	upper := Map([]string{"go", "rust"}, strings.ToUpper)
	fmt.Println("Upper:", upper) // [GO RUST]

	// ---- Filter ----
	evens := Filter([]int{1, 2, 3, 4, 5, 6}, func(n int) bool {
		return n%2 == 0
	})
	fmt.Println("Evens:", evens) // [2 4 6]

	// ---- Reduce ----
	sum := Reduce([]int{1, 2, 3, 4, 5}, 0, func(acc, n int) int {
		return acc + n
	})
	fmt.Println("Sum:", sum) // 15

	// Reduce with a different accumulator type: collect even numbers as strings
	evenStrs := Reduce([]int{1, 2, 3, 4, 5}, []string{}, func(acc []string, n int) []string {
		if n%2 == 0 {
			return append(acc, fmt.Sprintf("%d", n))
		}
		return acc
	})
	fmt.Println("Even strings:", evenStrs) // [2 4]

	// ---- Max / Min / Clamp ----
	fmt.Println("Max(3, 7):", Max(3, 7))              // 7
	fmt.Println("Max(3.14, 2.71):", Max(3.14, 2.71))  // 3.14
	fmt.Println("Max(\"a\", \"z\"):", Max("a", "z"))  // z

	fmt.Println("Clamp(15, 0, 10):", Clamp(15, 0, 10)) // 10
	fmt.Println("Clamp(-5, 0, 10):", Clamp(-5, 0, 10)) // 0
	fmt.Println("Clamp(5, 0, 10):", Clamp(5, 0, 10))   // 5

	m, _ := MaxSlice([]float64{3.14, 1.41, 2.71, 1.73})
	fmt.Println("MaxSlice:", m) // 3.14

	// ---- Zero — must specify type explicitly (no args to infer from) ----
	fmt.Println("Zero[int]:", Zero[int]())       // 0
	fmt.Println("Zero[string]:", Zero[string]()) // ""
	fmt.Println("Zero[bool]:", Zero[bool]())     // false

	// ---- Keys ----
	config := map[string]int{"host": 0, "port": 1, "timeout": 2}
	k := Keys(config)
	fmt.Println("Keys count:", len(k)) // 3 (order not guaranteed)
}
