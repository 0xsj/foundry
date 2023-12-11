function findSpecialInteger(arr: number[]): number {
  let result = -1;
  const targetFrequency = arr.length / 4;
  const map = new Map();

  for (const num of arr) {
    if (map.has(num)) {
      map.set(num, map.get(num) + 1);
    } else {
      map.set(num, 1);
    }
  }

  map.forEach((key, val) => {
    if (key > targetFrequency) {
      result = val;
    }
  });

  return result;
}
