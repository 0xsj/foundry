class Solution:
    def isPalindrome(self, x: int) -> bool:
        if x < 0:
            return False
        x_str = str(x)
        if x_str != x_str[::-1]:
            return False    
        return True
    def isPalindrome_try2(self, x:int) -> bool:
        if x < 0:
            return False
        original = x
        reversed = 0

        while (x > 0):
            lastDigit = x % 10
            reversed = reversed*10 +  lastDigit
            x = x // 10

        return original == reversed
    
solution = Solution()
print(solution.isPalindrome_try2(121))  # This will print True
