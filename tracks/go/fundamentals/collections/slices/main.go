// Slices — creation, append, capacity growth, backing array sharing, copy, nil vs empty.
//
// Run with: go run ./slices/
package main

import (
	"fmt"
)

func main() {
	creation()
	appendAndGrowth()
	backingArraySharing()
	fullSliceExpression()
	copyPreventsAliasing()
	nilVsEmpty()
}

// ============================================================================
// Creation
// ============================================================================

func creation() {
	fmt.Println("=== Creation ===")

	// Literal — you know the data upfront
	names := []string{"alice", "bob", "carol"}
	fmt.Printf("literal:    %v  len=%d cap=%d\n", names, len(names), cap(names))

	// make — you know the size but not the data yet
	// make([]T, len, cap): len=0 means "empty but allocated for 5"
	events := make([]string, 0, 5)
	fmt.Printf("make(0,5):  %v  len=%d cap=%d\n", events, len(events), cap(events))

	// make with length set — all elements are zero values
	counters := make([]int, 5)
	fmt.Printf("make(5):    %v  len=%d cap=%d\n", counters, len(counters), cap(counters))

	// From array — shares backing array!
	arr := [5]int{10, 20, 30, 40, 50}
	s := arr[1:4] // elements at index 1, 2, 3
	fmt.Printf("from array: %v  len=%d cap=%d\n", s, len(s), cap(s))
	// cap is 4 (from index 1 to end of arr = 4 slots remaining)

	fmt.Println()
}

// ============================================================================
// Append and Capacity Growth
// ============================================================================

func appendAndGrowth() {
	fmt.Println("=== Append and Capacity Growth ===")

	s := make([]int, 0, 4)
	fmt.Printf("initial:         len=%d cap=%d\n", len(s), cap(s))

	for i := 0; i < 8; i++ {
		s = append(s, i)
		fmt.Printf("after append(%d): len=%d cap=%d\n", i, len(s), cap(s))
		// Watch cap double when len exceeds current cap
	}

	// Append multiple values at once
	s2 := []int{1, 2, 3}
	s2 = append(s2, 4, 5, 6)
	fmt.Printf("\nmulti-append: %v\n", s2)

	// Append a whole slice using ...
	a := []int{1, 2}
	b := []int{3, 4, 5}
	c := append(a, b...)
	fmt.Printf("append slice: %v\n", c)

	// Critical: append returns a new header. The original is unchanged.
	original := []int{1, 2, 3}
	_ = append(original, 4) // BUG: result ignored — original not extended
	fmt.Printf("ignored append: %v (still len 3)\n", original)

	fmt.Println()
}

// ============================================================================
// Backing Array Sharing
// ============================================================================

func backingArraySharing() {
	fmt.Println("=== Backing Array Sharing ===")

	a := []int{1, 2, 3, 4, 5}
	// b shares a's backing array — it's a window into the same memory
	b := a[1:3] // [2, 3], len=2, cap=4
	fmt.Printf("a before: %v\n", a)
	fmt.Printf("b before: %v  len=%d cap=%d\n", b, len(b), cap(b))

	// Mutating b mutates a — they share memory
	b[0] = 99
	fmt.Printf("a after b[0]=99: %v\n", a) // a[1] is now 99
	fmt.Printf("b after:         %v\n", b)

	// The dangerous case: append within capacity overwrites tail of original
	fmt.Println("\n-- append within capacity overwrites original --")
	x := []int{1, 2, 3, 4, 5}
	y := x[0:2] // len=2, cap=5 — y has room to grow within x's array!
	fmt.Printf("x before: %v\n", x)
	fmt.Printf("y before: %v  len=%d cap=%d\n", y, len(y), cap(y))

	y = append(y, 99) // cap not exceeded — overwrites x[2] silently!
	fmt.Printf("x after append to y: %v  ← x[2] was overwritten!\n", x)
	fmt.Printf("y after:             %v\n", y)

	fmt.Println()
}

// ============================================================================
// Full Slice Expression — Controlling Capacity
// ============================================================================

func fullSliceExpression() {
	fmt.Println("=== Full Slice Expression (Three-Index) ===")

	x := []int{1, 2, 3, 4, 5}

	// Two-index: cap inherited from x — can overwrite x's tail
	y2 := x[0:2]
	fmt.Printf("two-index:   len=%d cap=%d\n", len(y2), cap(y2)) // cap=5

	// Three-index: cap explicitly bounded to 2 — append must reallocate
	y3 := x[0:2:2]
	fmt.Printf("three-index: len=%d cap=%d\n", len(y3), cap(y3)) // cap=2

	y3 = append(y3, 99) // reallocation — x is safe
	fmt.Printf("x after three-index append: %v  ← unchanged!\n", x)
	fmt.Printf("y3 after:                   %v\n", y3)

	fmt.Println()
}

// ============================================================================
// copy — Preventing Aliasing
// ============================================================================

func copyPreventsAliasing() {
	fmt.Println("=== copy: Preventing Aliasing ===")

	src := []int{1, 2, 3, 4, 5}
	dst := make([]int, len(src))
	n := copy(dst, src)
	fmt.Printf("copied %d elements\n", n)

	dst[0] = 999
	fmt.Printf("src after mutating dst: %v  ← unchanged!\n", src)
	fmt.Printf("dst:                    %v\n", dst)

	// copy copies min(len(dst), len(src)) — destination must be pre-sized
	small := make([]int, 3)
	copy(small, src) // only copies 3 elements
	fmt.Printf("partial copy: %v\n", small)

	// Clone pattern: make a private copy before passing to external code
	events := []string{"login", "purchase", "logout"}
	private := make([]string, len(events))
	copy(private, events)
	private[0] = "modified"
	fmt.Printf("events unchanged: %v\n", events)

	fmt.Println()
}

// ============================================================================
// nil vs empty slice
// ============================================================================

func nilVsEmpty() {
	fmt.Println("=== nil vs empty slice ===")

	var nilSlice []int                // nil, len=0, cap=0
	emptyLiteral := []int{}           // not nil, len=0, cap=0
	emptyMake := make([]int, 0)       // not nil, len=0, cap=0

	fmt.Printf("nilSlice == nil:      %v\n", nilSlice == nil)       // true
	fmt.Printf("emptyLiteral == nil:  %v\n", emptyLiteral == nil)   // false
	fmt.Printf("emptyMake == nil:     %v\n", emptyMake == nil)       // false

	// All have len=0 and can be appended to
	nilSlice = append(nilSlice, 1, 2, 3)
	fmt.Printf("nil slice after append: %v\n", nilSlice)

	// Ranging over nil slice: zero iterations, no panic
	count := 0
	for range nilSlice {
		count++
	}
	// (nilSlice is now [1,2,3] from the append above — reset for demo)
	var empty []int
	for range empty {
		count++
	}
	fmt.Printf("range over nil: %d iterations (no panic)\n", count)

	// JSON difference: nil → null, empty → []
	// (demonstrated in the maps example to keep imports clean)
	fmt.Printf("nil: %v, empty literal: %v\n", []int(nil), []int{})

	fmt.Println()
}
