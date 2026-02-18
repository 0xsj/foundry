// Run with: go run ./functions/
//
// This file demonstrates: function declarations, multiple return values,
// named returns, variadic functions, and first-class functions.

package main

import (
	"errors"
	"fmt"
	"strings"
)

// ============================================================================
// BASIC DECLARATIONS
// ============================================================================

// Standard function: named params, single return
func greet(name string) string {
	return "Hello, " + name
}

// Multiple params of the same type — share the type annotation
func add(a, b int) int {
	return a + b
}

// No params, no return — side effects only
func separator() {
	fmt.Println(strings.Repeat("-", 40))
}

// ============================================================================
// MULTIPLE RETURN VALUES
// ============================================================================

// The canonical Go pattern: (value, error)
func divide(a, b float64) (float64, error) {
	if b == 0 {
		return 0, errors.New("division by zero")
	}
	return a / b, nil
}

// Multiple non-error returns are fine too
func minMax(nums []int) (int, int) {
	if len(nums) == 0 {
		return 0, 0
	}
	min, max := nums[0], nums[0]
	for _, n := range nums[1:] {
		if n < min {
			min = n
		}
		if n > max {
			max = n
		}
	}
	return min, max
}

// ============================================================================
// NAMED RETURN VALUES
// ============================================================================

// Named returns serve as documentation and enable deferred modification.
// Use naked returns only in short, clear functions.
func parseHostPort(addr string) (host string, port string, err error) {
	idx := strings.LastIndex(addr, ":")
	if idx < 0 {
		err = fmt.Errorf("no port in address %q", addr)
		return // naked return: returns current values of host, port, err
	}
	host = addr[:idx]
	port = addr[idx+1:]
	if host == "" {
		err = fmt.Errorf("empty host in address %q", addr)
		return
	}
	return
}

// ============================================================================
// VARIADIC FUNCTIONS
// ============================================================================

// nums is []int inside the function — variadic is syntactic sugar
func sum(nums ...int) int {
	total := 0
	for _, n := range nums {
		total += n
	}
	return total
}

// Variadic with a required first argument
func joinWith(sep string, parts ...string) string {
	return strings.Join(parts, sep)
}

// ============================================================================
// FIRST-CLASS FUNCTIONS
// ============================================================================

// A function type alias makes signatures readable
type Predicate func(int) bool
type Transform func(int) int

func filter(nums []int, keep Predicate) []int {
	var result []int
	for _, n := range nums {
		if keep(n) {
			result = append(result, n)
		}
	}
	return result
}

func mapInts(nums []int, fn Transform) []int {
	result := make([]int, len(nums))
	for i, n := range nums {
		result[i] = fn(n)
	}
	return result
}

// ============================================================================
// MAIN
// ============================================================================

func main() {
	// Basic declarations
	fmt.Println("=== Basic Functions ===")
	fmt.Println(greet("Alice"))
	fmt.Println(add(3, 4))
	separator()

	// Multiple return values
	fmt.Println("=== Multiple Returns ===")
	result, err := divide(10, 3)
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Printf("10 / 3 = %.4f\n", result)
	}

	_, err = divide(10, 0)
	if err != nil {
		fmt.Println("Expected error:", err)
	}

	min, max := minMax([]int{3, 1, 4, 1, 5, 9, 2, 6})
	fmt.Printf("min=%d, max=%d\n", min, max)
	separator()

	// Named returns
	fmt.Println("=== Named Returns ===")
	host, port, err := parseHostPort("localhost:8080")
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Printf("host=%q, port=%q\n", host, port)
	}

	_, _, err = parseHostPort("no-port-here")
	fmt.Println("No port error:", err)
	separator()

	// Variadic functions
	fmt.Println("=== Variadic Functions ===")
	fmt.Println(sum())            // 0
	fmt.Println(sum(1))           // 1
	fmt.Println(sum(1, 2, 3, 4))  // 10

	// Spread a slice using ...
	nums := []int{10, 20, 30}
	fmt.Println(sum(nums...))  // 60

	fmt.Println(joinWith(", ", "alpha", "bravo", "charlie"))
	separator()

	// First-class functions
	fmt.Println("=== First-Class Functions ===")
	numbers := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}

	evens := filter(numbers, func(n int) bool { return n%2 == 0 })
	fmt.Println("Evens:", evens)

	squared := mapInts(numbers, func(n int) int { return n * n })
	fmt.Println("Squared:", squared)

	// Function stored in variable — same as any other value
	isPositive := Predicate(func(n int) bool { return n > 0 })
	positives := filter([]int{-3, -1, 0, 2, 5}, isPositive)
	fmt.Println("Positives:", positives)
}
