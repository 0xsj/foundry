function intersection(nums1: number[], nums2: number[]): number[] {
  const filtered = nums1.filter((value) => nums2.includes(value));
  const set = new Set(filtered);
  const result = Array.from(set);

  return result;
}

function intersection2(nums1: number[], nums2: number[]): number[] {
  const map = new Map<number, number>();
  const result: number[] = [];

  for (let num of nums1) {
    map.set(num, map.get(num) || 0 + 1);
  }

  for (let num of nums2) {
    if (map.has(num) && map.get(num)! > 0) {
      result.push(num);
      map.set(num, map.get(num)! - 1);
    }
  }
  return result;
}

function intersection3(nums1: number[], nums2: number[]): number[] {
  const result = nums1.reduce((acc: number[], value: number) => {
    if (nums2.includes(value) && !acc.includes(value)) {
      acc.push(value);
    }
    return acc;
  }, []);
  return result;
}

function intersection4(nums1: number[], nums2: number[]): number[] {
  let bitmask1 = 0;
  let bitmask2 = 0;
  let result: number[] = [];

  for (const num of nums1) {
    bitmask1 |= 1 << num;
  }

  for (const num of nums2) {
    bitmask2 |= 1 << num;
  }

  const common = bitmask1 & bitmask2;

  for (let i = 0; i < 31; i++) {
    if ((common & (1 << i)) !== 0) {
      result.push(i);
    }
  }

  return result;
}

console.log(intersection4([1, 3, 3, 1], [1]));
console.log(intersection4([4, 9, 5], [9, 4, 9, 8, 4]));
