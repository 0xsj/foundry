package main

import "fmt"

func isRunePalindrome(str string) bool {
	runes := []rune(str)
	left, right := 0, len(runes)-1

	for left < right {
		if runes[left] != runes[right] {
			return false
		}
		left++
		right--
	}

	return true
}

func main() {
	str := "A man, a plan, a canal: Panama"
	fmt.Println(isRunePalindrome(str))
}
