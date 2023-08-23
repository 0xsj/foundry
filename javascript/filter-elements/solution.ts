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

/**
 * if (fn(value, index)) { directly uses the result of calling the fn callback function as a condition in an if statement.
 * It doesn't explicitly store the result; instead, it checks whether the result is truthy, and if it is, it executes the code inside the if block.
 * let res = fn(value, index); allows you to explicitly capture the result and use it later
 * For example, you could log the result or perform additional operations on it.
 * if (fn(value, index)) { is a more concise way to check the truthiness of the result and conditionally execute code based on that result. It's useful when you want to take action based on whether the callback function's result is truthy or falsy.
 */
function filter_forEach(arr: number[], fn: (n: number, i: number) => any): number[] {
  const results: number[] = [];
  arr.forEach((value, index) => {
    if (fn(value, index)) {
      results.push(value);
    }
  });
  return results;
}

/**
 * 1. callback here takes in result, currentValue, and index
 * 2. inside the callback, we do a if check for truthy.
 * 3. push the current value n the results array.
 */

function filter_reduce(arr: number[], fn: (n: number, i: number) => any): number[] {
  return arr.reduce<number[]>((result, currentValue, index) => {
    if (fn(currentValue, index)) {
      result.push(currentValue);
    }
    return result;
  }, []);
}

console.log(filter_reduce(num2, greaterThanCB));
