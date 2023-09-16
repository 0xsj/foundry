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

// func twoSum_hashMap(nums []int, target int) []int {
// 	m := make(map[int]int, 0)

// 	for idx, num := range nums {
// 		m[num] = idx
// 	}

// 	for indexX, num := range nums {
// 		if indexY, ok := m[target - num]
// 	}
// }

func twoSum_hashMap(nums []int, target int) []int {
	m := make(map[int]int, 0)
	fmt.Println(m)

	return []int{-1, 1}
}

func main() {
	array1 := []int{2, 7, 3, 5}
	// array2 := []int{3, 6, 1, 0}

	target := 9

	fmt.Println(twoSum_bruceForce(array1, target))
}
