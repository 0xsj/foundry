// Run with: go run ./defer-patterns/
//
// This file demonstrates: defer execution order, argument evaluation timing,
// defer with named returns, defer for resource cleanup, and defer in error paths.

package main

import (
	"errors"
	"fmt"
	"os"
	"strings"
)

// ============================================================================
// EXECUTION ORDER: LIFO
// ============================================================================

// deferOrder demonstrates that defers execute in last-in-first-out order.
func deferOrder() {
	fmt.Println("start")
	defer fmt.Println("first registered — runs last")
	defer fmt.Println("second registered")
	defer fmt.Println("third registered — runs first")
	fmt.Println("end of function body")
}

// ============================================================================
// ARGUMENT EVALUATION TIMING
// ============================================================================

// deferArgTiming shows that defer arguments are evaluated IMMEDIATELY,
// not when the deferred call executes.
func deferArgTiming() {
	x := 10
	defer fmt.Printf("deferred: x was %d when defer ran (not %d)\n", x, 20)
	// ^ x is captured as 10 RIGHT NOW, even though the call happens later
	x = 20
	fmt.Printf("current: x is now %d\n", x)
}

// ============================================================================
// DEFER WITH NAMED RETURN VALUES
// ============================================================================

// withWrappedError uses defer to consistently wrap errors.
// The deferred function can see and modify the named return value `err`.
func withWrappedError(input string) (result string, err error) {
	defer func() {
		if err != nil {
			// Wrap the error with context — runs after the return value is set
			err = fmt.Errorf("withWrappedError(%q): %w", input, err)
		}
	}()

	if input == "" {
		err = errors.New("input is empty")
		return // naked return — deferred func wraps err before caller sees it
	}
	result = strings.ToUpper(input)
	return
}

// double shows how defer modifies the named return value.
// This is a textbook example from the Go spec.
func double(x int) (result int) {
	defer func() {
		// Runs after "return x" sets result = x, but before caller receives it
		result *= 2
	}()
	return x // sets result = x, then defer doubles it
}

// ============================================================================
// DEFER FOR RESOURCE CLEANUP
// ============================================================================

// writeAndRead creates a temp file, writes to it, reads it back, then cleans up.
// Defer ensures cleanup happens even if an error occurs mid-function.
func writeAndRead(content string) (string, error) {
	// Create temp file
	f, err := os.CreateTemp("", "defer-example-*.txt")
	if err != nil {
		return "", fmt.Errorf("create temp file: %w", err)
	}
	defer func() {
		name := f.Name()
		f.Close()
		os.Remove(name) // cleanup happens when writeAndRead returns, no matter what
	}()

	// Write content
	if _, err := f.WriteString(content); err != nil {
		return "", fmt.Errorf("write: %w", err)
	}

	// Seek back to beginning for reading
	if _, err := f.Seek(0, 0); err != nil {
		return "", fmt.Errorf("seek: %w", err)
	}

	// Read it back
	buf := make([]byte, len(content))
	if _, err := f.Read(buf); err != nil {
		return "", fmt.Errorf("read: %w", err)
	}

	return string(buf), nil
}

// ============================================================================
// LOCK/UNLOCK PATTERN
// ============================================================================

// fakeMu simulates a mutex for demonstration
type fakeMu struct{ name string }

func (m *fakeMu) Lock()   { fmt.Printf("[%s] acquired\n", m.name) }
func (m *fakeMu) Unlock() { fmt.Printf("[%s] released\n", m.name) }

// protectedOperation shows the lock/defer unlock pattern.
// This is the most common defer pattern in production Go code.
func protectedOperation(mu *fakeMu, op string) {
	mu.Lock()
	defer mu.Unlock() // always runs when function returns
	fmt.Printf("[%s] performing: %s\n", mu.Name(), op)
	// even if op panics, mu.Unlock() runs
}

func (m *fakeMu) Name() string { return m.name }

// ============================================================================
// DEFER IN LOOPS — THE TRAP AND FIX
// ============================================================================

// processFilesWrong demonstrates why defer inside a loop is a problem.
// Defers don't run until the function returns — so all files stay open
// until processFilesWrong returns, not at the end of each iteration.
func processFilesWrong(paths []string) {
	fmt.Println("WRONG: defer inside loop — all files stay open until function returns")
	for _, path := range paths {
		f, err := os.Open(path)
		if err != nil {
			fmt.Printf("  couldn't open %s: %v\n", path, err)
			continue
		}
		fmt.Printf("  opened %s (will close when function returns, NOT each iteration)\n", path)
		defer f.Close() // accumulates — runs after ALL iterations complete
	}
}

// processFilesRight: extract the per-file work into a helper.
// The defer runs at the end of the helper, not the outer loop.
func processOneFile(path string) error {
	f, err := os.Open(path)
	if err != nil {
		return fmt.Errorf("open: %w", err)
	}
	defer f.Close() // runs when processOneFile returns — each iteration

	info, err := f.Stat()
	if err != nil {
		return fmt.Errorf("stat: %w", err)
	}
	fmt.Printf("  %s: %d bytes\n", path, info.Size())
	return nil
}

func processFilesRight(paths []string) {
	fmt.Println("RIGHT: defer in helper function — closes each file per iteration")
	for _, path := range paths {
		if err := processOneFile(path); err != nil {
			fmt.Printf("  error: %v\n", err)
		}
	}
}

// ============================================================================
// PANIC AND RECOVER
// ============================================================================

// safeCall wraps a function call and recovers from panics, converting them to errors.
// This is a controlled use of panic/recover — not routine error handling.
func safeCall(fn func()) (err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("panic recovered: %v", r)
		}
	}()
	fn()
	return nil
}

// ============================================================================
// MAIN
// ============================================================================

func main() {
	fmt.Println("=== LIFO Order ===")
	deferOrder()

	fmt.Println("\n=== Argument Evaluation Timing ===")
	deferArgTiming()

	fmt.Println("\n=== Named Return Values ===")
	r, err := withWrappedError("hello")
	fmt.Printf("result=%q, err=%v\n", r, err)

	r, err = withWrappedError("")
	fmt.Printf("result=%q, err=%v\n", r, err)

	fmt.Printf("double(4) = %d\n", double(4)) // 8

	fmt.Println("\n=== Resource Cleanup ===")
	content, err := writeAndRead("hello from defer example")
	if err != nil {
		fmt.Println("Error:", err)
	} else {
		fmt.Printf("Read back: %q\n", content)
	}

	fmt.Println("\n=== Lock/Unlock Pattern ===")
	mu := &fakeMu{name: "db-mutex"}
	protectedOperation(mu, "update user record")

	fmt.Println("\n=== Panic and Recover ===")
	err = safeCall(func() {
		panic("something went wrong")
	})
	fmt.Println("Recovered error:", err)

	err = safeCall(func() {
		fmt.Println("Normal execution — no panic")
	})
	fmt.Println("No error:", err)

	fmt.Println("\n=== Defer in Loops ===")
	// Create some temp files to demonstrate
	var paths []string
	for i := 0; i < 3; i++ {
		f, _ := os.CreateTemp("", fmt.Sprintf("loop-demo-%d-*.txt", i))
		f.WriteString("content")
		f.Close()
		paths = append(paths, f.Name())
	}
	defer func() {
		for _, p := range paths {
			os.Remove(p)
		}
	}()

	processFilesRight(paths)
}
