def two_sum(nums: list[int], target: int) -> list[int]:
    num_map = {}
    
    for i, num in enumerate(nums):
        complement = target - num
        
        if complement in num_map:
            return [num_map[complement], i]
        else:
            num_map[num] = i
    
    return []
