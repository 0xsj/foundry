let nums = [1, 2, 3, 4];

const sumback = function sum(accum: number, curr: number) {
  return accum + curr;
};
let init = 0;

type Fn = (accum: number, curr: number) => number;

function reduce(nums: number[], fn: Fn, init: number): number {
  if (nums.length === 0) {
    return init;
  }

  return nums.reduce(fn, init);
}

console.log(reduce(nums, sumback, init));
