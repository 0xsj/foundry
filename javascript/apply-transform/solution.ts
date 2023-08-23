const arr = [1, 2, 3];

const add = function plusone(n: any) {
  return n + 1;
};

const genericForloop = (arr: number[], fn: (n: number, i: number) => number): number[] => {
  const results: number[] = [];
  for (let i = 0; i < arr.length; i++) {
    results.push(fn(arr[i], i));
  }
  return results;
};
