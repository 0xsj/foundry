function maxOperations(nums: number[], k: number): number {
  let count = 0;
  let left = 0;
  let right = nums.length - 1;

  // sor the nums
  const newArray = nums.sort((a, b) => a - b);

  console.log(newArray);

  while (left < right) {
    const sum = newArray[left] + newArray[right];
    if (sum === k) {
      newArray.splice(left, 1);
      newArray.splice(right - 1, 1);
      count++;
      right -= 2;
    } else if (sum < k) {
      left++;
    } else {
      right--;
    }
  }

  // count variable, to keep in track of the total number of operatios
  // keep a left variable, starting at 0 index
  // keep a right variable or end of the array. nums.length -1
  // we need to sort the array.
  // we are examining the array
  // left++
  // right--
  // we take a look at two "pairs". if nusms[pair1] + nums[pair2] === k,
  // pop nums[pair1] nums[pair2] out of the array
  // repeat
  // if there are no more possible outcomes, meaning none of the remaining numbers add up to k,
  // end and return the count
  return count;
}

maxOperations([1, 4, 3, 2, 5], 6);

// function maxOperations_try2(nums: number[], k: number): number {
//   // perhaps its possible to do it with a
// }

// function maxOperations_try3(nums: number[], k: number): number {}
