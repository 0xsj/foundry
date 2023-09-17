class Solution:
    def isPalindrome(self, x: int) -> bool:
        if x < 0:
            return False
        x_str = str(x)
        if x_str != x_str[::-1]:
            return False    
        return True
    
solution = Solution()
print(solution.isPalindrome(121))  # This will print True
