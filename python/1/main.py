# // Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.

# // You may assume that each input would have exactly one solution, and you may not use the same element twice.

# // You can return the answer in any order.

# // Example 1:

# // Input: nums = [2,7,11,15], target = 9
# // Output: [0,1]
# // Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].
# // Example 2:

# // Input: nums = [3,2,4], target = 6
# // Output: [1,2]
# // Example 3:

# // Input: nums = [3,3], target = 6
# // Output: [0,1]


class Solution:
    # we are taking the nums array, and running the first for loop.
    # then we are looping over a second time, but incremented by +1
    # we are saying that the sum is equal to the two numbers in the list
    # if the sum of the two indicies are the same as the target value provided,
    # return the two numbers in the array
    def bruteForce(self, nums, target):
        for i in range(len(nums)):
            for j in range(i + 1, len(nums)):
                sum = nums[i] = nums[j]
                if sum == target:
                    print("True")
                    return [i, j]
        return []

    # def optimized(self, nums, target):
    #     return nums, target


solution = Solution()
nums = [2, 7, 11, 15]
target = 9
nums2 = [3, 2, 4]
target2 = 6
nums3 = [3, 3]
target3 = 6
print(solution.bruteForce(nums, target))
