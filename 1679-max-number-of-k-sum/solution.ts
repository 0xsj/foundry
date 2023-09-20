function maxOperations(nums: number[], k: number): number {
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
}

function maxOperations_try2(nums: number[], k: number): number {}

function maxOperations_try3(nums: number[], k: number): number {}
