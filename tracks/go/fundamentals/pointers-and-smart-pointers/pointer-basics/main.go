// Package main demonstrates pointer fundamentals in Go:
// &x (address-of), *p (dereference), nil checks, new(), and &T{}.
//
// Run with: go run ./pointer-basics/
package main

import (
	"fmt"
	"unsafe"
)

// ============================================================================
// BASIC POINTER OPERATIONS
// ============================================================================

func basicPointers() {
	x := 42
	p := &x // p is *int — holds the address of x

	fmt.Println("--- basic pointer operations ---")
	fmt.Printf("x      = %d\n", x)
	fmt.Printf("&x     = %p  (address of x)\n", &x)
	fmt.Printf("p      = %p  (pointer value — same address)\n", p)
	fmt.Printf("*p     = %d  (value at that address)\n", *p)

	// Writing through the pointer
	*p = 100
	fmt.Printf("after *p = 100: x = %d\n", x) // x changed

	// Both p and q point to the same x
	q := p
	*q = 999
	fmt.Printf("after *q = 999: x = %d\n", x) // x changed again

	fmt.Println()
}

// ============================================================================
// NIL POINTERS
// ============================================================================

func nilPointers() {
	fmt.Println("--- nil pointers ---")

	var p *int
	fmt.Printf("zero value of *int: p = %v\n", p) // <nil>
	fmt.Printf("p == nil: %v\n", p == nil)          // true

	// Safe nil check before dereference
	if p != nil {
		fmt.Println("p points to:", *p)
	} else {
		fmt.Println("p is nil — skipping dereference")
	}

	// Assigning to nil is valid
	x := 42
	p = &x
	fmt.Printf("after p = &x: *p = %d\n", *p)

	p = nil // release — p no longer points to x
	fmt.Printf("after p = nil: p == nil: %v\n", p == nil)

	fmt.Println()
}

// ============================================================================
// TWO WAYS TO ALLOCATE
// new(T) vs &T{}
// ============================================================================

type ServiceConfig struct {
	Host    string
	Port    int
	Timeout int
	Debug   bool
}

func twoWaysToAllocate() {
	fmt.Println("--- new(T) vs &T{} ---")

	// new(T): allocates zero value, returns pointer
	p1 := new(int)
	fmt.Printf("new(int): *p1 = %d (zero value)\n", *p1)
	*p1 = 7
	fmt.Printf("after *p1 = 7: %d\n", *p1)

	// &T{}: composite literal — specify fields inline
	cfg := &ServiceConfig{
		Host:    "api.example.com",
		Port:    8080,
		Timeout: 30,
	}
	fmt.Printf("&ServiceConfig{}: %+v\n", *cfg)
	// cfg.Debug is false (zero value for bool — unspecified fields get zero values)

	// new(T) is equivalent to &T{} with all zero fields
	cfg2 := new(ServiceConfig)
	cfg3 := &ServiceConfig{}
	fmt.Printf("new(ServiceConfig) == &ServiceConfig{}: both have Port=%d\n", cfg2.Port)
	_ = cfg3

	// Pointer size: always 8 bytes on 64-bit systems regardless of T
	fmt.Printf("sizeof *ServiceConfig pointer: %d bytes\n", unsafe.Sizeof(cfg))
	fmt.Printf("sizeof ServiceConfig value:    %d bytes\n", unsafe.Sizeof(*cfg))

	fmt.Println()
}

// ============================================================================
// POINTER COMPARISON
// ============================================================================

func pointerComparison() {
	fmt.Println("--- pointer comparison ---")

	x := 1
	y := 1
	p := &x
	q := &x // same variable as p
	r := &y // different variable, same value

	fmt.Printf("p == q (same variable): %v\n", p == q)  // true
	fmt.Printf("p == r (diff variable): %v\n", p == r)  // false (different addresses)
	fmt.Printf("*p == *r (same value):  %v\n", *p == *r) // true

	// nil pointers are equal to each other
	var a, b *int
	fmt.Printf("nil == nil: %v\n", a == b) // true

	fmt.Println()
}

// ============================================================================
// AUTO-DEREF ON STRUCT FIELDS
// ============================================================================

type Point struct {
	X, Y int
}

func autoDeref() {
	fmt.Println("--- automatic pointer deref on struct fields ---")

	p := &Point{X: 3, Y: 4}

	// These are identical — Go auto-dereferences through pointer for field access
	fmt.Printf("p.X = %d  (auto-deref)\n", p.X)
	fmt.Printf("(*p).X = %d  (explicit deref)\n", (*p).X)

	p.X = 10 // same as (*p).X = 10
	fmt.Printf("after p.X = 10: (*p).X = %d\n", (*p).X)

	fmt.Println()
}

// ============================================================================
// POINTER SIZE DEMONSTRATION
// Pointers are always 8 bytes on 64-bit — size of T doesn't matter
// ============================================================================

type SmallStruct struct {
	N int // 8 bytes total
}

type LargeStruct struct {
	Data [1024]byte // 1024 bytes total
}

func pointerSize() {
	fmt.Println("--- pointer size is always 8 bytes ---")

	small := SmallStruct{}
	large := LargeStruct{}
	ps := &small
	pl := &large

	fmt.Printf("sizeof SmallStruct:   %d bytes\n", unsafe.Sizeof(small))
	fmt.Printf("sizeof LargeStruct:  %d bytes\n", unsafe.Sizeof(large))
	fmt.Printf("sizeof *SmallStruct: %d bytes (pointer)\n", unsafe.Sizeof(ps))
	fmt.Printf("sizeof *LargeStruct: %d bytes (pointer)\n", unsafe.Sizeof(pl))

	fmt.Println()
}

func main() {
	basicPointers()
	nilPointers()
	twoWaysToAllocate()
	pointerComparison()
	autoDeref()
	pointerSize()
}
