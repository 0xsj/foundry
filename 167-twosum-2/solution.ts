const arr = [2, 7, 11, 15];
const target167 = 9;

function twoSum(numbers: number[], target: number): number[] {
  const map = new Map<number, number>();
  for (let i = 0; i < numbers.length; i++) {
    let diff = target - numbers[i];

    if (map.has(diff)) {
      console.log(map);
      return [map.get(diff)!, i + 1];
    }

    map.set(numbers[i], i + 1);
  }

  return [];
}

// two pointer
function twoSum2(numbers: number[], target: number): number[] {
  let left = 0;
  let right = numbers.length - 1;

  while (left < right) {
    let current = numbers[left] + numbers[right];

    if (current > target) {
      right -= 1;
    } else if (current < target) {
      left += 1;
    } else {
      return [left + 1, right + 1];
    }
  }

  return [];
}

// filtered
function twoSum3(numbers: number[], target: number): number[] {
  const filtered = numbers.filter((value) => value <= target);
  let left = 0;
  let right = filtered.length - 1;

  while (left < right) {
    let current = filtered[left] + filtered[right];
    if (current > target) {
      right -= 1;
    } else if (current < target) {
      left += 1;
    } else {
      return [left + 1, right + 1];
    }
  }

  return [];

  return [];
}

console.log(twoSum3(arr, target167));
