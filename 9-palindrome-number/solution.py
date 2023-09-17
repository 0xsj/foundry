class Solution:
    def isPalindrome(self, x: int) -> bool:
        # Check for negative integers
        if x < 0:
            return False
        
        # Convert the integer to a string
        x_str = str(x)
        
        # Compare the string with its reverse
        return x_str == x_str[::-1]

solution = Solution()
print(solution.isPalindrome(121))  # This will print True
