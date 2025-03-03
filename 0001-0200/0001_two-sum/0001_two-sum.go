package main

func twoSum(nums []int, target int) []int {

	seen := make(map[int]int)

	for i, num := range nums {
		complement := target - num
		if idx, found := seen[complement]; found {
			return []int{idx, i}
		}

		seen[num] = i
	}

	return nil 
}