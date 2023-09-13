/** */
function uniqueOccurrences(arr: number[]): boolean {
  const map = new Map();
  for (const num of arr) {
    console.log(map);

    if (map.has(num)) {
      map.set(num, map.get(num) + 1);
    } else {
      map.set(num, 1);
    }
  }

  const occurenceSet = new Set();

  for (const count of map.values()) {
    if (occurenceSet.has(count)) {
      return false;
    }
    occurenceSet.add(count);
  }

  return true;
}

console.log(uniqueOccurrences([1, 2, 2, 1, 1, 3]));
console.log(uniqueOccurrences([1, 2]));
