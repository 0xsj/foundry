from typing import List

case1 = [[1, 3], [3, 0, 1], [2], [0]]
case2 = [[1], [2], [3], []]

class Solution:
    def canVisitAllRooms(self, rooms: List[List[int]]) -> bool:
        keys, rooms_left = set(rooms[0]), set(range(len(rooms)))
        keys.add(0)
        for _ in range(len(rooms)):
            rooms_left -= keys
            new_keys = [rooms[x] for x in keys]
            acquired_keys = set(item for row in new_keys for item in row)
            keys.update(acquired_keys)
            if not rooms_left:
                break

        return len(rooms_left) == 0

solution = Solution()

result1 = solution.canVisitAllRooms(case1)
result2 = solution.canVisitAllRooms(case2)

print("Result for case 1:", result1)
print("Result for case 2:", result2)
