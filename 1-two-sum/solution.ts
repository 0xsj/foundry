const dnums = [2, 7, 11, 15];
const target = 9;

/**
 * 1. we first start a loop through the nums array
 * 2. we then can start a nested loop inside, with the index incremented by 1
 * 3. if the sum of nums[i] and nums[j] === target, we return the indicies
 * 4. else we return [-1, 1], to satisfy the requirement of the number[]
 *
 * Time = O(n^2)
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
 * O(n)
 *
 */
const hashMap = (nums: number[], target: number): number[] => {
  const indicies = new Map<number, number>();
  for (let i = 0; i < nums.length; i++) {
    const diff = target - nums[i];
    if (indicies.has(diff)) {
      return [indicies.get(diff)!, i]; // non-null assertion
    }
    indicies.set(nums[i], i);
  }
  return [];
};

/**
 * 1. create a copy of the original array with the indces for sorting. we do this so we can associate each number with its original index.
 * 2. we sort the new array based on the first element of each sub array. we sort via ascending order. we need to do this for the two pointer two work properly.
 * 3. left, right - we initialize two pointers. we scan the new array from beginning and end of the array.
 * 4. we start a while loop, that will coontinue until the left pointer is less than the right.
 * 5. we calculate the sum of numbers pointed by left and right, and checks if they add up to the target.
 * 6. if we have a match, we return the pair of numbers that add up to the target.
 * 7. we continue, incrementing / decrementing the left / right respectively until we find something.
 *
 * O(n log n)
 */

const twoPointer = (nums: number[], target: number): number[] => {
  const newArray = nums.map((num, index) => [num, index]);
  newArray.sort((a, b) => a[0] - b[0]);

  let left = 0;
  let right = newArray.length - 1;

  while (left < right) {
    const sum = newArray[left][0] + newArray[right][0];

    if (sum === target) {
      return [newArray[left][1], newArray[right][1]];
    } else if (sum < target) {
      left++;
    } else {
      right--;
    }
  }
  return [];
};

/**
 * 1. start iterating over the nums array
 * 2. we initialize 3 variables, complement, left, right
 * 3. complement tells us whether or not when added to the current element being considered in nums, array would result in target sum
 * 4. we then start a while loop that continues as long as the left pointer is less than or equal to the right pointer
 * 5. the middle index is calculated using the average of the left / right values. we are dividng the space in halves
 * 6. if the middle index is equal to the complement, we know that we find the answer
 * 7. if the middle index is less than the complement, we move the left pointer to the right of the index
 * 8. if greater, the right pointer is moved to the left
 */
const binarySearchForTwoSum = (nums: number[], target: number): number[] => {
  for (let i = 0; i < nums.length; i++) {
    let complement = target - nums[i];
    let left = i + 1;
    let right = nums.length - 1;

    while (left <= right) {
      let middle = Math.floor((left + right) / 2);

      if (nums[middle] === complement) {
        return [nums[i], nums[middle]]; // Found the two numbers
      } else if (nums[middle] < complement) {
        left = middle + 1;
      } else {
        right = middle - 1;
      }
    }
  }

  return []; // If no such pair exists
};

hashMap(dnums, target);
