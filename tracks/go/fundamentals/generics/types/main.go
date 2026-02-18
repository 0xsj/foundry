// Package main demonstrates generic structs and methods on generic types.
// Run with: go run ./types/
package main

import (
	"errors"
	"fmt"
)

// ============================================================================
// STACK[T] — generic LIFO data structure
// ============================================================================

// Stack is a generic last-in-first-out container.
// T can be any type — the type is set when you declare a variable: var s Stack[int]
type Stack[T any] struct {
	items []T
}

// Push adds an item to the top of the stack.
// The receiver is Stack[T] — T is the same type parameter declared on the struct.
func (s *Stack[T]) Push(item T) {
	s.items = append(s.items, item)
}

// Pop removes and returns the top item.
// Returns the zero value of T and false if the stack is empty.
func (s *Stack[T]) Pop() (T, bool) {
	if len(s.items) == 0 {
		var zero T
		return zero, false
	}
	top := s.items[len(s.items)-1]
	s.items = s.items[:len(s.items)-1]
	return top, true
}

// Peek returns the top item without removing it.
func (s *Stack[T]) Peek() (T, bool) {
	if len(s.items) == 0 {
		var zero T
		return zero, false
	}
	return s.items[len(s.items)-1], true
}

// Len returns the number of items in the stack.
func (s *Stack[T]) Len() int {
	return len(s.items)
}

// IsEmpty reports whether the stack has no items.
func (s *Stack[T]) IsEmpty() bool {
	return len(s.items) == 0
}

// ============================================================================
// RESULT[T] — type-safe success/failure value (like Rust's Result<T, E>)
// ============================================================================

// Result holds either a successful value of type T or an error.
// This is a common pattern for wrapping operations that can fail
// when you want to pass the outcome around without losing type information.
type Result[T any] struct {
	value T
	err   error
}

// OK creates a successful Result.
func OK[T any](v T) Result[T] {
	return Result[T]{value: v}
}

// Err creates a failed Result.
func Err[T any](err error) Result[T] {
	return Result[T]{err: err}
}

// Unwrap returns the value and error.
func (r Result[T]) Unwrap() (T, error) {
	return r.value, r.err
}

// IsOK reports whether the result is a success.
func (r Result[T]) IsOK() bool {
	return r.err == nil
}

// OrDefault returns the value on success, or def on failure.
func (r Result[T]) OrDefault(def T) T {
	if r.err != nil {
		return def
	}
	return r.value
}

// ============================================================================
// PAIR[A, B] — a generic tuple with two values of potentially different types
// ============================================================================

// Pair holds two values of potentially different types.
type Pair[A, B any] struct {
	First  A
	Second B
}

// Swap returns a new Pair with First and Second exchanged.
func (p Pair[A, B]) Swap() Pair[B, A] {
	return Pair[B, A]{First: p.Second, Second: p.First}
}

// NewPair constructs a Pair; type arguments inferred from arguments.
func NewPair[A, B any](a A, b B) Pair[A, B] {
	return Pair[A, B]{First: a, Second: b}
}

// ============================================================================
// SET[T] — a generic set backed by a map
// ============================================================================

// Set is a collection of unique values.
// T must be comparable because it's used as a map key.
type Set[T comparable] struct {
	items map[T]struct{}
}

// NewSet creates an initialized Set.
// Note: you MUST use this (or initialize items manually) — a zero-value Set
// has a nil map and will panic on Add. This is the Go nil-map trap applied
// to generic types — see [[go-nil-map-panic]] in the vault.
func NewSet[T comparable]() *Set[T] {
	return &Set[T]{items: make(map[T]struct{})}
}

// Add inserts an element into the set (no-op if already present).
func (s *Set[T]) Add(item T) {
	s.items[item] = struct{}{}
}

// Remove deletes an element from the set.
func (s *Set[T]) Remove(item T) {
	delete(s.items, item)
}

// Contains reports whether item is in the set.
func (s *Set[T]) Contains(item T) bool {
	_, ok := s.items[item]
	return ok
}

// Len returns the number of elements.
func (s *Set[T]) Len() int {
	return len(s.items)
}

// Union returns a new set containing all elements from both s and other.
func (s *Set[T]) Union(other *Set[T]) *Set[T] {
	result := NewSet[T]()
	for k := range s.items {
		result.Add(k)
	}
	for k := range other.items {
		result.Add(k)
	}
	return result
}

// Intersection returns a new set containing elements present in both sets.
func (s *Set[T]) Intersection(other *Set[T]) *Set[T] {
	result := NewSet[T]()
	for k := range s.items {
		if other.Contains(k) {
			result.Add(k)
		}
	}
	return result
}

// ============================================================================
// MAIN — demonstrate each generic type
// ============================================================================

// parsePort is a helper that returns a Result[int] to show Result usage.
func parsePort(s string) Result[int] {
	if s == "" {
		return Err[int](errors.New("empty port string"))
	}
	// Intentionally simplified — real parsing would use strconv.Atoi
	if s == "8080" {
		return OK(8080)
	}
	if s == "443" {
		return OK(443)
	}
	return Err[int](fmt.Errorf("unknown port: %s", s))
}

func main() {
	// ---- Stack ----
	fmt.Println("=== Stack[string] ===")
	var s Stack[string]
	s.Push("first")
	s.Push("second")
	s.Push("third")

	fmt.Println("Len:", s.Len()) // 3

	if top, ok := s.Peek(); ok {
		fmt.Println("Peek:", top) // third
	}

	for !s.IsEmpty() {
		v, _ := s.Pop()
		fmt.Println("Pop:", v) // third, second, first
	}
	fmt.Println("Empty:", s.IsEmpty()) // true

	// Stack[int] — same code, different type
	var intStack Stack[int]
	intStack.Push(10)
	intStack.Push(20)
	v, _ := intStack.Pop()
	fmt.Println("IntStack pop:", v) // 20

	// ---- Result ----
	fmt.Println("\n=== Result[int] ===")
	good := parsePort("8080")
	bad := parsePort("9999")
	empty := parsePort("")

	port, err := good.Unwrap()
	fmt.Printf("good: %d, err=%v\n", port, err) // 8080, nil

	port, err = bad.Unwrap()
	fmt.Printf("bad: %d, err=%v\n", port, err) // 0, unknown port: 9999

	fmt.Println("empty.OrDefault(80):", empty.OrDefault(80)) // 80
	fmt.Println("good.IsOK():", good.IsOK())                 // true
	fmt.Println("bad.IsOK():", bad.IsOK())                   // false

	// ---- Pair ----
	fmt.Println("\n=== Pair[string, int] ===")
	p := NewPair("timeout", 30)
	fmt.Printf("First=%q, Second=%d\n", p.First, p.Second)

	swapped := p.Swap()
	fmt.Printf("Swapped: First=%d, Second=%q\n", swapped.First, swapped.Second)

	// Pair with two different types — mixed types demonstrate the A, B parameters
	kv := NewPair("region", []string{"us-east-1", "eu-west-1"})
	fmt.Println("Config key:", kv.First)
	fmt.Println("Config val:", kv.Second)

	// ---- Set ----
	fmt.Println("\n=== Set[string] ===")
	services := NewSet[string]()
	services.Add("auth")
	services.Add("payments")
	services.Add("notifications")
	services.Add("auth") // duplicate — no-op

	fmt.Println("Len:", services.Len())              // 3 (not 4)
	fmt.Println("Contains auth:", services.Contains("auth"))          // true
	fmt.Println("Contains billing:", services.Contains("billing"))    // false

	critical := NewSet[string]()
	critical.Add("auth")
	critical.Add("payments")
	critical.Add("database")

	union := services.Union(critical)
	fmt.Println("Union len:", union.Len()) // 4: auth, payments, notifications, database

	intersection := services.Intersection(critical)
	fmt.Println("Intersection len:", intersection.Len()) // 2: auth, payments
}
