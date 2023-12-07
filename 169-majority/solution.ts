// // pass 1 WIP
// function majorityElement(nums: number[]): number {
//   const set = new Set(nums);
//   // base case
//   if (nums.length === 0) {
//     return 0;
//   }

//   for (const num of nums) {
//     if (set.has(num)) {
//       set.add(num);
//     }
//   }

//   console.log(set);

//   return 0;
// }

// pass 2
function majorityElement2(nums: number[]): number {
  const map = new Map<number, number>();
  for (const num of nums) {
    if (map.has(num)) {
      map.set(num, map.get(num)! + 1);
    } else {
      map.set(num, 1);
    }
  }

  let max = 0;
  let current = 0;

  for (const [key, value] of map) {
    if (value > current) {
      current = value;
      max = key;
    }
  }

  return max;
}

// third pass
// function majorityElement(nums: number[]): number {
//   let largest = 0;
//   let window = 0;
//   const sorted = nums.sort((a, b) => a - b);

//   for (let i = 0; i < sorted.length; i++) {
//     if (i > 0 && sorted[i] === sorted[i - 1]) {
//       window++;
//     } else {
//       window = 1;
//     }

//     if (window > largest) {
//       largest = window;
//     }
//   }
//   return largest;
// }

// fourth pass - two pointer
// function majorityElement(nums: number[]): number {
//   let candidate = 0;
//   let count = 0;

//   for (const num of nums) {
//     if (count === 0) {
//       candidate = num;
//     }
//     count += num === candidate ? 1 : -1;
//   }

//   return candidate;
// }

// boyer
function majorityElement(nums: number[]): number {
  let candidate = 0;
  let count = 0;

  let sorted = nums.sort((a, b) => a - b);

  for (let i = 0; i < sorted.length; i++) {
    if (count === 0) {
      candidate = sorted[i];
    }

    count += sorted[i] === candidate ? 1 : -1;
  }

  return candidate;
}

console.log(majorityElement([3, 2, 3])); // 3 should return
console.log(majorityElement([2, 2, 1, 1, 1, 2, 2])); // 2 should return
