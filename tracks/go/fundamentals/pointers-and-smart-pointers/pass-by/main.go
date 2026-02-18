// Package main demonstrates pass-by-value vs pass-by-pointer semantics:
// - Go is always pass-by-value (even pointer arguments are copied)
// - How pointer passing simulates pass-by-reference
// - Mutation via pointer vs copy
// - Reference types (slices, maps) and when you still need *[]T
// - Large struct optimization
//
// Run with: go run ./pass-by/
package main

import (
	"fmt"
	"unsafe"
)

// ============================================================================
// PRIMITIVES: VALUE COPY vs POINTER
// ============================================================================

func doubleValue(n int) {
	n *= 2 // modifies local copy — caller unchanged
}

func doublePointer(n *int) {
	*n *= 2 // writes through pointer — caller's value changes
}

func primitivePassBy() {
	fmt.Println("--- primitive: pass by value vs pointer ---")

	x := 5
	doubleValue(x)
	fmt.Printf("after doubleValue: x = %d (unchanged)\n", x)

	doublePointer(&x)
	fmt.Printf("after doublePointer: x = %d (changed)\n", x)

	fmt.Println()
}

// ============================================================================
// STRUCTS: VALUE COPY vs POINTER
// The central use case for struct pointers
// ============================================================================

type AccountBalance struct {
	AccountID string
	Amount    int
	Currency  string
}

// Value receiver — modifies a copy, caller unchanged
func depositBroken(acc AccountBalance, amount int) {
	acc.Amount += amount
	// When this function returns, the modified copy is discarded
}

// Pointer receiver — modifies through pointer, caller sees the change
func deposit(acc *AccountBalance, amount int) {
	acc.Amount += amount
}

func structPassBy() {
	fmt.Println("--- struct: pass by value vs pointer ---")

	acc := AccountBalance{AccountID: "acct-001", Amount: 1000, Currency: "USD"}

	depositBroken(acc, 500)
	fmt.Printf("after depositBroken: Amount = %d (unchanged — worked on copy)\n", acc.Amount)

	deposit(&acc, 500)
	fmt.Printf("after deposit: Amount = %d (changed via pointer)\n", acc.Amount)

	fmt.Println()
}

// ============================================================================
// MULTIPLE RETURN VALUES vs OUTPUT PARAMETERS
// In Go, prefer multiple returns over pointer output parameters
// ============================================================================

// Idiomatic Go: return multiple values
func divide(a, b float64) (float64, error) {
	if b == 0 {
		return 0, fmt.Errorf("division by zero")
	}
	return a / b, nil
}

// Non-idiomatic: pointer output parameter (C-style — avoid in Go)
func divideViaPointer(a, b float64, result *float64) error {
	if b == 0 {
		return fmt.Errorf("division by zero")
	}
	*result = a / b
	return nil
}

func outputParams() {
	fmt.Println("--- output params: prefer multiple returns ---")

	result, err := divide(10, 3)
	fmt.Printf("divide(10,3) = %.4f, err=%v\n", result, err)

	// C-style output pointer — works but not idiomatic Go
	var r float64
	err = divideViaPointer(10, 3, &r)
	fmt.Printf("divideViaPointer(10,3) = %.4f, err=%v\n", r, err)

	fmt.Println()
}

// ============================================================================
// SLICES: ALREADY REFERENCE TYPES
// A slice header is 3 words: pointer + len + cap
// Passing a slice copies the header — both copies point to same array
// ============================================================================

// Modifying elements: visible to caller (same underlying array)
func zeroFirstElement(s []int) {
	if len(s) > 0 {
		s[0] = 0 // modifies element in the shared array
	}
}

// Appending: NOT visible to caller (modifies local copy of the header)
func appendToSlice(s []int, val int) {
	s = append(s, val) // s is a new header — caller's slice header unchanged
	fmt.Printf("  inside appendToSlice: len=%d, s=%v\n", len(s), s)
}

// Appending via pointer: visible to caller
func appendToSlicePtr(s *[]int, val int) {
	*s = append(*s, val) // modifies the caller's slice header
}

func slicePassBy() {
	fmt.Println("--- slices: reference type (mostly) ---")

	nums := []int{1, 2, 3}
	fmt.Printf("before zeroFirstElement: %v\n", nums)
	zeroFirstElement(nums)
	fmt.Printf("after zeroFirstElement: %v (element change visible)\n", nums)

	nums = []int{1, 2, 3}
	appendToSlice(nums, 99)
	fmt.Printf("after appendToSlice: %v (append NOT visible — header copy)\n", nums)

	appendToSlicePtr(&nums, 99)
	fmt.Printf("after appendToSlicePtr: %v (append visible via *[]int)\n", nums)

	// Idiomatic alternative: return the slice
	nums = appendReturn(nums, 200)
	fmt.Printf("after appendReturn: %v (idiomatic — return the slice)\n", nums)

	fmt.Println()
}

// Idiomatic: return the modified slice instead of using *[]T
func appendReturn(s []int, val int) []int {
	return append(s, val)
}

// ============================================================================
// MAPS: ALREADY REFERENCE TYPES
// Map variable is a pointer to the hash table — modifications always visible
// ============================================================================

func addToMap(m map[string]int, key string, val int) {
	m[key] = val // visible to caller — map is already a reference
}

func mapPassBy() {
	fmt.Println("--- maps: reference type ---")

	counts := map[string]int{"a": 1}
	addToMap(counts, "b", 2)
	fmt.Printf("after addToMap: %v (modification always visible)\n", counts)

	fmt.Println()
}

// ============================================================================
// LARGE STRUCT: POINTER AVOIDS COPY COST
// Profile before optimizing — Go is fast at copying small structs
// ============================================================================

type SensorReading struct {
	SensorID    string
	Timestamp   int64
	Samples     [512]float64 // 4KB of data
	Tags        [32]string
	Calibration float64
}

// Passes ~4KB+ on every call
func processByValue(r SensorReading) float64 {
	sum := 0.0
	for _, s := range r.Samples {
		sum += s
	}
	return sum / float64(len(r.Samples))
}

// Passes 8 bytes (pointer) on every call
func processByPointer(r *SensorReading) float64 {
	sum := 0.0
	for _, s := range r.Samples {
		sum += s
	}
	return sum / float64(len(r.Samples))
}

func largeStructDemo() {
	fmt.Println("--- large struct: pointer avoids copy ---")

	reading := SensorReading{SensorID: "sensor-001"}
	for i := range reading.Samples {
		reading.Samples[i] = float64(i)
	}

	fmt.Printf("SensorReading size: %d bytes\n", unsafe.Sizeof(reading))
	fmt.Printf("*SensorReading (pointer) size: %d bytes\n", unsafe.Sizeof(&reading))

	avgByValue := processByValue(reading)
	avgByPointer := processByPointer(&reading)
	fmt.Printf("avg by value:   %.1f\n", avgByValue)
	fmt.Printf("avg by pointer: %.1f\n", avgByPointer)
	// Same result — pointer just avoids the copy cost

	fmt.Println()
}

// ============================================================================
// POINTER IS STILL A VALUE
// When you pass *T, the pointer itself is copied — not the pointed-to value
// Reassigning the pointer variable inside a function doesn't affect caller
// ============================================================================

type Node struct {
	Value int
	Next  *Node
}

// This does NOT change what the caller's pointer points to
func tryToReplaceNode(n *Node) {
	n = &Node{Value: 999} // reassigns local copy of pointer — caller's unchanged
	fmt.Printf("  inside tryToReplaceNode: n.Value = %d\n", n.Value)
}

// This DOES change what the caller's pointer points to
func replaceNodePtr(n **Node) {
	*n = &Node{Value: 999} // dereferences the pointer-to-pointer — changes caller's pointer
}

func pointerIsAValue() {
	fmt.Println("--- pointer is still a value ---")

	node := &Node{Value: 1}
	fmt.Printf("before tryToReplaceNode: node.Value = %d\n", node.Value)
	tryToReplaceNode(node)
	fmt.Printf("after tryToReplaceNode: node.Value = %d (unchanged)\n", node.Value)

	replaceNodePtr(&node)
	fmt.Printf("after replaceNodePtr: node.Value = %d (changed)\n", node.Value)

	fmt.Println()
}

func main() {
	primitivePassBy()
	structPassBy()
	outputParams()
	slicePassBy()
	mapPassBy()
	largeStructDemo()
	pointerIsAValue()
}
