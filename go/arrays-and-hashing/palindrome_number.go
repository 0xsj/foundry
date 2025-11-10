package arrays_and_hashing

import "strconv"

func IsPalindrome(x int) bool {

	if x < 0 {
		return false
	}

	if x%10 == 0 && x != 0 {
		return false
	}

	numStr := strconv.Itoa(x)

	left := 0
	right := len(numStr) - 1

	for left < right {
		if numStr[left] != numStr[right] {
			return false
		}

		left++
		right--
	}
	return true
}