package main

// ============================================================================
// Variables and Types — Go
// ============================================================================
// Fill in each section. Run with: go run variables.go
// Or just save and ask for a review.
// ============================================================================

import "fmt"

func main() {
	// ========================================================================
	// 1. EXPLICIT DECLARATION
	// Declare a variable using the `var` keyword with an explicit type.
	// Assign it any value you like.
	// ========================================================================
	// a) a string
	var x string = "hello"

	// b) an int
	var y int = 2

	// c) a float64

	var a float64 = 1.0

	// d) a bool
	var isString bool = false

	fmt.Println(x, y, a, isString)

	// ========================================================================
	// 2. SHORT DECLARATION
	// Declare the same four types using Go's := shorthand.
	// Use different variable names than section 1.
	// ========================================================================
	// a) a string
	worldStr := "world"

	// b) an int
	year := 2000

	// c) a float64
	confidence := 0.9

	// d) a bool
	isDog := false
	fmt.Println(worldStr, year, confidence, isDog)

	// ========================================================================
	// 3. ZERO VALUES
	// Declare variables with `var` but DO NOT assign a value.
	// Then print each one. What does Go give you?
	// ========================================================================
	// a) var zeroStr string
	var zeroStr string
	fmt.Println(zeroStr)

	// b) var zeroInt int
	var zeroInt int
	fmt.Println(zeroInt)

	// c) var zeroFloat float64
	var zeroFloat float64
	fmt.Println(zeroFloat)

	// d) var zeroBool bool
	var zeroBool bool
	fmt.Println(zeroBool)

	// Print them:
	// fmt.Println(zeroStr, zeroInt, zeroFloat, zeroBool)

	// ========================================================================
	// 4. MULTIPLE DECLARATION
	// Go lets you declare multiple variables in one statement.
	// Declare at least 3 variables in a single var () block.
	// ========================================================================

	str1, str2, str3 := 1, 2, "string"
	fmt.Println(str1, str2, str3)

	// ========================================================================
	// 5. CONSTANTS
	// Declare a constant using `const`. Try declaring one without a type —
	// Go constants are "untyped" until they're used. What does that mean?
	// ========================================================================
	// a) a typed constant
	const val int = 3

	// b) an untyped constant
	const isFloat = false

	// ========================================================================
	// 6. PRINT EVERYTHING
	// Use fmt.Println or fmt.Printf to print all your variables.
	// For Printf, try using %T to print the TYPE of a variable.
	// Example: fmt.Printf("myVar: %v (type: %T)\n", myVar, myVar)
	// ========================================================================

	fmt.Println("done")
}
