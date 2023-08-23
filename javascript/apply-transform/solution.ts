const arr = [1, 2, 3];

const add = function plusone(n: any) {
  return n + 1;
};

class ApplyTransform {
  arr: number[];
  fn: (n: number, i: number) => number;

  constructor(arr: number[], fn: (n: number, i: number) => number) {
    this.arr = arr;
    this.fn = fn;
  }

  arrayPush(): number[] {
    const { arr, fn } = this;
    const results: number[] = [];
    for (let i = 0; i < arr.length; i++) {
      results.push(fn(arr[i], i));
    }
    return results;
  }

  // arrayForEach(): {}
  // arrayReduce() {}
  // arrayFrom() {}
  // arrayForOf {}
  // arrayFilter {}
  // recursive {}
  // flatMap
}

const test = new ApplyTransform(arr, add);

console.log(test.arrayPush());
