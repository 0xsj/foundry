package main

import (
	"fmt"
	"unsafe"
)

func main() {
	// ========================================================================
	// 1. PRINT ADDRESSES AND SIZES
	// Use &x for address, unsafe.Sizeof(x) for size in bytes.
	// Run this and observe the output.
	// ========================================================================

	var a int = 42
	var b float64 = 3.14
	var c bool = true
	var d string = "hello"

	fmt.Println("=== Sizes and Addresses ===")
	fmt.Printf("int     — value: %v, addr: %p, size: %d bytes\n", a, &a, unsafe.Sizeof(a))
	fmt.Printf("float64 — value: %v, addr: %p, size: %d bytes\n", b, &b, unsafe.Sizeof(b))
	fmt.Printf("bool    — value: %v, addr: %p, size: %d bytes\n", c, &c, unsafe.Sizeof(c))
	fmt.Printf("string  — value: %v, addr: %p, size: %d bytes\n", d, &d, unsafe.Sizeof(d))
	// Q: Why is string 16 bytes? What are those 16 bytes made of?
	//    8 bytes for a pointer to the character data, 8 bytes for the length.
	//    The actual characters ("hello") live elsewhere — the 16 bytes is just the header.

	// ========================================================================
	// 2. ZERO VALUES IN MEMORY
	// Declare without assigning. Print addresses and values.
	// Are zero-valued variables at different addresses than initialized ones?
	// ========================================================================

	var zeroInt int
	var zeroStr string
	var zeroBool bool

	fmt.Println("\n=== Zero Values ===")
	fmt.Printf("zeroInt  — value: %v, addr: %p\n", zeroInt, &zeroInt)
	fmt.Printf("zeroStr  — value: %q, addr: %p\n", zeroStr, &zeroStr)
	fmt.Printf("zeroBool — value: %v, addr: %p\n", zeroBool, &zeroBool)

	// ========================================================================
	// 3. POINTERS AND INDIRECTION
	// ========================================================================

	x := 42
	p := &x // p holds the address of x

	fmt.Println("\n=== Pointers ===")
	fmt.Printf("x value: %d, x addr: %p\n", x, &x)
	fmt.Printf("p value: %p, p addr: %p\n", p, &p)
	fmt.Printf("*p (dereferenced): %d\n", *p)

	*p = 100
	fmt.Printf("after *p = 100, x is now: %d\n", x)
	// Q: Explain in your own words why changing *p changed x.
	//    when we did x := 42, p := &x, at this point p is the pointer to int, holding memory address of x
	//	 and we changed the value of x, through the pointer when we did *p

	// ========================================================================
	// 4. VALUE SEMANTICS
	// Assigning a struct copies it. Prove it by printing addresses.
	// ========================================================================

	type Point struct {
		X, Y int
	}

	p1 := Point{10, 20}
	p2 := p1 // copy

	fmt.Println("\n=== Value Semantics ===")
	fmt.Printf("p1 — value: %v, addr: %p\n", p1, &p1)
	fmt.Printf("p2 — value: %v, addr: %p\n", p2, &p2)

	p2.X = 999
	fmt.Printf("after p2.X = 999:\n")
	fmt.Printf("p1.X = %d (unchanged?)\n", p1.X)
	fmt.Printf("p2.X = %d\n", p2.X)
	// Q: Are p1 and p2 at the same address? Why or why not?
	//    No. p2 := p1 copied the entire struct into a new location in memory.
	//    Go's default for structs is value semantics — assignment copies all the
	//    bytes to a new address. p2 is an independent copy, so mutating p2.X
	//    doesn't affect p1.

	// ========================================================================
	// 5. SLICES: REFERENCE-ISH SEMANTICS
	// Slices share underlying arrays. This is a common source of bugs.
	// ========================================================================

	s1 := []int{1, 2, 3}
	s2 := s1 // s2 shares the same underlying array

	fmt.Println("\n=== Slice Sharing ===")
	fmt.Printf("s1: %v\n", s1)
	fmt.Printf("s2: %v\n", s2)

	s2[0] = 999
	fmt.Printf("after s2[0] = 999:\n")
	fmt.Printf("s1: %v\n", s1)
	fmt.Printf("s2: %v\n", s2)
	// Q: Why did modifying s2 also change s1?
	//    How is this different from the struct copy above?
	//    s2 := s1 copied the slice header (pointer + len + cap), but both headers
	//    still point to the same underlying array. So s2[0] = 999 writes to the
	//    shared array, and s1 sees it too.
	//    The struct had its data inside it (value type). The slice has its data
	//    elsewhere, with just a pointer to it — so copying the header shares the data.

	// ========================================================================
	// 6. ESCAPE ANALYSIS
	// Run this file with: go build -gcflags="-m" memory.go
	// Look for "escapes to heap" in the output.
	// ========================================================================

	// Q: Which variables in this file do you think escape to the heap?
	//    Run the escape analysis and check.
	//    (answer here)
}
