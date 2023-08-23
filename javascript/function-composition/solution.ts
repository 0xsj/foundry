type F = (x: number) => number;

/**
 * 1. check if the length is 0, if true, we return itself
 * 2. otherwlse, we are returning the result of the reduce method on the functions array
 * 3. reduce, calls the specified callback function for all the elements in the array
 * - call back here is (composed, fn) => {...}
 * 4. the return value of the callback function is the acculmulated result, and is provided as an argument in the next call to the callback function
 * 5. we define a new function (x) => ...
 * 6. we compose the the current funciton with the previously composed function fn(x)
 */
function compose(functions: F[]): F {
  if (functions.length === 0) {
    return function (x) {
      return x;
    };
  }

  return functions.reduce(
    (composed, fn) => {
      return (x) => composed(fn(x));
    },
    (x) => x
  );
}

/**
 * reduce right
 */

function compose_reduceRight(functions: F[]): F {
  if (functions.length === 0) {
    return function (x) {
      return x;
    };
  }
  return functions.reduceRight((prev, next) => {
    return (x: any) => {
      return next(prev(x));
    };
  });
}
/**
 * const fn = compose([x => x + 1, x => 2 * x])
 * fn(4) // 9
 */
