function increasingTriplet_try1(nums: number[]): boolean {
  if (nums.length < 3) {
    return false; // Triplet cannot exist with fewer than 3 elements.
  }

  let firstMin = Infinity;
  let secondMin = Infinity;

  for (let num of nums) {
    if (num <= firstMin) {
      firstMin = num; // Update the first minimum.
    } else if (num <= secondMin) {
      secondMin = num; // Update the second minimum.
    } else {
      return true; // We found an increasing triplet.
    }
  }

  return false; // No increasing triplet found.
}

function increasingTriplet(nums: number[]): boolean {
  if (nums.length < 3) {
    return false; // Triplet cannot exist with fewer than 3 elements.
  }

  const result = nums.map((num, index) => {
    let firstMin = Infinity;
    let secondMin = Infinity;

    for (let i = 0; i < index; i++) {
      if (nums[i] < firstMin) {
        firstMin = nums[i]; // Update the first minimum.
      } else if (nums[i] < secondMin) {
        secondMin = nums[i]; // Update the second minimum.
      }
    }

    return num > secondMin;
  });

  return result.includes(true); // Check if any element in the result array is true.
}

console.log(increasingTriplet([1, 2, 3, 4, 5, 6]));
console.log(increasingTriplet([5, 4, 3, 2, 1]));
console.log(increasingTriplet([2, 1, 3, 0, 4, 6]));
