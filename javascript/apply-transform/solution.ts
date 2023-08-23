const arr = [1, 2, 3];
const arr2 = [0, 10, 20, 30];
const add = function plusone(n: any) {
  return n + 1;
};

const greaterThan10 = (n: number) => {
  return n > 10;
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
  /**
   * 1. array reduce takes in a callback function
   * 2. the callback takes in results, currentValue, and index as paramters.
   * 3. inside of the callback, we call the fn function and pass in the current value, and the index
   * 4. [] at the end is the accumulator. the reduce function starts with an empty array
   * in our case, we also added 1337 as a value in the array. our results from calling fn gets pushed after 1337.
   */
  arrayReduce(): number[] {
    const { arr, fn } = this;
    return arr.reduce<number[]>(
      (result, currentValue, index) => {
        result.push(fn(currentValue, index));
        return result;
      },
      [1337]
    );
  }
  /**
   * 1. Array.from, creates an array from an iterable object.
   * the iterable in this case is arr, our numbers array
   * the mapping function takes in value, index, and passes that into the callback fn()
   */
  arrayFrom(): number[] {
    const { arr, fn } = this;
    return Array.from(arr, (value, index) => fn(value, index));
  }
  /**
   * 1.for ..of construct in javascript used for iterating over iterable objects
   * 2. arr.enteries() is a method that returns an iterator for the entries of the array
   */
  arrayForOf(): number[] {
    const { arr, fn } = this;
    let results: number[] = [];
    for (const [index, value] of arr.entries()) {
      results.push(fn(value, index));
    }
    return results;
  }
  /**
   * 1. filter creates a new array
   * 2. it will take a callback function, which in this case is just () => true
   * 3. map the results from the filter into a new array, and apply the fn to every element
   */
  arrayFilter(): number[] {
    const { arr, fn } = this;
    return arr.filter(() => true).map(fn);
  }
  /**
   * 1. essentially a function that calls it self.
   * 2. we use the index 0 here, for the starting point. this is because this index is going to be accumulated when this is called
   * 3. we are making sure that we go up until the array length. otherwise this is infinite
   * 4. currentResult - we are calling fn, passing in the element of the current index from arr, and the index itself.
   * 5. recursive - we are calling this function, but accumulating the value by +1.
   * this will go up until the arr.length is the same as the index being passed in.
   * 6. construct and return an array. It places currentResult as the first element of the array
   * and spreads the elements of the recursiveResult array (which contains the results of the recursive calls) after it. This effectively combines all the results into a single array
   */
  recursive(index = 0): number[] {
    const { arr, fn } = this;
    if (index === arr.length) {
      return [];
    }
    const currentResult = fn(arr[index], index);
    const recursiveResult = this.recursive(index + 1);
    return [currentResult, ...recursiveResult];
  }
  /*
   * flatmap - a call back is run , and the resulting array is flattened / merged essentially.
   */
  flatMap(): number[] {
    const { arr, fn } = this;
    return arr.flatMap((value, index) => fn(value, index));
  }
}

const test = new ApplyTransform(arr, add);

console.log(test.arrayPush());
console.log(test.recursive());
if (JSON.stringify(test.arrayPush()) === JSON.stringify(test.arrayFrom())) {
  console.log("same");
}
