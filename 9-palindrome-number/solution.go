package main

import (
	"fmt"
	"strconv"
)

// this try using strconv works.
func isPalindromeNumber(x int) bool {

	if x < 0 {
		return false
	}

	numStr := strconv.Itoa(x)
	left := 0
	right := len(numStr) - 1

	for left < right {
		if numStr[left] != numStr[right] {
			fmt.Println(false)
			return false
		}
		left++
		right--
	}
	fmt.Println(true)
	return true
}

// is palindrome try 2
func isPalindromeNumber_try2(x int) bool {
	if x < 0 {
		return false
	}

	original := x
	reversed := 0

	for x > 0 {
		lastDigit := x % 10
		reversed = reversed*10 + lastDigit

		x /= 10
	}

	fmt.Println(original == reversed)

	return original == reversed
}

func main() {
	fmt.Println("main running")

	isPalindromeNumber_try2(122)
}
