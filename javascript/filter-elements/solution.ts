function filter(arr: number[], fn: (n: number, i: number) => any): number[] {
  return arr.filter((num, index) => fn(num, index));
}

console.log(filter(arr, add));
