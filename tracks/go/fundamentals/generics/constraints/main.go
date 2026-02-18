// Package main demonstrates custom constraints, the ~ operator, and union types.
// Run with: go run ./constraints/
package main

import (
	"fmt"
	"strings"
)

// ============================================================================
// THE ~ OPERATOR — underlying type constraints
// ============================================================================

// Domain-specific newtype aliases. These are NOT float64 — they are distinct
// named types whose underlying type is float64.
type Celsius float64
type Fahrenheit float64
type Kelvin float64

// Temperature is a constraint matching float64 AND any named type with
// underlying type float64. Without ~, Celsius and Fahrenheit wouldn't satisfy it.
type Temperature interface {
	~float64
}

// ToKelvin converts any Temperature to Kelvin.
// Works for float64, Celsius, Fahrenheit — any ~float64 type.
func ToKelvin[T Temperature](val T) Kelvin {
	return Kelvin(val + 273.15) // simplified: assumes Celsius input
}

// AbsDiff returns the absolute difference between two values.
// Works for int, int32, int64, float32, float64, and any named types thereof.
type SignedNumeric interface {
	~int | ~int32 | ~int64 | ~float32 | ~float64
}

func AbsDiff[T SignedNumeric](a, b T) T {
	d := a - b
	if d < 0 {
		return -d
	}
	return d
}

// ============================================================================
// UNION CONSTRAINTS — restricting to a specific set of types
// ============================================================================

// ID types used in a hypothetical service — distinct types prevent mixing up
// user IDs with order IDs.
type UserID string
type OrderID string
type ProductID string

// StringID matches string and any named type with underlying type string.
type StringID interface {
	~string
}

// ParseID parses a string into any string-based ID type.
// The ~ makes it work for UserID, OrderID, ProductID — not just raw string.
func ParseID[T StringID](raw string) (T, error) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		var zero T
		return zero, fmt.Errorf("ID cannot be empty")
	}
	return T(trimmed), nil // T(raw) works because T's underlying type is string
}

// ============================================================================
// CUSTOM CONSTRAINTS WITH METHODS
// ============================================================================

// Summarizable describes any type that can produce a summary string.
type Summarizable interface {
	Summary() string
}

// Renderable combines Summarizable with fmt.Stringer.
// You can embed interfaces inside constraint interfaces just like regular interfaces.
type Renderable interface {
	Summarizable
	fmt.Stringer
}

// PrintSummary prints the summary of any Summarizable.
func PrintSummary[T Summarizable](items []T) {
	for _, item := range items {
		fmt.Println(" -", item.Summary())
	}
}

// Event is a concrete type that satisfies Summarizable.
type Event struct {
	Type    string
	Payload string
}

func (e Event) Summary() string {
	return fmt.Sprintf("[%s] %s", e.Type, e.Payload)
}

// ============================================================================
// COMBINING ~ AND | — the full constraint toolkit
// ============================================================================

// Numeric covers all integer and floating-point types — built-in and named.
type Numeric interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 |
		~float32 | ~float64
}

// Sum adds all elements of a slice.
// Works for []int, []float64, []int32, and any slice of named numeric types.
func Sum[T Numeric](nums []T) T {
	var total T
	for _, n := range nums {
		total += n
	}
	return total
}

// Average returns the average as a float64, regardless of input type.
// Note the return type is always float64 — we convert inside.
func Average[T Numeric](nums []T) float64 {
	if len(nums) == 0 {
		return 0
	}
	return float64(Sum(nums)) / float64(len(nums))
}

// ============================================================================
// NAMED TYPE EXAMPLES — showing why ~ matters in practice
// ============================================================================

// Score is a domain type used for test or ranking scores.
type Score int32

// Temperature domain is already defined above. Here's another example:
type Milliseconds int64
type Seconds float64

// ============================================================================
// CONSTRAINT COMPOSITION — building constraints from other constraints
// ============================================================================

// OrderedID is something that is both an ID (string-based) and can be compared.
// In practice, all ~string types are comparable, but this shows composition.
type OrderedID interface {
	StringID
	comparable
}

// UniqueIDs returns a deduplicated slice of IDs preserving order.
func UniqueIDs[T OrderedID](ids []T) []T {
	seen := make(map[T]struct{})
	var result []T
	for _, id := range ids {
		if _, dup := seen[id]; !dup {
			seen[id] = struct{}{}
			result = append(result, id)
		}
	}
	return result
}

// ============================================================================
// WHEN CONSTRAINTS ENABLE OPERATIONS — a comparison
// ============================================================================

// With `any`, you can only assign, copy, and store in slices.
func StoreAny[T any](items []T) []T {
	result := make([]T, len(items))
	copy(result, items)
	return result
	// cannot do: items[0] < items[1]  — ERROR
	// cannot do: items[0] == items[1] — ERROR (use comparable for this)
}

// With `comparable`, you additionally get == and !=.
func Deduplicate[T comparable](items []T) []T {
	seen := make(map[T]struct{})
	var result []T
	for _, v := range items {
		if _, ok := seen[v]; !ok {
			seen[v] = struct{}{}
			result = append(result, v)
		}
	}
	return result
}

// With a custom Ordered constraint, you additionally get < > <= >=.
type Ordered interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 |
		~float32 | ~float64 |
		~string
}

func IsSorted[T Ordered](items []T) bool {
	for i := 1; i < len(items); i++ {
		if items[i] < items[i-1] {
			return false
		}
	}
	return true
}

func main() {
	// ---- ~ operator ----
	fmt.Println("=== ~ operator ===")
	c := Celsius(100)
	f := Fahrenheit(212)

	k1 := ToKelvin(c)
	k2 := ToKelvin(f) // Fahrenheit also satisfies ~float64
	fmt.Printf("Celsius(100) → Kelvin: %.2f\n", float64(k1))
	fmt.Printf("Fahrenheit(212) → Kelvin: %.2f\n", float64(k2))

	// AbsDiff works with both primitive and named types
	fmt.Println("AbsDiff(int):", AbsDiff(10, 3))        // 7
	fmt.Println("AbsDiff(float):", AbsDiff(3.14, 2.71)) // ~0.43

	// Named type: Score has underlying type int32
	a, b := Score(95), Score(82)
	fmt.Println("AbsDiff(Score):", AbsDiff(a, b)) // 13

	// ---- ParseID — ~ with named string types ----
	fmt.Println("\n=== ParseID with ~string types ===")
	uid, err := ParseID[UserID]("  user-abc-123  ")
	fmt.Printf("UserID: %q, err=%v\n", uid, err) // "user-abc-123", nil

	oid, err := ParseID[OrderID]("order-xyz-456")
	fmt.Printf("OrderID: %q, err=%v\n", oid, err) // "order-xyz-456", nil

	_, err = ParseID[ProductID]("   ")
	fmt.Printf("Empty ProductID: err=%v\n", err) // error

	// ---- Summarizable constraint ----
	fmt.Println("\n=== Method constraint ===")
	events := []Event{
		{Type: "user.login", Payload: "user-123"},
		{Type: "order.placed", Payload: "order-456"},
		{Type: "payment.failed", Payload: "order-456"},
	}
	PrintSummary(events)

	// ---- Numeric constraint with named types ----
	fmt.Println("\n=== Numeric constraint ===")
	scores := []Score{85, 92, 78, 95, 88}
	total := Sum(scores)
	avg := Average(scores)
	fmt.Printf("Scores: %v\n", scores)
	fmt.Printf("Sum: %d, Average: %.1f\n", total, avg)

	// Works with raw float64 too
	latencies := []float64{12.3, 8.7, 15.1, 9.4, 11.2}
	fmt.Printf("Avg latency: %.2fms\n", Average(latencies))

	// ---- Deduplicate and UniqueIDs ----
	fmt.Println("\n=== Deduplication ===")
	tags := []string{"go", "backend", "go", "api", "backend", "generics"}
	deduped := Deduplicate(tags)
	fmt.Println("Deduped tags:", deduped) // [go backend api generics]

	ids := []UserID{"user-1", "user-2", "user-1", "user-3", "user-2"}
	unique := UniqueIDs(ids)
	fmt.Println("Unique UserIDs:", unique) // [user-1 user-2 user-3]

	// ---- IsSorted with named type ----
	fmt.Println("\n=== IsSorted ===")
	times := []Milliseconds{100, 250, 300, 490}
	fmt.Println("Times sorted:", IsSorted(times)) // true

	outOfOrder := []Milliseconds{100, 490, 300}
	fmt.Println("Out of order sorted:", IsSorted(outOfOrder)) // false

	fmt.Println("Strings sorted:", IsSorted([]string{"alpha", "beta", "gamma"})) // true
}
