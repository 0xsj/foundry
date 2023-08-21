const arr1 = [1, 1, 1, 3, 3, 4, 3, 2, 4, 2]; // true
const arr2 = [1, 2, 3, 4]; // false

function hashmap(nums: number[]): boolean {
  return false;
}
/**
 * 1. we first define a function named bruceforce that takes in a number array, and returns a boolean.
 * 2. when we go through the first loop, we check each value in the array
 * 3. in the nested for loop, we increment the count by 1, and go to the end of the array.
 * j = i + 1 - This is done to ensure that you compare each element at index i with all elements that come after it in the array
 * 4. if the value at the index from the first for loop matches the nested for loop, we know we have a duplciate. we return true.
 * 5. else, we return false.
 */
function bruteforce(nums: number[]): boolean {
  for (let i = 0; i < nums.length; i++) {
    for (let j = i + 1; j < nums.length; j++) {
      if (nums[i] === nums[j]) {
        return true;
      }
    }
  }
  return false;
}

console.log("Bruce force : " + bruteforce(arr1));
