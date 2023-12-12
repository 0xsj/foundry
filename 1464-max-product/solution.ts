function maxProduct(nums: number[]): number {
  if (nums.length < 2) {
    return 0;
  }
  let first = 0;
  let second = 1;

  if (nums[1] > nums[0]) {
    first = 1;
    second = 0;
  }

  for (let i = 2; i < nums.length; i++) {
    if (nums[i] > nums[first]) {
      second = first;
      first = i;
    } else if (nums[i] > nums[second]) {
      second = i;
    }
  }

  nums[first]--;
  nums[second]--;

  const result = nums[first] * nums[second];
  console.log(result);

  return result;
}

maxProduct([3, 4, 5, 2]); // 3 * 4 = 12
maxProduct([1, 5, 4, 5]); // 4 * 4 = 16
