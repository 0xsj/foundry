word1 = "abc"
word2 = "pqr"

class Solution:
    def mergeAlternately(self, word1: str, word2: str) -> str:
        result = ''

        for i in range(max(len(word1), len(word2))):
            if i < len(word1):
                result += word1[i]
            if i < len(word2):
                result += word2[i]
                
        return result
    
solution = Solution()

print(solution.mergeAlternately(word1, word2))