// Maps — creation, nil map pitfall, comma-ok, delete, iteration, sets pattern.
//
// Run with: go run ./maps/
package main

import (
	"encoding/json"
	"fmt"
	"sort"
)

func main() {
	creation()
	nilMapPanic()
	readAndCommaOk()
	deleteAndIterate()
	randomizedOrder()
	structValuesInMaps()
	setsPattern()
	jsonDifference()
}

// ============================================================================
// Creation
// ============================================================================

func creation() {
	fmt.Println("=== Map Creation ===")

	// Literal — for known data
	config := map[string]string{
		"host": "localhost",
		"port": "8080",
		"env":  "production",
	}
	fmt.Printf("literal: %v\n", config)

	// make — for empty maps you'll populate
	cache := make(map[string][]byte)
	cache["key"] = []byte("value")
	fmt.Printf("make: %v entries\n", len(cache))

	// make with capacity hint — prevents rehashing for large maps
	// The hint is advisory; the map will grow beyond it.
	index := make(map[string]int, 10_000)
	_ = index // (not populated here)

	fmt.Println()
}

// ============================================================================
// nil map: readable but panics on write
// ============================================================================

func nilMapPanic() {
	fmt.Println("=== nil Map: Read-OK, Write-Panics ===")

	var m map[string]int // zero value: nil

	// Reading from nil map returns zero value — no panic
	v := m["any-key"]
	fmt.Printf("read from nil map: %d (zero value)\n", v)

	_, ok := m["any-key"]
	fmt.Printf("comma-ok on nil map: ok=%v\n", ok)

	// Writing to nil map panics — uncomment to see:
	// m["key"] = 1  // PANIC: assignment to entry in nil map

	// Fix: always make() before writing
	m = make(map[string]int)
	m["requests"] = 42
	fmt.Printf("after make: %v\n", m)

	fmt.Println()
}

// ============================================================================
// Reading and Comma-Ok
// ============================================================================

func readAndCommaOk() {
	fmt.Println("=== Reading and Comma-Ok Pattern ===")

	counts := map[string]int{
		"errors":   0, // present, value is zero
		"warnings": 5,
	}

	// Single-value form — zero if missing (ambiguous!)
	n1 := counts["requests"] // 0 — is it missing, or zero?
	n2 := counts["errors"]   // 0 — same value, different meaning!
	fmt.Printf("single-value: requests=%d errors=%d\n", n1, n2)

	// Comma-ok — unambiguous
	v, ok := counts["requests"]
	fmt.Printf("requests: v=%d ok=%v (key is missing)\n", v, ok)

	v, ok = counts["errors"]
	fmt.Printf("errors:   v=%d ok=%v (key exists, value is 0)\n", v, ok)

	// Real-world pattern: default if missing
	timeout, ok := counts["timeout"]
	if !ok {
		timeout = 30 // default
	}
	fmt.Printf("timeout with default: %d\n", timeout)

	fmt.Println()
}

// ============================================================================
// Delete and Iteration
// ============================================================================

func deleteAndIterate() {
	fmt.Println("=== Delete and Iteration ===")

	m := map[string]int{"a": 1, "b": 2, "c": 3}

	delete(m, "b")            // removes "b"
	delete(m, "nonexistent")  // safe no-op
	fmt.Printf("after delete: %v\n", m)

	// Checking membership after delete
	if _, ok := m["b"]; !ok {
		fmt.Println("b was deleted")
	}

	// Iterating — order is random
	fmt.Println("iteration (order varies):")
	for k, v := range m {
		fmt.Printf("  %s: %d\n", k, v)
	}

	// Keys-only iteration
	fmt.Println("keys only:")
	for k := range m {
		fmt.Printf("  %s\n", k)
	}

	fmt.Println()
}

// ============================================================================
// Randomized Iteration Order
// ============================================================================

func randomizedOrder() {
	fmt.Println("=== Randomized Order — Sort for Determinism ===")

	scores := map[string]int{
		"alice":   95,
		"bob":     87,
		"carol":   91,
		"dave":    78,
		"eve":     99,
	}

	// Collect keys, sort them, then iterate in order
	keys := make([]string, 0, len(scores))
	for k := range scores {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	fmt.Println("sorted output:")
	for _, k := range keys {
		fmt.Printf("  %-8s %d\n", k, scores[k])
	}

	fmt.Println()
}

// ============================================================================
// Struct Values in Maps — Read-Modify-Write
// ============================================================================

func structValuesInMaps() {
	fmt.Println("=== Struct Values: Read-Modify-Write ===")

	type Stats struct {
		Count   int
		LastSeen string
	}

	m := map[string]Stats{
		"alice": {Count: 5, LastSeen: "2026-01-01"},
	}

	// m["alice"].Count++ // compile error: cannot assign to map index expression

	// Fix: read-modify-write
	s := m["alice"]
	s.Count++
	s.LastSeen = "2026-02-18"
	m["alice"] = s
	fmt.Printf("after update: %+v\n", m["alice"])

	// Alternative: use pointer values in the map
	ptrMap := map[string]*Stats{
		"bob": {Count: 3, LastSeen: "2026-01-15"},
	}
	ptrMap["bob"].Count++ // fine — dereferencing pointer
	fmt.Printf("pointer map: %+v\n", *ptrMap["bob"])

	fmt.Println()
}

// ============================================================================
// Sets Pattern: map[T]struct{}
// ============================================================================

func setsPattern() {
	fmt.Println("=== Sets Pattern: map[T]struct{} ===")

	// struct{} is zero-size — no memory allocated for values
	seen := make(map[string]struct{})

	events := []string{"click", "hover", "click", "submit", "hover", "click"}
	unique := make([]string, 0)

	for _, e := range events {
		if _, exists := seen[e]; !exists {
			seen[e] = struct{}{}
			unique = append(unique, e)
		}
	}
	sort.Strings(unique) // deterministic output
	fmt.Printf("unique events: %v\n", unique)

	// Membership check
	if _, ok := seen["click"]; ok {
		fmt.Println("'click' was seen")
	}

	// Alternative with map[T]bool — same behavior, more readable
	seenBool := make(map[string]bool)
	seenBool["login"] = true
	if seenBool["login"] {
		fmt.Println("'login' was seen (bool set)")
	}
	// Missing key returns false — no comma-ok needed for bool sets
	if !seenBool["logout"] {
		fmt.Println("'logout' not seen")
	}

	fmt.Println()
}

// ============================================================================
// JSON Difference: nil slice → null, empty slice → []
// ============================================================================

func jsonDifference() {
	fmt.Println("=== JSON: nil vs empty slice ===")

	type Response struct {
		Items []string `json:"items"`
	}

	nilResp, _ := json.Marshal(Response{Items: nil})
	emptyResp, _ := json.Marshal(Response{Items: []string{}})

	fmt.Printf("nil slice:   %s\n", nilResp)   // {"items":null}
	fmt.Printf("empty slice: %s\n", emptyResp) // {"items":[]}

	// This matters: APIs often expect [] not null for empty collections.
	// Convention: return []T{} (or make([]T, 0)) from functions that
	// return collections — never return nil if the caller expects [].

	fmt.Println()
}
