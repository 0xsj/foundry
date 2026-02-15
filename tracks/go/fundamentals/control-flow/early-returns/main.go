package main

import (
	"fmt"
	"strings"
)

func main() {
	fmt.Println("=== Early Returns / Guard Clauses Demo ===\n")

	// Example 1: User validation
	fmt.Println("User validation:")
	if err := createUser("", "alice@example.com", 25); err != nil {
		fmt.Println("✗", err)
	}
	if err := createUser("Alice", "", 25); err != nil {
		fmt.Println("✗", err)
	}
	if err := createUser("Bob", "bob@example.com", 15); err != nil {
		fmt.Println("✗", err)
	}
	if err := createUser("Carol", "carol@example.com", 30); err != nil {
		fmt.Println("✗", err)
	} else {
		fmt.Println("✓ User created successfully")
	}

	// Example 2: Processing pipeline
	fmt.Println("\nProcessing pipeline:")
	data := []string{"hello", "world"}
	result, err := processPipeline(data)
	if err != nil {
		fmt.Println("✗", err)
	} else {
		fmt.Println("✓ Result:", result)
	}

	// Example 3: Nested conditionals vs guard clauses
	fmt.Println("\nComparing nested vs guard clause style:")
	processDataNested("valid", 42)
	processDataGuarded("valid", 42)
}

// ✅ Idiomatic: Guard clauses with early returns
func createUser(name, email string, age int) error {
	// Guard clause: check name
	if name == "" {
		return fmt.Errorf("name is required")
	}

	// Guard clause: check email
	if email == "" {
		return fmt.Errorf("email is required")
	}

	// Guard clause: validate email format
	if !strings.Contains(email, "@") {
		return fmt.Errorf("invalid email format")
	}

	// Guard clause: check age
	if age < 18 {
		return fmt.Errorf("must be 18 or older")
	}

	// Happy path: all validations passed
	fmt.Printf("Creating user: %s (%s), age %d\n", name, email, age)
	return nil
}

// ✅ Idiomatic: Processing pipeline with guard clauses
func processPipeline(data []string) (string, error) {
	// Step 1: Validate input
	if len(data) == 0 {
		return "", fmt.Errorf("empty data")
	}

	// Step 2: Normalize
	normalized, err := normalize(data)
	if err != nil {
		return "", fmt.Errorf("normalize failed: %w", err)
	}

	// Step 3: Transform
	transformed, err := transform(normalized)
	if err != nil {
		return "", fmt.Errorf("transform failed: %w", err)
	}

	// Step 4: Validate result
	if err := validateResult(transformed); err != nil {
		return "", fmt.Errorf("validation failed: %w", err)
	}

	// Happy path: return result
	return transformed, nil
}

func normalize(data []string) (string, error) {
	if len(data) == 0 {
		return "", fmt.Errorf("cannot normalize empty data")
	}
	return strings.Join(data, " "), nil
}

func transform(data string) (string, error) {
	if data == "" {
		return "", fmt.Errorf("cannot transform empty string")
	}
	return strings.ToUpper(data), nil
}

func validateResult(data string) error {
	if len(data) < 3 {
		return fmt.Errorf("result too short")
	}
	return nil
}

// ❌ Not idiomatic: Deeply nested
func processDataNested(input string, value int) {
	if input != "" {
		if value > 0 {
			if value < 100 {
				fmt.Println("Nested: Valid data")
			} else {
				fmt.Println("Nested: Value too large")
			}
		} else {
			fmt.Println("Nested: Value must be positive")
		}
	} else {
		fmt.Println("Nested: Input is empty")
	}
}

// ✅ Idiomatic: Guard clauses (early returns)
func processDataGuarded(input string, value int) {
	if input == "" {
		fmt.Println("Guarded: Input is empty")
		return
	}

	if value <= 0 {
		fmt.Println("Guarded: Value must be positive")
		return
	}

	if value >= 100 {
		fmt.Println("Guarded: Value too large")
		return
	}

	// Happy path at the top level
	fmt.Println("Guarded: Valid data")
}
