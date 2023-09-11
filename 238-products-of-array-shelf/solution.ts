/**
 * 1.
 */
function productExceptSelf_try1(nums: number[]): number[] {
  const resultArray = nums.map((_, index) => {
    return nums.reduce((accumulator, currentValue, currentIndex) => {
      if (currentIndex === index) {
        return accumulator;
      } else {
        return accumulator * currentValue;
      }
    }, 1);
  });

  return resultArray;
}

/** */
function productExceptSelf_try2(nums: number[]): number[] {
  const n = nums.length;
  const result = new Array(n).fill(1);

  let leftProduct = 1;
  for (let i = 0; i < n; i++) {
    result[i] *= leftProduct;
    leftProduct *= nums[i];
  }

  let rightProduct = 1;
  for (let i = n - 1; i >= 0; i--) {
    result[i] *= rightProduct;
    rightProduct *= nums[i];
  }

  return result;
}
console.log(productExceptSelf_try1([4, 3, 2, 1, 2]));
