from collections import defaultdict
from typing import List

class Solution:
    def groupAnagrams(self, strs: List[str]) -> List[List[str]]:
        # here we get key is a tuple representing char Count in the words + value that is a list of anagrams
        res = defaultdict(list)  # mapping charCount to list of anagrams

        for s in strs:
            count = [0] * 26  # a ... z

            for c in s:
                count[ord(c) - ord("a")] += 1  # ascii value math (z - a = 25)

            # Convert the count list to a tuple so it can be used as a dictionary key
            key = tuple(count)
            res[key].append(s)

        return list(res.values())

strs = ["eat", "tea", "tan", "ate", "nat", "bat"]
solution = Solution()
print(solution.groupAnagrams(strs))
