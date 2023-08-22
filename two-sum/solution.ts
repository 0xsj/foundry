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

/**
 * 1. we create a new empty map called indicies
 * 2. we start a for loop, go until the full length of the array.
 * 3. we look for a diff. this calculates the difference between the target and the current element in the nums array
 * 4. essentially, we just need to figure out what number we need to add to the current number in order to get to the target.
 * 5. the if block, we are just checking if we have seen a number in the nums array that when added to the current number, it would equal the target.
 * 6. if the indicies contain a diff value as a key, it means that there is a number in the nums array that meets our conditions
 * 7. we return an array containing two indicies.
 * - diff! - gets the index frosm the indicides map that corresponds to the complement diff.
 * - i - current index in the nums array , where we found the current number nums[i]
 *
 */
const hashMap = (nums: number[], target: number): number[] => {
  const indicies = new Map<number, number>();
  for (let i = 0; i < nums.length; i++) {
    const diff = target - nums[i];
    if (indicies.has(diff)) {
      return [indicies.get(diff)!, i];
    }
    indicies.set(nums[i], i);
  }
  return [];
};

console.log("hashmap : ", hashMap(nums, target));
console.log("bruteforce  : ", bruteForce(nums, target));
