const nums = [0, 1, 0, 3, 12];
const numsReversed = [12, 3, 0, 1, 0];

function moveZeroes(nums: number[]): void {
  let count = 0;
  for (let i = 0; i < nums.length; i++) {
    if (nums[i] === 0) {
      count++;

      nums.splice(i, 1);
      console.log(nums);

      i--;
    }
  }
  for (let i = 0; i < count; i++) {
    nums.push(0);
  }

  console.log(nums);
}

moveZeroes(nums);
