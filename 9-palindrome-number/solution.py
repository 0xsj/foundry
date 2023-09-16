class Solution:
    def isPalindrome(self, x: int) -> bool:
        left = 0
        right = -1

        # get rid of fail cases negative integers cannot be palindromes
        if x < 0:
            return False

        res = str(x).split(" ")[0]

        

        print(res)


        return False
    
solution = Solution()

solution.isPalindrome(121)