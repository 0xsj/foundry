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

  /**
   * 1. declare a new array results
   * 2. loop though the array that is passed in as args
   * 3. push each element back into the array and pass in the callback with the position and value as args
   * 4. return the results;
   */
  arrayPush(): number[] {
    const { arr, fn } = this;
    const results: number[] = [];
    for (let i = 0; i < arr.length; i++) {
      results.push(fn(arr[i], i));
    }
    return results;
  }

  /**
   * 1. we do the same thing as the array push
   * 2. this time we use forEach. The forEach() method of Array instances executes a provided function once for each array element.
   * 3. the results.push will execute every time for each iterable
   */
  arrayForEach(): number[] {
    const { arr, fn } = this;
    const results: number[] = [];
    arr.forEach((value, index) => {
      results.push(fn(value, index));
    });
    return results;
  }
  arrayReduce(): number[] {
    return [];
  }
  arrayFrom(): number[] {
    return [];
  }
  arrayForOf(): number[] {
    return [];
  }
  arrayFilter(): number[] {
    return [];
  }
  recursive(): number[] {
    return [];
  }
  flatMap(): number[] {
    return [];
  }
}

const test = new ApplyTransform(arr, add);

console.log(test.arrayPush());
console.log(test.arrayForEach());
