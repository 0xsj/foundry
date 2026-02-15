package main

import (
	"fmt"
	"os"
)

func main() {
	fmt.Println("=== Conditionals Demo ===\n")

	// Basic if
	temperature := 32
	if temperature > 30 {
		fmt.Println("It's hot outside")
	}

	// If with else
	age := 25
	if age >= 18 {
		fmt.Println("You can vote")
	} else {
		fmt.Println("Too young to vote")
	}

	// If with initialization statement
	if score := calculateScore(); score > 80 {
		fmt.Printf("Excellent! Score: %d\n", score)
	} else if score > 60 {
		fmt.Printf("Good! Score: %d\n", score)
	} else {
		fmt.Printf("Needs improvement. Score: %d\n", score)
	}
	// score is not accessible here

	// Idiomatic error handling with scoped initialization
	if err := validateUser("alice", 25); err != nil {
		fmt.Println("Validation failed:", err)
	} else {
		fmt.Println("User validated successfully")
	}

	// Guard clause pattern (early return)
	result, err := processFile("data.txt")
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println("File processed:", result)
}

func calculateScore() int {
	return 85
}

func validateUser(name string, age int) error {
	if name == "" {
		return fmt.Errorf("name cannot be empty")
	}
	if age < 18 {
		return fmt.Errorf("must be 18 or older")
	}
	return nil
}

func processFile(path string) (string, error) {
	// Simulate file processing with guard clauses
	if path == "" {
		return "", fmt.Errorf("path cannot be empty")
	}

	if _, err := os.Stat(path); err != nil {
		return "", fmt.Errorf("file not found: %w", err)
	}

	// Happy path: file exists
	return "processed successfully", nil
}
