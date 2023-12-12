// find the mode

function findMode(nums: number[]): number {
  let result = 0;

  const map = new Map<number, number>();

  console.log(typeof map);

  console.log(map);

  for (const num of nums) {
    if (map.has(num)) {
      map.set(num, map.get(num)! + 1);
    } else {
      map.set(num, 1);
    }
  }

  let frequency = 0;

  for (const [key, value] of map) {
    if (frequency < value) {
      frequency = value;
      result = key;
    }
  }

  console.log(result);

  return result;
}

// second pass

// // two pointer
// function findMode2(arr: number[]): number[] {
//   let result: number[] = [];
//   let map = {};

//   for (let i = 0; i < arr.length; i++) {
//     if (!map[arr[i]]) {
//       map[arr[i]] = 0;
//       map[arr[i]]++;
//     }
//   }

//   return [];
// }

findMode([
  1, 2, 2, 2, 3, 3, 4, 5, 5, 5, 5, 5, 2, 2, 13, 2, 54, 6, 4, 3, 1, 34, 4, 2, 5, 5, 6, 6, 13, 54,
]); // 2
