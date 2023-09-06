function topKFrequent(nums: number[], k: number): number[] {
  const map = new Map();

  for (const num of nums) {
    map.set(num, map.get(num || 0) + 1);
  }

  const sortedUniqueNumbers = Array.from(map.keys()).sort((a, b) => {
    return map.get(b)! - map.get(a)!;
  });

  return sortedUniqueNumbers.slice(0, k);
}

topKFrequent([1, 2, 3, 4], 2);
