package main

import "fmt"

func main() {
	fmt.Println("=== Loops Demo ===\n")

	// Traditional for loop
	fmt.Println("Traditional for loop:")
	for i := 0; i < 5; i++ {
		fmt.Printf("%d ", i)
	}
	fmt.Println()

	// While-style loop
	fmt.Println("\nWhile-style loop:")
	count := 0
	for count < 5 {
		fmt.Printf("%d ", count)
		count++
	}
	fmt.Println()

	// Infinite loop with break
	fmt.Println("\nInfinite loop with break:")
	n := 0
	for {
		if n >= 5 {
			break
		}
		fmt.Printf("%d ", n)
		n++
	}
	fmt.Println()

	// Range over slice (index and value)
	fmt.Println("\nRange over slice (index + value):")
	numbers := []int{10, 20, 30, 40, 50}
	for i, num := range numbers {
		fmt.Printf("numbers[%d] = %d\n", i, num)
	}

	// Range over slice (value only)
	fmt.Println("\nRange over slice (value only):")
	for _, num := range numbers {
		fmt.Printf("%d ", num)
	}
	fmt.Println()

	// Range over slice (index only)
	fmt.Println("\nRange over slice (index only):")
	for i := range numbers {
		fmt.Printf("%d ", i)
	}
	fmt.Println()

	// Range over map
	fmt.Println("\nRange over map:")
	scores := map[string]int{
		"Alice": 95,
		"Bob":   87,
		"Carol": 92,
	}
	for name, score := range scores {
		fmt.Printf("%s: %d\n", name, score)
	}

	// Range over string (runes)
	fmt.Println("\nRange over string:")
	for i, char := range "Go" {
		fmt.Printf("Index %d: %c (rune value: %d)\n", i, char, char)
	}

	// Continue example
	fmt.Println("\nContinue (skip even numbers):")
	for i := 0; i < 10; i++ {
		if i%2 == 0 {
			continue
		}
		fmt.Printf("%d ", i)
	}
	fmt.Println()

	// Break example
	fmt.Println("\nBreak (stop at 5):")
	for i := 0; i < 10; i++ {
		if i == 5 {
			break
		}
		fmt.Printf("%d ", i)
	}
	fmt.Println()

	// Labeled break (nested loops)
	fmt.Println("\nLabeled break (nested loops):")
outer:
	for i := 0; i < 3; i++ {
		for j := 0; j < 3; j++ {
			fmt.Printf("(%d,%d) ", i, j)
			if i*j >= 2 {
				break outer // exits both loops
			}
		}
	}
	fmt.Println()

	// Range creates copies (important gotcha)
	fmt.Println("\nRange creates copies:")
	nums := []int{1, 2, 3}
	fmt.Println("Original:", nums)

	// This won't modify the slice
	for _, num := range nums {
		num = num * 2 // modifies the copy
	}
	fmt.Println("After range (wrong):", nums)

	// This will modify the slice
	for i := range nums {
		nums[i] = nums[i] * 2
	}
	fmt.Println("After index modification:", nums)
}
