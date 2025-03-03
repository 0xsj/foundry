// 1. Two Sum
// Easy
// Topics
// Companies
// Hint

// Given an array of integers nums and an integer target, return indices of the two numbers such that they add up to target.

// You may assume that each input would have exactly one solution, and you may not use the same element twice.

// You can return the answer in any order.

 

// Example 1:

// Input: nums = [2,7,11,15], target = 9
// Output: [0,1]
// Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].

// Example 2:

// Input: nums = [3,2,4], target = 6
// Output: [1,2]

// Example 3:

// Input: nums = [3,3], target = 6
// Output: [0,1]

 

// Constraints:

//     2 <= nums.length <= 104
//     -109 <= nums[i] <= 109
//     -109 <= target <= 109
//     Only one valid answer exists.

 
// Follow-up: Can you come up with an algorithm that is less than O(n2) time complexity?



function twoSum_hashmap(nums: number[], target: number): number[] {
    const map = new Map<number, number>();
    for (let  i = 0; i < nums.length; i++) {
        const diff = target - nums[i];

        if (map.has(diff)) {
            return [map.get(diff)!, i];
        }

        map.set(nums[i], i)
    }
    return [];
}


function twoSum_twopointer(nums: number[], target: number): number[] {

    const sortedNums = nums.map((num, index) => ({num, index})).sort((a, b) => a.num - b.num)

    let left = 0, right = nums.length - 1;

    while (left < right) {
        const sum = sortedNums[left].num + sortedNums[right].num;
        if(sum === target) {
            return [sortedNums[left].index, sortedNums[right].index]
        } else if (sum < target) {
            left++;
        } else {
            right--;
        }
    }

    return [];
}

class TreeNode {
    val: number;
    left: TreeNode | null;
    right: TreeNode | null;
    constructor(val?: number, left?: TreeNode | null, right?: TreeNode | null) {
        this.val = val ?? 0;
        this.left = left ?? null;
        this.right = right ?? null;
    }
}

function twoSumBST(root: TreeNode | null, target: number): boolean {
    const seen = new Set<number>();
    function dfs(node: TreeNode | null): boolean {
        if(!node) return false;

        if(seen.has(target - node.val)) return true;
        seen.add(node.val)

        return dfs(node.left) || dfs(node.right)
    }
    return dfs(root)
}

class GraphNode {
    val: number;
    neighbors: GraphNode[];
    constructor(val: number) {
        this.val = val;
        this.neighbors = [];
    }
}

function twoSumGraphBFS(startNode: GraphNode, target: number): boolean {
    const queue: GraphNode[] = [startNode]
    const seen = new Set<number>();

    while(queue.length > 0) {
        const node = queue.shift()!;

        if(seen.has(target - node.val)) return true;
        seen.add(node.val);

        for (let neighbor of node.neighbors) {
            if (!seen.has(neighbor.val)) queue.push(neighbor)
        }
    }

    return false;
}

