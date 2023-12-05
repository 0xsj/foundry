function buildArray(nums: number[]): number[] {
  let ans: number[] = [];

  for (let i = 0; i < nums.length; i++) {
    ans.push(nums[nums[i]]);
  }

  console.log(ans);

  return ans;
}

buildArray([0, 2, 1, 5, 3, 4]);
buildArray([5, 0, 1, 2, 3, 4]);
