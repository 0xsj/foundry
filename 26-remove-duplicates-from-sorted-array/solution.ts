const ar1 = [1, 1, 2];
const ar2 = [0, 0, 1, 1, 1, 2, 2, 3, 3, 4];
const ar3 = [0, 0, 0, 0];

/**
 *
 */
const removeDuplicateSorted_try1 = (nums: number[]): number => {
  let i = 0;
  const map = new Map();

  for (let num of nums) {
    if (!map.has(num)) {
      map.set(num, true);
      nums[i] = num;
      i++;
    }
  }
  return i;
};

/**
 *
 */

console.log(removeDuplicateSorted_try1(ar1));
