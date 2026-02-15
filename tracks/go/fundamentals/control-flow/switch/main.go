package main

import (
	"fmt"
	"time"
)

func main() {
	fmt.Println("=== Switch Demo ===\n")

	// Basic switch
	day := "Friday"
	switch day {
	case "Monday":
		fmt.Println("Start of the week")
	case "Friday":
		fmt.Println("Almost weekend!")
	case "Saturday", "Sunday":
		fmt.Println("Weekend!")
	default:
		fmt.Println("Midweek")
	}

	// Switch with initialization
	fmt.Println("\nSwitch with initialization:")
	switch hour := time.Now().Hour(); {
	case hour < 12:
		fmt.Println("Good morning")
	case hour < 18:
		fmt.Println("Good afternoon")
	default:
		fmt.Println("Good evening")
	}

	// Expression-less switch (like if-else chain)
	fmt.Println("\nExpression-less switch:")
	temperature := 25
	switch {
	case temperature < 0:
		fmt.Println("Freezing")
	case temperature < 20:
		fmt.Println("Cold")
	case temperature < 30:
		fmt.Println("Warm")
	default:
		fmt.Println("Hot")
	}

	// Switch with fallthrough
	fmt.Println("\nSwitch with fallthrough:")
	num := 1
	switch num {
	case 1:
		fmt.Println("One")
		fallthrough
	case 2:
		fmt.Println("One or Two") // executes for both 1 and 2
	case 3:
		fmt.Println("Three")
	}

	// Type switch
	fmt.Println("\nType switch:")
	printType(42)
	printType("hello")
	printType(3.14)
	printType(true)

	// Switch for state machine
	fmt.Println("\nSwitch in state machine:")
	runStateMachine()

	// Switch for error handling
	fmt.Println("\nSwitch for error handling:")
	handleResult(doOperation(true))
	handleResult(doOperation(false))
}

func printType(value interface{}) {
	switch v := value.(type) {
	case int:
		fmt.Printf("Integer: %d\n", v)
	case string:
		fmt.Printf("String: %q\n", v)
	case float64:
		fmt.Printf("Float: %.2f\n", v)
	case bool:
		fmt.Printf("Boolean: %t\n", v)
	default:
		fmt.Printf("Unknown type: %T\n", v)
	}
}

func runStateMachine() {
	state := "start"
	steps := 0
	maxSteps := 5

	for steps < maxSteps {
		switch state {
		case "start":
			fmt.Println("State: start -> initializing")
			state = "processing"
		case "processing":
			fmt.Println("State: processing -> working")
			state = "done"
		case "done":
			fmt.Println("State: done -> complete")
			return
		default:
			fmt.Println("Unknown state")
			return
		}
		steps++
	}
}

func doOperation(success bool) error {
	if success {
		return nil
	}
	return fmt.Errorf("operation failed")
}

func handleResult(err error) {
	switch err {
	case nil:
		fmt.Println("✓ Operation succeeded")
	default:
		fmt.Printf("✗ Error: %v\n", err)
	}
}
