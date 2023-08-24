package main

import (
	"fmt"
)

/*
 */
func bruteForce(nums []int) bool {
	for i := 0; i < len(nums); i++ {
		for j := i + 1; j < len(nums); j++ {
			if nums[j] == nums[i] {
				return true
			}
		}
	}
	return false
}

/*
 */
func containsDuplicate(nums []int) bool {
	seen := make(map[int]bool)

	for _, num := range nums {
		if seen[num] {
			return true
		}
		seen[num] = true
	}
	return false
}

/**/

func main() {
	nums := []int{1, 2, 3, 4}
	nums2 := []int{1, 2, 3, 4, 5, 5}

	fmt.Println(containsDuplicate((nums)))
	fmt.Println(containsDuplicate((nums2)))
}
