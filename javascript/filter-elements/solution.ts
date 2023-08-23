const num = [1, 2, 3];
const num2 = [0, 10, 20, 30];

const addCB = function plusone(n: any) {
  return n + 1;
};

const greaterThanCB = (n: number) => {
  return n > 10;
};

class FilterElements {
  arr: number[];
  fn: (n: number, i: number) => any;
  constructor(arr: number[], fn: (n: number, i: number) => any) {
    this.arr = arr;
    this.fn = fn;
  }
}

function filtered_for(arr: number[], fn: (n: number, i: number) => any): number[] {
  return arr.filter((num, index) => fn(num, index));
}

console.log(filtered_for(num2, greaterThanCB));
