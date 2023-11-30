const arr1 = [1, 1, 1, 3, 3, 4, 3, 2, 4, 2]; // true
const arr2 = [1, 2, 3, 4]; // false

/**
 * 1. we declare a new map, a build in data structure that allows us to store key value pairs of any data type.
 * 2. we iterate over the nums array using a for of
 * 3. we do a check to determine if the current num / iterable is already present in the map using .get method
 * 4. if the get returns something else other than undefined, it tells us that the num has been encountered before, hence a duplciate.
 * 5. if the num is not found in the map, we add it to the map with a value of 1, to keep track of the encountered numbers
 *
 * O(n)
 */
function hashmap(nums: number[]): boolean {
  const map = new Map();
  for (let num of nums) {
    if (map.get(num) !== undefined) {
      return true;
    }
    map.set(num, 1);
  }
  return false;
}
/**
 * 1. we first define a function named bruceforce that takes in a number array, and returns a boolean.
 * 2. when we go through the first loop, we check each value in the array
 * 3. in the nested for loop, we increment the count by 1, and go to the end of the array.
 * j = i + 1 - This is done to ensure that you compare each element at index i with all elements that come after it in the array
 * 4. if the value at the index from the first for loop matches the nested for loop, we know we have a duplciate. we return true.
 * 5. else, we return false.
 *
 *  Time =  O(n^2)
 *
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

/**
 * 1. in this solution, we create a new "set" with the nums array
 * 2. a Set is a data structure that contains unique set of values.
 * 3. when we compare the size to the length
 * 4. if the size is different from the length, it means that there are duplicates.
 * this is because in a set, duplicate values are automatically removed.
 *
 * O(n)
 */
function set(nums: number[]): boolean {
  const results = new Set(nums);
  return results.size !== nums.length;
}

console.log("Bruce force : " + bruteforce(arr1));
console.log("Hash map : " + hashmap(arr1));
console.log("set : " + set(arr1));
