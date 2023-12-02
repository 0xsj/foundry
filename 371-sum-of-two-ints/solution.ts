// https://www.youtube.com/watch?v=NLKQEOgBAnw

function getSum(a: number, b: number): number {
  if (b == 0) {
    return a;
  } else {
    return getSum(a ^ b, (a & b) << 1);
  }
}

function getSum2(a: number, b: number): number {
  while (b !== 0) {
    const carry = a & b;
    a = a ^ b;
    b = carry << 1;
  }
  return a;
}

console.log(getSum2(10, 11));
console.log(getSum2(1, 2));
