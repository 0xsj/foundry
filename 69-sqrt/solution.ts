function mySqrt(x: number): number {
  return Math.floor(Math.sqrt(x));
}

// newton-raphson
function mySqrt3(x: number): number {
  if (x === 0 || x === 1) {
    return x;
  }

  let guess = x / 2;

  while (Math.abs(guess * guess - x) > 0.0001) {
    guess = 0.5 * (guess + x / guess);
  }

  return Math.floor(guess);
}

//
function mySqrt2(x: number): number {
  // recursive?

  // start a while loop
  while (x != 0) {}

  return 0;
}

console.log(mySqrt(4)); // 2
console.log(mySqrt3(4)); // 2
console.log(mySqrt3(12)); // 2
console.log(mySqrt3(9)); // 2
console.log(mySqrt3(8)); // 2
