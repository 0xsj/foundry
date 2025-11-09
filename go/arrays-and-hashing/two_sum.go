package arrays_and_hashing

func TwoSum(nums []int, target int) []int {
	numMap := make(map[int]int)

	for i := 0; i < len(nums); i++ {
		complement := target - nums[i]

		if index, exists := numMap[complement]; exists {
			return []int{index, i}
		}

		numMap[nums[i]] = i
	}

	return []int{}
}

