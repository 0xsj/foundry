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


function singleNumber2(nums: number[]): number {
    const seen = new Set();
    for(const num of nums) {
        if
    }
}

console.log(singleNumber([4, 3, 2, 2, 3]));
console.log(singleNumber([1, 3, 3]));
console.log(singleNumber([1]));
console.log(singleNumber([3, 3, 1]));
