function singleNumber(nums: number[]): number {
  // create a new map, where we keep track of numbers we encounter
  const map = new Map<number, number>();

  for (let num of nums) {
    map.set(num, (map.get(num) || 0) + 1);
  }

  for (let [num, count] of map) {
    if (count === 1) {
      return num;
    }
  }

  return 0;
}

// function singleNumber2(nums: number[]): number {
//   const seen = new Set<number>();
//   for (const num of nums) {
//     if (seen.has(num)) {
//       seen.delete(num);
//     } else {
//       seen.add(num);
//     }
//   }
//   return seen.values().next().value || -1;
// }

function singleNumber2(nums: number[]): number {
  let result = 0;

  for (const num of nums) {
    result ^= num;
  }

  return result;
}

function singleNumber3(nums: number[]): number {
  const bucket: Record<number, number> = {};
  nums.forEach((number, index) => {
    if (bucket[number]) {
      bucket[number]++;
    } else {
      bucket[number] = 1;
    }
  });

  for (const key in bucket) {
    if (bucket[key] == 1) {
      return parseInt(key);
    }
  }

  return 0;
}

console.log(singleNumber2([4, 3, 2, 2, 3]));
console.log(singleNumber2([1, 3, 3]));
console.log(singleNumber2([1]));
console.log(singleNumber2([3, 3, 1]));
