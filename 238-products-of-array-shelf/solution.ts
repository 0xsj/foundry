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

console.log(productExceptSelf_try1([4, 3, 2, 1, 2]));
