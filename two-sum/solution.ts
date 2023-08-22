const nums = [2, 7, 11, 15];
const target = 9;

/**
 * 1. we first start a loop through the nums array
 * 2. we then can start a nested loop inside, with the index incremented by 1
 * 3. if the sum of nums[i] and nums[j] === target, we return the indicies
 * 4. else we return [-1, 1], to satisfy the requirement of the number[]
 */
const bruteForce = (nums: number[], target: number): number[] => {
  for (let i = 0; i < nums.length; i++) {
    for (let j = i + 1; j < nums.length; j++) {
      if (nums[i] + nums[j] === target) {
        return [i, j];
      }
    }
  }
  return [-1, 1];
};

console.log(bruteForce(nums, target));
