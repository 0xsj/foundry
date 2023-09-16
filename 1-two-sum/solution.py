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
    
    def twoSum_try2():
        return
    def twoSum_try3():
        return
    def twoSum_try4():
        return

        
solution = Solution()

print(solution.twoSum_try1(nums, target))