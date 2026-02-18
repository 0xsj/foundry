// Run with: go run ./closures/
//
// This file demonstrates: closures, variable capture by reference,
// the loop variable gotcha, function factories, and practical closure patterns.

package main

import (
	"fmt"
	"strings"
)

// ============================================================================
// BASIC CLOSURE: VARIABLE CAPTURE
// ============================================================================

// makeCounter returns a closure that captures `count` by reference.
// Each call to the returned function reads and increments the same `count`.
func makeCounter(start int) func() int {
	count := start // lives on the heap because it escapes to the closure
	return func() int {
		count++
		return count
	}
}

// ============================================================================
// CLOSURES CAPTURE THE VARIABLE, NOT THE VALUE
// ============================================================================

// makeAccumulator: each closure has its own sum variable
func makeAccumulator() func(int) int {
	sum := 0
	return func(n int) int {
		sum += n
		return sum
	}
}

// ============================================================================
// THE LOOP VARIABLE GOTCHA
// ============================================================================

// demonstrateLoopGotcha shows the classic closure capture problem.
// NOTE: Go 1.22+ fixed loop variable capture for range loops — each iteration
// now gets its own variable. But the bug still exists in non-range loops
// (any loop using a shared variable), and in older codebases.
// We use a slice-based example to make the capture explicit regardless of Go version.
func demonstrateLoopGotcha() {
	fmt.Println("--- Loop gotcha (classic: shared variable) ---")

	// WRONG: All closures capture the same variable `v` by reference.
	// By the time any of them is called, the loop has finished and v is the last value.
	// This pattern shows up with any shared loop variable — not just the index.
	values := []string{"alpha", "bravo", "charlie"}
	wrong := make([]func(), len(values))
	for i := range values {
		// In Go < 1.22, `i` was shared across iterations.
		// In Go 1.22+, range loop variables are per-iteration — this is fixed.
		// To simulate the old behavior and understand the concept,
		// we capture a pointer to the slice element (a stable reference):
		ptr := &values[i] // capture pointer — demonstrates reference capture
		wrong[i] = func() {
			fmt.Print(*ptr, " ")
		}
	}
	// These print correctly here, but demonstrates: closures capture references.
	// If values were modified after the loop, all closures would see the change.
	for _, f := range wrong {
		f()
	}
	fmt.Println()

	fmt.Println("--- Same bug with a plain shared variable ---")

	// The bug is easy to reproduce with a plain mutable variable:
	shared := 0
	fns := make([]func(), 3)
	for i := 0; i < 3; i++ {
		shared = i
		fns[i] = func() {
			fmt.Print(shared, " ") // captures shared by reference — always reads current value
		}
	}
	for _, f := range fns {
		f()
	}
	fmt.Println() // all print 2 — shared == 2 after loop

	fmt.Println("--- Fix 1: Shadow with new variable ---")

	fixed1 := make([]func(), 3)
	for i := 0; i < 3; i++ {
		i := i // new independent variable per iteration
		fixed1[i] = func() {
			fmt.Print(i, " ")
		}
	}
	for _, f := range fixed1 {
		f()
	}
	fmt.Println() // prints 0 1 2

	fmt.Println("--- Fix 2: Pass as function argument ---")

	fixed2 := make([]func(), 3)
	for i := 0; i < 3; i++ {
		fixed2[i] = func(n int) func() {
			return func() { fmt.Print(n, " ") }
		}(i) // i is copied to n at call time
	}
	for _, f := range fixed2 {
		f()
	}
	fmt.Println() // prints 0 1 2
}

// ============================================================================
// FUNCTION FACTORIES
// ============================================================================

// makeMultiplier returns a closure that multiplies by n.
// Each call to makeMultiplier creates an independent closure with its own n.
func makeMultiplier(n int) func(int) int {
	return func(x int) int {
		return x * n
	}
}

// makePrefixer returns a closure that prepends a fixed prefix to strings.
func makePrefixer(prefix string) func(string) string {
	return func(msg string) string {
		return prefix + msg
	}
}

// ============================================================================
// CLOSURES AS CONFIGURATION
// ============================================================================

// Validator is a function that checks a string and returns an error message (or "").
type Validator func(string) string

// makeMinLength returns a validator that enforces minimum length.
func makeMinLength(min int) Validator {
	return func(s string) string {
		if len(s) < min {
			return fmt.Sprintf("must be at least %d characters", min)
		}
		return ""
	}
}

// makeMaxLength returns a validator that enforces maximum length.
func makeMaxLength(max int) Validator {
	return func(s string) string {
		if len(s) > max {
			return fmt.Sprintf("must be at most %d characters", max)
		}
		return ""
	}
}

// makeNoSpaces returns a validator that rejects strings containing spaces.
func makeNoSpaces() Validator {
	return func(s string) string {
		if strings.Contains(s, " ") {
			return "must not contain spaces"
		}
		return ""
	}
}

// validate runs all validators against a value and returns all errors.
func validate(value string, validators ...Validator) []string {
	var errs []string
	for _, v := range validators {
		if msg := v(value); msg != "" {
			errs = append(errs, msg)
		}
	}
	return errs
}

// ============================================================================
// MEMOIZATION: CLOSURES AS STATE
// ============================================================================

// memoize wraps a pure function and caches its results.
// The cache map is captured by the closure.
func memoize(fn func(int) int) func(int) int {
	cache := make(map[int]int)
	return func(n int) int {
		if v, ok := cache[n]; ok {
			fmt.Printf("  [cache hit: %d]\n", n)
			return v
		}
		result := fn(n)
		cache[n] = result
		return result
	}
}

// slowFib computes the nth Fibonacci number (intentionally recursive/slow for demo)
func slowFib(n int) int {
	if n <= 1 {
		return n
	}
	return slowFib(n-1) + slowFib(n-2)
}

// ============================================================================
// MAIN
// ============================================================================

func main() {
	fmt.Println("=== Basic Closure: Counter ===")
	c1 := makeCounter(0)
	c2 := makeCounter(10) // independent — its own captured count
	fmt.Println(c1(), c1(), c1()) // 1 2 3
	fmt.Println(c2(), c2())       // 11 12
	fmt.Println(c1())             // 4 — c1's count was not affected by c2

	fmt.Println("\n=== Accumulator ===")
	acc := makeAccumulator()
	fmt.Println(acc(5))  // 5
	fmt.Println(acc(3))  // 8
	fmt.Println(acc(10)) // 18

	fmt.Println("\n=== Loop Variable Gotcha ===")
	demonstrateLoopGotcha()

	fmt.Println("\n=== Function Factories ===")
	double := makeMultiplier(2)
	triple := makeMultiplier(3)
	fmt.Println(double(5), triple(5)) // 10 15

	errorLog := makePrefixer("[ERROR] ")
	infoLog := makePrefixer("[INFO] ")
	fmt.Println(errorLog("disk full"))
	fmt.Println(infoLog("server started"))

	fmt.Println("\n=== Closures as Validators ===")
	usernameValidators := []Validator{
		makeMinLength(3),
		makeMaxLength(20),
		makeNoSpaces(),
	}

	tests := []string{"ab", "alice", "alice smith", strings.Repeat("a", 25)}
	for _, t := range tests {
		errs := validate(t, usernameValidators...)
		if len(errs) == 0 {
			fmt.Printf("  %q: valid\n", t)
		} else {
			fmt.Printf("  %q: %s\n", t, strings.Join(errs, ", "))
		}
	}

	fmt.Println("\n=== Memoization ===")
	fastFib := memoize(slowFib) // note: this only memoizes the outer call, not recursion
	fmt.Println(fastFib(10))
	fmt.Println(fastFib(10)) // cache hit
	fmt.Println(fastFib(7))
	fmt.Println(fastFib(7)) // cache hit
}
