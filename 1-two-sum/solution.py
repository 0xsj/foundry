from typing import List

nums = [2,7,11,15]
target = 9

class Solution:
    def twoSum_try1(self, nums: List[int], target: int) -> List[int]:
        n = len(nums)
        for i in range(n):
            for j in range(i + 1, n):
                if nums[i] + nums[j] == target:
                    return [i, j]
        return [-1, 1]
    
    def twoSum_try2(self, nums: List[int], target: int) -> List[int]:
        n = len(nums)
        indicies = {}
        for i, num in enumerate(nums):
            diff = target - num
            if diff in indicies:
                return [indicies[diff], i]
            indicies[num] = i
        return [-1, 1]   


    def twoSum_try3(self, nums: List[int], target: int) -> List[int]:
        new_array = [(num, index) for index, num in enumerate(nums)]
        new_array = sorted(new_array, key=lambda x: x[0])

        left, right = 0, len(new_array) - 1

        while left < right:
            current_sum = new_array[left][0] + new_array[right][0]

            if current_sum == target:
                return [new_array[left][1], new_array[right][1]]
            elif current_sum < target:
                left += 1
            else:
                right -= 1

        return []


    def twoSum_try4(self, nums: List[int], target: int) -> List[int]:
        for i in range(len(nums)):
            complement = target - nums[i]
            left, right = i + 1, len(nums) - 1

            while left <= right:
                middle = (left + right) // 2

                if nums[middle] == complement:
                    return [nums[i], nums[middle]]
                elif nums[middle] < complement:
                    left = middle + 1
                else:
                    right = middle - 1
        return []

        
solution = Solution()

print(solution.twoSum_try3(nums, target))
