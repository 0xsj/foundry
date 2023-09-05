const nums = [-1, 0, 3, 5, 9, 12];

function search(nums: number[], target: number): number {
  let left = 0;
  let right = nums.length - 1;

  while (left <= right) {
    let middle = Math.floor((left + right) / 2);
    if (nums[middle] > target) {
      right = middle - 1;
    } else if (nums[middle] < target) {
      left = middle + 1;
    } else {
      return middle;
    }
  }

  return -1;
}

/**
 *
 */

function recursiveSearch(nums: number[], target: number): number {
  const recurse = (start: number, end: number): number => {
    if (nums[start] === undefined || nums[end] === undefined) {
      return -1;
    }
    if (start > end) {
      return -1;
    }
    if (start === end) {
      if (nums[start] === target) {
        return start;
      }
      return -1;
    }

    const middle = Math.floor((end + start) / 2);
    if (nums[middle] === target) {
      return middle;
    } else if (target < nums[middle]) {
      return recurse(start, middle - 1);
    } else {
      return recurse(middle + 1, end);
    }
  };

  return recurse(0, nums.length - 1);
}

console.log(recursiveSearch(nums, 9));
