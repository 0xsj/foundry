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

console.log(twoSum(arr, target167));
