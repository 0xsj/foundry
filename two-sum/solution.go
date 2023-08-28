package main

import "fmt"

func twoSum_bruceForce(nums []int, target int) []int {
	for i := 0; i < len(nums); i++ {
		for j := i + 1; j < len(nums); j++ {
			if nums[i]+nums[j] == target {
				return []int{i, j}
			}
		}
	}
	return []int{}
}

func main() {
	array1 := []int{2, 7, 3, 5}
	array2 := []int{3, 6, 1, 0}

	target := 9

	fmt.Println(twoSum_bruceForce(array1, target))
}
