from typing import List

class Solution:
    def search(self, nums: List[int], target:int) -> int:
        left = 0
        right = len(nums) - 1
        
